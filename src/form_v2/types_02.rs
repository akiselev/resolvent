#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum FormExprV2 {
    ScientificScalar {
        expression: Expr,
    },
    Argument {
        id: ArgumentIdV2,
    },
    Coefficient {
        field: ScientificFieldIdV2,
    },
    Constant {
        id: ConstantIdV2,
    },
    Neg {
        value: Box<FormExprV2>,
    },
    Add {
        values: Vec<FormExprV2>,
    },
    Product {
        values: Vec<FormExprV2>,
    },
    Apply {
        function: String,
        args: Vec<FormExprV2>,
    },
    Gradient {
        value: Box<FormExprV2>,
        frame: FrameIdV2,
    },
    TimeDerivative {
        value: Box<FormExprV2>,
    },
    Dot {
        left: Box<FormExprV2>,
        right: Box<FormExprV2>,
    },
    Inner {
        left: Box<FormExprV2>,
        right: Box<FormExprV2>,
    },
    Contract {
        left: Box<FormExprV2>,
        right: Box<FormExprV2>,
        left_axis: usize,
        right_axis: usize,
    },
    Conjugate {
        value: Box<FormExprV2>,
    },
    Transpose {
        value: Box<FormExprV2>,
    },
    HermitianTranspose {
        value: Box<FormExprV2>,
    },
    Restrict {
        value: Box<FormExprV2>,
        side: TraceSideV2,
    },
}

impl FormExprV2 {
    fn canonicalize(&mut self) {
        match self {
            Self::ScientificScalar { .. }
            | Self::Argument { .. }
            | Self::Coefficient { .. }
            | Self::Constant { .. } => {}
            Self::Neg { value }
            | Self::Gradient { value, .. }
            | Self::TimeDerivative { value }
            | Self::Conjugate { value }
            | Self::Transpose { value }
            | Self::HermitianTranspose { value }
            | Self::Restrict { value, .. } => value.canonicalize(),
            Self::Add { values } | Self::Product { values } => {
                for value in values.iter_mut() {
                    value.canonicalize();
                }
                values.sort_by_cached_key(canonical_json);
            }
            Self::Apply { args, .. } => {
                for argument in args {
                    argument.canonicalize();
                }
            }
            Self::Dot { left, right }
            | Self::Inner { left, right }
            | Self::Contract { left, right, .. } => {
                left.canonicalize();
                right.canonicalize();
            }
        }
    }

    fn referenced_arguments(&self, out: &mut BTreeSet<ArgumentIdV2>) {
        match self {
            Self::Argument { id } => {
                out.insert(id.clone());
            }
            Self::ScientificScalar { .. }
            | Self::Coefficient { .. }
            | Self::Constant { .. } => {}
            Self::Neg { value }
            | Self::Gradient { value, .. }
            | Self::TimeDerivative { value }
            | Self::Conjugate { value }
            | Self::Transpose { value }
            | Self::HermitianTranspose { value }
            | Self::Restrict { value, .. } => value.referenced_arguments(out),
            Self::Add { values } | Self::Product { values } => {
                for value in values {
                    value.referenced_arguments(out);
                }
            }
            Self::Apply { args, .. } => {
                for argument in args {
                    argument.referenced_arguments(out);
                }
            }
            Self::Dot { left, right }
            | Self::Inner { left, right }
            | Self::Contract { left, right, .. } => {
                left.referenced_arguments(out);
                right.referenced_arguments(out);
            }
        }
    }
}

fn canonical_json<T: Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("V2 form nodes are JSON serializable")
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct IntegralV2 {
    pub id: String,
    pub measure: MeasureV2,
    pub integrand: FormExprV2,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VariationalFormV2 {
    pub schema: String,
    pub name: String,
    pub arity: u16,
    #[serde(default)]
    pub arguments: Vec<FormArgumentV2>,
    #[serde(default)]
    pub coefficients: Vec<FormCoefficientV2>,
    #[serde(default)]
    pub constants: Vec<ConstantRefV2>,
    #[serde(default)]
    pub integrals: Vec<IntegralV2>,
    pub scalar_kind: ScalarKindV2,
    pub derivation: FormulationDerivationIdV2,
    #[serde(default)]
    pub obligations: Vec<String>,
}

impl VariationalFormV2 {
    pub const fn arity(&self) -> u16 {
        self.arity
    }

    fn computed_arity(&self) -> u16 {
        self.arguments
            .iter()
            .map(|argument| argument.number)
            .max()
            .map_or(0, |number| number + 1)
    }

    pub fn argument_parts(&self) -> BTreeMap<u16, Vec<Option<u16>>> {
        let mut out = BTreeMap::<u16, Vec<Option<u16>>>::new();
        for argument in &self.arguments {
            out.entry(argument.number).or_default().push(argument.part);
        }
        for parts in out.values_mut() {
            parts.sort_unstable();
            parts.dedup();
        }
        out
    }

    pub fn extract_block(
        &self,
        selection: &BTreeMap<u16, u16>,
    ) -> Result<Self, FormV2Error> {
        for (&number, &part) in selection {
            if !self
                .arguments
                .iter()
                .any(|argument| argument.number == number && argument.part == Some(part))
            {
                return Err(FormV2Error::BlockSelection {
                    number,
                    part,
                });
            }
        }

        let selected_arguments = self
            .arguments
            .iter()
            .filter(|argument| {
                selection
                    .get(&argument.number)
                    .is_none_or(|part| argument.part == Some(*part))
            })
            .cloned()
            .collect::<Vec<_>>();
        let selected_ids = selected_arguments
            .iter()
            .map(|argument| argument.id.clone())
            .collect::<BTreeSet<_>>();
        let excluded_ids = self
            .arguments
            .iter()
            .map(|argument| argument.id.clone())
            .filter(|id| !selected_ids.contains(id))
            .collect::<BTreeSet<_>>();
        let integrals = self
            .integrals
            .iter()
            .filter(|integral| {
                let mut referenced = BTreeSet::new();
                integral.integrand.referenced_arguments(&mut referenced);
                referenced.is_disjoint(&excluded_ids)
            })
            .cloned()
            .collect();
        let suffix = selection
            .iter()
            .map(|(number, part)| format!("a{number}p{part}"))
            .collect::<Vec<_>>()
            .join("-");
        let mut out = Self {
            name: if suffix.is_empty() {
                self.name.clone()
            } else {
                format!("{}::{suffix}", self.name)
            },
            arguments: selected_arguments,
            integrals,
            ..self.clone()
        };
        out.canonicalize();
        Ok(out)
    }

    fn canonicalize(&mut self) {
        self.arguments.sort_by(|left, right| {
            (left.number, left.part, &left.id).cmp(&(right.number, right.part, &right.id))
        });
        self.coefficients.sort_by(|left, right| left.field.cmp(&right.field));
        self.constants.sort_by(|left, right| left.id.cmp(&right.id));
        for integral in &mut self.integrals {
            integral.integrand.canonicalize();
        }
        self.integrals.sort_by(|left, right| left.id.cmp(&right.id));
        self.obligations.sort();
        self.obligations.dedup();
    }
}
