# Resolvent status

**Updated:** 2026-08-21
**Landed milestone:** R1 shared exact-algebra consolidation
**Active planning frontier:** RV0 exact-foundation stabilization; RV1 design may
overlap, but one-way public Term identity changes wait for RV0-E2.

## Ownership

Resolvent owns consumer-neutral exact algebra. The `.res` scientific compiler
has moved to Scientia; no compatibility facade remains here.

## Implemented

- stable, explicitly serialized arbitrary-precision rationals;
- shared scalar, approximate-scalar, uncertainty, and dual-number vocabulary;
- intervals, error-free transforms, expansions, filters, and exactness metrics;
- bounded deterministic canonicalization;
- exact symbolic differentiation for arithmetic and common scalar functions;
- exact evaluation and sign queries for decidable rational expressions;
- dense univariate polynomial arithmetic, division, gcd, and derivatives;
- exact Sylvester resultants with a matrix-dimension budget;
- Descartes/VCA real-root isolation with an explicit bisection budget and
  validated immutable root certificates;
- lazy exact reals, square-root extensions, Bernstein polynomials, and exact
  rational and polynomial matrices;
- deterministic algebra-operation receipts.

## R1 consolidation

CADabra is the second direct consumer. Its generic scalar/dual, rational,
interval/filter, polynomial/root, radical, lazy-real, Bernstein, and
exact-matrix machinery now lives here. CADabra consumes Resolvent directly;
the replaced crates and the stale unification experiment were deleted without
an adapter, compatibility facade, or parallel backend. Geometry policy remains
in CADabra and scientific meaning remains in Scientia.

## Validation

Passed locally on 2026-08-21:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets             # 117 tests
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
cargo test --locked --workspace --doc                     # 2 doctests
cargo bench --locked --bench ladder                       # determinant 7.9x
git diff --check
```

All 111 tests/doctests transferred from the two former CADabra algebra crates
remain present alongside Resolvent's existing and new certificate/budget tests.
