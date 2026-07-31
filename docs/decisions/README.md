# Architecture decision records

Every one-way door in resolvent has a file here. **ADRs are normative**: where a plan
document, `API.md`, or a research note contradicts a Ratified ADR, the ADR wins and the
contradicting text is a proposed amendment (ADR-021 §1).

**Template.** Every file carries, in order: a machine-readable `**Status:**` line
(`Draft` / `Proposed` / `Ratified YYYY-MM-DD` / `Superseded by ADR-NNN`), a
`**Reversibility:**` line, an optional `**Amended:**` line naming what changed, the lanes it
gates, its evidence; then **Context**, **Decision**, **Consequences**, **Alternatives
considered and why rejected**, **What would reverse this**. Each ADR lands on exactly one
option. Where a decision was corrected, the superseded position is recorded under
Alternatives with its reason — history is not rewritten silently, and a superseded decision
is never left standing as current.

**Ratification is the repository owner merging a commit that sets the status line.** Nothing
else counts. An agent may draft an ADR and may run the experiment an ADR needs; it may not
ratify one.

| ADR | Decision, in one line | Reversibility | Status |
|---|---|---|---|
| [001](ADR-001-license-posture.md) | MIT OR Apache-2.0; Tier A/B/C reading discipline; `cargo-deny` with a three-case regression corpus; `Derivation:` tethered to a committed research note | one-way | Ratified |
| [002](ADR-002-bignum-backend.md) | `dashu` behind the `resolvent-int` newtype wall, no re-export; modular methods *concentrate* rather than eliminate large integers, so the gcd ladder runs to 256 kbit and half-GCD is a triggered M1 contingency | costly | Ratified |
| [003](ADR-003-modular-arithmetic-in-house.md) | Hand-roll `resolvent-modular` (`Fp`, `Zn`, `GF(p^k)`, bulk kernels, prime registry); reject `ark-ff`, `crypto-bigint`, `num-modular` | cheap | Ratified |
| [004](ADR-004-z-primitive-coefficients.md) | Coefficients are ℤ-primitive; ℚ is a boundary façade; isolation works on dyadic intervals in ℤ[x] | one-way | Ratified |
| [005](ADR-005-workspace-crate-split.md) | Seven published crates + three unpublished, lockstep versioned; ten layering gates, including zero dev-dependencies on published crates | costly | Ratified |
| [006](ADR-006-generics-boundary.md) | Generics cross crate boundaries, never inner loops. **The corrected trait tower**: `Ring::Ctx`, `Liftable: Reducible`, `Reducible::Image: CommutativeRing` with fallible `reduce`, `BatchField::inv_batch`, no `BulkOps` | one-way | Ratified |
| [007](ADR-007-polynomial-representations.md) | Three representations; `UPoly<C>` defined first and standalone; `MPoly` carries an **owned** ring handle | one-way | Ratified |
| [008](ADR-008-monomial-representation-and-overflow.md) | Interned arena + packed key + divmask; **content-derived ids**; separate `W_KEY`/`W_RAW`; guard-bit overflow with widen-and-restart; arena exhaustion is an error | one-way (interning) | Ratified |
| [009](ADR-009-monomial-order-runtime.md) | Order is runtime ring data normalized into the key at intern time; **divisibility is order-free on `raw`**; FGLM gets a dual-key pair ring, not re-interning | one-way | Ratified |
| [010](ADR-010-modular-methods-and-certificates.md) | Modular everywhere; `Certified<T>` + `Certificate<C: Claim>` with a claim tether and no public mint; the two Gröbner modes **do not** share a reducer; batched lanes need a split driver | one-way | Ratified |
| [011](ADR-011-error-model.md) | Fail at construction, not at query; no panics; structured `Unsupported`; **a `Budget` on every looping entry point**, with bound-derived defaults so a decline is a bug where a bound exists | one-way | Ratified |
| [012](ADR-012-determinism.md) | Counter-based seeded RNG, index-addressed primes, ordered combination, replayable traces; INV-M1 (no tie-break consults id order); certificates and telemetry excluded from canonical bytes | one-way | Ratified |
| [013](ADR-013-algebraic-real-mutability.md) | `Arc<Inner>`, `&self` monotone refinement, `Send + Sync`, total `Ord`; **no `Equal` from bound exhaustion**; `try_cmp` alongside `Ord` with a measured diagnostic ceiling | one-way | Ratified |
| [014](ADR-014-algebraic-real-no-hash-no-arithmetic.md) | No `Hash`, no general arithmetic; `canonicalize()` opt-in; multiplicity in an `IsolatedRoot` struct; `SqrtExt` first-class with a resolvent-only generic parameter | one-way | Ratified |
| [015](ADR-015-no-float-interval-type.md) | No float interval type in the public API; rational bounds + an outward-correct `(f64, f64)` pair, specified by committed conformance vectors | cheap | Ratified |
| [016](ADR-016-oracles-are-subprocesses.md) | Subprocess-only oracles; two crate categories, no exceptions; every adapter ships a calibration corpus; benchmark families need a Tier-A citation | cheap | Ratified |
| [017](ADR-017-layer-4-egraph-seam.md) | L4 is a resolvent-owned hash-consed DAG with a caller-owned `FuncTable`; **no `simplify`, no `RuleSet`, no rewriter, no e-graph dependency in v1**; the deferred list is named | cheap | Ratified |
| [018](ADR-018-deferred-consumer-integration.md) | Defer the `arrangements` question; adapter-by-consumer is the default; the enumerated list of things not to do so options A and C stay open | cheap by design | Ratified |
| [019](ADR-019-numeric-type-seam.md) | One open trait tower covering coefficients *and* evaluation; no ops-surface scalar trait, no `resolvent-seam` crate; `Ring` gains defaulted in-place forms | one-way | Ratified |
| [020](ADR-020-arena-and-handle-ownership.md) | Every arena is a caller-owned value; handles are arena-relative and never escape into a result; `rebuild_from` for cross-store movement; optional `store-tags` | one-way | Ratified |
| [021](ADR-021-document-precedence-and-ratification.md) | **ADRs are normative**; `Status:` is machine-readable and ratification is a merge; `lanes.toml` makes the freeze a dependency edge; the twelve-item contradiction register | cheap | Ratified |
| [022](ADR-022-simd-unsafe-leaf.md) | One audited `unsafe` leaf, `resolvent-modular::simd`, with a CI-asserted bit-identical scalar fallback — or the published *Competitive* gate drops to 3–4× SOTA in the same commit | cheap | Ratified |
| [023](ADR-023-certificates-are-adversarially-validated.md) | Every certificate ships a mutant set and is observed rejecting a wrong answer; no certificate may invoke what it certifies; randomized certificates are graded over the fleet seed schedule | cheap now | Ratified |
| [024](ADR-024-corpus-tiering-and-gate-budgets.md) | `fast`/`full`/`slow` corpus tiers with a printed census and a hard `fast` budget; the sharpness-ceiling ratchet; declines classified before they are scored; five exit criteria rewritten as gates | cheap now | Ratified |
| [025](ADR-025-resultant-conventions.md) | Resultant value, degenerate-input table, the explicit `(−1)^{mn}` swap rule, Ducos's subresultant scalar convention; `ResultantOutcome` distinguishes a common component from a zero value | costly | Ratified |

## Reading order

- **Before writing any code:** 021 (how these documents relate), 001, 005, 016.
- **Before Layer 0:** 002, 003, 006, 011, 012, 019, 023, 024.
- **Before the univariate trunk:** 004, 007, 013, 014, 015, 025.
- **Before the multivariate trunk:** 008, 009, 010, 020, 022.
- **Before Layer 4:** 017, 020.
- **Whenever a consumer question arises:** 018, then `API.md`.

## Open experiments the ADRs gate on

Named here so they are not rediscovered. Each is specified in its ADR against a harness that
does **not** require the artifact it gates.

| Experiment | Gates | Specified in |
|---|---|---|
| **Z2** — `gcd`/`gcd_ext` ladder to 256 kbit + `rational_reconstruct` at Hexapod's modulus | `resolvent-int`; ADR-002 §Decision 7's half-GCD trigger | ADR-002 |
| **E-MUT** — four `AlgebraicReal` mutability prototypes, sorting 10³ degree-8 numbers | A1, i.e. all of M3 | ADR-013 |
| **E-MONO** — inline packed monomials vs ids+arena, replayed against a recorded S-pair trace | P1, P2, P3 | ADR-008 |
| **E-COFACTOR** — cofactor **reconstruction** prime count and wall time, Buchberger at Katsura-6/7 | whether `groebner_certified` is an API or an oracle | ADR-010 §5a |
| **Y1** — consumer workload profile, and the `cmp` step distribution on the M4 corpus | ADR-013 §5b's `Ord` ceiling; every geometry performance claim | ADR-013, `ROADMAP.md` |
