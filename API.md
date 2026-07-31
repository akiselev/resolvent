# resolvent — the consumer-facing API design

**Status:** canonical. **Supersedes `plans/api-shape.md`**, which remains readable as the
working notes it was.
**Inputs.** `docs/research/consumer-sinbad.md` (E1), `consumer-cadabra2.md` (E2),
`consumer-solverang.md` (E3); the two adversarial reviews `challenge-generality.md` (X1)
and `challenge-evidence.md` (X2), **which are authoritative wherever they contradict the
evaluations or the earlier API notes**; `plans/architecture.md` and
`docs/decisions/ADR-001…018`, written in parallel from the algorithm/verification side.
Where the architecture track and the consumer track disagreed, the resolution and its
reason are recorded in `docs/decisions/RECONCILIATION.md`; this document states the
outcome only.

Every claim about an existing repository is cited to a file and a line. Nothing here
invents a benchmark. Numbers that were measured are labelled measured; numbers that were
estimated are labelled estimated, and the two are never mixed in one figure.

---

## 0. The seven decisions this document makes

1. **Embedding is by owned values, not by a session.** No global state, no ambient
   context, no thread-local cache, no interpreter. Every arena — the L1 monomial arena
   and the L4 expression store — is a caller-owned value, and handles are arena-relative
   (§2, ADR-020).
2. **There is exactly one open numeric seam, and it is the coefficient-ring trait tower
   in `resolvent-base`.** A consumer may implement `Ring`/`CommutativeRing`/`Field`/
   `EuclideanDomain` for its own type and instantiate resolvent's generic algorithm texts
   over it. resolvent ships **no second, ops-surface "scalar" trait** and no
   `resolvent-seam` crate (§3, ADR-019). This reverses the earlier plan and it is the
   single largest change in this document.
3. **Coefficient homomorphisms are applied to polynomials, never inside evaluation
   loops.** `map_coefficients` is the only cross-ring path; `eval` is same-ring. The rule
   is enforced by the type system rather than by documentation (§3.4).
4. **Certainty is in the return type; certificates are unforgeable and readable.**
   `Certified<T>` carries a `Certainty`; certificate objects have private fields, no
   public mint, public read accessors, and a structural tether to the claim they attest
   (§5).
5. **L4 is a term-algebra container with a caller-owned function table, and it never
   rewrites anything implicitly.** One mechanism serves sinbad's transcendental
   differentiation and cadabra2's deliberately-un-rewritten `Cos2` atom (§4.5, §6.2).
6. **The 200-line adapter test is reported as two numbers, always.** The
   resolvent-facing *seam* and the *total* adapter are different quantities and only the
   first is resolvent's to control. Measured: sinbad's three adapters pass on both
   numbers (estimated); cadabra2 and solverang pass on the seam and fail on the total,
   for reasons that are invariant to resolvent's API shape (§7).
7. **Where a capability is general but no local consumer wants it, it is admitted or
   rejected on the record, once, in §4.2 — not left to the accident of who was
   surveyed.** This is the fix for the defect X1 §3 identified: the governance rule as
   previously *operated* was "≥2 consumers ∪ 1 consumer ∪ spec-named", which is not the
   rule as *stated*.

---

## 1. Design goal, and the governance rules that defend it

### 1.1 The goal, stated precisely

resolvent must be **embeddable directly** into an application layer — a consumer holds
resolvent values in its own structs, calls resolvent synchronously on its own thread, and
links it as a library — while remaining **general-purpose**: informed by real consumers,
not tailored to them.

Operationally that means three things, in priority order:

1. **Nothing consumer-specific reaches the public API.** No type, trait, method, name, or
   feature flag that mentions or presumes a consumer's domain. Features are
   capability-named (`parallel`, `serde`, `number-fields`). Verified by grep gates L4 and
   L5 (`plans/architecture.md` §1.3).
2. **A consumer plugs in by writing a thin adapter it owns.** resolvent ships no adapter
   crates and takes no optional dependency on any ecosystem crate. Integration is the
   consumer depending on resolvent, plus glue neither side owns.
3. **Where two consumers conflict, resolvent takes the general shape and the adapter
   absorbs the difference.** Each such conflict is recorded in §7.4 with the side that
   ate the cost named.

### 1.2 Governance rule 1 — the two-consumer rule, restated honestly

The previous statement was:

> core iff (a) two or more independent consumers need it, **or** (b) it is a general
> algebraic primitive any CAS would be expected to have.

X1 §3 enumerated all thirteen capabilities ever admitted under clause (b) and found that
**every one** was also wanted by a local consumer or named in `IDEAS-crates.md` §4, while
GF(p^k), public factorization over GF(p), RUR, multivariate resultants, Hermite/Smith
normal forms, p-adics and partial fractions — all present in Singular, Macaulay2, PARI,
Sage and Magma — were absent. Clause (b) never did independent work. The rule as operated
was "≥2 consumers ∪ 1 consumer ∪ spec-named".

**The rule, as it now binds:**

> A capability is **core** iff **(a)** two or more independent consumers need it, **or**
> **(b)** resolvent's own internals compute it anyway and exposing it costs no additional
> implementation, **or** **(c)** it survives the standing-CAS test in §4.2 — an
> admit/reject decision taken on the record against a list assembled from what every
> general-purpose CAS ships, *independent of who was surveyed*.
>
> One consumer plus none of (b) or (c) is evidence for an **adapter**, not for core.

Clause (b) is now falsifiable: "the Descartes/VCA test *is* a Bernstein coefficient sign
count" is a claim about resolvent's own code, and it either holds for a given item or it
does not. Where it holds for only part of an item, the item is split (§4.2, Bernstein).

Clause (c) is run once, in §4.2, and its output is a table of admissions **and
rejections**. A future addition argued on clause (c) must extend that table, not gesture
at it.

**Independence matters.** `/home/dev/sinbad/crates/sem1f-biquad-spike/` hand-rolls a
biquadratic ℚ-algebra and is the strongest algebraic-extension evidence in that
directory, but it is not a workspace member (`sinbad/Cargo.toml:5-45`) and it imports
`cadabra_geom`. It is counted once, under cadabra2. Counting it twice would manufacture a
two-consumer majority for algebraic extensions.

**Soft counts are marked.** Two items (`UPoly`, real root isolation) were previously
justified as "two consumers" where the second is sinbad's `crates/solverang` event
detection. That directory contains exactly `DESIGN.md` and `STATUS.md`; there is no
source (verified). Those counts read **1 shipping + 1 planned** in §4, and E1 §7 open
question 3 decides whether sinbad has any per-operation consumer at all.

**Two different things are called solverang.** `/home/dev/projects/solverang` is the CAD
constraint solver, legend **V**. `/home/dev/sinbad/crates/solverang` is sinbad's unwritten
DAE integrator. They are unrelated. Both are always written with their full path here.

### 1.3 Governance rule 2 — the 200-line adapter test, and how to report it

> For each consumer it must be possible to write the adapter in roughly **under 200
> lines, with zero changes to resolvent**. If an adapter would need resolvent to expose
> something new, resolvent's API is wrong — fix resolvent, do not special-case the
> consumer.

The test is reported as **two numbers, always**:

- **The seam** — the code that names a resolvent type or calls a resolvent function. This
  is the number the test is actually about, because it measures whether the consumer
  forces resolvent to expose something bespoke.
- **The total** — the seam plus consumer-side transcription and consumer-side vocabulary.
  This scales with the consumer's own domain and is byte-for-byte invariant to
  resolvent's API shape.

X1 §4 caught the earlier document reporting one number for cadabra2 (175, derived as a
prior estimate of ~250 minus four named savings summing to 77, with a block table that
sums to 175 — the sketch was sized to the subtraction and is therefore not independent
corroboration) while correctly reporting two for solverang. Both are reported both ways
here (§7), and the cadabra2 figure is restated at its **measured** range.

### 1.4 What this document does not do

It does not decide the ordering of work; that is `plans/roadmap.md`, with the corrections
in `docs/decisions/RECONCILIATION.md` §4. It does not decide algorithm choice; that is
`docs/research/algorithms-and-representation.md`. Where it states a *signature*, the
authority is this document plus the named ADR; where it states a *number*, the authority
is the evaluation that measured it.

---

## 2. The embedding model

### 2.1 What "directly embedded" means, and what it forbids

| Forbidden | The consumer that breaks, with evidence |
|---|---|
| `static mut`, `static` with interior mutability, `thread_local!` in any `publish = true` crate | sinbad D1 requires bit-identical output across thread counts (`sinbad/crates/sinbad-pal/src/repro.rs:20-21`). Any thread-local cache makes output a function of scheduling. |
| A `Ctx`/session/capability handle as a first parameter | sinbad's own conventions *want* one (`SINBAD-API-CONVENTIONS.md:86-90`) and resolvent still refuses: a session object owning every value is what makes a library un-embeddable. The adapter accepts `&Ctx` and drops it — ~5 lines (E1 §5.7). |
| Any I/O: `std::fs`, `std::net`, `std::env`, `std::time`, `std::process` | A CAS that reads the clock cannot be deterministic; one that reads the environment cannot be content-addressed (sinbad D6, artifacts BLAKE3-keyed at `sinbad/crates/rutter/src/lib.rs:11-14`). |
| Unseeded randomness | E1 D2 and E3 R2 independently demand it. Primes are a pure function of an index; evaluation points come from a counter-based RNG at index-derived positions (ADR-012). |
| `HashMap` iteration order in any position that can reach an output value | Same source. Ordering-visible positions use `BTreeMap` or a sorted `Vec`. |
| **A global allocator override** | Embedding must not fight the host's allocator choice. Note the narrowing: the earlier rule also forbade "an allocator parameter", which is backwards for an embedded consumer whose requirement is to *supply* the allocator (X1 §1.4). An allocator parameter is not offered today and is **not foreclosed**. |
| A lifetime parameter on any public **owned** type | A consumer must be able to store a resolvent value in its own struct without infecting that struct with a lifetime. Borrowed *views* (`RecursiveView<'a>`, iterators) are the only exception and are never returned by value from a constructor. |
| Panics on any input-dependent path | E1 D3 and E2 §6.8 both make this non-negotiable, and E2 shows the cost of violating it: `lazy_exact::SqrtExt::new` panics on a negative radicand (`arrangements/crates/lazy-exact/src/sqrt_ext.rs:38-41`) and cadabra2 hand-wrote a guard (`cadabra2/crates/cadabra-core/src/exact/radical.rs:77-88`). |

### 2.2 Arenas are caller-owned values (ADR-020)

Two arenas exist and they follow one rule.

**The L4 expression store.**

| Option | Verdict | Consequence |
|---|---|---|
| Global or thread-local interner | **Rejected** | Breaks bitwise reproducibility (output depends on interning history and therefore thread count) and content addressing (canonical bytes would depend on process-global insertion order). Two independent consumers in one process would share a table neither can reason about. |
| Per-call arena | **Rejected** | Destroys the only thing hash-consing buys. plexus differentiates the same equation set repeatedly across Pantelides rounds (E1 §1.2); cadabra2's certificate tether compares a claim built at mint time against one rebuilt later (E2 §11.2). |
| **Caller-owned `Store` value** | **Adopted** | `Store` is a plain owned struct, `Send`. `Expr` handles are `Copy`, store-relative, and never serialized. Canonical bytes are computed structurally, so two stores built by the same call sequence produce identical bytes and possibly different handles. |

**The L1 monomial arena.** `ADR-008` specifies an interned arena with a packed
order-normalized key and a divmask; the earlier API notes specified inline packed keys
and "no global monomial interner". These are reconcilable and are now reconciled in one
sentence: **the arena is owned by the `Ring` context value and is reached through it
explicitly; there is no global or implicit interner.** Whether terms carry
`(MonomialId, C)` into a shared arena or `(PackedMon, C)` inline remains open and is
settled by the microbenchmark in `plans/roadmap.md` §2.5 contradiction 2 — that
experiment decides the *term type*, not the ownership rule.

Because `MPoly` must be `Send + Sync` and must not carry a lifetime (§2.1), it carries its
ring by an **owned handle** (`Arc<Ring>` or an index into a caller-held ring table), never
as `&'a Ring`. `plans/architecture.md` §5.2's phrase "borrows `&Ring` by handle" is read
in that sense and should be restated in those words.

### 2.3 Handle identity, and the hazard that is not closed

Every entry point bounds-checks a handle and returns
`Error::Domain { fault: ForeignNode }` when it is out of range. **An in-range handle from
a different store yields a wrong answer, not an error.** The earlier justification for
accepting this — that a store tag "taxes every consumer for a bug none of the three would
make" — is falsified by two of the five outside consumers X1 evaluated: a Python binding
user with two `Store`s in one script makes the bug immediately, and any search loop that
rolls a store back makes it through a supported operation.

Three changes, all cheap now:

1. **`Store` growth is monotone for its lifetime, and L4 is not designed for a
   backtracking search loop.** Stated, not implied. An SMT consumer whose terms are
   polynomials stays on L1, where `MPoly` is a self-contained droppable value and serves
   backtracking well.
2. **`Store::rebuild_from(&Store, Expr) -> Result<Expr, Error>`** — the walk-and-rebuild
   that every parallel, multi-process, or distributed-cache consumer would otherwise
   write against `walk_topological`. ~30 lines, written once, in resolvent.
3. **An optional `store-tags` feature, default off.** `Store::with_tag(tag: u64)` records
   a **caller-supplied** tag; `Expr` carries it; every entry point checks it. Caller-
   supplied means no ambient counter and no violation of §2.1. The three local consumers
   ignore it; a binding author turns it on. If a checkpoint API is ever added, the tag
   becomes the generation counter and this is the mechanism that makes rollback safe.

### 2.4 The crate layout

The layout is `plans/architecture.md` §1.1 and `ADR-005`, unchanged, with one framing
correction. The earlier notes proposed a zero-dependency `resolvent-seam` crate pitched as
"the single highest-leverage hook" for the surrounding ecosystem. That crate does not
exist. `resolvent-base` carries `Sign`, `Verdict`, the `Ring` trait tower and its
orthogonal markers, `Error`, `Unsupported`, `Budget`, `Certified`, `Certainty`,
`ProofKind`, and the canonical serializer, and it is **resolvent's own base, not an
ecosystem standard**. §3.5 states why that framing change is load-bearing rather than
cosmetic.

`resolvent-base` has no third-party dependency except `thiserror`. A consumer that
implements `Ring` for its own coefficient type depends on `resolvent-base` alone and never
sees a version-pinned bignum in its tree (`plans/architecture.md` §1.2). Whether the crate
can be `#![no_std]` is an open item in §10: nothing in its contents needs `alloc`, and if
`Error` grows a `String` or a `Box` the claim fails — which is exactly why `Error` is
`String`-free by invariant (§9, INV-5).

---

## 3. The numeric-type seam

This is the hardest question in the design and the one on which the two founding tracks
disagreed outright.

### 3.1 Two questions wearing one coat

1. **What are a polynomial's coefficients?** ℤ, ℚ, GF(p), GF(p^k), ℤ/n, ℚ(α). Algorithms
   over them need *ring theory*: content and primitive part, exact division, gcd domains,
   Landau–Mignotte bounds, characteristic, reduction mod p, CRT lifting, bad-prime
   detection.
2. **What do you evaluate a polynomial at, and what does a generic algorithm text
   instantiate over?** An f64, an interval, a filtered lazy real, an exact rational, a
   dual number. These need only ring operations, sometimes a sign.

The earlier notes answered (1) with a **sealed set** `{Rational, Integer, FpElem, NfElem}`
and (2) with an **open six-method `Scalar` trait** in a zero-dependency crate.
`plans/architecture.md` §2.3 and `ADR-006` answered (1) with an **open trait tower** and
(2) with nothing at all.

### 3.2 The decision

> **One open trait tower, in `resolvent-base`, covering both questions. No second,
> ops-surface scalar trait, and no `resolvent-seam` crate.** (ADR-019)

```rust
pub trait Ring: Clone + PartialEq + Send + Sync + 'static {
    const LANES: usize;              // 1 for scalar rings; 4 for the batched tuple ring
    type Scalar: Ring;               // Self when LANES == 1
    fn zero() -> Self;  fn one() -> Self;
    fn add(&self, r: &Self) -> Self; fn sub(&self, r: &Self) -> Self;
    fn mul(&self, r: &Self) -> Self; fn neg(&self) -> Self;
    fn is_zero(&self) -> bool;

    // Defaulted in-place forms. A word-sized type ignores them; a bignum overrides them.
    fn add_assign(&mut self, r: &Self) { *self = self.add(r); }
    fn sub_assign(&mut self, r: &Self) { *self = self.sub(r); }
    fn mul_assign(&mut self, r: &Self) { *self = self.mul(r); }
}
pub trait CommutativeRing: Ring {}
pub trait Field: CommutativeRing { fn inv(&self) -> Option<Self>; }
pub trait EuclideanDomain: CommutativeRing { fn div_rem(&self, d: &Self) -> Option<(Self, Self)>; }
pub trait UniqueFactorizationDomain: CommutativeRing { /* content, primitive part */ }

// Orthogonal capability markers. Absence is a capability statement, not a defect.
pub trait Ordered:  Ring { fn sign(&self) -> Sign; }               // Integer, Rational. NOT Fp.
pub trait Reducible: Ring { type Image: Field;
    fn reduce(&self, m: &Modulus) -> Option<Self::Image>; }
pub trait Liftable:  Ring { fn crt_lift(images: &[Self::Image], moduli: &[Modulus]) -> Result<Self>; }
pub trait BulkOps:   Ring { fn axpy(dst: &mut [Self], a: &Self, src: &[Self]); /* … */ }
```

Three properties of this shape do the work:

**(a) It carries no obligation a word-sized type cannot discharge.** The argument
previously used to *reject* an open coefficient trait — that it "pushes bignum-shaped
obligations into a type whose entire purpose was to be word-sized" — is an argument
against a *badly factored* trait, and X1 §2 caught it being used six paragraphs later to
*justify* an open evaluation trait on the identical grounds. This tower is the well-
factored version: `Ring` is seven methods plus three defaulted ones, and the bignum-shaped
duties (`reduce`, `crt_lift`, content, primitive part) live in markers a word-sized type
simply does not implement.

**(b) The fast path is bounded by capability, not by identity.** The modular pipeline is
`where C: Reducible + Liftable`. A consumer's ring that cannot be reduced mod p **cannot
compile** into the fast path and gets the generic reference implementation instead. That
is "modular methods everywhere" as a type-level statement rather than a slogan, and it is
honest: the doc comment on `Ring` says in those words that a foreign `C` gets correctness,
not speed.

**(c) It has one seam, so a consumer's type is one impl.** cadabra2's `ExactScalar`
implements `Ring + CommutativeRing + Field + Ordered` and is done; there is no second
`Scalar`/`ScalarOrd`/`TryDiv` triple to also implement, and no question of which trait a
given generic text is written against.

`Ord` is **not** required on `Ring`. The batched tuple ring (four residues at once) has no
meaningful order, and requiring `Ord` would close that door permanently (ADR-006).
Sign-dependent algorithms carry `C: Ordered` explicitly.

**The consequence that must be stated plainly:** `Ring: Send + Sync + 'static` is a real
bound. A blanket impl from an ops-surface trait in a glue crate — `impl<T: SomeOpsTrait>
resolvent::Ring for T` — is legal only for thread-safe types. This is deliberate:
`MPoly<C>` must be `Send + Sync` (INV-13) and that requirement is downstream of the
determinism contract, not negotiable for the convenience of a blanket impl.

### 3.3 Why there is no separate evaluation-scalar trait

The earlier notes' `Scalar` was, in substance, `lazy_exact::exact::RingOps` with two
panics fixed (X2 §2.1 tabulates the correspondence:
`arrangements/crates/lazy-exact/src/exact/mod.rs:16-29, 58-72`). Shipping it would have
created a **fourth** scalar vocabulary in a workspace that already has three —
`scalar_seam::Scalar` (`arrangements/crates/scalar-seam/src/lib.rs`, 257 lines, zero
dependencies, MIT OR Apache-2.0 per `arrangements/Cargo.toml:9`),
`lazy_exact::exact::{RingOps, ExactRing, ExactField}`, and cadabra2's crate-private
`TierField` (`cadabra2/crates/cadabra-algorithms/src/fastpath/filter.rs:32`).

Three facts settle it:

1. **The attribution in the earlier notes was wrong, and the wrong attribution was the
   argument.** `scalar-seam` was described as something "cadabra2 already built … for
   exactly this". cadabra2 consumes it by path (`cadabra2/Cargo.toml:39`); it lives in
   `arrangements`, the very repository whose merge with resolvent is the deferred
   decision, and its own header says it exists so that `lazy-exact` and `~/sinbad` can
   both depend *down* on it with no repository cycle
   (`arrangements/crates/scalar-seam/src/lib.rs:5-17`).
2. **A competing vocabulary makes the deferred merge more expensive, not less.** Under
   the orphan rule a geometry crate cannot `impl resolvent::Scalar for lazy_exact::Real`
   unless it owns one of the two; neither repository owns both; a glue crate becomes
   mandatory and nobody owns it. The earlier claim that "nothing in this document makes
   that merge more expensive" was false, and X2 §2.4 calls that sentence the most
   dangerous line in the plan because it licenses future agents to stop thinking about it.
3. **The two traits are not the same kind of object.** `RingOps` is explicitly an *ops
   surface* and "not an algebraic claim" — `Interval` implements it too. resolvent's
   `Ring` is an algebraic claim: `Field::inv` means a multiplicative inverse, not a
   best-effort division. Two similarly-named traits with different contracts across an
   adapter boundary is a bug generator (`plans/architecture.md` §5.6, ADR-018 §6.4).

So: `Interval<f64>` is not a `Ring` and resolvent does not ship one (ADR-015). A consumer
that wants one algorithm text at three tiers — cadabra2's de Boor, rendered at T0/T1/T2 —
keeps `scalar-seam` for its float and interval tiers and implements `resolvent::Ring` for
its exact tier. resolvent's generic texts (Horner, Bareiss, de Casteljau, sign ladders)
instantiate at the exact tier. **The adapter absorbs the difference, which is what rule 5
requires.**

### 3.4 The boundary rules

- **The only f64 ingress is `Rational::try_from_f64` — exact dyadic, `Err` on NaN/±∞.**
  No "nice rational" heuristic exists, ever. E3 §5 shows why: silently turning `sin(30°)`'s
  f64 into `1/2` analyses a different system than the one the user authored.
- **The only f64 egress is `approx_lossy()` and the outward-correct `enclosure()` pair.**
  No `impl From<X> for f64`, no `as`, no `Display` that rounds. See INV-7 for what
  `Display` *is* permitted to do.
- **Lift-then-operate is the only expressible order.** No `Rational::from_f64(a - b)`-shaped
  convenience exists. cadabra2 codified this as SEM-0 and enforced it structurally
  (`cadabra2/crates/cadabra-geom/src/exact/algebra.rs:36-45`); resolvent does not hand back
  the footgun.
- **Homomorphisms are applied to polynomials, not inside evaluation loops.** This is a
  boundary rule *and* a type-level fact:

  ```rust
  impl<C: Ring> UPoly<C> {
      pub fn map_coefficients<D: Ring, E>(&self, f: impl Fn(&C) -> Result<D, E>)
          -> Result<UPoly<D>, E>;
      pub fn eval_horner(&self, at: &C) -> C;      // same ring. No hom parameter.
  }
  impl MPoly<C> { /* the same two, plus `derivative(var)` */ }
  ```

  There is no `evaluate_with(hom, point)`. X1 §5.2 caught the earlier acceptance sketch
  folding a ℚ→GF(p) reduction into the innermost column loop — a bignum reduction on
  numerator and denominator plus a modular inverse of the denominator, per coefficient per
  evaluation point — for the one consumer whose entire case is "no bignums appear
  anywhere; all arithmetic is single-word modular" (E3 §3.2), inside a per-edit loop that
  a MUS extraction calls O(k) times (E3 §4.1). Removing the signature removes the idiom.
- **Which algorithm texts are generic** is a closed list fixed at design time and not grown
  casually: Horner evaluation, Bernstein/de Casteljau, Bareiss determinant, dense row
  echelon, matrix multiplication, sign-of-expression ladders. Everything else is
  monomorphic (ADR-006 Tier M). This is the compile-time budget, written down.

### 3.5 What this costs, and what it buys the deferred integration decision

The cost is real and is recorded rather than argued away. resolvent no longer offers the
surrounding ecosystem a numeric vocabulary to standardize on. A geometry crate that wants
"write the algorithm once, run it at f64 / interval / exact" gets that from
`arrangements/crates/scalar-seam`, which already exists, is zero-dependency, is the same
license, and is already consumed by two repositories. resolvent's contribution to that
picture is a set of *algebraic* traits and a set of algorithms, not a scalar standard.

What it buys: the overlap inventory below stays a merge rather than a fork. These are the
five landed, first-party, MIT-OR-Apache-2.0 components resolvent's plan would otherwise
duplicate, with the one design decision per item that determines merge cost.

| resolvent item | Incumbent | Lines | The decision that sets merge cost |
|---|---|---|---|
| filtered eager-interval / lazy-exact real | `arrangements/crates/lazy-exact/src/real.rs` | 724 | resolvent does **not** ship one (§4.1, L0-11). Root isolation's internal filter is a private module, never a published tier. |
| `Interval<f64>` with directed rounding, no global FPU mode | `lazy-exact/src/interval.rs` | 431 | resolvent ships **no float interval type** (ADR-015). One enclosure semantics at the boundary, and it is the consumer's. |
| Bernstein coefficients, exact de Casteljau, certified range | `lazy-exact/src/bernstein.rs` (`from_power:43`, `range_bound:108`, `range_interval:123`, `sign_over:135`, `subdivide_at:157`, `subdivide:193`) | 298 | resolvent's method set matches the incumbent's **deliberately**; the deviations are the fixes (rational endpoints not an interval type; `Verdict<Sign>` not a guess; a tracked Unknown-rate ceiling). |
| forward-mode `Dual<S>` | `arrangements/crates/scalar-seam/src/dual.rs` | 412 | resolvent does not ship one (§4.1, L0-10). It is a generic construction over whichever scalar seam the consumer already has. |
| `AlgebraicReal` | `lazy-exact/src/roots.rs:317-322` `RealRoot { poly, lo, hi, multiplicity }` | — | Field-for-field identical except that multiplicity is a pair element, not a field (ADR-014 §3), and refinement is `&self` on an `Arc<Inner>` rather than `&mut self` (ADR-013). Both deviations are the fix. |

A merge is then "take resolvent's version of these five decisions", not "reconcile two
designs". Whether `resolvent-base` should ever supersede, wrap, or coexist with
`arrangements/crates/scalar-seam` remains open (§10); **coexistence is the default, is the
most expensive of the three, and is chosen by inaction.**

---

## 4. Core vs adapter, capability by capability

Legend: **S** = sinbad, **C** = cadabra2, **V** = `/home/dev/projects/solverang`.
"1 shipping + 1 planned" marks a soft count per §1.2.

### 4.1 The capability table

#### Layer 0 — coefficient rings

| # | Capability | Wanted by | Placement | Rule | Rationale |
|---|---|---|---|---|---|
| L0-1 | `Integer`, `Rational` with `sign()` and by-reference arithmetic | S, C, V | **core** | (a) | The floor of the library. By-reference arithmetic is E1 §1.4's explicit ask (`sinbad/crates/meshwright/src/predicates.rs:33-70`). |
| L0-2 | `Rational::try_from_f64` — exact dyadic, typed failure on non-finite | S, C, V | **core** | (a) | Three consumers. No heuristic sibling, ever (§3.4). |
| L0-3 | `num_bits()`/`den_bits()` and explicit `round_to_f64_grid()` | S, C | **core** | (a) | meshwright snaps circumcenters to 53 bits by hand because unbounded growth is a *measured* performance failure (`sinbad/crates/meshwright/src/triangulate.rs:500-512`). Policy stays with the caller. |
| L0-4 | The three exits: `demote_exact` / `enclosure` / `approx_lossy` | C explicit, S as D5 | **core** | (a) | E2 §10.2 calls this "the single most transplantable design decision in the consumer" and it is clippy-enforced there (`cadabra2/crates/cadabra-core/src/exact/mod.rs:40-46`). `enclosure` returns `(f64, f64)` outward-correct, not an interval type (ADR-015). |
| L0-5 | A public float `Interval` type | C (already owns one) | **out of scope** | — | ADR-015. Two enclosure semantics at one adapter boundary produce a wrong *verdict*, not a wrong number. The incumbent is 431 lines and works. resolvent's internal dyadic filter is private. |
| L0-6 | `Fp` — prime field, **runtime** modulus, `Copy` word-sized elements | V explicit; C explicitly does not want it user-facing | **core, public** | (b) | Every modular method needs it internally; making it public costs zero marginal implementation. E3 §5's ask is that it be callable with **no forced CRT/rational-reconstruction lift** — solverang never wants the ℚ answer. cadabra2 pays nothing by not importing the module. |
| L0-7 | Seeded uniform random points over GF(p) | V explicit, S as a prohibition on the alternative | **core** | (a) | Schwartz–Zippel, sparse interpolation and modular gcd all need it. The seed is a parameter, never ambient. |
| L0-8 | `GF(p^k)` | none of the three | **core, public** | (c) | §4.2. Already built by `plans/architecture.md:57` and scheduled at `roadmap.md:90`. Zero marginal implementation given the modular layer. Its absence was the single defect that made a cryptography consumer fail the 200-line test outright (X1 §1.3). |
| L0-9 | `Zn` for composite n | none | **core** | (c) | Same lane, same argument, already scheduled. Cheap, and its absence is arbitrary once GF(p^k) is in. |
| L0-10 | `SqrtExt` — `a + b√r` with a total order **across distinct radicands** | C | **core** | (b) | Not merely a degree-2 convenience: `arrangements/crates/arrangements/src/geoms/circle_segments.rs` is 931 lines that use `SqrtExt` exclusively and never import `RealRoot` or `QPoly`, and `cmp_cross` has 31 call sites (ADR-014 §4). Subsuming it into `AlgebraicReal` would be a large silent regression on the cheapest and most common case. |
| L0-11 | `NumberField` — ℚ(α) with a known minimal polynomial, degenerate-tower detection | C (three fail-closed sites) | **core, `number-fields` feature** | (b)(c) | One consumer, but it is what unblocks *number-field linear algebra* with no new resolvent linear algebra (§4.4, L2-7) and it needs factorization to exist anyway. It closes cadabra2's largest fail-closed site (`cadabra2/crates/cadabra-algorithms/src/intersection/quadric/classification.rs:78-87`, `:441-445`). |
| L0-12 | Forward-mode `Dual<S>` | C (eventual) | **adapter** | — | A generic construction over whichever scalar seam the consumer has. It already ships at `arrangements/crates/scalar-seam/src/dual.rs` (412 lines). |
| L0-13 | Lazy filtered real (eager interval + lazy exact DAG) | C, which already owns one | **out of scope** | — | A *strategy*, not an algebraic object. `lazy-exact/src/real.rs` is 724 lines and designs out the CGAL failure modes. resolvent's root isolation uses a private eager filter, which is not a published tier. |
| L0-14 | A bignum ℚ offered as a units/exponent type | nobody — actively wrong | **out of scope** | — | `league::Exp` is `{num: i16, den: i16}`, `Copy`, `const fn` gcd, frozen wire form (`sinbad/crates/league/src/exp.rs:14-17`). resolvent's `Rational` is the wrong type and must not be advertised as one. |

#### Layer 1 — polynomials

| # | Capability | Wanted by | Placement | Rule | Rationale |
|---|---|---|---|---|---|
| L1-1 | `UPoly<C>` dense univariate, defined **before and independently of** the multivariate type | C (every path), S (1 shipping + 1 planned) | **core, first** | (a)(b) | The entire consumer-unblocking surface is dense univariate. Defining it standalone is what lets the multivariate/F4 program run on a track that never blocks the consumer track (ADR-007). |
| L1-2 | `MPoly` sparse distributed, packed exponents, **runtime arity** | V (blocked on runtime arity), C (weakly) | **core** | (a) | The one-way door. E3 R9 is binding: per-constraint arity runs 2..14 (`solverang/src/sketch3d/constraints.rs` `Parallel3D` 12 params; `assembly/constraints.rs` `Insert` 14), so a const-generic arity makes an adapter that builds rings from constraint data impossible. |
| L1-3 | The `Ring` context carried by an **owned handle**, never by `&'a Ring` | V, embedding | **core** | (a) | §2.2. A lifetime on `MPoly` would infect every consumer struct that stores one. |
| L1-4 | Monomial arena owned by the `Ring` context, reached explicitly | embedding | **core** | (b) | §2.2, ADR-020. Whether terms are `(MonomialId, C)` or `(PackedMon, C)` is the open experiment, not the ownership rule. |
| L1-5 | `derivative(var)` | V | **core** | (b) | Textbook. It also halves solverang's transcription: the adapter transcribes residuals and resolvent produces the Jacobian (E3 R4). |
| L1-6 | `map_coefficients` — the only cross-ring path, fallible | V, and every modular algorithm | **core** | (a)(b) | §3.4. Fail-closed when `p ∣ den`, not silent (E3 R3). |
| L1-7 | `eval_horner` at a point in the same ring | S, C, V | **core** | (a) | |
| L1-8 | `total_degree()` | V | **core** | (b) | Trivial, and it is *all* Bézout counting needs — which is why Bézout gets no dedicated API (E3 §4 #2). |
| L1-9a | Bernstein coefficients + exact de Casteljau + certified range enclosure, **univariate** | C (lift-now) | **core** | (b) | The Descartes/VCA test *is* a Bernstein coefficient sign count, so resolvent computes these anyway. E2 §4.2 is emphatic that naive interval evaluation provably rejects true identities (`cadabra2/docs/notes/design/whole_carrier_enclosure.md:29-46`). |
| L1-9b | The same, **bivariate over a rational box** | C | **core, later, on its own merits** | (a)? | Split out per X1 §3: bivariate de Casteljau over a box is *not* computed by univariate root isolation, so clause (b) does not cover it. One consumer. Decide when L1-9a ships and the torus lane's shape is known. |
| L1-10 | `RecursiveView<'a>` for subresultant PRS | internals | **core** (borrowed view) | (b) | A view, not an owned tree. |
| L1-11 | Kronecker substitution | internals | **core** utility | (b) | Not a representation. |
| L1-12 | `RatFunc` | nobody | **out of scope** | — | E3 §5: the two rational residuals are cleared by the adapter, which records the extraneous `len_sq = 0` factor itself. |

#### Layer 2 — the engine

| # | Capability | Wanted by | Placement | Rule | Rationale |
|---|---|---|---|---|---|
| L2-1 | Real root isolation over ℚ with **multiplicities**, over an optional window, under a budget | C (lift-now), S (1 shipping + 1 planned) | **core** | (a)(b) | Multiplicity comes back as a pair element `(AlgebraicReal, u32)` (ADR-014 §3): available without recomputation, which is cadabra2's actual requirement (a double radicand root *is* the sheet-junction signature, `cadabra2/crates/cadabra-core/src/exact/algebraic.rs:106-108`), while remaining impossible to tie-break a comparison on. The window matters: sinbad isolates only within `[t_n, t_n+h]`. |
| L2-2 | Yun square-free decomposition, `gcd`, `gcd_ext`, `div_rem`, `square_free_part` | C, plus every internal path | **core** | (a)(b) | |
| L2-3 | `resultant` / subresultant PRS eliminating one variable | C (next milestone) | **core** | (b)(c) | resolvent needs it internally for degree bookkeeping and curve topology. E2 §3.5 is the largest net-new demand: the unbuilt torus lane is pure resultant work. `arrangements` currently reaches degree ≤ 8 by *double squaring* because no general resultant was available — that is the shape of the hole. |
| L2-4 | Univariate factorization over ℚ (Zassenhaus, then van Hoeij) | C (degree-4 plane curve) | **core** | (b)(c) | A hard prerequisite for L0-11 (minimal polynomials) and for `canonicalize()`/`Hash` on algebraic numbers. E2 §3.4 is the cleanest single lift in the whole evaluation: one general capability replaces three hand-coded circle strata *and* covers the generic case they were carved out of. |
| L2-5 | Factorization over GF(p) — distinct-degree / equal-degree, public | none of the three | **core, public** | (b)(c) | An internal step of L2-4, so the code exists; the same "zero marginal implementation" argument that made L0-6 public applies and was simply not applied. It is also the one factorization with a **complete** certificate (irreducibility over a finite field is decidable and cheap). |
| L2-6 | Dense `row_echelon` over a field returning **rank, pivot rows, dependent rows, and the transform** | V explicit, C (ℚ linear algebra) | **core, public** | (a) | The transform is not a bonus: it is solverang's `implied_by` certificate, shipped unconditionally empty at two sites (`solverang/src/system.rs:803` and `solverang/src/pipeline/analyze.rs:98`), and it is the same object as a Gröbner cofactor representation. **No lane currently builds this** — see RECONCILIATION §4. |
| L2-7 | Fraction-free (Bareiss) determinant over an integral domain, incl. ℚ[λ] | C (2.448 ms recursive Laplace today) | **core** | (b) | The same routine L2-6 needs over non-fields, and modular determinants are internal to L2-3. **Primes must not appear in the signature**: cadabra2 asks for a fast exact determinant, and modular is *how* you give it one, not *what* it asked for (E2 §4.3). |
| L2-8 | Sylvester inertia / congruence diagonalization | C only | **adapter** | — | The rule working correctly. cadabra2 already has 49 lines of it (`classification.rs:292-338`). Once `Ring`/`Ordered` and `NumberField` exist, that *existing* routine becomes generic and instantiates at ℚ(α) **for free** — closing cadabra2's largest fail-closed site with **zero new resolvent API**. |
| L2-9 | Rank of a polynomial matrix at an algebraic root, by minor vanishing | C only | **adapter** | — | ~20 lines over `is_root_of` and L2-6. cadabra2 already wrote it (`classification.rs:242-260`). |
| L2-10 | Factorization of a quadratic form into linear factors over ℚ / ℚ(√d) | C only | **adapter** | — | Diagonalize (L2-8) + square-root detection (L0-11) + split. Two lines once L0-11 exists. Replaces cadabra2's guessed factor pair plus ten-coefficient identity check (`carrier_cylinder_cylinder.rs:216-222, 561-596`). |
| L2-11 | Gröbner / F4, ideal membership, Nullstellensatz certificate | V (eventual, gated); C zero; S zero | **core, explicitly not in the first fan-out** | (c) | E3 §0.6 is unambiguous: solverang's algebra demand is gated behind a Laman/DR-planner decomposition it has not begun, and a whole-sketch cluster is ~250 quadratics — intractable for anyone's engine. **Do not build F4 for solverang.** Build it because a CAS has one, after L0–L3 close real consumer sites. **The Nullstellensatz certificate `1 ∈ ⟨f₁…f_k⟩` decides *complex* inconsistency only; real infeasibility is not decided by it and resolvent ships no Positivstellensatz.** See §4.3. |
| L2-12 | Topology of a real bivariate curve `G(a,b)=0` | C (next milestone) | **core, later** | (b)(c) | E2 §13 flags it as genuinely unsettled whether the torus lane needs this or whether L2-3 suffices. Working one generic plane×torus case by hand settles it. Do not build it first. |
| L2-13 | RUR / primitive element for 0-dimensional real solving | none of the three | **core** | (c) | §4.2 and §8.2. Already in `plans/architecture.md:68` and M8. It is the mechanism that keeps `AlgebraicReal` ℚ-only *and sufficient* for a multivariate sample point. |
| L2-14 | Multivariate factorization at scale | nobody — both C and V reject it | **core, post-v1** | (c) | E2 §6.2: the only factorizations wanted anywhere are degree ≤ 4 in ≤ 3 variables. Sequence it last. |
| L2-15 | BKK / mixed volume root counting | V, rejected by V | **out of scope** | — | Convex geometry over Newton polytopes, not algebra. It belongs in a polytope crate. |
| L2-16 | Numeric root polishing, Newton/corrector, homotopy | nobody — C calls it an "attractive nuisance" | **out of scope** | — | `cadabra2/.../quadric/roots.rs:11-12` exists precisely so "no numeric root polishing enters the decision path". E3 §4 #5: continuation is a Davidenko-ODE predictor-corrector needing only the residual and Jacobian solverang already has. |
| L2-17 | Interval-Newton / Krawczyk | V names it as the right tool for a job resolvent should not do | **out of scope** | — | A numerics library, not a CAS. |
| L2-18 | Any API taking `eps: f64` for an equality or sign decision | nobody — forbidden | **out of scope** | — | Equality by tolerance is intransitive and is *the* canary failure of exact arithmetic (ADR-011). |
| L2-19 | Full CAD — projection operators, discriminant/psc chains as user-facing objects, cell decomposition | none of the three | **out of scope for v1; the pieces are core** | (c) | §4.2. Subresultant chains, principal subresultant coefficients and RUR are admitted individually (L2-3, L2-13, M8). Assembling them into a CAD is a large component with its own literature and no local consumer; an SMT consumer that wants one builds it on the admitted pieces. |

#### Layer 3 — algebraic numbers

| # | Capability | Wanted by | Placement | Rule | Rationale |
|---|---|---|---|---|---|
| L3-1 | `AlgebraicReal { poly, isolating bounds }`, `Arc`-backed, `Send + Sync`, `&self` monotone refinement, total `Ord` | C (inner loop); S no; V no | **core** | (b)(c) | **One consumer**, stated plainly. Core status rests on clause (c) plus its being resolvent's headline differentiator, not on consumer count. Thread-safety is decided in ADR-013 and is `Send + Sync` — see §9 INV-15 and §8.5. |
| L3-2 | Construction is fail-closed | invariant | **core** | (b) | `new(poly, lo, hi)` returns `Err` if `poly` is not square-free, if `(lo,hi)` does not isolate exactly one root, or if `poly(lo) == 0` or `poly(hi) == 0`. This makes the query surface total (ADR-011 §6). |
| L3-3 | `is_root_of(h)`, `sign_of(h)` | C | **core** | (a)(b) | The sign query settles zero-ness **algebraically first** (gcd) before entering any refinement loop, or it hangs rather than answering. A hang in a library is worse than a wrong answer because it is undebuggable in production. |
| L3-4 | `rational_between` — a certified strictly-between witness | C — hand-rolled **twice in two crates of one consumer** | **core** | (b) | `cadabra2/crates/cadabra-arrange/src/trim.rs:842-854` and `cadabra2/crates/cadabra-geom/src/exact/harmonic.rs:793-818`. Two independent implementations of one primitive inside a single consumer is the clearest possible signal it belongs in the library. Both carry a hard 256-step budget. |
| L3-5 | Sign of an element of a real radical tower at an algebraic abscissa, **at arbitrary depth** | C (inner loop) | **core, in general form** | (b) | Exposed as *sign of Σ cᵢ(α)·√hᵢ(α)*, not as `sign_radical2`. cadabra2's two-radical ladder is a two-line instantiation. This is what keeps predicates in degree 4 instead of 65536. |
| L3-6 | General field arithmetic on `AlgebraicReal` | nobody asked | **opt-in module, loudly documented** | (c) | `α + β` has degree ≤ `deg α · deg β`; three operations take degree 4 to 65536 without a factorization after each step. The documented fast path is L3-5; `tower::materialize` is the general fallback and says its cost in its doc comment (ADR-014 §2). |
| L3-7 | `Hash` on `AlgebraicReal` | nobody | **out of scope; `CanonicalAlgebraicReal` has one** | — | `x²−2` and `x⁴−4` are equal numbers with different polynomials. A "cheap" `Hash` silently puts two entries in a map for one number and shows up as nondeterminism in a consumer, never in a unit test. `canonicalize()` costs a factorization and says so (ADR-014 §1). |
| L3-8 | Algebraic numbers over an extension — `UPoly<NumberFieldElem>` plus a second algebraic-number type | none of the three; the SMT consumer | **core, added instantiation** | (c) | Because `UPoly<C>` is generic from day zero (ADR-006, ADR-014 §5), this arrives as an *instantiation*, not as a rewrite of `AlgebraicReal`. That is what turns X1 §1.2(c) from a breaking change into an additive one. |

#### Layer 4 — expressions

| # | Capability | Wanted by | Placement | Rule | Rationale |
|---|---|---|---|---|---|
| L4-1 | Hash-consed `Store` with `Expr` handles and **structural** equality | S, C | **core** | (a) | They want it for opposite reasons — sinbad for a canonical content address, cadabra2 for a certificate tether a canonicalizer would *break*. §7.4 conflict 1. |
| L4-2 | Open, **caller-owned** `FuncTable`: `Apply(FuncId, args)` with a per-function derivative rule | S (sin/sinh/exp/cos), C (opaque `Cos2`, `Radical` with *no* rule), V (must not see `Atan2`) | **core** | (a) | The synthesis. One mechanism serves all three because resolvent ships **no transcendental semantics in core** — only a table the caller constructs. `FuncTable::standard_elementary()` is a constructor, not a default; `FuncTable::empty()` plus `register` is what cadabra2 uses; solverang never builds one, so `Atan2` structurally cannot appear in its world. |
| L4-3 | `diff` and `diff_with(expr, sym, &LeafRules)` | S (plexus is a stub blocked on exactly this) | **core** | (b) | Differentiation with respect to an implicit variable, where the derivative of an unknown is a *new* unknown the caller mints, is textbook CAS. Plain `diff` cannot express it. Signature changed from sinbad's ask — §7.4 conflict 6. |
| L4-4 | `walk_topological` with stable ids | S | **core** | (a) | Shared-subexpression let-binding falls out of hash-consing for free and is the main value the DAG adds over a tree. |
| L4-5 | `is_polynomial_in(&syms) -> Option<MPoly>` — the L4→L1 bridge as a **predicate** | S (speculative), C (morally) | **core** | (b) | A coercion would lie; a predicate cannot. |
| L4-6 | `canonical_bytes(expr)` + `SCHEMA_VERSION` | S (content addressing), C (certificate tether) | **core** | (a) | Independent of interning order, handles, arena addresses, insertion history and build configuration. A canonical-form change is a re-key event for every downstream artifact, so it is versioned explicitly. |
| L4-7 | `Store::rebuild_from(&Store, Expr)` | no local consumer; ≥2 hypothetical | **core** | (b) | §2.3. Otherwise every parallel, multi-process or distributed-cache consumer writes the same walk-and-rebuild. ~30 lines, written once. |
| L4-8 | A code emitter (Rust/C/WASM printer) | S wants one and says resolvent must **not** ship it | **out of scope** | — | sinbad needs Rust closures; the next consumer needs its own opcode tape. resolvent exposes L4-4 and stops. |
| L4-9 | e-graph / equality-saturation simplifier | nobody — C actively hostile, S says "anvil should call `egg` directly" | **out of scope for core; external glue** | — | cadabra2 keeps `Cos2` as a first-class atom *deliberately* rather than rewriting it to `2cos²t−1` (`cadabra2/crates/cadabra-check/src/carrier.rs:162-165`); a canonicalizing e-graph rewrites exactly the thing that must be left alone. anvil's want is Herbie-style FP-accuracy rewriting whose rewrites **change the computed value** — resolvent's must not (ADR-017). |
| L4-10 | A general `simplify()` | nobody | **out of scope** | — | `canonicalize()` exists, is explicit, opt-in, and is defined as *value-preserving normalization*, not cleverness. |

#### Cross-cutting

| # | Capability | Wanted by | Placement | Rule | Rationale |
|---|---|---|---|---|---|
| X-1 | `Budget` on every loop with no a-priori termination proof, `BudgetExhausted` distinct from malformed input | S (D4 rungs), C (two 256-step witness loops) | **core** | (a) | sinbad's `Decline` "always means the next rung may succeed" (`sinbad/crates/tiered-core/src/rung.rs:11`); cadabra2's budgets "turn a surprise into a typed refusal instead of a hang". Note the two regimes in §9 INV-6: where a proven bound exists, the budget is a bug detector and the query stays total. |
| X-2 | Small closed error enum: `PartialEq + Clone`, no `Box<dyn>`, no `String`, offending data on the variant, structured `Unsupported` | C explicit, S (maps to `DiagCode`) | **core** | (a) | Consumers `matches!` on it in tests, which `String` payloads defeat. It must **not** try to be the consumer's kernel error type — cadabra2's existing 6-line `From<lazy_exact::…>` (`cadabra-core/src/exact/algebraic.rs:81-90`) is the proof that upward mapping is cheap. |
| X-3 | Typed `Unsupported` refusal naming the missing capability | C (CORE RULE), S (D4) | **core** | (a) | "Even if a fallback is implemented, it should still fail if the main algorithm is not." |
| X-4 | `Certified<T>` with `Certainty::{Proved, Probable}` on every modular result | S's grading, C's `ProofStrength`, and resolvent's own design rule | **core** | (a) | §5. `Probable` is allowed to exist (Gröbner over ℚ needs it) but must be visible in the type, and the default path is `Proved`. |
| X-5 | Certificates: private fields, crate-private mint, carry the claim, `certifies(claim)`, public read accessors | C, S | **core** | (a) | §5. |
| X-6 | Determinism: no ambient RNG, index-addressed primes, seeded counter-based points, ordered combination | S (D1/D2 hard), V (R2 hard, cross-platform) | **core** | (a) | Two consumers with independent hard requirements (ADR-012). |
| X-7 | `#![forbid(unsafe_code)]` above one auditable leaf | S (D7), C (§10.10) | **core** | (a) | 11 of 28 sinbad library crates already forbid it; both `lazy-exact` and `cadabra-check` do. |
| X-8 | Warm-start telemetry (tier reached, bisections spent, precision attained) as plain data with **no proof type attached** | C, plus the general argument | **core** | (b)(c) | The general argument, which the earlier notes omitted: **any library with a tiered cost model wants performance metadata cacheable without the trust boundary following it.** cadabra2 is the instance — `cadabra-hints` is defined by "a hint is never evidence … hint values need no unforgeability" (`cadabra2/crates/cadabra-hints/src/lib.rs:19-47`) — but the property is general, and the type system is what keeps the two apart. |
| X-9 | An adapter crate for any consumer | nobody | **out of scope** | — | resolvent ships no adapter crates and has no optional dependency on any ecosystem crate. |

### 4.2 The standing-CAS test, run once, on the record

Clause (c). The list is assembled from what Singular, Macaulay2, PARI, Sage and Magma
ship, deliberately **without** reference to the three local consumers. Each is admitted or
rejected here; a future addition on clause (c) extends this table.

| Capability | Verdict | Reason |
|---|---|---|
| GF(p^k) | **admit** (L0-8) | Zero marginal implementation over the modular layer that is already core. Its absence blocked an entire consumer class from the 200-line test. |
| ℤ/n for composite n | **admit** (L0-9) | Same lane. Arbitrary to exclude once GF(p^k) is in. |
| Public factorization over GF(p) (Cantor–Zassenhaus, DDF/EDF) | **admit** (L2-5) | Already an internal step of factorization over ℚ. The one factorization with a complete certificate. |
| RUR / primitive element | **admit** (L2-13) | The mechanism that makes a ℚ-only `AlgebraicReal` sufficient for a multivariate sample point. Without it, §8.2's consumer needs a breaking change to the headline type. |
| Subresultant chains + principal subresultant coefficients as **returned data** | **admit** (L2-3, M8) | Already computed; returning the chain rather than only the resultant costs nothing and is what a CAD or an SMT projection needs. |
| Multivariate resultants / discriminants | **admit as post-v1** (L2-3 generalization) | Genuinely general, genuinely large, and no consumer. Sequenced after the bivariate case closes real sites. |
| Full CAD: projection operators, cell decomposition, sample-point lifting | **reject for v1** (L2-19) | A large component with its own literature and its own failure modes, and everything it is built from is admitted individually. An SMT consumer assembles it; resolvent supplies the pieces. Recorded as a rejection so that "resolvent does CAD" is never claimed. |
| Positivstellensatz / SOS certificates of **real** infeasibility | **reject** | Real infeasibility is not decided by anything resolvent ships. Stated here so the Nullstellensatz claim (§4.3) cannot be quietly widened. |
| Hermite and Smith normal forms | **reject** | Integer-matrix theory. No consumer, not needed by any admitted algorithm, and additive later on top of L2-6/L2-7. |
| p-adic numbers as a public ring | **reject** | Hensel lifting is an internal step of factorization and stays internal. A public p-adic ring is a different library. |
| Cyclotomic fields as a distinct type | **reject** | A constructor over `NumberField`, not a capability. `NumberField::cyclotomic(n)` is a helper a consumer writes in three lines. |
| General polynomial decomposition (`f = g ∘ h`) | **reject** | `compose_affine` is admitted (it is the Taylor-shift/scaling primitive root isolation needs). General functional decomposition has no consumer and no internal need. |
| Partial fraction decomposition | **reject** | Derivable by the caller from `gcd_ext` and `div_rem`, both core. Shipping it would be a convenience, not a capability. |
| Symbolic integration, limits, series | **reject** | The source spec calls symbolic calculus "a thin layer on top, not the point". Out of scope permanently, not merely deferred. |

Rejections are as load-bearing as admissions. Six of fourteen were rejected, which is the
evidence that clause (c) is now doing work rather than ratifying whatever was asked for.

### 4.3 One correction that prevents a milestone being justified on a capability that does not answer the question

E3 §4 #10 called the Nullstellensatz certificate `1 ∈ ⟨f₁…f_k⟩` "a *proof* of
unsatisfiability … stronger diagnosis than the licensed reference oracle produces", and
the earlier API notes carried it forward as "the most attractive *new* capability found in
E3". X2 §1.3 refutes both halves and it is correct on both.

**Mathematically**, the Nullstellensatz certifies emptiness of the **complex** variety.
CAD over-constraint is routinely complex-satisfiable and real-unsatisfiable. In
solverang's own vocabulary: fix `A = (0,0)` and impose `DistancePtPt(A,B) = 1`,
`DistancePtPt(B,C) = 1`, `DistancePtPt(A,C) = 5` — three squared-distance residuals
(`solverang/src/sketch2d/constraints.rs:18`) in four unknowns. No real configuration
exists, but the complex variety is positive-dimensional and non-empty, so `1 ∉ ⟨f⟩` and
**no certificate exists**. E3's worked example (distance = 10 versus distance = 7 on the
same pair) lands in the trivial subclass, where the difference of the two residuals is the
unit 51.

**Factually**, it would not beat the reference: `solverang/TODO.md:206-214` and
`solverang/tests/differential_oracle.rs:305-313` show D-Cubed getting the case *right* and
solverang getting it wrong, for a status-classification bug solverang's own TODO says to
fix in the status enum. E3 §4.1 states this correctly and §4 #10 then contradicts it.

**The API consequence** is one sentence in L2-11 and it is already there: the certificate
decides complex inconsistency only.

### 4.4 One capability the placement rule pushes out, and why that is the rule working

Sylvester inertia over ℚ(α) (L2-8) is cadabra2's largest fail-closed site
(`classification.rs:78-87`, `:441-445`) and it is placed in the **adapter**. That is not a
refusal to serve the consumer. cadabra2 already owns 49 lines of exact congruence
diagonalization over ℚ (`classification.rs:292-338`); once the `Ring`/`Ordered` tower and
`NumberField` exist, that *existing* routine becomes generic and instantiates at ℚ(α) with
**zero new resolvent API**. Putting `inertia` in core would add a public routine one
consumer calls and would not close the site one day sooner.

---

## 5. The certificate API

### 5.1 Shape

```rust
pub struct Certified<T> { pub value: T, pub certainty: Certainty }

pub enum Certainty { Proved(ProofKind), Probable(ProbableReason) }
pub enum ProofKind {
    BoundDriven { bound_bits: u64, primes_used: u32 },  // Landau–Mignotte / Hadamard
    DivisibilityAndDegree,                              // the gcd certificate
    CofactorRepresentation,                             // Gröbner: f = Σ hᵢgᵢ
    Identity,
    Enclosure,
    RootCount,                                          // see §5.3
}

pub struct Certificate<C: Claim> {
    claim:     C,           // private
    evidence:  C::Evidence, // private
    certainty: Certainty,   // private
}

impl<C: Claim> Certificate<C> {
    pub fn claim(&self)     -> &C;
    pub fn evidence(&self)  -> &C::Evidence;
    pub fn certainty(&self) -> Certainty;
    pub fn certifies(&self, claim: &C) -> bool;                 // structural tether
    pub fn verify(&self, budget: Budget) -> Result<(), Error>;  // re-checks via public ops
}
```

**No public constructor exists on any certificate type.** Mints are `pub(crate)`. A
certificate exists iff resolvent proved the claim. But **the accessors expose the
mathematical content**, which is what lets a consumer with its own trusted computing base
re-verify with its own arithmetic instead of trusting `verify()`. That is the resolution of
the tension in E2 §8: cadabra2 wants resolvent's certificates *and* expects to cross-check
them independently with a from-scratch `BigInt`
(`cadabra2/crates/cadabra-testkit/src/oracle/exact.rs:27-38`). **Unforgeable** means no
public mint; **checkable** means public read. Both, not either.

**Tether.** Every certificate carries the claim it attests, and `certifies` is structural
equality against it, so "a transplanted certificate fails the comparison instead of riding
along" (`cadabra2/crates/cadabra-check/src/certificate.rs:41-42`). Claims hold `Arc`-shared
operands so carrying them is cheap.

### 5.2 Cost tiering — the answer to "always produced, or opt-in?"

Neither. **Opt-in by choosing an entry point, tiered by what the evidence actually costs.**
A boolean flag would be wrong because the tiers differ by orders of magnitude.

| Tier | Definition | Behaviour | Examples |
|---|---|---|---|
| **F — free** | The evidence *is* the answer's shape | Always returned, part of the return type | The echelon transform from `row_echelon`; the factor list from `factor`; multiplicities from Yun |
| **C — cheap** | Verification is `O(one multiplication)` or less | Runs **by default**; the escape is a separately named `*_unchecked` entry point returning `Certainty::Probable` | `gcd` (`H∣A`, `H∣B`, degree match); factorization *product* (multiply back); resultant (two independent implementations plus three structural invariants); **root isolation** (§5.3) |
| **X — expensive** | Evidence requires tracing the computation | **Separate entry point**; the uncertified path does not pay | `groebner` vs `groebner_certified` — cofactor tracing costs memory and time, and the "verification is cheap" claim is *false* for Gröbner specifically |

Written as a rule: **no certificate may add more than a documented constant factor to the
answer path; where it would, it lives behind a separate entry point.**

### 5.3 The correction: an isolating interval certifies nothing

The earlier notes listed "isolating intervals from `isolate_roots`" as tier-F free
evidence. X1 §1.1 is right that this is wrong twice over. The claim "`f` has exactly one
root in `[a,b]`" is established by a Descartes/VCA sign-variation count or a Sturm chain;
**the interval is the conclusion, not the evidence.** A consumer handed `Vec<(AlgebraicReal, u32)>`
and nothing else cannot check an isolation result at all — it must redo the isolation.

Why the three local consumers did not surface it: all three consume isolating intervals as
*data*. cadabra2 uses a certificate's *presence* as an admission ticket and never reads it;
sinbad grades on the certainty tag alone; solverang has no L3 demand.

**Fix, adopted:** `isolate_roots` returns the sign-variation witness per interval,
`ProofKind::RootCount` names it, and the item moves from tier F to **tier C** with the
constant factor documented. Retaining the witness is not free.

### 5.4 Composition with a consumer's grading lattice — illustrative, never normative

```rust
// cadabra2 adapter, 6 lines
fn strength(c: Certainty) -> Option<ProofStrength> {
    match c {
        Certainty::Proved(ProofKind::Enclosure) => Some(ProofStrength::IntervalEnclosed),
        Certainty::Proved(_)                    => Some(ProofStrength::AlgebraicallyExact),
        Certainty::Probable(_)                  => None,   // fail closed: refuse to certify
    }
}

// sinbad adapter, 5 lines
fn grade(c: Certainty) -> Grade {
    match c {
        // NOT Grade::Proven. sinbad's D5 rule — "a partial bound cannot masquerade as a
        // total one" (sinbad/crates/tiered-core/src/lib.rs:21-22) — means an enclosure is
        // Proven only if it accounts for the CALLER's whole error budget, which resolvent
        // cannot know. Promoting it requires the caller to certify its budget is total.
        Certainty::Proved(ProofKind::Enclosure) => Grade::Estimated,
        Certainty::Proved(_)                    => Grade::Proven,
        Certainty::Probable(_)                  => Grade::Estimated,
    }
}
```

Two constraints, and one disclaimer that is now part of the design:

1. **`Probable` maps to a refusal, not to a weaker yes, in a fail-closed consumer.**
2. **Certificates are separable from warm-start telemetry** (X-8). resolvent returns
   `(Certified<T>, Telemetry)` where `Telemetry { tier_reached, bisections, precision_bits,
   primes_used }` is plain `Copy` data carrying no proof type and no `Certainty`. One goes
   to the trust boundary, the other to the cache, and the type system keeps them apart.
3. **These mappings are illustrative, not normative.** resolvent's job is to make the
   distinction visible in the type. Deciding what a given `ProofKind` is worth inside a
   consumer's lattice is the consumer's judgement and resolvent must not ship a table that
   pre-empts it. Note that cadabra2's readiness ladder is three stages
   (`Record → Verified → Ready`, `cadabra2/docs/notes/design/dual-path-architecture.md:38-41`)
   and the `Ready` gate additionally requires the whole-carrier contract
   (`cadabra2/ROADMAP.md:47-52`); nothing in resolvent composes that, and nothing should.

### 5.5 What a certificate is, and what it is not, for the three surveyed consumers

cadabra2 consumes a certificate **as an admission ticket, not as a proof to read** — its
*presence* classifies a Parasolid disagreement as a certified divergence rather than a bug,
and nobody re-verifies it (E2 §8). So for that consumer the value is: it cannot be forged,
it names what it attests, it is cheap to carry.

The earlier notes generalized that into a design rule — "elaborate proof objects a consumer
must interpret are not wanted by anyone" — which is true of the three surveyed and **false
of the entire certified-computation category** (§8.1). The rule as it now stands is scoped:
*none of the three consumers evaluated reads a certificate's evidence, so evidence payloads
are kept to the minimum that makes `verify` possible and re-derivation possible — not to
the minimum that makes `verify` possible alone.*

---

## 6. Consumer dossiers, condensed

### 6.1 sinbad — `would-benefit`; a shape constraint, not a demand driver

**What it is.** `/home/dev/sinbad` @ `d5726c8`, 61,419 lines across 42 crates, 33 workspace
members.

**Verdict.** Real, evidenced demands exist; none blocks a shipped code path today; and
every one of them lands in **L4**, the layer `IDEAS-crates.md:114-115` itself calls "a thin
layer on top, not the point". sinbad does not use L1 multivariate polynomials, L2
Gröbner/factorization, or L3 algebraic numbers **at all**.

**What it needs.**
- **L4 with non-polynomial function symbols.** Every shipped manufactured solution is
  transcendental: `sin(πx)·sinh(πy)`, `exp(x)·cos(y)`, `sin(πx)·sin(πy)`
  (`sinbad/crates/sinbad-testkit/src/mms.rs:158-196`). If L4 admits only ring operations
  over L0 elements, resolvent cannot serve sinbad's strongest use case at all.
- **`diff_with` with a caller-supplied leaf rule.** `plexus/src/index_reduction.rs:1-6` is
  verbatim "**Not implemented in this slice.** These passes need a *symbolic
  differentiation* (`d/dt`) layer that does not yet exist in the federation", and the whole
  module is a `NoStructuralPass` identity at `:57-62`. Pantelides differentiates with
  respect to an implicit `t` and *grows new variables*; a `diff` that treats every other
  symbol as constant cannot express it.
- **Canonical bytes** for content addressing (`sinbad/crates/rutter/src/lib.rs:11-14`).
- **`Rational` that exposes its size and its rounding.** meshwright snaps circumcenters to
  53 bits by hand because unbounded bit growth is an observed performance failure
  (`meshwright/src/triangulate.rs:500-512`).
- **A budget-and-decline mode**, so a resolvent call can be a *rung*
  (`tiered-core/src/rung.rs:13-26`).

**What it does NOT need.** No Gröbner, no factorization, no algebraic numbers, no
simplification, no code emitter (it wants one and says resolvent must not ship it), no
e-graph — anvil's want is Herbie-style FP-accuracy rewriting whose rewrites **change the
computed value**, and anvil's IR is a flat `Vec<ConstraintOp>` over `Reg(u16)` with
`LoadConst { value: f64 }`, no interning and no structural identity
(`sinbad/crates/anvil/src/opcodes.rs:14, 41-58, 278, 309`; `lower.rs:48-53`). anvil should
call `egg` directly. This is the best-supported negative finding in the three evaluations
and it survived the audit intact.

**Layers stressed.** L4 heavily, L0 lightly (through `meshwright`, already served by
`lazy-exact`), L2 speculatively.

**Latency classes.** Build-time for MMS generation and Pantelides (bignum cost is free).
Inner-loop for `meshwright` predicates. Per-operation, thousands of calls per transient
solve, for the *unwritten* `sinbad/crates/solverang` event detection — that directory
contains `DESIGN.md` and `STATUS.md` and no source, so this is the only demand that would
constrain resolvent's performance envelope and it is unimplemented in fact.

**Two corrections to E1 that change the urgency, not the placement.** (i) "The plan
hardened to 'plexus needs a small CAS'" overstates: the cited text says plexus "**likely**
needs a *small symbolic layer* of its own (Q6 **leans** 'yes, a tiny CAS')", the same
document at `:592` gives the opposite instruction ("must reuse solverang's IR-AD, not a
second engine"), and it lists the symbolic-vs-numerical fork as open question #1 with the
leaning toward numerical-first. Nothing hardened. L4-3 keeps core status on clause (b);
the honest schedule urgency is lower. (ii) MMS's real blocker is a numerical assembly seam,
not a symbolic one: `residua`'s volumetric source is
`SourceField { per_region: BTreeMap<RegionTag, f64> }`
(`sinbad/crates/residua/src/lib.rs:358-361`), piecewise constant, so a spatially varying
`f(x,y)` cannot be assembled and `poisson_sine` cannot be run against residua today
regardless of how `f` is derived. Shipping resolvent would not unblock MMS. This finding
argues *against* resolvent's own interest and is the most valuable thing in E1.

### 6.2 cadabra2 — `strong-consumer`; substitution plus extension, not unblocking

**What it is.** `/home/dev/projects/cadabra2`, a clean-slate exact CAD kernel, 9 crates,
57,302 lines, MIT OR Apache-2.0.

**Verdict.** Already a consumer of a narrower proto-resolvent: `lazy-exact` is a production
dependency of five of its nine crates including its trusted computing base, with 37
`use lazy_exact` sites, ratified by name with the alternative considered and rejected
(`cadabra2/docs/notes/design/ssi-boolean-plan.md:515-521`). `AlgebraicNumber` is a
one-tuple newtype over `RealRoot` (`cadabra-core/src/exact/algebraic.rs:54`).

**Why not "blocked-today", and the correction that matters most.** cadabra2 is not waiting
on resolvent. Its own evaluation says so in its header and its critical path (ROADMAP item
B, topology publication) touches none of this. E2 §4.6's summary table then tags **eleven**
rows `blocked-now` — a value defined nowhere in that document — of which at least seven are
capabilities cadabra2 **runs in production today** via `lazy-exact`: exact order of two
algebraic numbers (`roots.rs:549`), radical sign (`sign_radical2`), root isolation
(`roots.rs:327`), gcd/`is_root_of` (`roots.rs:480`), Bernstein enclosure
(`bernstein.rs:108,123,135,157`), interval arithmetic (`interval.rs`, 431 lines), the lazy
filtered real (`real.rs`, 724 lines), and the scalar seam (`arrangements/crates/scalar-seam`).
**The correct tagging is `substitute-now / lift-now / eventual`, and only six rows are
`lift-now`:** number-field linear algebra (`classification.rs:441-445`), quadratic-form
factorization (`carrier_cylinder_cylinder.rs:216-222`), degenerate-tower detection,
the plane×torus spiric quartic (`plane_torus.rs:24-27`, refusal minted at `:373-378`),
resultant/subresultant elimination, and bivariate curve topology. A roadmap reading only
that table would sequence eleven items as unblocking work when they are substitution work
with no user-visible outcome.

**What it needs (the genuine lifts).** Number-field linear algebra; factorization of a
quadratic form over ℚ/ℚ(√d); minimal polynomials and degenerate-tower detection;
factorization of a degree-4 plane curve; resultant/subresultant PRS; and the topology of a
real bivariate curve. The unbuilt torus lane is **pure resultant work** and is the largest
net-new demand found anywhere in the three evaluations.

**What it does NOT need.** No Gröbner/F4/ideal membership (zero occurrences of "Gröbner",
"Buchberger", "F4" anywhere in its crates). No multivariate factorization at scale — the
only factorizations wanted are degree ≤ 4 in ≤ 3 variables. No numeric root polishing
(`quadric/roots.rs:11-12` exists precisely to keep it out of the decision path). No Newton
or corrector — it owns those and owns them well. No epsilon and no tolerance model. No
`String`/`Display` symbolic API on production paths. **Certificates must not feed
`cadabra-hints`**, whose whole simplification is "a hint is never evidence".

**Layers stressed.** L0 heavily and in an unusual shape (ℚ, interval, lazy filtered real,
one-radical extension, multiquadratic tower — behind one generic seam). L1 **dense
univariate only**, which makes cadabra2 a *weak witness* for the sparse-multivariate
representation decision: treat it as a constraint that dense-small must be cheap, not as
evidence about sparse-large. L2 is the centre of gravity. L3 is the load-bearing bridge.
L4 is stressed but not in the shape the source spec assumes (§7.4 conflict 1).

**Latency classes and the budgeting fact that inverts the usual CAS priority.** Measured by
cadabra2's own microbench: exact QSIC classification 2.448 ms median, lowered filter
1.846 µs median, ratio 1326×, with `p_straddle = 0` on the generic corpus. On generic input
the exact arm is *never called*. So resolvent may cost milliseconds on the classification
path and cadabra2 will pay it; it may **not** cost milliseconds in the arrangement sweep
inner loop. **cadabra2 does not need resolvent to be fast in order to be useful. It needs
resolvent to be total, certified, and honest about refusal.**

### 6.3 `/home/dev/projects/solverang` — `would-benefit`; API pressure, not a priority driver

**Stated explicitly, because the evaluation bore it out.** solverang is a source of API
pressure on L0/L1/L2 primitives and **not** a priority driver for resolvent's roadmap. Its
demands on resolvent's *interesting* layers — F4, algebraic numbers, the expression DAG —
are all either gated behind a Laman/DR-planner decomposition it has not begun, or actively
rejected. **Do not build F4 for solverang.**

**What it needs.** A public, directly callable `Fp` with a runtime modulus and **no forced
CRT/rational-reconstruction lift**; seeded random points over GF(p); `MPoly` with **runtime
arity** (per-constraint arity runs 2..14, so a const-generic arity makes the adapter
impossible — this is the single highest-stakes representation constraint in the plan and
its evidence is sound); `map_coefficients`; `derivative`; and a dense `row_echelon` over a
field returning rank, pivot rows, dependent rows **and the transform**.

**What it does NOT need.** Bézout counting (it is `Π deg fᵢ`, and `total_degree` is all it
needs). BKK/mixed volume (convex geometry, belongs in a polytope crate). Homotopy start
systems. Exact verification of a converged solution (its residual is essentially never
exactly zero; the right tool is an interval Krawczyk test, which is numerics). Certified
branch tracking (speculative, on top of a feature that does not exist). resolvent's L4 —
its expression layer is compile-time, transcendental, and downstream of a Cranelift JIT
that does its own CSE.

**Layers stressed.** L0 and one L2 primitive. Nothing above.

**Latency class.** Per-edit interactive diagnosis (`solverang/src/system.rs:765`), called
O(k) times inside a MUS-extraction loop. Bignum rational arithmetic is unaffordable, which
is the whole reason `Fp` must be public and callable without a lift.

**The finding that must be restated before it reaches a roadmap.** E3 headlined "18× faster
at 200×200, 40× faster at 800×800, against LAPACK … a conservative lower bound on the
speedup". That is measured against the *incremental k-SVD loop*, a bad algorithm regardless
of arithmetic. Against the single-pass float column of E3's own table, the GF(p) echelon is
**4.4× faster at n=200 (130 ms vs 29.3 ms), 1.5× faster at n=400 (333 ms vs 228 ms), and
2.9× SLOWER at n=800 (639 ms vs 1.86 s)**. The claim is false at the largest size measured.
The honest statement: *it replaces a k-pass SVD loop with a single pass; against a
single-pass float baseline the unoptimized modular echelon is 4.4× faster at n=200 and 2.9×
slower at n=800. The durable wins are exactness at near-degenerate configurations and the
`implied_by` certificate, not wall-clock.* A column-pivoted float QR delivers one pass plus
rank, pivot columns, dependent rows **and** a dependency certificate, and is the alternative
E3 never considered. L2-6 keeps core status on two consumers; **the roadmap must not carry a
40× into a priority argument.**

A second correction: E3 §0.2 says the GF(p) echelon "produces the same rank, the same
dependent-row set" as the SVD it replaces, while §3.1 spends a page arguing that exact rank
at the current floats is the wrong question and that generic rank deliberately answers a
different one. Both cannot be true. Substituting generic rank for numerical rank is a
**semantics change**, not a drop-in speedup — and the disagreement between the two is
itself the diagnostic.

Census corrections, none load-bearing: there are exactly 31 `impl Constraint for` blocks;
`Spline2D` is an **entity** (`sketch2d/entities.rs:510`), so "exactly three are not
polynomial" should be **two** — `Gear` (`assembly/constraints.rs:633-635`, `atan2`) and
`Insert` (`:520-522`, `sqrt` with a `.max(1e-15)` clamp) — plus one piecewise entity. Six
constraints are missing from E3's table (`Collinear`, `SymmetricAboutLine`, `Coincident3D`,
`Fixed3D`, `Coplanar`, `Coaxial`), all plausibly polynomial, so the ~90% conclusion holds.
The quaternion parametrization is free **for rank**; for any solving computation it costs
one extra variable, one degree-2 equation, and a 2:1 double cover.

---

## 7. Adapter sketches

Real Rust against the API above. **Line counts are labelled estimated or measured and the
two are never combined into one figure.**

### 7.1 sinbad — three adapters, three call sites (estimated)

sinbad's demands land in three unrelated crates, so it gets three adapters.

**(a) `sinbad-testkit` MMS forcing generation — ~110 lines estimated, build-time.**

```rust
// xtask/src/mms_gen.rs — runs offline, emits a committed .rs file.
use resolvent::expr::{Store, Expr, Sym, FuncTable};

struct Gen { st: Store, x: Sym, y: Sym, pi: Sym }

impl Gen {
    fn new() -> Self {
        let mut st = Store::with_functions(FuncTable::standard_elementary());
        let (x, y, pi) = (st.sym("x"), st.sym("y"), st.sym("pi"));
        Gen { st, x, y, pi }                       // pi is a SYMBOL, not a numeric value
    }

    // u* = sin(pi*x) * sinh(pi*y)
    fn harmonic_trig(&mut self) -> Expr {
        let (x, y, pi) = (self.st.var(self.x), self.st.var(self.y), self.st.var(self.pi));
        let a = self.st.sin(self.st.mul(pi, x));
        let b = self.st.sinh(self.st.mul(pi, y));
        self.st.mul(a, b)
    }

    // f = -div(kappa grad u) = -(kx*ux + ky*uy) - kappa*(uxx + uyy)
    fn poisson_forcing(&mut self, u: Expr, kappa: Expr) -> Result<Expr, resolvent::Error> {
        let d = |st: &mut Store, e, s| st.diff(e, s);
        let ux  = d(&mut self.st, u, self.x)?;
        let uy  = d(&mut self.st, u, self.y)?;
        let uxx = d(&mut self.st, ux, self.x)?;
        let uyy = d(&mut self.st, uy, self.y)?;
        let kx  = d(&mut self.st, kappa, self.x)?;
        let ky  = d(&mut self.st, kappa, self.y)?;
        let lap = self.st.add(uxx, uyy);
        let adv = self.st.add(self.st.mul(kx, ux), self.st.mul(ky, uy));
        Ok(self.st.neg(self.st.add(adv, self.st.mul(kappa, lap))))
    }
}

/// ~55 lines: match on NodeRef, emit f64 Rust, bind Sym("pi") -> std::f64::consts::PI,
/// bind Sym("x")/Sym("y") -> closure params, `let t{k} = ...;` for every shared node.
/// resolvent hands back a topological walk; sinbad chooses the target language.
fn emit_rust(st: &Store, e: Expr, out: &mut String) {
    for (id, node) in st.walk_topological(e) { /* ... */ }
}
```

Uses L4-1, L4-2, L4-3, L4-4. **resolvent changes: none.**

**(b) `plexus` symbolic `d/dt` for Pantelides — ~150 lines estimated, build-time.**

```rust
// crates/plexus/src/symdiff.rs
use resolvent::expr::{Store, Expr, Sym, LeafRules, LeafDefault};
use std::collections::BTreeMap;

/// plexus's variable convention: state variable `v` at differentiation order `n`.
/// The adapter owns this mapping; resolvent never learns what a "state variable" is.
struct DerVars {
    by_key: BTreeMap<(VarId, u32), Sym>,
    by_sym: BTreeMap<Sym, (VarId, u32)>,
}
impl DerVars { fn sym(&mut self, st: &mut Store, v: VarId, n: u32) -> Sym { /* ~10 lines */ } }

/// d/dt of an equation, with d/dt(der(v,n)) = der(v,n+1).
/// Two-phase: ask which symbols occur, mint their derivative symbols, then differentiate.
fn ddt(st: &mut Store, dv: &mut DerVars, e: Expr, t: Sym) -> Result<Expr, resolvent::Error> {
    let mut rules = LeafRules::new(LeafDefault::Zero);   // parameters differentiate to 0
    for s in st.symbols_in(e) {                          // BTreeSet -> deterministic
        if let Some(&(v, n)) = dv.by_sym.get(&s) {
            let next = dv.sym(st, v, n + 1);
            let node = st.var(next);
            rules.set(s, node);
        }
    }
    st.diff_with(e, t, &rules)
}

fn pantelides_step(st: &mut Store, dv: &mut DerVars, sys: &mut FlatSystem, unmatched: &[EqId])
    -> Result<(), resolvent::Error> { /* ~55 lines */ }

/// Alias elimination a = b / a = -b, on the canonical form so the Schedule content-addresses.
fn is_alias(st: &mut Store, e: Expr) -> Option<(Sym, Sym, bool)> {
    let c = st.canonicalize(e).ok()?;                    // explicit, opt-in
    /* ~25 lines of structural match */
}
```

Uses L4-1, L4-3, L4-6. **resolvent changes: none.** `diff_with`'s leaf-rule table is the
difference between this adapter existing and not.

**(c) `sinbad/crates/solverang` DAE event root isolation — ~70 lines estimated,
per-operation.**

```rust
// crates/solverang/src/events/exact_roots.rs
use resolvent::{Rational as Q, UPoly, Integer, Budget, isolate_roots_in, AlgebraicReal};

fn crossings_in_step(coeffs: &[f64], h: f64, budget: Budget)
    -> Result<Vec<(Q, Q)>, Decline>
{
    let mut c = Vec::with_capacity(coeffs.len());
    for &a in coeffs {
        c.push(Q::try_from_f64(a).map_err(|_| Decline::CannotCertify)?);   // fails closed
    }
    let (p, _den) = UPoly::<Q>::from_coeffs_low_to_high(c).clear_denominators(); // -> UPoly<Integer>
    let hi = Q::try_from_f64(h).map_err(|_| Decline::CannotCertify)?;
    let roots = isolate_roots_in(&p, &Q::ZERO, &hi, budget)
        .map_err(|e| if e.is_decline() { Decline::Budget } else { Decline::CannotCertify })?;
    Ok(roots.value.into_iter().map(|(r, _mult)| r.bounds()).collect())
}
```

Uses L0-2, L1-1, L2-1, L3-2, X-1. **resolvent changes: none.** Note what changed against
the earlier sketch: no `Interval<Q>` type appears (ADR-015), the square-free precondition
is discharged inside `isolate_roots_in` rather than by a caller-visible `SqfrPoly`
constructor, and multiplicity comes back as a pair element rather than a field.

**Verdict: all three pass, on both numbers, comfortably.** The seam *is* the adapter here.

### 7.2 cadabra2 — the seam passes; the total does not, and it is measured

**Measured** (X1 §4: `wc -l`, then strip comment-only and blank lines above the
`#[cfg(test)]` marker) on the existing `lazy-exact` delegation, which is the closest thing
to a real measurement anyone has:

| File | Total | Code above tests | Composition |
|---|---|---|---|
| `cadabra-core/src/exact/scalar.rs` | 428 | **149** | all delegation — 21 inherent methods, operators, `From`, `cmp_exact`; **zero CAD vocabulary** |
| `cadabra-core/src/exact/radical.rs` | 410 | **137** | mostly delegation |
| `cadabra-core/src/exact/algebraic.rs` | 606 | **211** | ~65 delegation (`:54-160`); ~145 geometry (`:163-430`) |
| `cadabra-core/src/exact/interval.rs` | 218 | **86** | delegation |
| `cadabra-core/src/exact/mint.rs` | 259 | **69** | cadabra2's own mint guard |
| **Total** | | **652** | |

The earlier estimate of 175 was 4.4× under on `scalar.rs` alone. Its `ExactScalar` sketch
had 11 methods; the shipped one has 21 (`zero`, `one`, `from_f64`, `from_finite`,
`from_i64`, `from_ratio`, `from_rational`, `negated`, `squared`, `abs`, `checked_div`,
`sign`, `is_zero`, `is_positive`, `is_negative`, `demote_exact`, `enclosure`,
`approx_lossy`, `as_rational`, plus operators and `cmp_exact`) — **none of which is CAD
vocabulary**. They are the constructors and predicates any consumer newtyping a rational
writes. It also silently dropped `cadabra-arrange/src/lift.rs` from the block table;
excluding it is defensible on inspection (most of it is geometry) but the exclusion was
unstated and it ran in the direction that helped the number.

**Honest restatement:**

| Quantity | Figure | Status |
|---|---|---|
| The resolvent-facing seam — newtypes, `Ring`/`Ordered`/`Field` impls, error mapping, the three exits | **~120–180 lines** | estimated; **passes** |
| The total delegation adapter with the shipped ergonomics | **~250–400 lines** | measured range; **does not pass the literal 200-line test** |

Sketch of the seam against the API above:

```rust
// cadabra-core/src/exact/ — the resolvent delegation core
use resolvent::{Sign, Budget, Rational, SqrtExt, AlgebraicReal, UPoly, Integer,
                Error as RErr, Unsupported, DomainFault};
use resolvent::base::{Ring, CommutativeRing, Field, Ordered};
use crate::error::{KernelError, KernelResult, Capability, Subject};

// ---- error mapping ------------------------------------------------------ ~22 lines
impl From<RErr> for KernelError {
    fn from(e: RErr) -> Self {
        match e {
            RErr::Unsupported(u)      => KernelError::not_implemented(cap(u), Subject::None),
            RErr::BudgetExhausted{..} => KernelError::not_implemented(Capability::Budget, Subject::None),
            RErr::Overflow { .. }     => KernelError::resource_exhausted(),
            RErr::Domain { fault, .. }=> KernelError::invalid_geometry(reason(fault)),
            RErr::Internal { .. }     => KernelError::internal(),
        }
    }
}

// ---- ExactScalar --------------------------------------------------------- ~60 lines
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ExactScalar(Rational);
impl ExactScalar {
    pub fn lift(x: f64) -> KernelResult<Self> { Ok(Self(Rational::try_from_f64(x)?)) }
    pub fn demote_exact(&self) -> KernelResult<i64> { Ok(self.0.demote_exact()?) }
    pub fn enclosure(&self)    -> (f64, f64)        { self.0.enclosure() }   // NOT an Interval
    pub fn approx_lossy(&self) -> f64               { self.0.approx_lossy() }
    pub fn num_bits(&self) -> u32 { self.0.num_bits() }
    pub fn den_bits(&self) -> u32 { self.0.den_bits() }
    /* ~15 more constructors and predicates the shipped type has and resolvent does not owe */
}
// ONE seam, not three: no Scalar/ScalarOrd/TryDiv triple.
impl Ring            for ExactScalar { /* 7 fns, one line each; in-place forms defaulted */ }
impl CommutativeRing for ExactScalar {}
impl Field           for ExactScalar { fn inv(&self) -> Option<Self> { self.0.inv().map(Self) } }
impl Ordered         for ExactScalar { fn sign(&self) -> Sign { self.0.sign() } }

// ---- ExactRadical -------------------------------------------------------- ~30 lines
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ExactRadical(SqrtExt<Rational>);
impl ExactRadical {
    /// Total, not panicking: a negative radicand is an Err, not an abort.
    pub fn new(a: ExactScalar, b: ExactScalar, r: ExactScalar) -> KernelResult<Self> {
        Ok(Self(SqrtExt::new(a.0, b.0, r.0)?))     // replaces 12 lines of hand-written guard
    }
    /* sign / cmp_cross / arithmetic forwards, one line each */
}

// ---- AlgebraicNumber ----------------------------------------------------- ~40 lines
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AlgebraicNumber(AlgebraicReal);          // Ord is resolvent's; no Rc<RefCell<_>>
impl AlgebraicNumber {
    pub fn is_root_of(&self, h: &UPoly<Integer>) -> bool { self.0.is_root_of(h) }
    pub fn sign_of(&self, h: &UPoly<Integer>) -> Sign    { self.0.sign_of(h) }   // total
    pub fn between(a: &Self, b: &Self) -> ExactScalar {
        ExactScalar(resolvent::rational_between(&a.0, std::slice::from_ref(&b.0)))
    }                                                // replaces TWO hand-rolled 256-step loops
    pub fn enclosure(&self)    -> (f64, f64) { self.0.enclosure_f64() }
    pub fn approx_lossy(&self) -> f64        { let (l, h) = self.0.enclosure_f64(); 0.5*(l+h) }
}
// The self-comparison guard is gone: resolvent's `cmp` handles pointer-equal operands
// internally, and the type is Send + Sync so a parallel sweep needs no wrapper (ADR-013).
```

**What the seam unlocks with no further resolvent API.** cadabra2's existing 49-line
`inertia` (`classification.rs:292-338`) becomes generic over `Ordered + Field` and
instantiates at `NumberField` — closing its largest fail-closed site
(`classification.rs:78-87`, `:441-445`) without resolvent shipping an `inertia` function.
That is the seam paying for itself, and it is the two-consumer rule working correctly.

**Judgement.** The acceptance criterion passes **in substance** — nothing cadabra2 needs is
bespoke to cadabra2 — and fails **on the literal total**. Both are reported. §0 decision 6
is stated in those terms rather than as "all three adapters pass".

### 7.3 solverang — a ~45-line seam over an afternoon of transcription

```rust
// crates/solverang/src/exact/mod.rs
use resolvent::{Fp, FpElem, MPoly, Rational as Q, Ring as PolyRing, Order, linalg};
use crate::{constraint::Constraint, id::ParamId, param::SolverMapping};

/// Consumer-side trait. Lives in solverang. Resolvent knows nothing about it.
pub trait AlgebraicConstraint: Constraint {
    fn poly_vars(&self) -> &[ParamId];
    fn residual_polys(&self, ring: &PolyRing) -> Option<Vec<MPoly<Q>>>;  // None = transcendental
}

pub fn generic_rank(
    cs: &[(usize, &dyn AlgebraicConstraint)],
    mapping: &SolverMapping,
    seed: u64,
) -> Option<GenericRank> {
    let fp    = Fp::new(2_147_483_647).ok()?;                     // runtime modulus     L0-6
    let n     = mapping.len();
    let point = fp.random_point(n, seed);                         // seeded, never ambient L0-7

    let (mut rows, mut owner) = (Vec::new(), Vec::new());
    for (idx, c) in cs {
        let vars  = c.poly_vars();
        let ring  = PolyRing::new(vars.len() as u32, Order::GrevLex).ok()?;   // runtime arity L1-2/3
        let polys = c.residual_polys(&ring)?;                     // bail on Gear / Insert
        let cols: Vec<usize> = vars.iter().map(|p| mapping.col(*p)).collect();
        let local: Vec<FpElem> = cols.iter().map(|&j| point[j]).collect();
        for f in &polys {
            // HOISTED, once per polynomial: the Q -> GF(p) reduction never enters a loop.
            let f_p: MPoly<FpElem> = f.map_coefficients(|q| fp.reduce(q)).ok()?;   // L1-6
            let dfs: Vec<_> = (0..cols.len())
                .map(|k| f_p.derivative(k as u32))                // hoisted too         L1-5
                .collect();
            let mut row = vec![fp.zero(); n];
            for (k, &j) in cols.iter().enumerate() {
                row[j] = dfs[k].eval(&local);                     // pure FpElem arithmetic L1-7
            }
            rows.push(row);
            owner.push(*idx);
        }
    }
    let ech = linalg::row_echelon(&fp, rows).ok()?;               //                     L2-6
    Some(GenericRank {
        rank:       ech.rank(),
        dependent:  ech.dependent_rows().iter().map(|&r| owner[r]).collect(),
        implied_by: ech.transform_rows(&owner),                   // the implied_by certificate
    })
}
```

**~45 lines estimated, zero resolvent changes.** The two hoists are not cosmetic: X1 §5.2
showed the earlier sketch performing a bignum numerator/denominator reduction plus a modular
inverse of the denominator *per coefficient per evaluation point*, and allocating a fresh
`MPoly` per (constraint, polynomial, column) — up to 14 per residual for `assembly::Insert`.
With `evaluate_with` removed from the API (§3.4) the slow idiom is no longer expressible.

The second half is transcription, `× 28` algebraic constraint types at 5–12 lines each ≈
**250–450 lines estimated**, using nothing but `Ring::var`, `MPoly::constant`, and `+ − ×`.

**Both numbers reported.** The seam **passes**; the total **does not**, and that is not
resolvent's to fix: the transcription scales with solverang's constraint vocabulary and is
byte-for-byte identical under any polynomial API with a runtime-arity ring. The test as
intended — "does this consumer force resolvent to expose something bespoke?" — passes:
every item the seam touches is justified in §4 as a general primitive independent of
solverang. The one thing that would fail it is a const-generic arity, which is why L1-2 is a
one-way door settled before fan-out.

### 7.4 Conflicts, and who ate the cost

| # | Conflict | Resolution | Who absorbs it |
|---|---|---|---|
| 1 | cadabra2 needs **structural, non-canonical** L4 equality (its `certifies` tether; `Cos2` deliberately un-rewritten); sinbad needs a **canonical** form so its `Schedule` content-addresses | resolvent never rewrites implicitly. Construction hash-conses and constant-folds and stops. `canonicalize(expr) -> Expr` is explicit, value-preserving, and returns a *new* node; `canonical_bytes` hashes whatever it is handed | **resolvent** takes the general shape; **sinbad's adapter** pays one extra line per call site |
| 2 | sinbad needs transcendental function nodes; solverang forbids `Atan2`/`Sqrt`/piecewise nodes; cadabra2 needs opaque domain atoms with **no** derivative rule | resolvent ships **no transcendental semantics** — only an open `FuncTable` the caller owns. `standard_elementary()` is a constructor, not a default | **resolvent** — a genuine scope addition over the spec's polynomial-only L4, and the only way any of the three gets served |
| 3 | solverang needs GF(p) **public and callable with no CRT lift**; cadabra2 explicitly does not want GF(p) user-facing | `Fp` is public in `resolvent-modular` and appears in no signature cadabra2 calls. Modular methods are an *exposed layer*, not only an internal strategy | **resolvent** (public surface it builds anyway); cadabra2 pays zero by not importing it |
| 4 | cadabra2 needs `&self` comparison and `impl Ord`; a refinement cache makes the naive design non-`Sync` | `Arc<Inner>`, monotone `&self` refinement, `Send + Sync`, self-comparison handled inside resolvent (ADR-013). Refinement progress is **shared across clones**, which is the thing the cache exists for | **resolvent** pays a synchronization mechanism; **every consumer** — including the parallel and FFI ones outside the three — gains |
| 5 | cadabra2 wants `impl Ord` (infallible); sinbad's rung protocol wants a budget on everything | The separation bound makes comparison terminate in a *computable* number of steps, so `Ord` is total and honest; a budgeted sibling ships alongside for latency-bounded callers | **both get what they asked for**; resolvent pays two entry points |
| 6 | sinbad asked for `diff_with(e, sym, impl FnMut(Sym) -> Option<Expr>)`; a closure that mints nodes needs `&mut Store` while `diff_with` holds it | Signature changed to `symbols_in(e) -> BTreeSet<Sym>` plus `diff_with(e, sym, &LeafRules)` where `LeafRules` is a `BTreeMap<Sym, Expr>` with a `Zero`/`Refuse` default. Borrow-clean, reentrancy-free, deterministic by construction | **resolvent** chose a different shape than the consumer asked for; **sinbad's adapter** pays ~6 lines and gains determinism |
| 7 | sinbad wants a `Ctx` first parameter and a `DiagCode` on every error | resolvent takes no capability handle and no diagnostics registry; its error enum is small, closed and stable | **adapter** (~5 lines to drop `Ctx`, ~20 to map variants to codes) |
| 8 | sinbad demands bitwise reproducibility across thread counts; modular methods want randomness | Index-addressed primes; counter-based seeded RNG with per-index substreams; parallel results combined in index order | **resolvent** — non-negotiable, and solverang independently requires the same |
| 9 | solverang's coefficients are f64 bakes of transcendentals (`sin(30°)` at `sketch2d/constraints.rs:797-798`) | `try_from_f64` is exact-dyadic and no "nice rational" sibling exists. The adapter documents that it analyses the system as authored in f64 | **adapter** (a doc comment); **resolvent** absorbs the cost of refusing a convenience users will ask for |
| 10 | cadabra2's arrangement sweep cannot afford a bignum allocation per call; resolvent's ℚ is a bignum | resolvent ships no filtered-arithmetic layer and no float interval. The consumer keeps its own filter and descends to ℚ only when the filter fails | **consumer** keeps the filtering policy it already owns; **resolvent** keeps one enclosure semantics at the boundary |
| 11 | cadabra2 wants one algorithm text at three tiers; resolvent ships only an *algebraic* trait tower | The exact tier instantiates `resolvent::Ring`; the f64 and interval tiers stay on the consumer's own seam | **consumer** (it already has both); **resolvent** gives up being the ecosystem's scalar vocabulary (§3.5) |
| 12 | cadabra2's TCB admission budget is roughly `dashu + smallvec + thiserror` plus a zero-dep base crate; resolvent wants `rayon` and `serde` too | `rayon` and `serde` are default-off features. Core runtime dependencies stay inside the measured budget | **resolvent** |
| 13 | cadabra2 wants certificates unforgeable; its TCB also wants to re-verify them with its own from-scratch arithmetic | Private fields, no public mint, **public read accessors** on the evidence | **resolvent** |

---

## 8. Generality evidence

The claim "general-purpose" is defensible only against consumers the design was not written
for. Five were evaluated in X1, outside the three surveyed. This section states, for each,
what the **corrected** API does.

### 8.1 Proof assistant / certified computation — **served, additively**

Wants Gröbner with cofactor certificates feeding a kernel that trusts nothing.

Served by: `groebner_certified` as a separate entry point with cofactor tracing (§5.2 tier
X); private fields, no public mint, **public read accessors** (§5.1); `Certainty::Probable`
visible in the type so a kernel can reject it; `resolvent-base` at one dependency
(`thiserror`) so admission is plausible at all.

The one real defect, invisible from inside the three surveyed consumers, was calling
isolating intervals free evidence. **Fixed** in §5.3: `ProofKind::RootCount`, the
sign-variation witness returned per interval, tier C rather than tier F. And §5.5's design
rule is now scoped to the three consumers evaluated rather than asserted of everyone.

### 8.2 SMT NRA theory — **served, additively; previously breaking**

Wants CAD, sign determination at a multivariate sample point, and incremental use inside a
search loop with frequent backtracking.

**(a) The sample point.** `AlgebraicReal` is ℚ-only and every L3 query takes a
`UPoly<Integer>`, so there is no representation for a real algebraic point in several
variables. Two routes exist; the design now names one. **RUR / primitive element (L2-13) is
the route**: it pushes the tower down to a single ℚ-algebraic number plus rational
coordinate functions, which is what msolve does and what makes `AlgebraicReal` sufficient
as written. A two-variable sign query becomes: compute the RUR of the zero-dimensional
system; the sample point is `(α, g₁(α)/g₀(α), g₂(α)/g₀(α))` for one ℚ-algebraic α;
`sign_of(h)` at that point is the sign of a single univariate polynomial in α obtained by
substituting the coordinate functions and clearing denominators — one `sign_of` call on the
existing API, no new type.

**(b) The general route stays open and is additive, not breaking.** Because `UPoly<C>` is
generic from day zero, `UPoly<NumberFieldElem>` behind the `number-fields` feature plus a
second algebraic-number type arrives as an *added instantiation*, never as a change to
`AlgebraicReal` (L3-8, ADR-014 §5). This is the resolution of the sharpest consumer
incompatibility in the whole analysis: geometry provably never needs algebraic numbers over
ℚ(α); SMT provably does; ship the simple one and keep the general one an instantiation away.

**(c) Backtracking.** MCSAT's terms are polynomials, so this consumer stays on L1, where
`MPoly` is a self-contained droppable value with no shared arena state — which serves
backtracking well and is a direct consequence of L1-4. If it does reach L4, §2.3 states
plainly that `Store` growth is monotone and L4 is not designed for a search loop, and the
`store-tags` feature is what makes a future checkpoint safe rather than silently wrong.

**(d) CAD itself** is rejected for v1 on the record (§4.2, L2-19), with every piece it is
built from admitted individually. That is a scoping decision stated rather than an omission.

### 8.3 Cryptography / coding theory — **served; previously failed the acceptance test outright**

Wants GF(p^k), factorization over finite fields; does not care about real root isolation.

Previously this consumer hit a **sealed** coefficient set `{Rational, Integer, FpElem,
NfElem}` and stopped: GF(p^k) appeared nowhere in the API notes, ℤ/n was out of scope, and
the stated answer to a fourth consumer needing its own ring was "add it to the sealed set" —
i.e. **resolvent must change**, which is exactly what the acceptance criterion forbids. For
a crypto consumer that is usually terminal, because the whole point is *their* tower with
*their* basis chosen for speed.

Three changes fix it, none of which is a redesign:

1. **GF(p^k) and ℤ/n are core and public** (L0-8, L0-9). They were already in
   `plans/architecture.md:57` and scheduled at `roadmap.md:90, 256, 539`; the API notes had
   simply read the spec's "algebraic extensions" as extensions of ℚ — the direction cadabra2
   needs — rather than of GF(p), which is the same words pointing the other way.
2. **Factorization over GF(p) is a public capability** (L2-5), on the same
   zero-marginal-implementation argument that made `Fp` public.
3. **The coefficient seam is an open trait tower** (§3.2). A consumer with a chosen normal
   basis, a Galois ring, or a Montgomery-friendly modulus implements `Ring + Field` for its
   own type and gets every Tier-G algorithm. It does not get the modular pipeline unless it
   also implements `Reducible + Liftable`, and the doc comment says so.

The layering serves it well besides: a consumer that never imports `resolvent-real` pays
nothing for `AlgebraicReal`, exactly as cadabra2 pays nothing for `Fp`. Determinism, seeded
randomness, budgets and `row_echelon` over a field (which is Berlekamp's matrix step) are
all directly useful.

### 8.4 Robotics / kinematics — **served offline; explicitly out of scope online**

**Offline is the realistic architecture and it is served**: run resolvent at build time,
eliminate the loop-closure system symbolically with L2-3, walk the DAG with L4-4, and emit
your own fixed-point or SIMD tape — which is precisely why L4-8 refuses to ship a code
emitter. L2-16's exclusion of numeric root polishing is correct here too: the online solver
is the consumer's.

**Online is out of scope, and the boundary is now stated rather than left to be discovered.**

- **resolvent is a build-time tool for a hard-real-time consumer.** `Store`, `MPoly` and
  `Rational` are heap-backed; `Rational` sits on a bignum with no custom-allocator story.
  This is a substrate property, not an oversight.
- **The allocator rule is narrowed** to "no global allocator override" (§2.1). An allocator
  parameter is not offered and is **not foreclosed**, which the previous blanket prohibition
  did foreclose for exactly the consumer that would need it.
- **`resolvent-base` is a `no_std` candidate.** Nothing in `Sign`, `Verdict`, the `Ring`
  tower, `Error`, `Unsupported` or `Budget` needs `alloc`. Whether it compiles that way is
  an open item (§10) and it is cheap to check once the crate exists — the claim fails if
  `Error` grows a `String` or a `Box`, which INV-5 forbids anyway.
- **`Budget` bounds steps, not memory.** A latency-budgeted consumer needs a bit-size or
  allocation cap; that is additive and is not built.

### 8.5 Teaching / scripting from Python — **served**

Bindings are glue outside resolvent; `serde` is a default-off feature; INV-4's no-panic rule
is exactly right for FFI, where unwinding is UB; `FuncTable::standard_elementary()` gives a
REPL its function set for free.

Three costs the earlier design imposed, all now removed:

1. **`!Sync` on the headline type is gone.** `pyo3` requires `Send + Sync` for
   `#[pyclass]`; `!Sync` forces `#[pyclass(unsendable)]`, which **panics** on foreign-thread
   access — reintroducing outside resolvent the panic INV-4 forbids inside it, on the type
   §4 calls resolvent's headline differentiator. ADR-013's `Arc<Inner>` + `Send + Sync` is
   what fixes this, and every outside consumer in X1 favours that side: a parallel SMT solver
   no longer re-refines a shared sample point from scratch per clone, and `Arc<AlgebraicReal>`
   in a shared cache becomes possible. The `RefCell` alternative's advantage was confined to
   the single access pattern that motivated it.
2. **The store-identity hazard has a mechanism.** §2.3's `store-tags` feature, default off,
   caller-supplied, ambient-free. The justification previously given for declining a tag —
   "a bug none of the three would make" — was falsified by this consumer and by 8.2, and it
   is not left standing.
3. **There is a stated way to show a number to a human.** INV-7 previously forbade "`Display`
   that rounds", which — generalized from one consumer's *production-path clippy ban* to a
   library law — left a REPL with no supported rendering for a type that has no finite exact
   decimal form. INV-7 is now scoped to decision paths: **exact `Display` on exactly
   representable types (`Integer`, `Rational`, `Fp` elements, `UPoly`, `MPoly`) is permitted
   and expected**, and inexact types get an explicit
   `to_decimal_string(digits) -> (String, ProofKind::Enclosure)`.

### 8.6 What would still flip the verdict

The corrected design is `general` for four of these five and `explicitly out of scope` for
the online half of the fifth. It would revert to overfitted if any of the following were
ratified as written:

- the sealed coefficient set returning (§3.2 reversed) — then crypto, p-adic and Galois-ring
  consumers are upstream-blocked by construction;
- `AlgebraicReal` staying ℚ-only **without** RUR and **without** the `UPoly<NumberFieldElem>`
  instantiation (§8.2 (a) and (b) both dropped);
- `!Sync` on the headline type (§8.5);
- a `resolvent-seam` scalar vocabulary shipping as a fourth ecosystem standard (§3.3).

---

## 9. API invariants

Any future change must preserve every one of these or be argued as an explicit override with
the consumer cost named.

**INV-1 — No ambient state.** No `static mut`, no `static` with interior mutability, no
`thread_local!`, no global interner, no session object, no capability-handle parameter, in
any `publish = true` crate. CI greps.

**INV-2 — No I/O and no clock.** No `std::fs`, `std::net`, `std::env`, `std::time`,
`std::process` in any published crate.

**INV-3 — No unseeded randomness.** Primes are a pure function of an index. Every random
choice that can reach an output comes from a counter-based RNG at an index-derived position,
seeded by an explicit parameter with a fixed checked-in default. No `thread_rng`, no OS
entropy. No `HashMap` iteration order may reach a result value or a decision.

**INV-4 — Total functions. No panics on any input-dependent path.** Overflow, coefficient
blowup, division by zero, square root of a negative, non-finite f64 ingress,
exponent-packing overflow: all `Result`. A violated internal invariant returns
`Error::Internal { invariant: &'static str }`; it does not panic. An adapter cannot absorb a
panic, and behind an `extern "C"` boundary unwinding is UB.

**INV-5 — Errors are a small, closed, `Clone + PartialEq`, `String`-free enum** with the
offending data on the variant, a structured `Unsupported` value that names the missing
capability, and declines (`BudgetExhausted`, `Unsupported`) distinguishable from faults
(`Domain`, `Overflow`, `Internal`) via `is_decline()`. resolvent's error type never tries to
be a consumer's error type.

**INV-6 — Two budget regimes, and which one applies is stated per entry point.** Where a
proven bound exists (Mignotte–Davenport separation, Landau–Mignotte, Hadamard) the default
budget is *derived from the bound*, exhaustion is proven impossible, the budget is a bug
detector, and **the query is total** — `sign_of` returns `Sign`, not `Result<Sign>`. Where
no proven bound exists (van Hoeij lattice iteration, stabilization-driven reconstruction) the
budget **is** the exit and exhaustion returns a typed decline carrying resumable state. A
budgeted sibling ships alongside every total query that can allocate unboundedly, so a
latency-bounded caller can decline instead of waiting. Exhaustion is a value, never a hang
and never an abort.

**INV-7 — Exactly three exits from every exact type**: `demote_exact` (lossless or typed
error), `enclosure` (outward-correct `(f64, f64)`; the true value lies in the closed
interval), `approx_lossy` (nearest double, diagnostic). No `as f64`, no `impl From<_> for
f64`. **Scoped to decision paths:** exact `Display` on exactly representable types is
permitted and expected; inexact types expose `to_decimal_string(digits)` whose result carries
`ProofKind::Enclosure`. No rounding value ever re-enters a decision.

**INV-8 — No `eps` parameter on any equality, sign, or ordering decision, anywhere, under any
name.** Equality is decided algebraically or not at all. `refine_to(width)` is not a
tolerance: it never affects a verdict, and the verdict is identical whether or not it is
called (property-tested as idempotence under refinement).

**INV-9 — Certainty is visible in the type.** Every modular or heuristic result is
`Certified<T>` carrying `Certainty::{Proved(ProofKind), Probable(ProbableReason)}`; the
default path is `Proved`; certificates have private fields, no public mint, public read
accessors, and a `certifies(claim)` structural tether. Warm-start telemetry is a separate,
proof-free `Copy` value.

**INV-10 — No public owned type carries a lifetime parameter.** Consumers store resolvent
values in their own structs. Borrowed views and iterators are the only exception and are
never returned by value from a constructor.

**INV-11 — Nothing consumer-specific in the public API.** No type, trait, method, name, or
feature flag that mentions or presumes a consumer's domain. Features are capability-named.
resolvent ships no adapter crates and has no optional dependency on any ecosystem crate.

**INV-12 — Canonical bytes are a pure function of mathematical content**, independent of
interning order, handles, arena addresses, insertion history and build configuration, and
carry an explicit `SCHEMA_VERSION`. Changing canonical form is a breaking, versioned event.

**INV-13 — Polynomial ring arity is a runtime value.** No const-generic arity. `MPoly` is a
self-contained `Send + Sync` value carrying its ring by an **owned handle**, never as
`&'a Ring`.

**INV-14 — The coefficient seam is an open, capability-factored trait tower, and it imposes
no obligation a word-sized type cannot discharge.** This replaces the previous method-count
invariant, which was the wrong property: what matters is that `Ring` carries seven methods
plus three defaulted in-place forms and no bignum-shaped duty, while `Reducible`,
`Liftable`, `Ordered` and `BulkOps` carry the duties a given type may or may not have. The
modular fast path is bounded by `Reducible + Liftable`. There is **no second, ops-surface
scalar trait** and no `resolvent-seam` crate (ADR-019).

**INV-15 — `AlgebraicReal` is `Send + Sync`, `Arc`-backed, with `&self` monotone
refinement and a total `Ord`.** Refinement only narrows, always contains the root, and can
never change a verdict — only how much work the verdict took. Refinement progress is shared
across clones. Self-comparison is safe without a caller-side guard, no two locks are ever
held simultaneously, and no lock order is derived from an address. Pinned by the
trichotomy / transitivity / sort-stability / idempotence suite under a step budget, where
"did not finish" counts as "wrong" (ADR-013).

**INV-16 — Homomorphisms are applied to polynomials, not inside evaluation loops.**
`map_coefficients` is the only cross-ring path; `eval` is same-ring. No API takes a
homomorphism and a point together.

**INV-17 — Every arena is a caller-owned value and every handle is arena-relative.** No
global or implicit interner, at L1 or at L4. A handle used against the wrong arena is
bounds-checked; the residual in-range hazard is documented, `Store` growth is monotone for
its lifetime, and the optional caller-supplied `store-tags` feature closes it for consumers
that need it (ADR-020).

**INV-18 — `Verdict<T>` and bare values are not interchangeable.** A function returns a bare
`Sign` iff it is total and exact. A function that can be indeterminate returns
`Verdict<Sign>` and never `Sign`. `Verdict` is produced **only** by enclosure and filter
APIs and never by an algebraic-decision API. `Unknown` means "this cheap rung declined";
the caller climbs, never guesses.

---

## 10. What this document does not settle

Stated so it is not mistaken for settled.

1. **Whether `resolvent-base` should supersede, wrap, or coexist with
   `arrangements/crates/scalar-seam`.** Coexistence is the default, is the most expensive of
   the three, and is chosen by inaction. §3.5 lists the five overlapping components and the
   one decision per item that sets merge cost; ADR-018's deferral stands.
2. **Whether `resolvent-base` compiles as `#![no_std]`.** Nothing in its contents needs
   `alloc`. Cheap to check once the crate exists; the claim fails if `Error` grows a `String`
   or a `Box`, which INV-5 forbids anyway.
3. **Whether terms are `(MonomialId, C)` into a ring-owned arena or `(PackedMon, C)`
   inline.** The ownership rule is settled (§2.2); the term type is not, and it is decided by
   `plans/roadmap.md` §2.5 contradiction 2's microbenchmark before the multivariate trunk
   starts.
4. **Whether L1-9b (bivariate Bernstein over a rational box) earns core.** One consumer, no
   internal-need argument, and the univariate half's justification does not reach it.
5. **The `Ord` allocation bound.** Comparison is decidable and the separation bound makes it
   terminate in a computable number of steps, but memory use on a Mignotte-style near-equal
   pair is unbounded in principle. Whether it ever bites is unmeasured; the budgeted sibling
   exists because it might.
6. **Whether L2-12 (bivariate curve topology) is real demand or whether L2-3 suffices.**
   Working one generic plane×torus case by hand settles it. Do not build it first.
7. **The compile-time cost of the generic straight-line texts** (§3.4). Asserted negligible
   on a closed list of six. Measure once L1 and L2 exist; if it is not negligible the list
   shrinks rather than the seam disappearing.
8. **Whether a checkpoint API is ever added to `Store`.** §2.3 states the monotone-growth
   position; `store-tags` is the mechanism that would make a checkpoint safe. No consumer
   asks for one today.
