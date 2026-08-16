use crate::id::{ExprId, FieldId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DiscreteValueId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BasisEvaluation {
    Value,
    Gradient,
    Curl,
    Divergence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RestrictionDirection {
    Gather,
    ScatterAdd,
}

/// A libCEED-style structured discrete dialect. It keeps element restriction, basis
/// evaluation, quadrature-point physics and transpose operations explicit so the same
/// mathematical discretization can lower to assembled, partial-assembly or matrix-free
/// execution without changing its scientific identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiscreteOp {
    FieldInput {
        field: FieldId,
    },
    Restrict {
        input: DiscreteValueId,
        field: FieldId,
        direction: RestrictionDirection,
    },
    Basis {
        input: DiscreteValueId,
        field: FieldId,
        evaluation: BasisEvaluation,
        transpose: bool,
    },
    Pointwise {
        inputs: Vec<DiscreteValueId>,
        expressions: Vec<ExprId>,
    },
    QuadratureWeight {
        input: DiscreteValueId,
        rule: String,
    },
    Sum {
        inputs: Vec<DiscreteValueId>,
    },
    Custom {
        operator: String,
        inputs: Vec<DiscreteValueId>,
        metadata: BTreeMap<String, String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscreteInstruction {
    pub output: DiscreteValueId,
    pub op: DiscreteOp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscreteProgram {
    pub name: String,
    #[serde(default)]
    pub instructions: Vec<DiscreteInstruction>,
    #[serde(default)]
    pub outputs: Vec<DiscreteValueId>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl DiscreteProgram {
    pub fn next_value(&self) -> DiscreteValueId {
        DiscreteValueId(
            self.instructions
                .iter()
                .map(|i| i.output.0)
                .max()
                .map_or(0, |v| v + 1),
        )
    }

    pub fn push(&mut self, op: DiscreteOp) -> DiscreteValueId {
        let output = self.next_value();
        self.instructions.push(DiscreteInstruction { output, op });
        output
    }
}
