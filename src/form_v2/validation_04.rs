fn lift_scientific_expr(
    model: &ScientificModel,
    expression: &Expr,
) -> Result<FormExprV2, FormV2Error> {
    Ok(match expression {
        Expr::Number { .. } | Expr::String(_) => FormExprV2::ScientificScalar {
            expression: expression.clone(),
        },
        Expr::Name(name)
            if model.fields.iter().any(|field| {
                field.name == *name
                    && matches!(
                        field.role,
                        FieldRoleV1::State
                            | FieldRoleV1::Unknown
                            | FieldRoleV1::Trial
                            | FieldRoleV1::Coefficient
                    )
            }) =>
        {
            FormExprV2::Coefficient {
                field: ScientificFieldIdV2::new(name.clone()),
            }
        }
        Expr::Name(_) => FormExprV2::ScientificScalar {
            expression: expression.clone(),
        },
        Expr::Unary { arg, .. } => FormExprV2::Neg {
            value: Box::new(lift_scientific_expr(model, arg)?),
        },
        Expr::Binary { op, lhs, rhs } => {
            use crate::scientific::BinaryOp;
            let left = lift_scientific_expr(model, lhs)?;
            let right = lift_scientific_expr(model, rhs)?;
            match op {
                BinaryOp::Add => FormExprV2::Add {
                    values: vec![left, right],
                },
                BinaryOp::Sub => FormExprV2::Add {
                    values: vec![
                        left,
                        FormExprV2::Neg {
                            value: Box::new(right),
                        },
                    ],
                },
                BinaryOp::Mul => FormExprV2::Product {
                    values: vec![left, right],
                },
                BinaryOp::Div => FormExprV2::Apply {
                    function: "divide".into(),
                    args: vec![left, right],
                },
                BinaryOp::Pow => FormExprV2::Apply {
                    function: "power".into(),
                    args: vec![left, right],
                },
                BinaryOp::Eq
                | BinaryOp::Lt
                | BinaryOp::Le
                | BinaryOp::Gt
                | BinaryOp::Ge => FormExprV2::Apply {
                    function: format!("comparison::{op:?}"),
                    args: vec![left, right],
                },
            }
        }
        Expr::Call { function, args } => match (function.as_str(), args.as_slice()) {
            ("grad", [argument]) => {
                let frame = expression_domain_frame(model, argument)?;
                FormExprV2::Gradient {
                    value: Box::new(lift_scientific_expr(model, argument)?),
                    frame,
                }
            }
            ("dt", [argument]) => FormExprV2::TimeDerivative {
                value: Box::new(lift_scientific_expr(model, argument)?),
            },
            ("dot", [left, right]) => FormExprV2::Dot {
                left: Box::new(lift_scientific_expr(model, left)?),
                right: Box::new(lift_scientific_expr(model, right)?),
            },
            ("inner", [left, right]) => FormExprV2::Inner {
                left: Box::new(lift_scientific_expr(model, left)?),
                right: Box::new(lift_scientific_expr(model, right)?),
            },
            ("conj" | "conjugate", [value]) => FormExprV2::Conjugate {
                value: Box::new(lift_scientific_expr(model, value)?),
            },
            ("transpose", [value]) => FormExprV2::Transpose {
                value: Box::new(lift_scientific_expr(model, value)?),
            },
            ("adjoint" | "hermitian_transpose", [value]) => {
                FormExprV2::HermitianTranspose {
                    value: Box::new(lift_scientific_expr(model, value)?),
                }
            }
            ("div" | "curl" | "sym_grad", _) => {
                return Err(FormV2Error::UnloweredTerm {
                    stage: "scalar_h1_compatibility".into(),
                    detail: format!("unsupported differential call `{function}`"),
                });
            }
            _ => FormExprV2::Apply {
                function: function.clone(),
                args: args
                    .iter()
                    .map(|argument| lift_scientific_expr(model, argument))
                    .collect::<Result<Vec<_>, _>>()?,
            },
        },
        Expr::Index { .. } | Expr::Vector(_) => {
            return Err(FormV2Error::UnloweredTerm {
                stage: "scalar_h1_compatibility".into(),
                detail: format!("non-scalar expression `{expression:?}`"),
            });
        }
    })
}

fn expression_domain_frame(
    model: &ScientificModel,
    expression: &Expr,
) -> Result<FrameIdV2, FormV2Error> {
    let mut names = BTreeSet::new();
    expression.names(&mut names);
    let mut frames = names
        .iter()
        .filter_map(|name| {
            model
                .fields
                .iter()
                .find(|field| field.name == *name)
                .map(|field| FrameIdV2::new(format!("frame::{}", field.domain)))
        })
        .collect::<BTreeSet<_>>();
    if frames.len() != 1 {
        return Err(FormV2Error::UnloweredTerm {
            stage: "scalar_h1_compatibility".into(),
            detail: format!(
                "gradient operand must identify exactly one domain frame, found {}",
                frames.len()
            ),
        });
    }
    Ok(frames.pop_first().expect("one frame was checked above"))
}

fn contains_differential_operator(expression: &Expr) -> bool {
    match expression {
        Expr::Call { function, args } => {
            matches!(
                function.as_str(),
                "dt" | "grad" | "div" | "curl" | "sym_grad"
            ) || args.iter().any(contains_differential_operator)
        }
        Expr::Unary { arg, .. } => contains_differential_operator(arg),
        Expr::Binary { lhs, rhs, .. } => {
            contains_differential_operator(lhs) || contains_differential_operator(rhs)
        }
        Expr::Index { value, indices } => {
            contains_differential_operator(value)
                || indices.iter().any(contains_differential_operator)
        }
        Expr::Vector(values) => values.iter().any(contains_differential_operator),
        Expr::Number { .. } | Expr::String(_) | Expr::Name(_) => false,
    }
}

fn digest_serialized<T: Serialize + ?Sized>(value: &T) -> Result<Digest, FormV2Error> {
    serde_json::to_vec(value)
        .map(|bytes| Digest::blake3(&bytes))
        .map_err(|error| FormV2Error::Serialization(error.to_string()))
}
