//! `resolvent` — exact algebra and a symbolic scientific compiler.
//!
//! The public crate intentionally contains several semantic dialects rather than one giant
//! expression enum. The RSL authoring surface, Rust macros and Lean bridge all elaborate into
//! the same caller-owned semantic context.

#![forbid(unsafe_code)]

extern crate self as resolvent;

pub mod author;
pub mod calculus;
pub mod compile;
pub mod context;
pub mod diagnostic;
pub mod discrete;
pub mod evidence;
pub mod expr;
pub mod form;
pub mod form_diff;
pub mod freeze;
pub mod generated_verify;
pub mod id;
pub mod latex;
pub mod lean;
pub mod migration;
pub mod model;
pub mod operator;
pub mod reference;
pub mod refinement;
pub mod semantic_check;
pub mod structural;
pub mod units;
pub mod verify;

pub use resolvent_macros::{include_physics, physics};

pub use compile::{
    CompileError, CompilerDiagnostic, DiagnosticSeverity as CompileDiagnosticSeverity, Dialect,
    LegalityTarget, Lowering, LoweringPass, PassContract, declare_refinement,
};
pub use context::Context;
pub use diagnostic::{Diagnostic, DiagnosticSeverity, SourceLabel, SourceSpan, SuggestedFix};
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
pub use freeze::SemanticLock;
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
pub use reference::{
    AssembledMass, AssembledScalarOperator, BoundaryEdge, DirichletValue, DofLayout,
    EvolutionOperator, NeumannValue, PiecewiseConstant, ReferenceError, ScalarEllipticProblem2d,
    SparseEntry, SparseMatrix, TetCell, TetMesh, TriangleCell, TriangleMesh, assemble_mass_p1,
    assemble_scalar_elliptic_p1, assemble_tet_diffusion_p1,
};
pub use refinement::{
    ArtifactKind, ArtifactRef, RefinementError, RefinementProvenance, RefinementRecord,
    RefinementRelation, ScopeTransition,
};
pub use structural::dae::{
    AliasGroup, DerivativeVariable, DummyDerivative, PantelidesStep, StructuralDaeAnalysis,
    analyze_dae,
};
pub use structural::scc::{Digraph, GraphError, Sccs, tarjan_scc};
pub use structural::{
    Block, BlockKind, IncidenceSystem, Matching, Schedule, StructuralCompileError, StructuralError,
    compile_schedule, compile_schedule_without_tearing, maximum_matching,
};
pub use units::{Dimension, UnitError, UnitExpr, parse_unit};
pub use verify::{CheckResult, CheckStatus, ValidationBundle, ValidationCheck, ValidationKind};

pub const SCIENTIFIC_SCHEMA_VERSION: &str = "resolvent-science/0.2";
