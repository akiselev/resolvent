# RV0 - Exact Foundation Stabilization

## Status

R1 shared exact-algebra consolidation is already landed. Resolvent now owns the former CADabra generic scalar/dual, rational, interval/filter, root, radical, lazy-real, Bernstein and exact-matrix machinery; CADabra consumes Resolvent directly and the duplicate crates are gone.

RV0 is therefore a short post-consolidation stabilization phase, not a migration phase.

## Goal

Freeze the public exact/scalar invariants that RV1-RV9 will rely on, close residual resource/error/serialization gaps, and establish repeatable cross-consumer/performance baselines before the general term/domain redesign begins.

RV0 must not become an artificial blocker for CADabra R2. CADabra may proceed on the landed R1 substrate. Coordination is required only when an RV0 fix changes an exact decision or public algebra contract used by CADabra.

## Landed starting point

Current `STATUS.md` records:

- stable explicitly serialized arbitrary-precision rationals;
- shared scalar, approximate-scalar, uncertainty and dual-number vocabulary;
- intervals, error-free transforms, expansions, filters and exactness metrics;
- bounded deterministic canonicalization;
- exact symbolic differentiation for current arithmetic/common scalar functions;
- exact rational expression evaluation/sign queries;
- dense univariate polynomial arithmetic, division, gcd and derivatives;
- exact Sylvester resultants;
- Descartes/VCA root isolation with explicit budgets and immutable root certificates;
- lazy exact reals, square-root extensions, Bernstein polynomials and exact rational/polynomial matrices;
- deterministic algebra receipts;
- 117 tests plus 2 doctests;
- the migrated 111 CADabra algebra tests/doctests;
- filtered determinant benchmark around the current 7.9x reference.

RV0 treats that state as a baseline to harden, not work to repeat.

## Work packages

### RV0-A1 - Public API and invariant census

Document the exact/scalar public surface and classify every exported item by role:

- exact primitive/value;
- approximate/enclosure value;
- uncertainty/decision value;
- scalar-kernel trait;
- polynomial/root/matrix algebra;
- lazy exact runtime value;
- serialization artifact;
- budget/error/evidence type.

For each exported type/function, record:

- mathematical invariant;
- exactness/approximation contract;
- panic/error preconditions;
- serialization status;
- whether the type is safe to reference from long-lived RV1/RV2 public APIs;
- current Scientia/CADabra consumers.

Exit: no public item has ambiguous ownership or an undocumented exactness contract.

### RV0-A2 - Serialization audit and golden fixtures

R1 established explicitly serialized rationals and immutable root certificates. Extend the audit across every value that is expected to survive process/repository boundaries.

Requirements:

- canonical rational bytes do not depend on backend-private serde structure;
- immutable root certificates round trip; decode performs bounded envelope
  validation without algebra, and budgeted restoration validates the
  mathematical certificate;
- public serialized matrices/polynomials use Resolvent-owned schema versions where they are durable artifacts;
- lazy runtime-only state is not serialized as authoritative identity;
- schema golden vectors cover normal, degenerate and boundary cases;
- a golden change requires an explicit schema/version decision.

Do not serialize caches, mutex state, local arena IDs or mutable refinement state as semantic truth.

### RV0-B1 - Panic and fallibility audit

Search the public exact/scalar path for assertions/panics reachable from valid untrusted caller data.

Convert to:

- validated constructors;
- typed domain/input errors;
- typed resource/indeterminate outcomes;
- private assertions only where a public invariant proves the condition.

Priority cases:

- zero denominators/divisors;
- malformed intervals;
- invalid root certificates;
- matrix shape mismatches;
- non-finite approximate ingress;
- unsupported scalar operations;
- forced lazy-exact evaluation edge cases.

Exit: public APIs fail closed instead of using panic as ordinary error control flow.

### RV0-B2 - Resource budget coverage

Audit every potentially explosive operation against the current `AlgebraBudget` model and extend it or define successor budget components for:

- expression/work nodes;
- coefficient bit growth;
- polynomial degree/term count;
- resultant/matrix work;
- root subdivisions/refinements;
- lazy exact nodes forced;
- recursive/tower depth where relevant.

Budget exhaustion must be deterministic with respect to counted mathematical work, not an implicit wall-clock timeout.

RV3 later generalizes this into operation/plan resource accounting.

### RV0-B3 - Exact/approximate capability table

Make the distinctions inherited from CADabra explicit before RV2 domains arrive.

Record which operations are available for:

- `f64`;
- interval/enclosure values;
- exact rationals;
- lazy exact reals;
- square-root/algebraic extensions;
- `Dual<S>` combinations.

In particular, keep the distinction between exact-closed field operations and approximate/transcendental operations. Do not silently add transcendental methods to exact values by evaluating them in `f64`.

### RV0-C1 - Consumer boundary regression gate

Maintain direct integration cases for both current primary consumers.

Scientia:

- existing exact differentiation bridge remains green;
- no scientific semantics or source types move into Resolvent;
- current corpus/semantic compiler gates remain green where part of the federation validation run.

CADabra:

- no `cadabra-exact`/`cadabra-scalar` references remain;
- stable CADabra crates consume public Resolvent types directly;
- geometry-specific checked-number/predicate/topology policy remains above Resolvent;
- exact/filter changes run relevant arrangement/SSI and licensed Parasolid oracle cases when available.

### RV0-C2 - Dependency and ownership grep gates

Add mechanical checks where cheap:

- no `.res`/scientific semantic vocabulary in production Resolvent modules;
- no CAD topology/entity vocabulary in production Resolvent modules;
- no Methodus numerical-solver or Solverang constraint-system semantics;
- no compatibility facade restoring deleted CADabra algebra crates;
- no stable consumer reaching around Resolvent to the underlying exact backend where the public abstraction is intended to own the decision.

### RV0-D1 - Baseline performance corpus

Record representative non-gating baselines for the exact stack before RV1/RV2 representation changes:

- rational arithmetic by bit size;
- interval filter versus exact fallback rates;
- determinant/filter benchmark;
- polynomial/resultant by degree/coefficient size;
- root isolation/refinement by degree/root separation;
- lazy exact DAG forcing and sharing behavior;
- exact matrix operations.

Correctness gates and score gates remain distinct. A baseline change is not automatically a correctness failure, but change-point data makes regressions visible.

### RV0-D2 - Deep/shared/concurrency stress

Preserve and broaden the strongest inherited lazy-exact tests:

- deep DAG evaluation without recursive stack overflow;
- iterative teardown;
- shared-subgraph forcing;
- overlapping concurrent forcing;
- deterministic exact decisions independent of forcing history;
- interval cache tightening never changes the exact mathematical value.

This specifically protects RV1 from accidentally conflating the retained symbolic term model with the existing lazy-exact runtime DAG.

### RV0-E1 - Federation baseline record

Record exact repository commits and successful gates for the post-R1 baseline used to start RV1 work.

Minimum local Resolvent gate:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
cargo test --locked --workspace --doc
git diff --check
```

Cross-repository validation should include Scientia and CADabra when a change touches their consumed algebra contracts, plus any downstream Sinbad/Finitum/Krasis gate required by the active federation plan.

### RV0-E2 - RV1 readiness freeze

Before replacing/augmenting the public symbolic expression identity in RV1, freeze these facts:

- canonical exact-number serialization;
- error/fallibility conventions;
- exact versus approximate scalar capability boundaries;
- root-certificate identity;
- public consumer ownership boundaries;
- benchmark/stress corpus locations.

RV1 may build a new term/wire schema without destabilizing the already-working exact numeric substrate.

## Exit gate

RV0 exits when:

- the landed R1 exact/scalar API has documented mathematical invariants and exactness semantics;
- durable serialized exact values are schema-owned by Resolvent;
- public panic/resource gaps found by the audit are closed or explicitly tracked as blockers;
- exact/approximate/dual capability boundaries are explicit;
- Scientia and CADabra integration gates are repeatable;
- baseline performance/stress corpora are recorded;
- RV1 can change symbolic representation without reopening numeric ownership.

RV0 does **not** require CADabra to stop R2 work while these audits run.

## Completion ledger

| Package | Landed evidence |
|---|---|
| RV0-A1 | [`../RV0-PUBLIC-API-CENSUS.md`](../RV0-PUBLIC-API-CENSUS.md) classifies the complete public surface by family and freezes invariants, persistence, consumer, and long-lived API status. |
| RV0-A2 | Normal, degenerate, boundary, and negative golden tests cover rational, canonical `QPoly`, rational-matrix, root-certificate, and receipt schemas. Root serde performs bounded envelope validation; budgeted restoration validates the mathematical certificate. Lazy state remains non-serializable. Receipt v1 retains its pre-QPoly-wire polynomial digest projection explicitly. |
| RV0-B1 | Checked ingress/construction/matrix paths, typed exact-zero evaluation, and bounded positive-width root refinement close the audit findings; remaining convenience/index/trait panics are documented preconditions with total alternatives where untrusted data enters. |
| RV0-B2 | `AlgebraBudget` counts expression/minor/Euclidean work, live polynomial division, matrix multiply/determinant/RREF arithmetic, certificate Horner/affine restoration, polynomial degree, intermediate coefficient bits, matrix/resultant dimension, root bisections, and lazy nodes. Root isolation and each PolyMat top-level operation retain one shared meter across every nested algebra stage; hostile aggregate tests prove subcalls cannot reset it. Convenience forms are documented proven-input contracts. |
| RV0-B3 | [`../RV0-CAPABILITY-TABLE.md`](../RV0-CAPABILITY-TABLE.md) freezes exact/enclosure/approximate/dual distinctions. |
| RV0-C1 | Resolvent tests retain the Scientia differentiation and CADabra exact/root/filter/matrix contracts; federation validation is recorded separately. |
| RV0-C2 | `tests/ownership_boundaries.rs` and `scripts/check-ownership.sh` reject consumer-owned vocabulary/deleted facades in production/dependencies. |
| RV0-D1 | [`../RV0-BASELINES.md`](../RV0-BASELINES.md) names the non-gating corpus; `benches/ladder.rs` reports rational, filter/fallback, lazy/eager, resultant/root, and matrix observations. |
| RV0-D2 | The `real` suite pins 200,000-node iterative evaluation/teardown, sharing, overlapping concurrency, monotone cache refinement, exact stability, and resumable budget exhaustion. |
| RV0-E1 | [`../RV0-FEDERATION-BASELINE.md`](../RV0-FEDERATION-BASELINE.md) records exact starting commits and repeatable local/cross-consumer gates. |
| RV0-E2 | Serialization, fallibility, capability, certificate, ownership, benchmark, and stress locations above are frozen; RV1 may add structural Terms without replacing `Expr`, `Real`, or numeric ownership. |

## Parallelism

A1/A2/B1/B2/B3 are largely parallel audits/hardening lanes. C1/C2 and D1/D2 can run concurrently against the landed implementation. E2 is the short convergence step before RV1's one-way identity decisions.

## Non-goals

- redoing the R1 migration;
- restoring deleted CADabra algebra crates;
- new general CAS language;
- new hash-consed symbolic store (RV1);
- assumptions/rewrite engine (RV4);
- broad multivariate algebra (RV6);
- notebook/protocol work (RV8);
- geometry semantics;
- Methodus numerical algorithms;
- Solverang constraint semantics.
