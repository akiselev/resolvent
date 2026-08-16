//! `resolvent` — exact algebra and a symbolic scientific compiler.
//!
//! The public crate intentionally contains several semantic dialects rather than one giant
//! expression enum:
//!
//! `expr -> model/system -> form -> discrete -> operator -> executable`
//!
//! Not every model visits every dialect. A circuit or geometric constraint system may lower
//! from `model` directly to `operator`; a continuum model may visit `form` and `discrete`.
//! Every semantic transition is represented by [`RefinementRecord`], which carries the
//! claimed relation, scope change, assumptions, obligations, evidence and provenance.
//!
//! This crate does **not** own numerical solve policy (Solverang), machine scheduling/codegen
//! (Anvil), simulation orchestration (Sinbad), theorem proving (Lean/Ferris–Howard), theorem
//! mining (Lean Atlas), or scientific campaign authority (Pi Lab). It provides the common
//! mathematical objects those systems can relate without inventing parallel ASTs.

#![forbid(unsafe_code)]

pub mod compile;
pub mod context;
pub mod discrete;
pub mod evidence;
pub mod expr;
pub mod form;
pub mod id;
pub mod lean;
pub mod model;
pub mod operator;
pub mod p1;
pub mod refinement;
pub mod structural;
pub mod verify;

pub use compile::{
    CompileError, CompilerDiagnostic, DiagnosticSeverity, Dialect, LegalityTarget, Lowering,
    LoweringPass, PassContract, declare_refinement,
};
pub use context::Context;
pub use discrete::{
    BasisEvaluation, DiscreteInstruction, DiscreteOp, DiscreteProgram, DiscreteValueId,
    RestrictionDirection,
};
pub use evidence::{
    EmpiricalGrade, EvidenceArtifact, EvidenceAxis, EvidenceGrade, EvidenceItem, EvidenceProfile,
    FormalGrade, NumericalGrade, Obligation, ObligationStatus,
};
pub use expr::{ExprNode, ExprStore, ScalarLiteral, Symbol, SymbolRole, SymbolTable};
pub use form::{
    Continuity, Field, FieldRole, FormExpr, FormProgram, FunctionSpace, Integral, Measure,
    ValueShape,
};
pub use id::{
    Digest, DiscreteProgramId, ExprId, FieldId, FormId, ObligationId, ObservableId, OperatorId,
    RefinementId, SymbolId, SystemId,
};
pub use lean::{LeanBridgeError, LeanDeclaration, LeanExportManifest, ReificationReceipt};
pub use model::{
    Assumption, Equation, Event, Observable, PropertyContract, PropertyKind, ScientificSpec, Scope,
    System,
};
pub use operator::{
    DerivativeCapability, OperatorBlock, OperatorBlockKind, OperatorProgram, OperatorProperty,
    SparsityContract,
};
pub use p1::{
    BoundaryEdge, BoundaryFlux, CsrMatrix, DirichletBoundary, DofMap, EvolutionAssembly,
    EvolutionClass, MassAssembly, MassInput, MassLumping, P1DiscretizationRequest, P1Error,
    P1LoweringResult, P1Mesh, PiecewiseConstant, PiecewiseSource, Point2, ScalarEllipticAssembly,
    ScalarEllipticInput, Triangle, assemble_mass, assemble_scalar_elliptic, lower_p1,
};
pub use refinement::{
    ArtifactKind, ArtifactRef, RefinementError, RefinementProvenance, RefinementRecord,
    RefinementRelation, ScopeTransition,
};
pub use structural::scc::{Digraph, GraphError, Sccs, tarjan_scc};
pub use structural::{
    Block, BlockKind, IncidenceSystem, Matching, Schedule, StructuralCompileError, StructuralError,
    compile_schedule, compile_schedule_without_tearing, maximum_matching,
};
pub use verify::{CheckResult, CheckStatus, ValidationBundle, ValidationCheck, ValidationKind};

/// Wire-level schema family for the unified scientific compiler surface.
pub const SCIENTIFIC_SCHEMA_VERSION: &str = "resolvent-science/0.1";
