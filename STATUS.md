# Resolvent status

Updated: 2026-08-20

Branch: `master`

Milestone: FC2 formulation and typed forms complete

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
  from `ScientificModel`.
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
- Scalar point-QFunction lowering returns Malleus IR with a lowering receipt. Finitum owns
  quadrature selection/traversal; the empty Malleus iteration domain means one point invocation.
- One `resolvent` CLI for check, format, parse, inspect, freeze, explain, coupling, structural
  analysis, and form output.

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
- `cargo test --all-targets` -- passed: 52 tests, 0 failed.
- `cargo doc --no-deps` -- passed.
- `cargo run --quiet --bin resolvent -- check` over all 50 Sinbad corpus models -- passed: 50 of
  50 parsed and elaborated.
- `derive-form` passed for Poisson, transient diffusion, nonlinear heat, linear elasticity, and
  both Stokes equations; the same set is covered by the FC2 integration gate.
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
- Public downstream sequence: `compile_variational_form` -> `factor_local_integral` ->
  `lower_local_program`. The first function now requires a `SemanticModule`; the last returns
  `LoweredKernel { kernel, receipt }` rather than a bare `StructuredKernel`.
- Finitum must map `LocalIterationContract::QuadraturePoint` across its selected element and
  quadrature points. Any later fixed-axis batching remains realization-owned and must preserve
  point-QFunction semantics; see `ITERATION-OWNERSHIP.md`.

## Next compiler work

1. Add indexed tensor/QFunction factorization and basis/transformation requirements.
2. Lower Poisson completely through Malleus and bind it to a Finitum realization.
3. Add primal/JVP/VJP/parameter kernel products and verify them independently.
