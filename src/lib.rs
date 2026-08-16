//! `resolvent` — exact algebra and a symbolic scientific compiler.
//!
//! Public semantic progression:
//! `RSL/Lean -> expr/system -> form -> discrete -> operator -> execution plan`.
//! Source languages are intentionally outside the trusted semantic core; every meaningful
//! transition can carry a refinement receipt and every execution artifact is tied to the
//! frozen context that gives its handles meaning.

#![forbid(unsafe_code)]

pub mod backend;
pub mod calculus;
pub mod compile;
pub mod context;
pub mod discrete;
pub mod evidence;
pub mod expr;
pub mod field;
pub mod form;
pub mod form_compile;
pub mod generated_verify;
pub mod id;
pub mod latex;
pub mod lean;
pub mod migration;
pub mod model;
pub mod operator;
pub mod physics;
pub mod reference;
pub mod reference_mixed;
pub mod refinement;
pub mod rsl;
pub mod scientific;
pub mod source;
pub mod structural;
pub mod units;
pub mod verify;

pub use backend::{
    BackendCapabilities, BackendError, ExecutionBackend, ExecutionPlan, ExecutionPolicy,
    IdentityBackend, Realization, build_execution_plan,
};
pub use calculus::{CalculusError, differentiate, evaluate_f64};
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
pub use field::{
    BoundaryRef, Continuity, DomainRef, ElementFamily, Field, FieldRole, FunctionSpace,
    InterfaceRef, ValueShape,
};
pub use form::{
    EssentialBoundary, FormExpr, FormProgram, Integral, Measure, NaturalBoundary, RobinBoundary,
};
pub use form_compile::{
    CompiledForm, FormCompileError, FormCompileOptions, RecognizedFormTerm, compile_form,
};
pub use generated_verify::{
    ConvergenceObservation, DerivativeGate, GeneratedVerifyError, ManufacturedSolution,
    adjoint_dot_gate, finite_difference_gate, infer_dimension, manufacture_forcing,
    observed_orders, substitute,
};
pub use id::{
    Digest, DiscreteProgramId, ExprId, FieldId, FormId, ObligationId, ObservableId, OperatorId,
    RefinementId, SymbolId, SystemId,
};
pub use latex::{MathExpr, parse_scientific_latex};
pub use lean::{LeanBridgeError, LeanDeclaration, LeanExportManifest, ReificationReceipt};
pub use migration::{DifferentialMismatch, MigrationCase, NumericTolerance, compare_json};
pub use model::{
    Assumption, Equation, Event, Observable, PropertyContract, PropertyKind, ScientificSpec, Scope,
    System,
};
pub use operator::{
    DerivativeCapability, OperatorBlock, OperatorBlockKind, OperatorProgram, OperatorProperty,
    SparsityContract,
};
pub use physics::{
    PhysicsError, PhysicsLock, freeze as freeze_physics, parse_and_elaborate, validate_lock,
};
pub use reference::{
    BoundaryEdge, DofLayout, PiecewiseConstant, ReferenceError, ReferenceMesh2, ReferenceOperator,
    RobinData, ScalarH1Problem, SparseMatrix, compile_scalar_h1_p1,
};
pub use reference_mixed::{
    ElasticityOperator2, IsotropicElasticity2, NedelecOperator2, StokesOperatorP1P1,
    compile_elasticity_p1_2d, compile_nedelec0_2d, compile_stokes_p1p1_2d,
};
pub use refinement::{
    ArtifactKind, ArtifactRef, RefinementError, RefinementProvenance, RefinementRecord,
    RefinementRelation, ScopeTransition,
};
pub use rsl::{ElaboratedRsl, RslFieldDecl, RslModel, RslSymbolDecl, parse_rsl};
pub use scientific::{
    CanonicalHeatCase, CouplingGraph, DiscretizationCatalog, PropertyDefinition,
    ScientificExecutionPlan, ScientificModel, ScientificModule, TimeStateSemantics,
    derive_coupling_graph, execution_plan, format_scientific_module, parse_scientific_module,
    resolve_modules, semantic_digest,
};
pub use source::{RelatedSpan, SourceDiagnostic, SourceSeverity, SourceSpan, Spanned};
pub use structural::scc::{Digraph, GraphError, Sccs, tarjan_scc};
pub use structural::{
    AliasAnalysis, AliasClass, Block, BlockKind, DerivativeVariable, DifferentiationStep,
    EquationDerivativeProfile, IncidenceSystem, IndexReductionPlan, Matching, Schedule,
    StructuralCompileError, StructuralError, analyze_aliases, compile_schedule,
    compile_schedule_without_tearing, derivative_profile, maximum_matching, pantelides_plan,
};
pub use units::{Dimension, UnitError};
pub use verify::{CheckResult, CheckStatus, ValidationBundle, ValidationCheck, ValidationKind};

pub use resolvent_quantities as quantities;

/// Wire-level schema family for the unified scientific compiler surface.
pub const SCIENTIFIC_SCHEMA_VERSION: &str = "resolvent-science/0.3";
