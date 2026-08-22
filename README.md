# Resolvent

Resolvent is the consumer-neutral algebraic CAS used directly by Scientia and
CADabra. It owns exact rational expressions, scalar and dual-number vocabulary,
bounded deterministic canonicalization, exact symbolic differentiation,
interval and expansion filters, univariate polynomial arithmetic and
resultants, certified real-root isolation, radical and lazy-exact numbers,
exact matrices, exact sign queries, and algebra receipts.

It does not own `.res`, scientific fields/forms/methods, geometry/topology,
meshes, runtime state, numerical solver policy, or constraint-system semantics.
The FC0-FC11 scientific compiler moved to the standalone Scientia repository
during R1; Git history preserves its earlier residence here.

The current crate is intentionally one cohesive package. Split packages only
when real consumers require independently versioned capability boundaries.
The R1 consolidation moved CADabra's former generic exact/scalar implementation
and tests here and deleted the old crates. There is no adapter, compatibility
facade, or second algebra backend.

## Resolvent Vision

[`PLAN.md`](PLAN.md) defines the proposed RV0-RV9 program to evolve this exact
algebra substrate into a full embeddable CAS with:

- a caller-owned immutable symbolic term kernel and stable wire identity;
- explicit mathematical domains, capabilities and canonical coercions;
- deterministic algorithm planning, budgets, receipts and certificates;
- assumptions, definitions, patterns and bounded rewriting;
- consumer-critical algebra for CADabra and Scientia/Sinbad;
- broad exact algebra, equation solving, calculus, special functions and
  certified scalar numerics;
- CLI/REPL, a transport-neutral kernel protocol, Jupyter and a native notebook;
- packages, optional providers, broad bindings, content-addressed caching and
  agent-facing structured APIs.

The roadmap keeps federation ownership explicit. Methodus owns generic numerical
methods. Solverang owns generic constraint solving and its reusable 2D/3D
constraint vocabulary, using Methodus for numerical algorithms. Resolvent may
supply reusable algebra to either but does not absorb their solver semantics.

The first RV1 slice is public as `TermStore`: a caller-owned, structurally
hash-consed arena whose local `TermId` handles never serve as persistent
identity. `canonical_bytes` and `TermDigest` preserve retained syntax exactly,
including child order and nesting; they perform no algebraic simplification.
Exact decimal, exact IEEE-bit ingress, machine-float, and precision-bearing
atoms remain distinct. Stores support bounded iterative traversal, explicit
cross-store import, construction-time node/depth/width budgets, and fail-closed
canonical decoding. Store metrics use a fixed portable logical-schema formula,
not Rust layout or allocator capacity.

Notebook lifetime is explicit epoch rebuild: `rebuild_roots` copies only
selected reachable DAGs into a fresh store, after which callers may discard the
old epoch. Old handles remain tied to the old store and are foreign in the new
one. Structural queries expose ordered children/heads, DAG statistics, free
symbols, and de Bruijn closure requirements. Substitution and exact-path
replacement accept only closed replacement terms, preventing binder capture.
Optional `OriginMap` sidecars attach multiple generic authored/generated
origins to stable digests without entering canonical Term identity. Batch root,
substitution, and path requests are bounded before allocation and share one DAG
budget; mutating replacements preflight completely before committing.
`OriginBudget` independently caps per-term/total records, retained text, and
attachment work, with hash-indexed deduplication and atomic refusal.

See [`docs/resolvent-vision/README.md`](docs/resolvent-vision/README.md) for the
architecture and phase execution model, and [`STATUS.md`](STATUS.md) for landed
implementation truth.
