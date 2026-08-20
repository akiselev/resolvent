# Resolvent status

Updated: 2026-08-20

Branch: `master`

Milestone: compiler federation reset

## Role

Resolvent owns `.res` source and scientific/form semantics. It parses and resolves modules,
maintains one canonical model, performs semantic and structural analysis, derives variational and
local mathematical artifacts, and lowers local work into Malleus-owned structured kernel types.

Quantitas owns dimensions, quantity kinds, units, and registries. Malleus owns local kernel IR and
execution. Finitum owns concrete discretization/global operators. Krasis owns coupled runtime
state. Solverang owns numerical algorithms. Sinbad owns product orchestration.

## Implemented

- Recovering `.res` parser, canonical formatter, source spans, module resolution, and stable
  semantic digest.
- Canonical `ScientificModule`, `ScientificModel`, and `Expr` used by every active compiler pass.
- Direct Quantitas quantity/unit types and validation; the internal quantity crate is removed.
- Property tables/expressions, derivative contracts, constitutive semantics, coupling graphs,
  time/state semantics, and evidence profiles.
- Structural incidence, matching, SCC/BLT, tearing, alias, and DAE planning projected directly
  from `ScientificModel`.
- `VariationalForm` compilation for authored forms.
- Realization-neutral `LocalFormProgram` factorization and direct lowering into
  `malleus::StructuredKernel` for scalar pointwise arithmetic.
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
- `cargo test --all-targets` -- passed: 31 tests, 0 failed.
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
  `lower_local_program`.

## Next compiler work

1. Elaborate expression dimensions, shapes, field roles, and diagnostics across every declaration.
2. Derive variational forms from strong equations with integration-by-parts and boundary receipts.
3. Add indexed tensor/QFunction factorization and basis/transformation requirements.
4. Lower Poisson completely through Malleus and bind it to a Finitum realization.
5. Add primal/JVP/VJP/parameter kernel products and verify them independently.
