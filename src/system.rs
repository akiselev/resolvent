//! Scientific equation-system dialect.
//!
//! This is the shared semantic home for lumped equations/DAEs and the structural view used
//! by former Plexus-style passes. Continuum problems may reference the same variables and
//! equations before lowering their spatial operators into [`crate::form`].

use crate::{EquationId, ExprId};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct VariableId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum VariableKind {
    State,
    Derivative,
    Algebraic,
    Parameter,
    Input,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Variable {
    pub id: VariableId,
    pub name: String,
    pub kind: VariableKind,
    pub expression: ExprId,
    pub derivative_of: Option<VariableId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemEquation {
    pub id: EquationId,
    pub name: String,
    pub residual: ExprId,
    /// Exact structural incidence supplied by construction/analysis. This is deliberately
    /// separate from numeric Jacobian sparsity at one evaluation point.
    pub variables: Vec<VariableId>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct System {
    pub name: String,
    pub variables: Vec<Variable>,
    pub equations: Vec<SystemEquation>,
    pub events: Vec<ExprId>,
    pub assumptions: Vec<String>,
}

impl System {
    pub fn variable(&self, id: VariableId) -> Option<&Variable> {
        self.variables.iter().find(|v| v.id == id)
    }

    pub fn equation(&self, id: EquationId) -> Option<&SystemEquation> {
        self.equations.iter().find(|e| e.id == id)
    }

    pub fn incidence(&self) -> Incidence<'_> {
        Incidence { system: self }
    }
}

/// Read-only structural projection consumed by matching/BLT/index-reduction passes.
pub struct Incidence<'a> {
    system: &'a System,
}

impl<'a> Incidence<'a> {
    pub fn equation_count(&self) -> usize {
        self.system.equations.len()
    }

    pub fn variable_count(&self) -> usize {
        self.system.variables.len()
    }

    pub fn variables_of(&self, equation: usize) -> &[VariableId] {
        &self.system.equations[equation].variables
    }

    pub fn equations(&self) -> impl Iterator<Item = &'a SystemEquation> {
        self.system.equations.iter()
    }
}
