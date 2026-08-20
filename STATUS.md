# Resolvent status

Updated: 2026-08-20
Branch: `agent/fc0-fc1-form-v2`
Milestone: FC0–FC1 form-compiler V2 foundation

## Current role

Resolvent owns scientific authoring semantics, quantity/kind semantics, property and
constitutive IR, formulation and variational meaning, coupling dependency semantics,
differentiation, assumptions, obligations, and refinement evidence. It does not own
finite-precision kernel scheduling, global field runtime, or numerical solve strategy.

## Implemented on this branch

- FC0 artifact infrastructure: schema-versioned, content-addressed V2 envelopes; stable
  artifact IDs; explicit compiler stages; JSON round trips; verification; inspection;
  diagnostics; and refinement receipts.
- FC0 truthful capability boundary: V2 forms carry no derivative or operator-property
  claim without a referenced evidence artifact. Unsupported scalar-H1 input fails with
  structured diagnostics rather than executable custom nodes.
- FC0 scalar compatibility adapter: existing mass, diffusion, and pointwise weak programs
  are embedded losslessly beside V1 with a scientific-to-form receipt and explicit
  deferred boundary-term obligations. Assembly level is absent from V2 form identity.
- FC1 `VariationalFormV2`: physical coefficients are separate from numbered/partitioned
  form arguments; arity and mixed block extraction are explicit; cell, exterior-facet,
  interior-facet, and interface measures carry explicit sides.
- FC1 value semantics: real/complex scalar kinds, bilinear dot versus sesquilinear inner,
  conjugation, transpose/Hermitian adjoints, typed tensor axes, frames, variance, and
  checked contractions.
- FC1 formulation provenance: integration-by-parts steps and generated boundary terms are
  explicit and must be reconciled rather than discarded.

## Validation gate

The branch must pass the repository's required commands before its commit is consumed by
Malleus:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```

Unit gates cover deterministic artifact serialization/digests, lossless scalar-H1
compatibility, absent unevidenced claims, invalid frame and variance contractions, mixed
block digest stability, explicit facet-side rejection, and complex Hermitian semantics.
Exact CI run results and the passing commit SHA are recorded in the coordinated Sinbad
integration PR.

## Cross-repository contract

Malleus must pin the exact passing commit from this branch and admit only a post-form
TensorIR/QFunction boundary; it must not compile `VariationalFormV2` directly. Sinbad must
pin that Malleus commit and exercise the compatibility bundle beside the V1 scalar oracle.

## Next work

FC2 separates space requirements from concrete element realization and adds reference-cell,
pullback, interpolation, transformation, orientation, and constraint contracts. V1 remains
the differential oracle until later gates explicitly retire it.
