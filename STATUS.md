# Resolvent status

Updated: 2026-08-20

Branch: `fc3-review-fixes`

Milestone: FC3 spaces, mappings, and preprocessing complete

## Role

Resolvent owns `.res` source and scientific/form semantics. It parses and resolves modules,
maintains one canonical model, performs semantic and structural analysis, derives variational and
local mathematical artifacts, and lowers local work into Malleus-owned structured kernel types.

Quantitas owns dimensions, quantity kinds, units, and registries. Malleus owns local kernel IR and
execution. Finitum owns concrete discretization/global operators. Krasis owns coupled runtime
state. Solverang owns numerical algorithms. Sinbad owns product orchestration.

## Implemented

- Recovering `.res` parser, canonical formatter, byte-precise expression/reference spans,
  deterministic module resolution, and presentation-invariant source digest.
- One typed `SemanticModel` arena with stable domain, region, symbol, expression, and declaration
  identities; resolved roles and references; shapes and axes; Quantitas dimensions/kinds/units;
  domain frames; typed declaration payloads; and presentation-invariant semantic digest. The
  typed-declaration wire shape is `resolvent-semantic/3`.
- Stable structured diagnostics with exact spans for malformed syntax, imports, domains, names,
  units, quantity kinds, roles, shapes, axes, dimensions, and frames.
- `compile_semantics` is the complete FC1 library boundary. CLI `check`, `inspect`, `freeze`, and
  all analysis/form commands require it; `elaborate` emits the typed arena directly.
- Direct Quantitas quantity/unit types and validation; no internal quantity representation exists.
- Property tables/expressions, derivative contracts, constitutive semantics, coupling graphs,
  time/state semantics, and evidence profiles.
- Structural incidence, matching, SCC/BLT, tearing, alias, and DAE planning projected directly
  from `ScientificModel`; incidence follows model-defined value, property, and constitutive chains
  to the same transitive field dependencies reported by coupling analysis.
- `VariationalForm` compilation consumes only `SemanticModule`; arguments, captures, measures, and
  integrands retain arena IDs and typed roles instead of source-name keys.
- Strong equations derive into residual forms with generated typed test arguments, physical-field
  captures, space-aware integration by parts, and stable capability refusals when test selection,
  boundary partitions, curl orientation, or Robin flux meaning is not available.
- Cell, exterior/interior-facet, interface, and point sides are explicit. Differential,
  contraction, tensor/facet trace, jump, average, normal-component, and conjugation operations are
  typed semantic nodes; ambiguous two-sided field access is rejected.
- Form receipts record residualization, test multiplication, integration by parts, boundary-data
  substitution, an explicit complex convention, typed boundary-partition/test-trace assumptions,
  and retained/substituted/eliminated boundary terms. Multiple flux terms cannot consume the same
  Neumann datum without explicit correspondence.
- The deterministic form interpreter evaluates scalar, complex, vector/tensor, differential,
  contraction, side/trace, indexing, and common scalar operations from caller-supplied point data.
  Caller-supplied weights are accumulated without taking ownership of quadrature traversal.
  `required_evaluations` exposes the expression/evaluation bindings needed by an integral.
- Realization-neutral `LocalFormProgram` factorization preserves test/trial/field/coefficient/value
  roles and required value/gradient/time-derivative/trace evaluations.
- Form, local-program, and kernel-lowering artifacts have schema-versioned, span-independent
  digests and receipts linking each artifact to its parent.
- Mesh-free `FormRequirements` inference covers H1, L2, product/mixed, Hcurl, Hdiv, DG, and trace
  spaces; abstract element family/order/shape; H1/L2/broken and Piola pullbacks; tangential,
  normal, and two-sided orientation; basis evaluation sites; geometry preprocessing; conservative
  quadrature intent; essential constraints; and exterior-boundary partition requirements.
- Requirement inference expands model-defined value/property/constitutive dependency chains to
  recover hidden physical-field spaces and derivative evaluations. Inputs explicitly distinguish
  basis-backed values, external values, model-defined values/properties, and model-defined
  constitutive preprocessing, so non-space symbols are never presented as basis data.
- Integral expression signatures are normalized without arena IDs or spans and grouped only when
  measure/domain/region, output type, inputs/evaluations, geometry, quadrature, and integrand all
  match. The mathematical requirement digest is invariant to integral, domain, and field
  declaration order, while its receipt retains the exact parent form digest.
- Stable `REQ_*` refusals cover non-scalar axes, cross-domain measures, invalid region kinds,
  incompatible space continuity/value shapes/differentials/normal traces, and incompatible
  essential boundary data.
- Scalar point-QFunction lowering returns Malleus IR with a lowering receipt. Finitum owns
  quadrature selection/traversal; the empty Malleus iteration domain means one point invocation.
- One `resolvent` CLI for check, format, parse, inspect, freeze, explain, coupling, structural
  analysis, forms, and requirements. Multi-model modules require explicit model selection;
  form/equation commands accept `Model:item` and model-wide commands accept `Model`.

## Removed

- The pre-form expression/context/system pipeline and its RSL, LaTeX, and Lean frontends.
- Form/discrete/operator/backend types that duplicated the new compiler direction.
- Reference FEM implementations and scientific bridge layers.
- Old comparison tooling, runtime plans, diagnostic logs, and the internal quantity crate.
- The exact-CAS roadmap/research/ADR corpus that no longer described this product.

Git history is the archive. None of the removed implementation is an acceptance oracle.

## Validation

Verified locally on 2026-08-20:

- `cargo fmt --all -- --check` -- passed.
- `cargo check --all-targets` -- passed.
- `cargo clippy --all-targets -- -D warnings` -- passed.
- `cargo test --all-targets` -- passed: 65 tests, 0 failed.
- `cargo doc --no-deps` -- passed.
- `cargo test --doc` -- passed.
- `cargo run --quiet --bin resolvent -- check` over all 50 Sinbad corpus models -- passed: 50 of
  50 parsed and elaborated.
- `derive-form` passed for Poisson, transient diffusion, nonlinear heat, linear elasticity, and
  both Stokes equations; the same set is covered by the FC2 integration gate.
- `derive-requirements` passed for Poisson, transient diffusion, nonlinear heat, linear
  elasticity, and both Stokes equations from the Sinbad corpus; the same set is covered by the FC3
  integration gate.
- Stokes momentum requirements include velocity H1/order-2 space and basis-backed
  `symmetric_gradient`, with viscosity/strain/stress typed as model-defined preprocessing.
- The electrothermal corpus model's coupling graph includes Joule-source and constitutive
  dependencies, and structural analysis returns a nonsingular coupled 2x2 schedule.
- A three-model CLI fixture passes qualified form, requirement, coupling, structural, and explain
  selection and refuses ambiguous unqualified item selection.
- Resolvent tests contain no compile-time or runtime path into Sinbad's product corpus; standalone
  validation uses only repository-local fixtures plus the declared Quantitas/Malleus dependencies.
- `cargo run --quiet --bin resolvent -- check examples/nonlinear_heat.res` -- passed.
- `cargo run --quiet --bin resolvent -- structural examples/nonlinear_heat.res` -- passed with
  one explicit structural block.

## Cross-repository contract

- Quantitas path: `../quantitas`, validated at
  `734d78cd6ff516afee54201bc70cd59fd34e67e3`; API types used directly include `Dimension`,
  `Quantity`, `QuantityLiteral`, `QuantityKindId`, `UnitId`, and `UnitRegistry`.
- Malleus path: `../malleus`, validated at
  `cd24813b29e5909a01b654477de99e9d4adde79b`; Resolvent constructs `StructuredKernel` and uses
  Malleus operands, indexing maps, scalar expressions, statements, and numeric policy directly.
- Public downstream sequence: `compile_variational_form`/`derive_variational_form` ->
  `infer_form_requirements` -> indexed tensor/QFunction work. The existing narrow scalar path
  remains `factor_local_integral` -> `lower_local_program`, with `LoweredKernel { kernel, receipt }`
  rather than a bare `StructuredKernel`.
- Finitum must map `LocalIterationContract::QuadraturePoint` across its selected element and
  quadrature points. Any later fixed-axis batching remains realization-owned and must preserve
  point-QFunction semantics; see `ITERATION-OWNERSHIP.md`.

## Next compiler work

1. Add indexed tensor/QFunction factorization and operator factorization from `FormRequirements`.
2. Lower Poisson completely through Malleus and bind it to a Finitum realization.
3. Add primal/JVP/VJP/parameter kernel products and verify them independently.
