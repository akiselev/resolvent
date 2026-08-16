# Resolvent status

Updated: 2026-08-16
Branch: `agent/r13-r20-wave-a-f`
Milestone: Waves A-F / R13-R20

## Current role

Resolvent owns scientific authoring semantics, quantity/kind semantics, property and constitutive IR, variational/discrete meaning, coupling dependency semantics, and time/state meaning. It does not own finite-precision kernel compilation, global field runtime, or numerical solve strategy.

## Implemented on this branch

- R14a: structured scientific-v1 lexer/parser with modules/imports, source spans, typed model declarations, Pratt expressions, error synchronization, canonical formatting, and module-cycle detection.
- R14 acceptance corpus: 50 valid and 50 invalid generated modules, multiple-diagnostic recovery, formatting idempotence, semantic-digest stability, and declaration-order coupling invariance.
- R15a: standalone `resolvent-quantities` crate with SI dimension vectors, quantity kinds, unit IDs, exact decimal/rational scales, affine point/interval handling, standard registry metadata, bounds, and kind strictness.
- R15a: deterministic offline `tools/update-sirp` snapshot normalizer.
- R13: canonical nonlinear transient heat manufactured case, explicit execution-stage plan, and generic scalar-H1 weak lowering. `scientific_weak` expands properties/constitutive aliases and lowers mathematical `dt`, `div(coefficient*grad(field))`, and pointwise residual structure to mass/diffusion/pointwise weak terms without a named heat operator.
- R14b/c: typed domains/fields/parameters/sources/properties/equations/forms/conditions/observables plus `resolvent-science` parse/check/fmt/elaborate/freeze/coupling/plan CLI.
- R15b/c: property signatures/models, 1-D and 2-D tables, physical vs validity bounds, derivative contracts, evidence and uncertainty metadata, and symbolic expression derivatives.
- R16: stateless/stateful constitutive semantic contracts and standard law catalog identities.
- R17: triangle/tetrahedron H1, L2, H(curl), H(div), P1/P2 and lowest-order compatible element catalog plus orientation semantics.
- R18: derived coupling graph, dependency explanation paths, structurally nonzero block derivative map, and generic weak lowering of coupled scalar blocks.
- R20: differential/algebraic field roles, initial-state semantics, event/history schema types, and canonical `F(t,y,ydot,p)=0` representation.

## Validation state

Local Rust validation is unavailable because the execution sandbox cannot resolve the rustup download host. GitHub-hosted runners install Rust successfully and are the validation authority.

Mechanical rustfmt and the quantity canonicalization clippy fix were applied at `6924144570bd528855eb462d8272036e5923954e`. GitHub marks bot-authored follow-up checks `action_required`, so this user-authored status commit retriggers the normal format/clippy/all-feature test workflow on the corrected tree. The implementation remains unverified until that run is green.

## Cross-repository contract

Malleus and Sinbad must pin the exact passing Resolvent Wave commit. Sinbad's `scientific-stack.lock` records the final federation tuple.

## Remaining before merge

1. Fix any remaining normal-CI compile/clippy/test failures.
2. Pin the green Resolvent revision into Malleus and Sinbad.
3. Re-run downstream CI, then freeze the federation tuple.
4. Update this file with the exact green revision and compact evidence summary.
