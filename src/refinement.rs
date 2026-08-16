use crate::{ArtifactHash, EvidenceSet, Scope};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Stage {
    FormalSpec,
    SymbolicModel,
    ReducedSystem,
    VariationalForm,
    DiscreteProgram,
    OperatorProgram,
    ExecutableProgram,
    SimulationResult,
    ObservablePrediction,
}

/// Semantic claim made by a transformation.
///
/// These variants are intentionally more precise than a generic `LoweredTo`: they prevent
/// exact equivalence, approximation, specialization, and empirical adequacy from collapsing
/// into the same status word.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RefinementRelation {
    DefinitionallyEqual,
    MathematicallyEquivalent,
    LogicalConsequence,
    Specialization,
    Reformulation,
    StrongToWeakForm,
    IndexReduced,
    Discretization {
        method: String,
        consistency: Option<String>,
        convergence: Option<String>,
    },
    Approximation {
        metric: String,
        bound: Option<String>,
    },
    FinitePrecisionImplementation {
        arithmetic: String,
        error_model: Option<String>,
    },
    CompiledImplementation {
        backend: String,
    },
    ObservableInterpretation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Assumption {
    pub name: String,
    pub statement: String,
    pub source: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AssumptionSet(Vec<Assumption>);

impl AssumptionSet {
    pub fn push(&mut self, assumption: Assumption) {
        self.0.push(assumption);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Assumption> {
        self.0.iter()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ObligationKind {
    FormalProof,
    Regularity,
    Existence,
    Uniqueness,
    Stability,
    Consistency,
    Convergence,
    QuadratureExactness,
    Orientation,
    Conservation,
    AdjointIdentity,
    FloatingPointError,
    ReferenceCrosscheck,
    ExperimentalAdequacy,
    ScopeTransport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Obligation {
    pub kind: ObligationKind,
    pub statement: String,
    pub source: Option<String>,
    pub discharged: bool,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ObligationSet(Vec<Obligation>);

impl ObligationSet {
    pub fn push(&mut self, obligation: Obligation) {
        self.0.push(obligation);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Obligation> {
        self.0.iter()
    }

    pub fn open(&self) -> impl Iterator<Item = &Obligation> {
        self.0.iter().filter(|o| !o.discharged)
    }

    pub fn all_discharged(&self) -> bool {
        self.0.iter().all(|o| o.discharged)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Provenance {
    pub producer: String,
    pub producer_version: String,
    pub source_locator: Option<String>,
    pub hash_algorithm: String,
    pub notes: Vec<String>,
}

impl Default for Provenance {
    fn default() -> Self {
        Self {
            producer: String::new(),
            producer_version: String::new(),
            source_locator: None,
            hash_algorithm: "sha256".into(),
            notes: Vec::new(),
        }
    }
}

/// Auditable receipt connecting two compiler/scientific artifacts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refinement {
    pub source_stage: Stage,
    pub target_stage: Stage,
    pub source_hash: ArtifactHash,
    pub target_hash: ArtifactHash,
    pub relation: RefinementRelation,
    pub source_scope: Scope,
    pub target_scope: Scope,
    pub assumptions: AssumptionSet,
    pub obligations: ObligationSet,
    pub evidence: EvidenceSet,
    pub provenance: Provenance,
}

impl Refinement {
    /// A scope change is never implicit. Callers can gate promotion on this helper and require
    /// a discharged `ScopeTransport` obligation whenever it returns true.
    pub fn changes_scope(&self) -> bool {
        self.source_scope != self.target_scope
    }

    pub fn has_open_obligations(&self) -> bool {
        !self.obligations.all_discharged()
    }
}
