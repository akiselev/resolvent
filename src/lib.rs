//! Resolvent is a proof-producing symbolic scientific compiler and computer algebra system.
//!
//! The crate deliberately keeps several semantic dialects distinct. Algebraic expressions,
//! scientific models, variational forms, discrete programs, and executable operators are
//! different kinds of objects even though they share one provenance-bearing context.
//!
//! The load-bearing abstraction is [`Refinement`]: every lowering states what relation it
//! claims between source and target, which assumptions it introduced, which obligations are
//! still open, and what evidence warrants the claim.

#![forbid(unsafe_code)]

pub mod context;
pub mod evidence;
pub mod ids;
pub mod refinement;
pub mod spec;

pub use context::{
    Context, ContextError, EquationNode, ExprNode, FormNode, ModelNode, OperatorNode,
};
pub use evidence::{
    EmpiricalEvidenceGrade, Evidence, EvidenceAxis, EvidenceGrade, EvidenceSet,
    FormalEvidenceGrade, NumericalEvidenceGrade,
};
pub use ids::{
    ArtifactHash, EquationId, ExprId, FormId, ModelId, ObservableId, OperatorId, ScopeId,
};
pub use refinement::{
    Assumption, AssumptionSet, Obligation, ObligationKind, ObligationSet, Provenance, Refinement,
    RefinementIssue, RefinementRelation, Stage,
};
pub use spec::{
    BoundaryCondition, InitialCondition, Law, Observable, Parameter, ScientificSpec, Scope,
    StateVariable, ValidationContract,
};
