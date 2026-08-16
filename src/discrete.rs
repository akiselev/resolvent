//! Discretization dialect between variational mathematics and solver-facing operators.
//!
//! The representation follows the restriction/basis/pointwise/integration decomposition used
//! by high-performance matrix-free FEM systems. Retaining these operations makes assembled,
//! partial-assembly and matrix-free realizations alternative lowerings of the same discrete
//! semantics rather than unrelated implementations.

use crate::ExprId;
use crate::form::FieldId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DiscreteOpId(pub u32);
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BasisId(pub u32);
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct QuadratureId(pub u32);
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RestrictionId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BasisAction {
    Interpolate,
    Gradient,
    Divergence,
    Curl,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiscreteOp {
    Restrict {
        field: FieldId,
        restriction: RestrictionId,
    },
    Basis {
        basis: BasisId,
        action: BasisAction,
        input: DiscreteOpId,
    },
    Pointwise {
        /// Mathematical expression evaluated at quadrature points. The expression remains a
        /// Resolvent semantic object; Anvil decides how to execute it.
        expression: ExprId,
        inputs: Vec<DiscreteOpId>,
    },
    Integrate {
        quadrature: QuadratureId,
        input: DiscreteOpId,
    },
    BasisTranspose {
        basis: BasisId,
        action: BasisAction,
        input: DiscreteOpId,
    },
    ScatterAdd {
        restriction: RestrictionId,
        input: DiscreteOpId,
    },
    Sum(Vec<DiscreteOpId>),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DiscreteProgram {
    pub operations: Vec<DiscreteOp>,
    pub outputs: Vec<DiscreteOpId>,
}

impl DiscreteProgram {
    pub fn push(&mut self, op: DiscreteOp) -> DiscreteOpId {
        let id = DiscreteOpId(self.operations.len() as u32);
        self.operations.push(op);
        id
    }

    pub fn op(&self, id: DiscreteOpId) -> Option<&DiscreteOp> {
        self.operations.get(id.0 as usize)
    }
}
