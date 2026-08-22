# Resolvent status

**Updated:** 2026-08-21
**Landed milestone:** RV1-A1/A2/A3 contract freeze and first RV1-B1 Term-store slice
**Active planning frontier:** RV1-B2 lifetime policy and RV1-B3 structural queries.

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

## RV1 structural Term kernel

- A1 freezes distinct canonical atoms for integers, rationals, exact decimals,
  exact IEEE-754 bit-pattern ingress, machine floats, precision-bearing reals,
  strings, bytes, namespaced symbols, booleans, constants, and bound variables;
- A2 freezes an ordered consumer-neutral node vocabulary for application,
  relations, booleans, conditions, piecewise values, collections, ordered maps,
  indexing/slicing, rules, binders, and held syntax. Binders use de Bruijn
  variables; open fragments may exist in a store, but stable wire roots must be
  closed;
- A3 freezes the `RESOLVENT-TERM` schema version 1 tagged DAG encoding. It uses
  deterministic child-first order, canonical unsigned varints and explicit
  tags, preserving retained syntax without Serde or algebraic normalization;
- the first B1 slice provides a caller-owned strongly retaining `TermStore`,
  local cross-store-safe handles, structural hash-consing, iterative bounded
  traversal, stable BLAKE3 digests, deterministic logical metrics, explicit
  import, and fail-closed canonical decode;
- node/depth/width budgets apply during construction as well as traversal and
  decode. Hash-cons hits are resolved before the node cap, and import preflights
  only genuinely new nodes so all-duplicate and partially shared DAGs work at
  exact capacity without partial mutation;
- logical byte accounting is a portable `u64` schema formula: one-byte tags,
  eight-byte lengths and term references, four-byte interned symbol references,
  fixed-width scalar fields, and length-prefixed payloads. Unique symbol names
  are charged once in the symbol table; Rust layout and allocator overhead are
  excluded;
- construction is mutable through `&mut TermStore`; immutable shared stores are
  thread-safe for concurrent reads. Weak retention, epochs, compaction,
  binder-safe substitution, free-symbol queries, provenance sidecars,
  renderers, and Scientia projection remain later RV1 work.

## R1 consolidation

CADabra is the second direct consumer. Its generic scalar/dual, rational,
interval/filter, polynomial/root, radical, lazy-real, Bernstein, and
exact-matrix machinery now lives here. CADabra consumes Resolvent directly;
the replaced crates and the stale unification experiment were deleted without
an adapter, compatibility facade, or parallel backend. Geometry policy remains
in CADabra and scientific meaning remains in Scientia.

## Validation

Passed locally on 2026-08-21 after the RV1-A/B1 slice:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets             # 142 tests
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
cargo test --locked --workspace --doc                     # 2 doctests
./scripts/check-ownership.sh
cargo bench --locked --bench ladder                       # observational timing only
git diff --check
```

All tests/doctests transferred from the former CADabra algebra crates remain
present alongside the RV0 wire, fallibility, ownership, budget, and stress
contracts. RV1 adds frozen atom and exhaustive node/subtag byte-and-digest wire
vectors plus hostile structural-identity, insertion-permutation, binder,
deep-DAG, cross-store, exact-capacity, accounting, budget, and decoder tests.
Benchmark timing remains observational rather than a correctness gate.
