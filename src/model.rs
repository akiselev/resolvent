use crate::id::{ExprId, ObservableId, SymbolId, SystemId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Explicit model scope. The compiler never infers that a result proved on a restricted
/// family applies to a larger one; any change of scope is carried by a refinement record.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scope {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_region: Option<String>,
    #[serde(default)]
    pub regularity: Vec<String>,
    #[serde(default)]
    pub conventions: BTreeMap<String, String>,
    #[serde(default)]
    pub restrictions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Assumption {
    pub name: String,
    pub statement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formal_declaration: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Equation {
    pub lhs: ExprId,
    pub rhs: ExprId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub guard: ExprId,
    #[serde(default)]
    pub updates: Vec<(SymbolId, ExprId)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// A model/system dialect above variational forms. Acausal equation systems, circuit/MNA
/// systems, geometric algebraic constraints, and continuum models can all inhabit this IR;
/// only continuum models need to continue through `form`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct System {
    pub name: String,
    #[serde(default)]
    pub unknowns: Vec<SymbolId>,
    #[serde(default)]
    pub parameters: Vec<SymbolId>,
    #[serde(default)]
    pub equations: Vec<Equation>,
    #[serde(default)]
    pub events: Vec<Event>,
    #[serde(default)]
    pub children: Vec<SystemId>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Observable {
    pub id: ObservableId,
    pub name: String,
    /// Mathematical observable before an instrument/measurement model is applied.
    pub expression: ExprId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimension: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub measurement_model: Option<ExprId>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyKind {
    Equality,
    Inequality,
    Conservation,
    Symmetry,
    Monotonicity,
    Positivity,
    Stability,
    Passivity,
    Reciprocity,
    Convergence,
    Custom,
}

/// A theorem/invariant/expectation that can later be turned into an executable validator.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PropertyContract {
    pub name: String,
    pub kind: PropertyKind,
    pub statement: ExprId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formal_declaration: Option<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

/// The object at the Lean/Resolvent boundary. A PDE alone is not a scientific
/// specification: assumptions, scope, observables and invariants are part of the meaning.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScientificSpec {
    pub name: String,
    pub model: SystemId,
    #[serde(default)]
    pub assumptions: Vec<Assumption>,
    #[serde(default)]
    pub scope: Scope,
    #[serde(default)]
    pub observables: Vec<Observable>,
    #[serde(default)]
    pub properties: Vec<PropertyContract>,
    #[serde(default)]
    pub sources: Vec<String>,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}
