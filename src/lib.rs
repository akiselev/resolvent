//! `resolvent` parses `.res` source and derives scientific compiler semantics.
//!
//! Source syntax is represented by [`scientific::ScientificModule`]; resolved scientific meaning
//! lives exclusively in the typed [`semantic::SemanticModule`] arena. Quantity values use
//! Quantitas directly, while numerical realization and solve strategy belong downstream.

#![forbid(unsafe_code)]

pub mod evidence;
pub mod formulation;
pub mod id;
pub mod kernel;
pub mod property_tensor;
pub mod scientific;
pub mod semantic;
pub mod source;
pub mod structural;

pub use evidence::{
    EmpiricalGrade, EvidenceArtifact, EvidenceAxis, EvidenceGrade, EvidenceItem, EvidenceProfile,
    FormalGrade, NumericalGrade, Obligation, ObligationStatus,
};
pub use formulation::{
    FormArgument, FormArgumentRole, FormCapture, FormCaptureRole, FormCompileError, FormReceipt,
    FormTransformation, VariationalForm, VariationalIntegral, compile_variational_form,
};
pub use id::{Digest, ObligationId};
pub use kernel::{
    InputEvaluation, KernelLoweringError, KernelLoweringMethod, KernelLoweringReceipt,
    LocalFactorizationReceipt, LocalFormProgram, LocalInput, LocalInputRole,
    LocalIterationContract, LocalOutput, LocalOutputRole, LocalTransformation, LoweredKernel,
    factor_local_integral, lower_local_program,
};
pub use property_tensor::SymmetricTensor2;
pub use scientific::{
    CouplingGraph, PropertyDefinition, ScientificError, ScientificModel, ScientificModule,
    TimeStateSemantics, canonicalize_authored_quantity, derive_coupling_graph,
    format_scientific_module, parse_scientific_module, parse_scientific_module_diagnostics,
    resolve_modules, semantic_digest, validate_quantities,
};
pub use semantic::{
    Axis, DeclarationId, DomainId, ExprId, Frame, RegionId, RegionKind, SemanticCompilation,
    SemanticDeclaration, SemanticDeclarationKind, SemanticDomain, SemanticExpr, SemanticExprKind,
    SemanticIntegral, SemanticMeasure, SemanticModel, SemanticModule, SemanticRegion, SemanticRole,
    SemanticShape, SemanticSymbol, SemanticType, SymbolId, compile_semantics, elaborate_module,
    semantic_arena_digest,
};
pub use source::{RelatedSpan, SourceDiagnostic, SourceSeverity, SourceSpan, Spanned};
pub use structural::scc::{Digraph, GraphError, Sccs, tarjan_scc};
pub use structural::{
    AliasAnalysis, AliasClass, Block, BlockKind, DerivativeVariable, DifferentiationStep,
    EquationDerivativeProfile, IncidenceSystem, IndexReductionPlan, Matching, Schedule,
    StructuralCompileError, StructuralError, analyze_aliases, compile_schedule,
    compile_schedule_without_tearing, derivative_profile, maximum_matching, pantelides_plan,
};
