# X1 — Is this API general, or three local projects described as general?

**Status:** adversarial review of `plans/api-shape.md` against consumers it was not written
for. Research input, not a commitment.
**Method:** read `plans/api-shape.md` (951 lines), `docs/research/consumer-{sinbad,cadabra2,
solverang}.md`, and — where the answer turned on a decision made elsewhere —
`plans/architecture.md` and `plans/roadmap.md`. Then took five consumers outside the local
three and asked, for each, whether the API as written serves them, needs an *additive*
extension, or needs a *breaking* change. Every measurement below was run against the real
repositories and is cited to file and line count.

**Verdict: `general-with-fixes`.**

Stated precisely, because both halves matter:

- **The skeleton is genuinely general and would survive almost any consumer set.** The
  embedding model (§1: no ambient state, no I/O, no unseeded RNG, caller-owned everything),
  the error/budget/certainty vocabulary (X-1, X-2, X-3, X-4), the runtime-arity ring
  (INV-13), and the caller-owned `FuncTable` (L4-2) all serve my five outside consumers
  *better* than they serve the local three. L4-2 in particular is the best decision in the
  document: "resolvent ships no transcendental semantics, only a table the caller
  constructs" is exactly the move that lets a proof assistant, an SMT solver and a teaching
  REPL share one mechanism with no changes.
- **The capability list (§2) and two type-level decisions are consumer-shaped**, and the
  mechanism by which they got that way is visible: clause (b) of the two-consumer rule
  ("a general algebraic primitive any CAS would be expected to have") has **never once**
  admitted a capability that is not either wanted by one of the three local consumers or
  named in `IDEAS-crates.md`. §3 below proves that by enumeration. The rule as *operated* is
  "≥2 consumers, **or** 1 consumer, **or** spec-named" — which is not the rule as *stated*,
  and the difference is exactly the overfit.

Scored against the task's own threshold — "a design that needs breaking changes for two or
more of these is overfitted" — the count is: 2 additive, 1 upstream-blocking, 2 breaking, and
both breaking cases are contingent on a usage mode the document never claims to support. That
lands on `general-with-fixes` rather than `three-special-cases-in-a-trenchcoat`. §7 states
what would flip it.

---

## 1. The five outside consumers

### 1.1 Proof assistant / certified computation — **additive**

Wants Gröbner with cofactor certificates feeding a kernel that trusts nothing.

**Served.** L2-10's `groebner_certified` is a separate entry point with cofactor tracing;
§4.1 gives certificates private fields, no public mint, and **public read accessors**; §4.3
says a distrusting consumer reads `evidence()` and re-checks with its own arithmetic. That is
the correct shape and it was derived from cadabra2's TCB posture (E2 §8), which happens to
generalize cleanly. `Certainty::Probable` visible in the type (X-4) is exactly what a kernel
needs to reject. `resolvent-seam` at zero dependencies (§1.4) is what makes admission
plausible at all.

**The one real defect, and it is invisible from inside the three consumers.** §4.2's tier F —
"free evidence: the evidence *is* the answer's shape; producing it costs nothing extra" —
lists *"isolating intervals from `isolate_roots`"* as an example. **An isolating interval
certifies nothing.** The claim "`f` has exactly one root in `[a,b]`" is established by a sign
variation count (Descartes/VCA) or a Sturm sequence; the interval is the *conclusion*, not the
evidence. A proof assistant handed `Vec<Interval<Q>>` cannot check anything — it must redo the
isolation. Retaining the witness (the Bernstein coefficient sign counts on the returned
interval, or the Sturm chain sign sequence at the endpoints) is not free, so the item is
mis-tiered as well as mis-typed.

Why the local three did not surface it: all three consume isolating intervals as *data*.
cadabra2 uses a certificate's *presence* as an admission ticket and never reads it (E2 §8:
"Nobody re-verifies it"); sinbad grades on `Certainty` alone; solverang has no L3 demand at
all. §4.5 then generalizes cadabra2's non-reading into a design rule — *"Elaborate proof
objects a consumer must interpret are not wanted by anyone"* — which is true of the three and
false of the entire certified-computation category.

**Fix (additive):** add `ProofKind::RootCount`, make `isolate_roots` return the sign-variation
witness per interval, and move it from tier F to tier C with the constant factor documented.

### 1.2 SMT NRA theory (`IDEAS-crates.md` #12) — **breaking, contingent**

Wants CAD, sign determination, incremental use inside a search loop with frequent
backtracking.

**Three separate problems, in increasing severity.**

**(a) Backtracking against the `Store` — additive, with a hazard the design explicitly
declined to close.** §1.2 adopts a caller-owned monotone `Store` and there is no
`mark()`/`rollback_to()` anywhere in the document (grep for `backtrack`, `incremental`,
`checkpoint` across `plans/`: zero hits). A search loop that mints terms per decision level
grows the store for the whole solve. Adding a checkpoint is additive — but it collides with
the handle decision. §1.2 says an out-of-range `Expr` returns `Error::Domain { fault:
ForeignNode }` while *"an in-range id from a different store yields a wrong answer, not an
error"*, and rejects a store tag because it *"taxes every consumer for a bug none of the three
would make."* After a rollback, every outstanding `Expr` is in-range and points at a different
node — the same silent-wrong-answer failure, now reachable through a supported operation. The
justification for the decision is stated in terms of the three consumers' access patterns;
that is the sentence to look at when asking whether this document is overfitted.

Mitigating: MCSAT's terms are polynomials, so this consumer can stay on L1 and never touch
L4 — and L1-4's inline packed keys (no monomial interner) make `MPoly` a droppable
self-contained value, which serves backtracking *very* well. The L1 decision survives this
attack; the L4 decision does not, but is avoidable.

**(b) `!Sync` on the headline type — see §5.** A portfolio/parallel NRA solver shares learned
lemmas containing algebraic sample points. `Clone` is cheap, so this is survivable, but each
clone re-refines from scratch: a point refined to high precision in one thread is refined
again in every other. That is a real cost with no API workaround.

**(c) The breaking one: there is no representation for a multivariate real algebraic sample
point.** `AlgebraicReal` is `{ poly: Arc<SqfrPoly<Rational>>, cache, mult }` (§1.3), and L3-3's
queries take `h: &UPoly<Rational>`. NRA needs sign determination of a polynomial at a point
whose coordinates are real algebraic numbers in several variables, which requires either:

- **algebraic numbers over an extension** — `AlgebraicReal` whose defining polynomial has
  coefficients in ℚ(α₁…α_k). The sealed coefficient set does contain `NfElem`, so
  `UPoly<NfElem>` is expressible, but `AlgebraicReal`'s field is hard-wired to
  `SqfrPoly<Rational>` and L2-1 is specified as *"Real root isolation over ℚ"*. Making it
  generic changes the type in every L3 signature. **Breaking.**
- **or a primitive-element / RUR presentation** that pushes the tower down to a single
  ℚ-algebraic number plus rational coordinate functions. This is the standard route (msolve
  does it) and it would make `AlgebraicReal` sufficient as written. **It appears nowhere in
  `api-shape.md`** — grep for `RUR`, `primitive element`, `sample point`: zero hits.
  `plans/architecture.md:64` lists RUR in `resolvent-real`, so the sibling track has it; the
  API document does not.

Likewise `CAD` appears exactly once in 951 lines, inside L2-11's justification column, scoped
to *bivariate* curve topology, marked one-consumer and *"do not build it before that is
settled."* Multivariate resultants, discriminant/psc chains, and projection operators are
absent. The spec's own milestone M4 is *"resultants, CAD (this is when #12's NRA unblocks)"* —
so the *spec's* named beneficiary of CAD is this consumer, and the API document scoped CAD
down to the one bivariate case cadabra2 might need.

**Verdict:** breaking, contingent on which route is chosen. If RUR ships as
`architecture.md:64` says, this drops to additive and the finding becomes "the API document
omitted the mechanism that makes its headline type sufficient."

### 1.3 Cryptography / coding theory — **needs a resolvent-side change (fails rule 4)**

Wants GF(p^k), factorization over finite fields; does not care about real root isolation.

**The layering serves it well** — a consumer that never imports `resolvent-alg` pays nothing
for `AlgebraicReal`, exactly as §2 promises cadabra2 pays nothing for `Fp`. Determinism
(X-6), seeded randomness (L0-7), `Budget` (X-1) and `row_echelon` over a field (L2-5, which is
Berlekamp's matrix step) are all directly useful.

**Then it hits the sealed set and stops.**

- **GF(p^k) is not in `api-shape.md` at all.** L0-6 is `Fp` (prime field, runtime modulus)
  only. L0-12 puts ℤ/n out of scope. The sealed coefficient set (§3.3) is
  `{Rational, Integer, FpElem, NfElem}`.
- **INV-14 makes this unfixable by the consumer**: *"Coefficient rings are a sealed set… A
  consumer cannot add a coefficient ring."* §8.4 anticipates the case and answers *"add it to
  the sealed set, not open the trait"* — i.e. **resolvent must change**, which is precisely
  what rule 4's acceptance criterion ("under 200 lines, with zero changes to resolvent")
  forbids. The change is source-additive for other consumers, but the consumer is blocked on
  an upstream release, and for a crypto consumer that is usually terminal: the whole point is
  *their* tower with *their* basis (GF(((2²)²)²) with a chosen normal basis; a Galois ring;
  a Montgomery-friendly modulus) chosen for speed. A sealed set can never carry that.
- **`plans/architecture.md:57` lists `GF(p^k)` and `Zn` in `resolvent-modular`**, and
  `plans/roadmap.md:90, 256, 539` schedule building both. So the sibling founding document
  already disagrees. See §2 — this is an unflagged contradiction on the L0 one-way door.
- **Factorization over GF(p) is not public.** L2-4 is *"Univariate factorization over ℚ
  (Zassenhaus, then van Hoeij)"*. Cantor–Zassenhaus / distinct-degree / equal-degree
  factorization is an internal step of that, so the code exists — but it is not a listed
  capability, and grep for `Cantor` in `api-shape.md` returns nothing. Note L0-6 was made
  public on exactly this argument: *"Making it public costs zero marginal implementation."*
  The argument was available and was not applied, because no local consumer asked.

**The reading of the spec that produced this.** `IDEAS-crates.md:120-121` says L0 is
*"`Z`, `Q`, `GF(p)`, `Z/n`, and algebraic extensions."* "Algebraic extensions" was read as
extensions **of ℚ** (SqrtExt, NumberField) — the direction cadabra2 needs — and not as
extensions of GF(p), which is the same words pointing the other way. `architecture.md` read it
the other way. Two readings of one clause, and api-shape took the one its consumers wanted.

### 1.4 Robotics / kinematics, hard latency, no allocator — **breaking online, works offline**

**Works offline, and this is the realistic architecture.** The honest form of this consumer
runs resolvent at build time — eliminate the loop-closure system symbolically, emit a numeric
solver — and that is served: L2-3 resultants, L4-4 `walk_topological`, and L4-7's refusal to
ship a code emitter (so the consumer emits its own fixed-point / SIMD tape). L2-14's exclusion
of numeric root polishing is correct here too: the online solver is the consumer's, not
resolvent's.

**Breaks online, and the document never draws the line.**

- **`no_std` and no-alloc appear nowhere in 951 lines** (nor anywhere in `plans/`). `Store`,
  `MPoly`, `Rational` are all heap-backed; `Rational` is a newtype over `dashu`, which has no
  custom-allocator story, so this is a substrate change, not an additive one.
- **§1.1 forbids the one mechanism that would help**: *"A custom global allocator, or an
  allocator parameter | Embedding must not fight the host's allocator choice."* The
  justification is backwards for an embedded consumer, whose entire requirement is to *supply*
  the allocator or avoid allocation. The rule is written from the perspective of a desktop
  host that already has an allocator it likes — which is what all three local consumers are.
- **The cheapest generality win in the document is unclaimed.** `resolvent-seam` is specified
  as zero-dependency (§1.4, §5.1) and contains `Sign`, `Scalar`, `ScalarOrd`, `TryDiv`, `Hom`,
  `Budget`, `Error`, `Certainty` — none of which needs `alloc`. Declaring it `#![no_std]`
  costs nothing and makes §5.1's "single highest-leverage hook" reachable from firmware. That
  it was not declared is diagnostic: three std desktop consumers defined what "embeddable"
  means.
- **`Budget` bounds steps, not memory.** §8.3 admits `Ord` on `AlgebraicReal` *"may allocate
  unboundedly"*. INV-4 guarantees totality, not a footprint. A latency-budgeted consumer needs
  a bit-size/allocation cap on `Budget`; that is additive.

### 1.5 Teaching / scripting from Python — **additive, and it refutes two stated justifications**

**Structurally fine.** Bindings are glue outside resolvent (rule 6, §5's dependency arrow),
`serde` is a default-off feature, and INV-4's no-panic rule is exactly right for FFI, where
unwinding is UB. `FuncTable::standard_elementary()` gives a teaching REPL its function set for
free.

**Three costs, all landing on justifications the document states explicitly.**

1. **The store-tag decision is refuted by this consumer.** §1.2 declined a store tag because it
   *"taxes every consumer for a bug none of the three would make."* A Python user with two
   notebook cells, two sessions, or two `Store`s in one script makes that bug immediately and
   gets a silently wrong answer. The binding author's fix is to wrap handles with an
   `Rc<Store>` plus a store id it maintains itself — ~30 lines of glue, so the cost is
   absorbable — but the *reason given* for the decision is now known to be false.
2. **`!Sync` costs a panic in the glue.** `pyo3` requires `Send + Sync` for `#[pyclass]`;
   `AlgebraicReal` forces `#[pyclass(unsendable)]`, whose access from a non-owning thread
   **panics**. INV-4 forbids panics in resolvent, and the `!Sync` choice reintroduces one
   immediately outside it, on the type §2.4 calls *"resolvent's headline differentiator."*
3. **There is no stated way to show a number to a human.** INV-7 permits only `demote_exact`
   / `enclosure` / `approx_lossy` and forbids *"`Display` that rounds"*; E2 anti-finding 7
   adds *"No `String`/`Display`-based symbolic API"*, which in cadabra2 is a **production-path
   clippy ban** (*"display readouts clippy-banned in production"*), not a library design rule.
   Generalized to an invariant, it leaves a REPL with no supported rendering for
   `AlgebraicReal` — which has no finite exact decimal form, so any human-facing string is an
   approximation, i.e. banned by INV-7's letter. An exact `Display` for `Rational` ("3/4") is
   presumably fine, but the document never says so, and a binding author reading INV-7 in good
   faith concludes there is no supported path. **This is one consumer's production discipline
   promoted to library law with an out-of-domain cost.**

---

## 2. Unflagged contradiction on the L0 one-way door

The brief says *"Layer 0/1 representation is a one-way door and must be settled before any
fan-out."* `plans/roadmap.md` §2.5 runs a contradiction census over D1's output and finds two
(`AlgebraicReal` mutability; interned vs inline monomials), each with a named deciding
experiment. **It misses a third, and the third is the one my crypto consumer dies on.**

| Source | The coefficient seam |
|---|---|
| `plans/api-shape.md` §3.2(b), §3.3, INV-14 | Option (b) — a consumer-implementable coefficient trait — **"Rejected."** Coefficients are a **sealed** set `{Rational, Integer, FpElem, NfElem}`. *"A consumer cannot add a coefficient ring."* |
| `plans/architecture.md` §2.3 | `pub trait Ring: Clone + PartialEq + Send + Sync + 'static` with `CommutativeRing`, `Field`, `EuclideanDomain`, and orthogonal markers `Ordered`, `Reducible`, `Liftable`, `BulkOps`. *"The modular pipeline is bounded by `C: Reducible + Liftable`, not by `C: Ring`. That is what makes 'modular methods everywhere' a type-level statement rather than a slogan."* |

These are not two phrasings of one decision. One is a closed enum of four types; the other is
a public open trait tower with a documented depth cap and a compile-time budget (§2.4). They
imply different signatures for every L1 and L2 function.

**And api-shape's argument against (b) does not survive contact with architecture's version of
(b).** §3.2(b) rejects the coefficient trait because *"to be useful the trait must expose
everything §3.1(1) lists… implementing `CoefficientRing` for an inner-loop number type pushes
bignum-shaped obligations — exact division, content, bit-length, reconstruction — into a type
whose entire purpose was to be word-sized."* That is an argument against a badly factored
trait. `architecture.md` §2.3 is the well-factored version: `Ring` has seven methods and no
bignum obligation; the bignum-shaped duties live in `Reducible`/`Liftable`, which a word-sized
type simply does not implement.

Note the inversion. The identical argument — *"Six methods, one sign, one fallible division.
Nothing in `Scalar` obliges an implementor to be a bignum"* (§3.3) — is used to **justify** the
open `Scalar` seam six paragraphs after it is used to **reject** the open coefficient seam. The
two seams differ in what they must support, but the *reason given* for rejecting one is the
reason given for accepting the other.

This is a fatal-severity process defect: a declared one-way door has two incompatible
specifications in two founding documents, the fan-out is scheduled against it, and the
contradiction census that exists specifically to catch this did not.

---

## 3. Is clause (b) an escape hatch? — yes, in one direction

The rule (§2): core iff **(a)** ≥2 independent consumers, **or (b)** *"it is a general
algebraic primitive any CAS would be expected to have."*

**Clause (a) is applied honestly.** The exclusion of `sem1f-biquad-spike` from sinbad's demand
set (§2, citing E1 §8) is a genuine act of discipline: counting it would have manufactured a
two-consumer majority for algebraic extensions, and the document refuses. L2-7, L2-8, L2-9 are
pushed to adapter with one consumer each, correctly, and L2-7's reasoning — that the seam plus
`NumberField` makes cadabra2's *existing* 49-line `inertia` generic for free — is the rule
working exactly as designed. L3-5's insistence on *"sign of an element of a real extension
tower over ℚ(ξ), not `sign_radical2`"* is the general-shape rule applied correctly.

**Clause (b) is applied in one direction only.** Enumerate every core item admitted on clause
(b) and check what else is true of it:

| Item | Consumers | Also true |
|---|---|---|
| L0-8 `SqrtExt` | C | admitted as a specialization of L0-9; doc concedes clause (b) *"holds but weakly on its own"* |
| L0-9 `NumberField` | C | spec-named ("algebraic extensions") |
| L1-9 Bernstein / de Casteljau | C | internal-need argument (see below) |
| L2-3 resultant | C | spec-named |
| L2-4 factorization over ℚ | C | spec-named |
| L2-6 Bareiss | C | internal need for L2-3 |
| L2-10 Gröbner / F4 | **none** | spec-named |
| L2-11 bivariate curve topology | C | spec-named ("CAD") |
| L2-12 multivariate factorization | **none** (both C and V reject) | spec-named |
| L3-1 `AlgebraicReal` | C | spec-named, called "the whole bridge" |
| L3-6 `AlgebraicReal` arithmetic | **none** | follows from L3-1 |
| L4-3 `diff_with` | S | textbook CAS |
| L4-5 `is_polynomial_in` | S (speculative) | small |

**Every single one is wanted by a local consumer, or named in `IDEAS-crates.md`, or both.
Clause (b) has never once admitted a capability that is neither.** The capabilities that are
general-primitive-by-any-standard and are neither consumer-wanted nor spec-named are, without
exception, absent:

GF(p^k) · Cantor–Zassenhaus factorization over GF(p) as public API · multivariate
resultants and discriminants · CAD projection and sample points · RUR / primitive element ·
Hermite and Smith normal forms · p-adics · cyclotomics · polynomial composition/decomposition ·
partial fractions.

Every one of those is in Singular, Macaulay2, PARI, Sage and Magma. So the operative rule is
**"≥2 consumers ∪ 1 consumer ∪ spec-named"**, and clause (b) as written is decorative — it
never does independent work. That is the honest answer to the question, and it is the single
strongest piece of evidence for the overfit charge.

**The smuggling is mild but real where it occurs.** Two instances:

- **L1-9.** The internal argument is *"the Descartes/VCA test **is** a Bernstein coefficient
  sign count, so resolvent computes these anyway."* True for the **univariate** half. The item
  as scoped is *"Univariate/bivariate only"*, and bivariate de Casteljau over a rational box is
  not computed by univariate root isolation. Half the item is admitted with one consumer and no
  internal need.
- **X-8** (warm-start telemetry as proof-free data) is **core** on one consumer, with the
  justification column citing only E2 §6.6 — cadabra2's *internal* invariant that "a hint is
  never evidence." No clause-(b) argument is offered. It is good design, but by the document's
  own mechanics it should have been "the adapter drops the certificate."

**One count is soft and should be marked.** L2-1 (root isolation) and L1-1 (`UPoly`) are both
justified as "two consumers", where the second is sinbad's `crates/solverang` event detection.
Verified: `/home/dev/sinbad/crates/solverang/` contains exactly `DESIGN.md` and `STATUS.md` —
no source. E1 §1.3 carries the caveat plainly (*"The integrator is not written. This demand is
real in shape and unimplemented in fact"*) and E1 §7 makes it open question 3, which *"decides
whether resolvent has any per-operation consumer in sinbad at all."* `api-shape.md`'s
justification column drops the caveat and reports "Two consumers." Both items survive on other
grounds; the count should still be marked soft. (Separately: this `solverang` is a different
project from `/home/dev/projects/solverang`, which the legend calls **V** — two unrelated
things share a name across the evidence base, and §6.1(c) titles a sinbad adapter "solverang".)

---

## 4. Are the adapter line counts credible? — spot-check on cadabra2

I checked the one with the most prior measurement behind it, so the estimate had the best
chance of being right. `api-shape.md` §6.2 claims **175 lines**, derived as E2's measured ~250
minus four named savings (~40 + ~15 + ~12 + ~10 = 77; 250 − 77 = 173 ≈ 175).

**Measured, non-doc non-test code in the actual delegation files** (`wc -l`, then strip
comment-only and blank lines above the `#[cfg(test)]` marker):

| File | Total | Code above tests | Composition (my classification) |
|---|---|---|---|
| `cadabra-core/src/exact/scalar.rs` | 428 | **149** | **all delegation** — 21 inherent methods + `Add`/`Sub`/`Mul`/`Neg` + `From` + `cmp_exact` |
| `cadabra-core/src/exact/radical.rs` | 410 | **137** | mostly delegation |
| `cadabra-core/src/exact/algebraic.rs` | 606 | **211** | ~65 delegation (`AlgebraicNumber`, 14 methods, `:54-160`); ~145 geometry (`SeamBranch`, `WeierstrassParam/Sample/Span`, `:163-430`) |
| `cadabra-core/src/exact/interval.rs` | 218 | **86** | delegation |
| `cadabra-core/src/exact/mint.rs` | 259 | **69** | cadabra2's own mint guard |
| **Total** | | **652** | |

Set against §6.2's block table:

| Block | api-shape budget | Nearest measured object | Ratio |
|---|---|---|---|
| `ExactScalar` | 34 | `scalar.rs`, 149 lines, zero CAD vocabulary | **4.4×** |
| `IntervalScalar` + four `Scalar`/`ScalarOrd` impls | 34 | `interval.rs`, 86 — and four impls of a 6-method trait is ~27 one-line bodies plus headers on its own | **~2.5×** |
| `AlgebraicNumber` | 33 | delegation half of `algebraic.rs`, ~65, before the 65 lines of claimed savings | **~2×** |

Three problems, in order:

1. **`scalar.rs` is the killer.** api-shape's `ExactScalar` sketch has 11 methods. The shipped
   one has 21: `zero`, `one`, `from_f64`, `from_finite`, `from_i64`, `from_ratio`,
   `from_rational`, `negated`, `squared`, `abs`, `checked_div`, `sign`, `is_zero`,
   `is_positive`, `is_negative`, `demote_exact`, `enclosure`, `approx_lossy`, `as_rational`,
   plus operators and `cmp_exact`. **None of those is CAD vocabulary.** They are the
   constructors and predicates any consumer newtyping a rational writes. A leaner adapter is
   possible — but it is not 34 lines, and the 34 was reached by sketching a subset.
2. **`lift.rs` is silently dropped.** E2 §12 says *"The adapter already exists. It is
   `cadabra-core/src/exact/` **plus `cadabra-arrange/src/lift.rs`**"* and tables it at ~90
   code lines of 238 total. §6.2's block table has no `lift.rs` row. Reading the file, most of
   it is genuinely cadabra2↔arrangements geometry (`LiftedRadicalGraph`, `radical_graph`,
   `chart_level`) with one delegation function (`fn rational(&ExactScalar) -> Rational`,
   `:141`), so excluding it is *defensible* — but the exclusion is unstated and it runs in the
   direction that helps the number.
3. **The arithmetic and the sketch are not independent.** 250 − 77 = 173 and the block table
   sums to 175. The block table was sized to land on the subtraction, so it is not
   corroboration.

**Honest range: 250–400 lines** for a delegation adapter with the shipped one's ergonomics.
cadabra2 therefore fails the literal 200-line test and passes only under the "resolvent-facing
seam" reading — **which is exactly the reading §6.3 applies to solverang, and applies
honestly** (40-line seam reported passing, ~300-line total reported failing, both numbers
given, with the correct observation that transcription is invariant to resolvent's API shape).
The document is fully capable of this rigor. It did not apply it to cadabra2, where prior
measurement existed and the conclusion was pre-committed by §0's decision 5 (*"All three
adapters pass the 200-line test"*).

The acceptance criterion still passes in substance — nothing cadabra2 needs is bespoke to
cadabra2 — but §0 decision 5 as written is not supported by its own evidence.

---

## 5. The numeric seam: two concrete traces

The decision (§3.3): (a) concrete resolvent-owned coefficients reached by `From`/`TryFrom`;
(c) an open six-method `Scalar` for evaluation, restricted to straight-line texts; (b) never.

**The split is right.** Separating "what are a polynomial's coefficients" from "what do you
evaluate it at" is the correct cut and is the document's second-best decision. Two costs it
hides, both traceable to a call site the document itself names.

### 5.1 `Scalar` returns by value, and INV-14 freezes that

```rust
pub trait Scalar: Clone + PartialEq + Sized {
    fn add(&self, rhs: &Self) -> Self;   // by reference, never consuming
    …
}
```

By-reference *arguments* with by-value *returns*. Every operation on `Rational` allocates a
fresh dashu numerator and denominator; there is no `add_assign`, no `add_into(&mut out)`, no
scratch buffer.

Trace the call the document wants to speed up. L2-6 exists to replace cadabra2's recursive
Laplace determinant, measured at **2.448 ms** (E2 §2, `perf_r0.rs`). Bareiss over ℚ is one of
the six texts §3.4 declares generic. At *n* = 4 with `LambdaPoly` entries it is
Θ(n³) ≈ 64 multiply-subtract-divide steps, each allocating two or three fresh bignum rationals
that are dead one line later. That is the allocation profile the incumbent already has, so the
speedup comes entirely from the algorithm, not from the arithmetic — while the seam forecloses
recovering the rest.

The foreclosure is explicit: **INV-14** — *"`Scalar` stays at six methods plus a sign plus a
fallible division"* — is a hard invariant, so adding accumulating forms is a stated violation.

Note the fix does not reopen the trap the seam was built to avoid. Adding
`fn add_assign(&mut self, rhs: &Self) { *self = self.add(rhs) }` with a default body obliges no
implementor to do anything; a word-sized type ignores it, a bignum overrides it. E1 §1.4's
ask — *"`add/sub/mul/div` by reference without consuming"* (`predicates.rs:33-70`) — is
satisfied either way. The invariant should be *"`Scalar` imposes no obligation that a
word-sized type cannot discharge"*, which is the property that actually matters, rather than a
method count.

### 5.2 The flagship adapter sketch puts a bignum reduction in the innermost loop

This one is worse, because it is in the acceptance evidence. `api-shape.md` §6.3:

```rust
for (k, &j) in cols.iter().enumerate() {
    row[j] = f.derivative(k as u32)                 // L1-5
              .evaluate_with(&fp, &local).ok()?;    // hom Q -> GF(p),  L1-6/L1-7
}
```

and §3.3 presents `evaluate_with` as *"The evaluation signature that ties them together, and
which satisfies E3 R5 exactly."*

It satisfies R5 and silently regresses R3. `Hom<Rational, FpElem>::apply` must reduce a bignum
rational mod *p*: a `Natural % u64` on numerator and denominator plus a **modular inverse of
the denominator** (extended Euclid). Folding the hom into `evaluate_with` applies that **per
coefficient, per evaluation point**, in the innermost loop.

Compare E3 §6.1's own sketch, which hoists it:

```rust
let f_p = f.map_coefficients(|q| fp.reduce(q))?;    // R3 — once per polynomial
…
row[j] = f_p.derivative(k).evaluate(&local);        // pure FpElem arithmetic
```

`api-shape.md` §6.3 **deleted that line.** And this is the consumer whose entire case (E3 §3.2)
is *"**No bignums appear anywhere.** All arithmetic is single-word modular"*, whose accepted
demand is a latency win in a per-edit diagnosis loop (`system.rs:765`), and which E3 §4.1 notes
will be called O(k) times inside a MUS extraction loop, compounding it.

L1-7 (the up-front coefficient hom) is still core, so the fast path exists — the API is not
broken. The **acceptance sketch demonstrates the slow idiom and the prose endorses it as
"exactly" right.** For a document whose purpose is to fix the API shape before fan-out, an
adapter sketch is a specification of intended use. Fix: restore the hoist in §6.3, and state
the rule (*homomorphisms are applied to polynomials, not inside evaluation loops*) in §3.4's
boundary rules.

**Second trace, minor:** the same sketch calls `f.derivative(k)` inside the column loop,
allocating a fresh `MPoly` per (constraint, polynomial, column) — up to 14 per residual for
`assembly::Insert`. Hoist to a `Vec<MPoly>` computed once per polynomial.

---

## 6. Arena / interner ownership under three access patterns

| Pattern | Survives? | Detail |
|---|---|---|
| **Multi-threaded** | Partly | `Store: Send`, construction needs `&mut Store`, so no concurrent building — correct and cheap. But **there is no supported way to move an expression between two `Store`s.** `canonical_bytes` (L4-6) is specified as an addressing function, not a decodable form, and no `Store::import(bytes) -> Expr` exists. Every parallel, multi-process or distributed-cache consumer writes the same walk-and-rebuild over `walk_topological`. That is ≥2 hypothetical consumers by the document's own rule; it is absent because none of the three local ones is parallel at L4 (sinbad's two L4 uses are build-time; cadabra2 does not use L4; solverang must not). Additive: `Store::rebuild_from(&Store, Expr) -> Expr`, ~30 lines, and it belongs in resolvent so it is written once. |
| **Backtracking** | No | §1.1(a). No checkpoint; monotone growth for the life of a search; and generation tags were declined with a justification that names the three consumers' access patterns. Additive to add, but the handle-safety half needs the tag §1.2 rejected. |
| **`no_std`** | No | §1.4. Not mentioned anywhere. `resolvent-seam` could be `no_std` today at zero cost and is not declared so; §1.1 forbids an allocator parameter, foreclosing the alternative. |

**And the `!Sync` decision loses on every outside consumer.** §1.3 chooses inline
`RefCell<Isolation>` + `std::ptr::eq` guard, `Send + !Sync`, on evidence from E2 §4.1 alone
(cadabra2's `Rc<RefCell<RealRoot>>` and `Rc::ptr_eq` guard at `trim.rs:857-862`). The argument
is good — the guard is exactly correct *because* the cache is inline, so address equality means
value identity — and it is optimal for a single-threaded sweep. Outside:

- parallel SMT: correct but re-refines per clone;
- `pyo3`: forces `#[pyclass(unsendable)]`, which **panics** on foreign-thread access;
- any consumer wanting `Arc<AlgebraicReal>` in a shared cache: blocked.

`plans/roadmap.md` §2.5 already flags this as contradiction 1 against
`architecture.md:589-591` (`Arc<Inner>`, lock-free monotone, `Send + Sync`) and §2.6 correctly
identifies the lock-free option as a *fourth* candidate that R3's table does not enumerate,
where *"a stale read is merely a wider valid enclosure, so there is no self-comparison hazard
and no atomic per compare."* **Every consumer in this review favors that side.** This review is
independent evidence for the experiment roadmap §2.5 already schedules, and the outside-consumer
input should be entered into ADR-013 as a tiebreaker: the `RefCell` option's advantage is
confined to the single access pattern that motivated it.

---

## 7. What would flip the verdict

`general-with-fixes` is contingent. It becomes `three-special-cases-in-a-trenchcoat` if all
three of the following are ratified as written:

1. **The sealed coefficient set survives** (§2). Then crypto/coding theory, p-adic, and Galois
   ring consumers are all upstream-blocked by construction, and the L0 seam is defined by the
   four rings three local consumers use.
2. **`AlgebraicReal` stays ℚ-only with no RUR** (§1.2c). Then the spec's own named beneficiary
   of L2 — #12's NRA theory, `IDEAS-crates.md` milestone M4 — needs a breaking change to the
   headline type, and "the whole bridge to computational geometry" is a bridge to one
   geometry.
3. **`!Sync` is ratified without the outside evidence** (§6). Then the headline type is shaped
   by exactly one consumer's inner loop.

Conversely, it becomes `genuinely-general` if:

- the open `Ring`/`Reducible`/`Liftable` tower from `architecture.md` §2.3 wins §2's
  contradiction, and GF(p^k) + public GF(p) factorization ship;
- ADR-013 resolves to lock-free monotone `Arc<Inner>`;
- `resolvent-seam` is declared `#![no_std]`, `Scalar` gains defaulted in-place forms, and
  INV-14 is restated as an obligation bound rather than a method count;
- §4.2's tier F stops calling isolating intervals evidence;
- `Store` gains `rebuild_from` and either a checkpoint with generation-tagged handles or an
  explicit statement that L4 is not for search loops;
- §6.2's cadabra2 count is restated at its measured range and §0 decision 5 is softened to
  match §6.3's honesty.

None of those is a redesign. That is why the verdict is what it is.

---

## 8. What this review could not settle

Stated so it is not mistaken for settled.

1. **Whether the RUR route makes `AlgebraicReal` sufficient for NRA.** `architecture.md:64`
   lists RUR; `api-shape.md` does not mention it. Writing one two-variable sign-determination
   query by hand against both candidate designs would settle whether §1.2(c) is breaking or
   merely undocumented.
2. **The true delegation line count for cadabra2.** §4 classifies `radical.rs` and the halves
   of `algebraic.rs` by reading; a per-line attribution by whoever owns that code would replace
   my 250–400 range with a number.
3. **Whether the `evaluate_with` hom cost is material at solverang's actual sizes.** §5.2
   argues from operation counts, not measurement. E3 §3.3's proposed port
   (`identify_dependent_blocks` behind a feature flag, both paths, real sparsity) would answer
   it with the hoisted and unhoisted forms as two arms.
4. **Whether a `no_std` `resolvent-seam` actually compiles.** `Error` carries `Op` and
   `DomainFault`; if either grows a `String` or a `Box`, the claim in §1.4 fails. Cheap to
   check once the crate exists.
