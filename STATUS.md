# Resolvent status

Updated: 2026-08-16
Branch: `agent/r13-r20-wave-a-f`
Milestone: Waves A-F / R13-R20

## Current role

Resolvent owns scientific authoring semantics, quantity/kind semantics, property and constitutive IR, variational/discrete meaning, coupling dependency semantics, and time/state meaning. It does not own finite-precision kernel compilation, global field runtime, or numerical solve strategy.

## Implemented on this branch

- R13: canonical nonlinear transient heat MMS plus generic scalar-H1 weak lowering into mass/diffusion/pointwise terms, with an explicit execution plan and no named heat opcode.
- R14: scientific-v1 parser/IR, source spans including weak-form integrals, recovery, canonical formatting, semantic digest independent of spans/whitespace and declaration ordering, and CLI `check|fmt|parse|elaborate|inspect|freeze|explain|coupling|plan` surfaces.
- R14 acceptance: generated valid/invalid corpus, multi-error recovery, formatting idempotence/semantic preservation, nonempty declaration spans, and ordering-invariant semantics.
- R15: `resolvent-quantities` dimension/kind/unit semantics, affine point-vs-interval handling, exact scales, offline SIRP snapshot tooling, scalar/table property IR, bounds/validity/evidence/uncertainty/derivative contracts, and 2-D symmetric tensor frame transforms.
- R16: stateless/stateful constitutive semantic contracts and standard law identities.
- R17: H1/L2/H(curl)/H(div) element semantics plus executable P1 scalar/vector, mixed Stokes, Nedelec H(curl), and Raviart-Thomas H(div) reference paths with orientation tests.
- R18: recursively derived coupling through nested properties and constitutive aliases, form/condition dependencies, explanation paths, and structurally nonzero Jacobian-block structure; declaration reordering is tested to preserve the graph.
- R20: differential/algebraic field roles, initial-state semantics, event/history schemas, and canonical `F(t,y,ydot,p)=0` representation.

## Validation state

Local rustup is blocked by sandbox DNS; GitHub-hosted Rust jobs are authoritative. This user-authored status update retriggers the normal format/clippy/all-feature workflow on the current R14/R15/R17/R18 implementation after bot-applied migrations.

## Cross-repository contract

Malleus and Sinbad must pin the exact passing Resolvent Wave commit. Sinbad's `scientific-stack.lock` records the final passing federation tuple.

## Remaining before merge

1. Resolve any normal-CI findings from the new acceptance tests.
2. Synchronize Malleus/Sinbad to the final green Resolvent revision.
3. Close remaining roadmap-level gaps called out by the cross-repo R13-R20 audit, then freeze the federation tuple.
