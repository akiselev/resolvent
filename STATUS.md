# Resolvent status

Updated: 2026-08-20
Branch: `agent/fc0-fc1-v2-form-artifacts`
Milestone: Physics Factory FC0-FC1

## Current role

Resolvent owns scientific authoring semantics, formulation derivation, typed variational meaning,
structural coupling, differentiation contracts, generated obligations, and refinement receipts. It
does not own mesh realization, global assembly, local finite-precision code generation, or solver
strategy.

## Implemented on this branch

- Introduced the self-contained `resolvent-variational-form/2` and
  `resolvent-stage-artifact/2` schemas beside the V1 `FormProgram`/`WeakOperatorProgram` path.
- Added content digests, canonical serialization, round-trip verification, formulation receipts,
  provenance, artifact inspection, and stable diagnostic codes.
- Separated scientific coefficients from form arguments and made form arity, mixed argument
  number/part, block extraction, measures, trace sides, scalar kind, tensor axes, frames, and index
  sets explicit.
- Added distinct `dot`, sesquilinear `inner`, conjugation, transpose, Hermitian transpose, and typed
  contraction nodes with validation for frames, axes, scalar rank, and facet-side legality.
- Added the scalar-H1 V1-to-V2 compatibility adapter for mass, diffusion, pointwise/source, and
  coupled gradient-dependent terms. The V1 weak program is retained as an explicit digest-bound
  differential oracle rather than being silently re-derived downstream.
- Generated integration-by-parts boundary terms and compatibility assumptions are recorded in the
  derivation receipt.
- New artifacts make no derivative or operator-property claim unless an artifact and evidence are
  present. Assembly level is absent from form identity.
- Added unit gates for residual/objective/Jacobian arity, mixed block extraction, canonical digests,
  complex semantics, invalid contractions/frames/sides, evidence requirements, round trips, and
  structured unsupported-term failures.

## Validation state

Pending the branch CI tuple:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

## Cross-repository contract

Malleus must consume this exact branch revision only as a semantic audit boundary; local/tensor
kernel lowering remains FC4+. Sinbad must pin the passing Resolvent and Malleus revisions, retain
V1 scalar execution as the differential oracle, and keep assembly selection outside the form
digest.

## Next work

FC2 splits abstract space requirements from concrete element families and mapping/constraint
realization. FC3 adds preprocessing and quadrature planning without changing FC0-FC1 artifact
identity.
