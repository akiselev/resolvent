use crate::{ExprId, ObservableId, ScopeId};

/// Explicit applicability of a claim/model. Scope changes must be represented by a
/// [`crate::Refinement`] rather than silently widening during lowering or summarization.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Scope {
    pub id: Option<ScopeId>,
    pub label: String,
    pub parameter_region: Option<String>,
    pub spatial_domain: Option<String>,
    pub temporal_domain: Option<String>,
    pub regularity: Vec<String>,
    pub conventions: Vec<String>,
    pub restrictions: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub value: ExprId,
    pub unit: Option<String>,
    pub scope: Option<ScopeId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StateVariable {
    pub name: String,
    pub symbol: ExprId,
    pub unit: Option<String>,
    pub domain: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Law {
    pub name: String,
    /// A residual-style symbolic expression. Zero means the law is satisfied.
    pub residual: ExprId,
    pub scope: Option<ScopeId>,
    /// Optional stable locator back to the formal declaration or source statement.
    pub source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitialCondition {
    pub state: String,
    pub condition: ExprId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BoundaryCondition {
    pub state: String,
    pub boundary: String,
    pub condition: ExprId,
}

/// A formally named quantity that can be compared with simulation or measurement.
///
/// Observables are first-class because validation compares interpretations of the same
/// quantity, not arbitrary internal solver state against raw instrument output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Observable {
    pub id: ObservableId,
    pub name: String,
    pub expression: ExprId,
    pub unit: Option<String>,
    pub measurement_model: Option<ExprId>,
    pub uncertainty_model: Option<ExprId>,
    pub source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationContract {
    Invariant {
        name: String,
        predicate: ExprId,
    },
    Conservation {
        name: String,
        balance: ExprId,
    },
    Bound {
        name: String,
        predicate: ExprId,
    },
    Symmetry {
        name: String,
        relation: ExprId,
    },
    Convergence {
        name: String,
        expected_order: Option<String>,
    },
    ReferenceCase {
        name: String,
        predicate: ExprId,
    },
    ObservableAgreement {
        observable: ObservableId,
        metric: String,
    },
}

/// The semantic center of the scientific stack.
///
/// A `ScientificSpec` states what system is claimed to be modeled, under which assumptions
/// and scope, and what observable/invariant contracts matter. It deliberately contains no
/// mesh, nonlinear solver choice, timestep controller, kernel target, or experimental data.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ScientificSpec {
    pub name: String,
    pub scope: Scope,
    pub assumptions: Vec<String>,
    pub parameters: Vec<Parameter>,
    pub state: Vec<StateVariable>,
    pub laws: Vec<Law>,
    pub initial_conditions: Vec<InitialCondition>,
    pub boundary_conditions: Vec<BoundaryCondition>,
    pub observables: Vec<Observable>,
    pub validation_contracts: Vec<ValidationContract>,
    /// Stable locator for the formal source (for example a Lean declaration + statement hash).
    pub formal_source: Option<String>,
}

impl ScientificSpec {
    pub fn observable(&self, id: ObservableId) -> Option<&Observable> {
        self.observables.iter().find(|o| o.id == id)
    }

    pub fn is_structurally_empty(&self) -> bool {
        self.state.is_empty() && self.laws.is_empty()
    }
}
