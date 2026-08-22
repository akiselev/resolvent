# RV0 public API and invariant census

This is the RV0-A1/B1/B2 freeze of the post-CADabra-consolidation surface. The
classification is exhaustive by exported family: every public item under a
listed module inherits that row's role, exactness, persistence, and consumer
contract. Per-item mathematical details remain beside the item in rustdoc.

| Public family | Role and invariant | Failure/resource contract | Durable? | Long-lived RV1/RV2 surface? | Current consumer |
|---|---|---|---|---|---|
| `Rational`, `RingOps`, `ExactRing`, `ExactField` | Exact primitive/traits; reduced arbitrary-precision rational field | Total arithmetic except documented field division; `try_from_ratio`, `checked_recip`, and finite `from_f64` are total ingress | `Rational` schema is explicit canonical numerator/denominator decimal strings | yes, as domain values/traits, not Term identity | Scientia, CADabra |
| `Interval`, `AtomicInterval`, EFT and expansion functions | Approximate outward enclosure/filter machinery; bounds are non-NaN and ordered | `try_point`/`try_new` reject invalid ingress; expansion operations return `None` when finite error bounds are unavailable | no; cache/enclosure state is derived | implementation/support surface only | CADabra |
| `Sign`, `Uncertain`, `UBool`, `UOrd`, `USign` | Certified decision vocabulary; `Unknown` means no claim | `try_of_f64` rejects NaN; filters fail closed to `Unknown` | no standalone artifact contract | yes as outcome vocabulary | CADabra |
| `Scalar`, `FallibleScalar`, `ApproxScalar`, `Dual<S>` | Consumer-neutral scalar kernel, total ingress companion, and forward derivative pair | `Scalar::from_f64` preserves all IEEE values for `f64` but panics on non-finite exact-`Real` ingress; `FallibleScalar` rejects non-finite/zero-denominator generic ingress; `ApproxScalar` is deliberately floating/transcendental | no | traits yes; concrete dual is runtime value | Scientia, CADabra |
| `Expr` | Existing bounded exact arithmetic expression, not RV1 `Term` | canonicalize/differentiate count expression nodes; evaluation returns typed missing/unsupported/division-by-zero errors | serde is implementation-era input to receipts, not frozen Term wire identity | no; RV1 may replace/augment it | Scientia differentiation bridge |
| `QPoly`, polynomial/root/radical functions | Exact dense univariate rational polynomial and certified algebraic-root operations | budgeted multiply/division/GCD/resultant paths bound work and growth; root isolation shares one work/bisection meter across decomposition, Descartes, collapse, and multiplicity proof; bounded refinement rejects nonpositive widths | `QPoly` is canonical `resolvent-qpoly/1` (empty zero; no trailing zeros); root-certificate serde performs bounded envelope validation and `from_certificate_with_budget` meters Horner/affine mathematical restoration | yes as typed domain values/certificates | CADabra |
| `Bernstein` | Exact polynomial enclosure on a strict rational interval | `try_from_power`/`try_subdivide_at` reject malformed intervals; legacy convenience forms document panic contracts | no; reconstruct from polynomial and interval | implementation/domain value only | CADabra |
| `Mat`, `PolyMat`, `bilinear` and pencil helpers | Exact rational/polynomial matrix algebra | budgeted multiplication/determinant/RREF charge exact arithmetic before execution; each PolyMat top-level minor/divisor/invariant operation shares one meter across recursion, polynomial combination, GCD, and division; checked construction/add/multiply/congruence/bilinear reject malformed shapes | `Mat` is `resolvent-rational-matrix/1`; `PolyMat` is runtime classification state | `Mat`/`QPoly` values yes; algorithms remain evolvable | CADabra |
| `SqrtExt<T>` | Exact one-square-root coordinate value with nonnegative radicand | `try_new` rejects negative radicands; cross-root arithmetic returns `None`; exact comparisons remain total | no durable schema yet | candidate RV2 domain value after descriptor work | CADabra |
| `Real<E>`, `Formula`, `TupleFormula` | Lazy exact runtime DAG with derived interval cache and exact memo | finite ingress is fallible; `exact_with_budget` counts forced DAG nodes; formulas retain documented algebra preconditions | explicitly runtime-only: never serialize node/cache/mutex/arena identity | no; distinct from retained RV1 structural Terms | CADabra |
| `AlgebraBudget`, `AlgebraError`, `RootError` | Deterministic resource/error vocabulary | mathematical work counts only; no wall-clock outcome | errors/budgets are Rust API values, not frozen RV3 wire contracts | temporary predecessor to RV3 | both |
| `AlgebraOperation`, `AlgebraReceipt` | Deterministic evidence envelope over canonical serde payloads | serialization failure is typed | `resolvent-algebra-receipt/1` | predecessor to RV3, versioned and replaceable | both |
| `metrics`, `certify`, determinant ladder | Filter instrumentation and exact fallback policy | counters are observational; exact fallback provides the decision | no | no | CADabra |

## Panic boundary

Untrusted data has a total entry point, including generic scalar ingress through
`FallibleScalar`. The older `from_ratio`, `recip`,
`Interval::point/new`, `Bernstein::from_power/subdivide_at`, `SqrtExt::new`, and
unchecked matrix/polynomial conveniences remain for callers whose types already prove the
precondition; each has a corresponding checked form. Indexing (`Mat::get/set`,
`PolyMat::get/minor`) is an ordinary in-range caller contract. Exact backend
trait implementations may panic if a consumer-defined `Formula` violates field
preconditions; arbitrary formula bodies cannot be made fallible without a later
trait contract change.

## Ownership freeze

No row owns scientific meaning, CAD topology, numerical-solver policy,
constraint semantics, or executable kernel IR. Scientia projects supported
scalar operations here without replacing `ExprId`; CADabra owns geometry and
certification policy above these values. RV1 structural Terms remain separate
from `Expr` and `Real`.
