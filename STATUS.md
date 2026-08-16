# Resolvent status

Updated: 2026-08-16
Branch: `agent/r13-r20-wave-a-f`
Milestone: Waves A-F / R13-R20

## Current role

Resolvent owns scientific authoring semantics, quantity/kind semantics, property and constitutive IR, variational/discrete meaning, coupling dependency semantics, and time/state meaning. It does not own finite-precision kernel compilation, global field runtime, or numerical solve strategy.

## Implemented on this branch

- R14a: structured scientific-v1 lexer/parser with modules/imports, source spans, typed model declarations, Pratt expressions, canonical formatting, and module-cycle detection.
- R15a: standalone `resolvent-quantities` crate with SI dimension vectors, quantity kinds, unit IDs, exact decimal/rational scales, affine point/interval handling, standard registry metadata, bounds, and kind strictness.
- R15a: deterministic offline `tools/update-sirp` snapshot normalizer.
- R13: canonical nonlinear transient heat manufactured case and explicit execution-stage plan.
- R14b/c: typed domains/fields/parameters/sources/properties/equations/forms/conditions/observables plus `resolvent-science` parse/check/fmt/elaborate/freeze/coupling/plan CLI.
- R15b/c: property signatures/models, 1-D and 2-D tables, physical vs validity bounds, derivative contracts, evidence and uncertainty metadata, symbolic expression derivatives.
- R16: stateless/stateful constitutive semantic contracts and standard law catalog identities.
- R17: production discretization catalog for triangle/tetrahedron H1, L2, H(curl), H(div), P1/P2 and lowest-order compatible elements, plus orientation parity support.
- R18: derived coupling graph, dependency explanations/paths, and structurally nonzero block derivative map.
- R20: differential/algebraic field roles, initial-state semantics, event/history schema types, and canonical `F(t,y,ydot,p)=0` representation.

## Validation state

Local Rust validation is unavailable in the execution sandbox because outbound DNS/network access prevents rustup from downloading a toolchain. The branch therefore requires GitHub Actions (`cargo fmt`, clippy with warnings denied, and `cargo test --all-features`) before it can be considered verified.

Do not treat the implementation as verified until CI is green. Any CI failures should be fixed on this branch and reflected here.

## Cross-repository contract

Downstream Wave A-F branches in Malleus, Solverang, and Sinbad must consume the exact Resolvent commit selected after this branch passes CI. Sinbad's `scientific-stack.lock` is the federation-level record.

## Next

1. Run/fix GitHub Actions on this branch.
2. Pin the passing commit in Malleus and Sinbad.
3. Validate Malleus kernel bundles, Solverang block/time contracts, then the stacked Sinbad runtime integration.
