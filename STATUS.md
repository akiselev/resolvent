# Resolvent status

**Updated:** 2026-08-21
**Landed milestone:** RV0 exact-foundation stabilization
**Active planning frontier:** RV1 immutable structural Term kernel.

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

## RV0 readiness freeze

- the complete public exact/scalar surface and panic boundaries are classified
  in `docs/RV0-PUBLIC-API-CENSUS.md`;
- exact versus approximate/enclosure/dual capabilities are frozen in
  `docs/RV0-CAPABILITY-TABLE.md`;
- rational, `QPoly`, rational-matrix, root-certificate, and receipt wire
  identities are Resolvent-owned and covered by golden/negative decode tests;
  receipt v1 explicitly preserves its original polynomial digest projection;
- checked ingress and matrix APIs fail closed for invalid untrusted data, and
  exact expression division by zero is typed;
- deterministic budgets charge live polynomial division, matrix
  multiplication/determinant/RREF, certificate Horner/affine restoration,
  expression/minor/Euclidean work, degree, intermediate coefficient bits,
  dimensions, root bisections/refinement, and lazy DAG forcing;
- root isolation and PolyMat invariant operations share one aggregate meter
  across all nested algebra and refinement stages;
- executable ownership checks reject scientific, CAD topology, Methodus,
  Solverang, Malleus, and deleted-facade vocabulary in production code;
- benchmark/stress locations and exact federation starting commits are recorded
  in `docs/RV0-BASELINES.md` and `docs/RV0-FEDERATION-BASELINE.md`.

RV1 may now introduce caller-owned structural Term identity. It must not reuse
the existing `Expr` or lazy `Real` DAG as that identity, and it does not reopen
numeric ownership.

## R1 consolidation

CADabra is the second direct consumer. Its generic scalar/dual, rational,
interval/filter, polynomial/root, radical, lazy-real, Bernstein, and
exact-matrix machinery now lives here. CADabra consumes Resolvent directly;
the replaced crates and the stale unification experiment were deleted without
an adapter, compatibility facade, or parallel backend. Geometry policy remains
in CADabra and scientific meaning remains in Scientia.

## Validation

Passed locally on 2026-08-21 after RV0 hardening:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets             # 129 tests
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
cargo test --locked --workspace --doc                     # 2 doctests
./scripts/check-ownership.sh
cargo bench --locked --bench ladder                       # observational timing only
git diff --check
```

All tests/doctests transferred from the former CADabra algebra crates remain
present alongside the RV0 wire, fallibility, ownership, budget, and stress
contracts. Benchmark timing remains observational rather than a correctness
gate.
