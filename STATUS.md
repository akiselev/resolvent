# Resolvent status

Updated: 2026-08-16
Branch: `agent/r13-r20-wave-a-f`
Milestone: Waves A-F / R13-R20

## Current role

Resolvent owns scientific authoring semantics, quantity/kind semantics, property and constitutive IR, variational/discrete meaning, coupling dependency semantics, and time/state meaning. It does not own finite-precision kernel compilation, global field runtime, or numerical solve strategy.

## Implemented on this branch

- R14a: scientific-v1 lexer/parser with modules/imports, source spans, typed declarations, Pratt expressions, error recovery, canonical formatting, and cycle detection.
- R14 corpus: 50 valid + 50 invalid generated modules, multi-diagnostic recovery, format idempotence, semantic-digest roundtrip stability, and coupling-order invariance.
- R15a: standalone `resolvent-quantities` with dimensions, quantity kinds, unit IDs, exact scales, affine point/interval handling, bounds, kind strictness, and offline deterministic SIRP snapshot tooling.
- R13: nonlinear transient heat manufactured case, explicit execution staging, and generic scalar-H1 weak lowering into mass/diffusion/pointwise terms without a named heat operator.
- R14b/c: typed domains/fields/parameters/sources/properties/equations/forms/conditions/observables plus scientific CLI parse/check/fmt/elaborate/freeze/coupling/plan surfaces.
- R15b/c: property signatures/models/tables, physical vs validity bounds, derivative contracts, evidence/uncertainty metadata, and symbolic derivatives.
- R16: stateless/stateful constitutive semantic contracts.
- R17: triangle/tetrahedron H1, L2, H(curl), H(div), P1/P2 and lowest-order catalog plus orientation semantics.
- R18: derived coupling graph, dependency paths, structurally nonzero block derivative map, and generic weak lowering of coupled scalar blocks.
- R20: differential/algebraic field roles, initial state, event/history schemas, and `F(t,y,ydot,p)=0` semantics.

## Validation state

Local rustup is blocked by sandbox DNS; GitHub-hosted Rust jobs are authoritative.

Rustfmt and the quantity lint fix are applied. A diagnostic run then exposed two Rust ownership errors in coupling-graph construction; both were fixed at the current code head (`3164ddafe53d5cb593661a9bdba9f3f579485a9e`) and temporary diagnostics were removed. This status commit retriggers the normal format/clippy/all-feature test workflow on that corrected tree.

## Cross-repository contract

Malleus and Sinbad must pin the exact passing Resolvent Wave commit; Sinbad's `scientific-stack.lock` records the final federation tuple.

## Remaining before merge

1. Confirm normal CI is green or fix any further compiler/test findings.
2. Pin the green revision into Malleus and Sinbad and rerun downstream CI.
3. Freeze the exact federation tuple and record final evidence here.
