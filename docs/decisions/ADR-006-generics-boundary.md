# ADR-006 — Coefficient-ring traits, and where generics stop

**Status:** Ratified 2026-07-31
**Reversibility:** one-way (the trait signature is inherited by everything above it)
**Amended:** 2026-07-31 — **four corrections to the trait block, all load-bearing**:
`zero`/`one` take a ring context; `Liftable`'s supertrait is `Reducible`;
`Reducible::Image` is a `CommutativeRing` and `reduce` is fallible; `BulkOps` is deleted
and `BatchField::inv_batch` is added. The previous block **did not compile** and was
**unimplementable for five of the seven rings in its own instantiation set**
(critique-engineering §1, §4, §14, §20).
**Amends:** ADR-019 §1 and §3 restate this tower; `API.md` §3.2's code block predates this
amendment and is superseded on these four points (ADR-021 §2: ADRs win on signatures).
**Gates lanes:** Z0, Z1, Z3, Z4, Z5, U1, P3 — i.e. everything above Layer 0.
**Evidence:** `docs/research/prior-art-and-licensing.md` §2.1, §3.2;
`docs/research/algorithms-and-representation.md` §3.5, §1.6, §4.4;
`docs/research/critique-engineering.md` §1, §4, §14, §20.

---

## Context

"Trait-generic everywhere with specialization for the hot cases" is what the source spec
says for Layer 0. It is also a well-known way to produce a CAS that is simultaneously slow
to compile and slow to run, because the two failure modes are opposite and both are easy to
hit:

- **Too generic**: `Box<dyn Ring>` or ring-object arithmetic (`ring.add(&a, &b)`) puts an
  indirect call in the inner loop of the hottest code in the library.
- **Too monomorphized**: every algorithm × every coefficient ring × every packing width
  instantiated, and the front end spends minutes on code nobody calls.

There is direct evidence in the Rust CAS prior art that this is hard rather than merely
fiddly. `feanor-math` (MIT, the best permissive Rust reference) uses a **two-trait
`RingBase`/`RingStore` split**, which its own documentation describes as a deliberate
workaround for Rust's borrow and blanket-impl limitations. That is a warning about the
design space, not a template to copy.

There is also one measured constraint that must be honoured *now* or lost forever.
Groebner.jl computes over `Z/p₁ × … × Z/p_N` as **tuples**, sharing all non-arithmetic work
(matrix construction, pair handling) across `N` primes and exposing SIMD, for up to ~2.7×
amortized speedup; `N = 4` is their production choice. If the Layer-0 trait signature
cannot express a ring whose element is `[u32; 4]`, that trick is unavailable later — and
Layer 0 is a one-way door.

### The four defects in the original block, and why they had to be fixed before fan-out

Recorded rather than silently patched, because the original shape is what an agent will
otherwise reconstruct from the surrounding documents.

**(1) `fn zero() -> Self` is unimplementable for five of the seven rings this ADR names.**
`zero()` and `one()` are receiverless associated functions: only a type that knows its ring
*statically* can implement them. Of `{Fp, Fp4, Integer, Rational, Zn, GFpk,
NumberFieldElem}`, exactly **two** — `Integer` and `Rational` — qualify. This ADR itself
says `Fp` "carries `p` plus its Barrett reciprocal by value", so `Fp::zero()` must answer
"zero of *which* prime field?" from no information at all. Lane Z3 — the plan's designated
best agent lane — hits this on its first `impl` and has three options, all bad: a `p = 0`
sentinel that poisons every `PartialEq` and `is_zero`; ambient state, which violates
ADR-012 and `API.md` INV-1; or changing a one-way-door trait *after* fan-out. Lane U1 needs
`C::zero()` with no `C` in hand in at least four places (the zero polynomial's `lc`,
trailing-zero trimming on an empty vector, `eval_horner` of zero, every `resize` in
add/sub). The plan's own worked example already worked around it without noticing —
`plans/api-shape.md:794` writes `fp.zero()`, which is the ring-object arithmetic §Forbidden
shapes bans.

**(2) `Liftable: Ring` with `&[Self::Image]` in its signature does not compile.** `Image`
is an associated type of `Reducible`. A one-way-door signature that has never been
typechecked is not a settled decision.

**(3) `Reducible::Image: Field` is false over algebraic extensions, and not marginally.**
For ℚ(α) with minimal polynomial `f`, reduction mod `p` lands in `GF(p)[x]/(f mod p)`,
which is a field **iff `p` is inert**. The set of inert primes can be *empty*: `f` has one
only if the Galois group of its splitting field contains an `n`-cycle, and for the
multiquadratic towers geometry actually produces — ℚ(√2, √3), Galois group `(ℤ/2)²`, no
4-cycle — **no prime is inert and `Reducible` has no valid implementation at all.** This is
the same Chebotarev obstruction the plan documents for Swinnerton–Dyer factorization
certificates (`plans/verification.md` §2.4) and never connected to the trait bound. The
consequence is precise and bad: M8's `UPoly<NumberField>` compiles as "an added
instantiation, not a rewrite", cannot satisfy `Reducible + Liftable`, and silently gets the
Tier-G reference path — correctness without speed — for SMT NRA, the consumer M8 exists for.

**(4) Batched lanes have no way to report a per-lane fault.** `Field::inv(&self) ->
Option<Self>` must return `None` if *any* lane of an `Fp4` is non-invertible, and `Option`
cannot say which. F4 row reduction needs `inv(pivot)`; a zero pivot in one lane is certain
to happen. Lane Z5's "componentwise equality with N scalar runs" is called a *free complete
oracle*; it is complete for arithmetic and silent on control flow.

---

## Decision

### The boundary rule

> **Generics may cross a crate boundary. They may not cross into an inner loop.**
> Every generic algorithm has a `where C: CoeffRing` entry point and delegates to a
> monomorphic kernel selected by at most one runtime `match` **per call**, never per
> element.

### Three tiers

**Tier G — generic, monomorphized, source-level.** `UPoly<C>`, `MPoly` arithmetic, and the
*reference* implementation of every algorithm. Generic over `C`, and instantiated **by
resolvent** over a closed set: `Fp`, `Fp4`, `Integer`, `Rational`, `Zn`, `GFpk`, and —
behind a `number-fields` feature — `NumberFieldElem`. The set is closed for resolvent's own
compile-time budget; it is **not closed to consumers** (ADR-019 §2). A consumer may
instantiate over a foreign `C`; it gets correctness, not speed, **and the trait's own doc
comment says so in those words**.

`Zn` (composite modulus) is in the set and stays. `plans/api-shape.md` L0-12 declared ℤ/n
out of scope on the grounds that "it is not needed by any modular method (all of which use
prime moduli)"; that is false — **Hensel lifting to `p^k` is arithmetic modulo a
composite**, it is lane K2, it has a certificate row in `plans/verification.md` §2.4, and
M1's exit gate requires it (ADR-021 §3, item 11).

**Tier M — monomorphic, concrete, no trait bounds.** The kernels. Rule of thumb: *any loop
whose body is a single coefficient operation and whose trip count is data-dependent and
unbounded is written over a concrete type.* The list is in `plans/architecture.md` §2.1 and
is exhaustive by intent: F4 row reduction, GF(p) bulk vector ops, Descartes Taylor shift
and sign-variation counting, CRT and rational reconstruction, monomial SWAR ops
(const-generic over word count `W ∈ {1,2,4,8}` — **const** generics, not trait generics),
and the Horner loop inside `AlgebraicReal` refinement.

**Tier D — dynamic, runtime data.** The `Ring` context object: variable count, monomial
order, exponent field width, characteristic, coefficient-ring tag. Consulted once per
phase, never per element (ADR-009).

### The trait vocabulary (`resolvent-base`)

Depth is capped at three levels plus orthogonal capability markers:

```rust
pub trait Ring: Clone + PartialEq + Send + Sync + 'static {
    const LANES: usize;          // 1 for scalar rings, 4 for Fp4
    type Scalar: Ring;           // == Self when LANES == 1

    /// The data needed to name this ring's identity elements.
    /// `()` for Integer/Rational; FpParams for Fp; (p, k, modulus) for GFpk.
    type Ctx: Clone + PartialEq + Send + Sync + 'static;

    fn zero(ctx: &Self::Ctx) -> Self;
    fn one(ctx: &Self::Ctx) -> Self;
    /// Free for every ring in the instantiation set: each already carries its Ctx by value.
    fn ctx(&self) -> &Self::Ctx;

    // Element-to-element arithmetic. Unchanged, and no context enters the inner loop.
    fn add(&self, r: &Self) -> Self;  fn sub(&self, r: &Self) -> Self;
    fn mul(&self, r: &Self) -> Self;  fn neg(&self) -> Self;
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
pub trait Ordered: Ring { fn sign(&self) -> Sign; }

pub trait Reducible: Ring {
    /// NOT `Field`. Reduction of an algebraic-extension element mod p lands in a
    /// product ring whenever p splits, and for some towers no prime is inert.
    type Image: CommutativeRing;
    /// `Err(BadPrime)` when the image is not a domain (split or ramified prime),
    /// or when p divides a denominator. Never a silent zero divisor.
    fn reduce(&self, m: &Modulus) -> Result<Self::Image, BadPrime>;
}
pub trait Liftable: Reducible {
    fn crt_lift(images: &[Self::Image], moduli: &[Modulus]) -> Result<Self>;
}

/// Only for rings with LANES > 1. Makes a per-lane fault expressible.
pub trait BatchField: Ring {
    /// Bit `i` set ⇔ lane `i` is non-invertible. `Ok` ⇒ every lane inverted.
    fn inv_batch(&self) -> Result<Self, LaneMask>;
}
```

Seven load-bearing details:

1. **`Ctx` splits *element* from *ring* without putting the ring in the arithmetic path.**
   This is the fix for defect (1). Only *construction* consults a context, and construction
   is per-call, which is exactly this ADR's own boundary rule. `UPoly<C>` and `MPoly` store
   one `C::Ctx` alongside their coefficients — which they need anyway, because a
   `UPoly<Fp>` that does not know its own `p` cannot be printed, serialized, or compared.
   `Ctx = ()` for `Integer`/`Rational`, so the two rings that had a static zero pay nothing.
   **This is deliberately *not* `feanor-math`'s `RingBase`/`RingStore` split**: that design
   puts the ring object in every *arithmetic* call, which stays forbidden. The dismissal of
   `RingStore` was right about the cure and wrong to conclude nothing was needed.
2. **`Liftable: Reducible`**, so `Self::Image` resolves. Fix for defect (2).
3. **`Reducible::Image: CommutativeRing` and `reduce` returns `Result<_, BadPrime>`.** Fix
   for defect (3). The corollary is a *scoping* decision and it belongs here rather than in
   a lane brief: **the modular path over an algebraic extension is multi-modular over split
   factors** — reduce, factor `f mod p`, work in each `GF(p^{d_i})`, CRT back — which is a
   different algorithm with its own bad-prime predicate. **It is a lane (M8), not an
   instantiation.** ADR-010 §4 carries the predicate; the M8 corpus carries ℚ(√2, √3)
   specifically, because that is the instance where a naive implementation divides by a
   zero divisor.
4. **`BatchField::inv_batch` replaces nothing and enables the split driver.** Fix for
   defect (4). `Field::inv` is unchanged for `LANES == 1`. ADR-010 §7 carries the
   consequence: batched Gröbner requires a batch-split driver, and lane G6's brief is
   "batching **and** splitting".
5. **`LANES` and `Scalar` are on the base trait from day one.** This is the only reason
   `Fp4` remains possible later. One associated const and one associated type.
6. **`Ord` is NOT required on the coefficient ring.** `Fp4` has no meaningful order.
   Ordering is the orthogonal `Ordered` marker, implemented by `Integer` and `Rational` and
   not by `Fp`, `Fp4`, or `GFpk`. Requiring `Ord` on `Ring` would close the batching door
   permanently.
7. **Arithmetic is by-reference (`&self, &Self -> Self`)**, so generic formula bodies never
   deep-copy bignum operands. For `Copy` rings the reference is elided by the optimizer;
   for `Integer` it is the difference between one allocation and three. The defaulted
   in-place forms (ADR-019 §3) exist because Bareiss over ℚ allocates two or three fresh
   bignum rationals per step that are dead one line later; a defaulted body obliges no
   implementor to do anything.

**There is no `BulkOps`.** It was in the original block and is deleted. Tier M exists so
GF(p) bulk vector ops are written concrete over `u32`/`u64` with `FpParams` by value;
re-exposing that kernel as a trait method means either implementing it once per `C` — which
duplicates the kernel across the instantiation set, exactly what Tier M prevents — or a
thin forwarder that buys nothing but a bound every generic call site must carry. Worse, it
blurs the single boundary rule the design rests on, and an agent reads it as licence to add
`fn row_reduce(...)` next to it. **Bulk kernels are free functions in `resolvent-modular`
over concrete types, selected by one `match` on the `RingTag` at the top of each phase.** A
generic caller over an arbitrary `C` gets the naive loop, and the doc comment says so.

### The typecheck obligation

> **This trait block, with a real `impl` for `Fp` and a real `impl` for `Integer`, must
> compile before the freeze.** Not after Wave 1, not "when Z0 lands" — before any lane is
> unblocked against it.

It is one afternoon. The original block was a one-way door that had never been through a
compiler, and two of its four defects were type errors a `cargo check` would have caught in
seconds. Lane Z0's first deliverable is that compiling block plus its trait-law property
tests; nothing else in Wave 1 starts until it is green.

### Explicitly forbidden shapes

- **`Box<dyn Ring>` / `&dyn Ring` in a hot path.** An indirect call per coefficient
  operation. Permitted only in `resolvent-oracles` and in diagnostic/printing code.
- **Ring-object arithmetic — `ring.add(&a, &b)`.** `Fp` is `Copy`, carries `p` and its
  Barrett reciprocal by value, and exposes `#[inline]` inherent methods. This is the
  concrete divergence from `feanor-math`'s model and the reason for it.
- **Specialization (`min_specialization`).** It is a nightly feature; `feanor-math` pins
  `nightly-2026-03-01` and is disqualified as a dependency partly for that reason
  (ADR-002/003). resolvent is stable-Rust-only. Where the spec says "specialization for the
  hot cases", the mechanism is Tier M, not the `specialization` feature.
- **Associated-type projections in public bounds** beyond one level. `where C: Reducible,
  C::Image: Field` is the ceiling; anything deeper produces error messages an agent cannot
  act on.

### Compile-time budget

Monomorphization count is `|generic algorithms| × |instantiations|`. Controls: the closed
instantiation set resolvent itself compiles (ADR-019 §2: closed for resolvent, **open to
consumers**); `number-fields` feature-gated; kernels in Tier M so the expensive code
compiles once; the inner-function trick (a thin generic wrapper converting to a concrete
representation and calling a non-generic body) for large cold generic functions.

**The gate is an absolute per-crate ceiling, not a relative regression.** *Amended
2026-07-31.* The original gate — "fails on a >20% regression in total front-end time" — is
measured against the previous workspace, and in Wave 0 the workspace has no algebra: adding
`resolvent-int` is a >20% regression, and so is adding `resolvent-modular`. Every early
lane trips it against a near-empty baseline, so it is disabled within a fortnight, and a
compile-time budget disabled once never returns — which is precisely how a monomorphization
explosion arrives unannounced. Instead:

- **Absolute per-crate front-end ceilings**, set at the M1 boundary and revised at each
  milestone boundary, **ratcheting down only**, recorded in the same file as the tuning
  thresholds (`tuning-thresholds.toml`). Indicative shape, to be replaced by measurement:
  `resolvent-poly` front-end ≤ 20 s, workspace clean debug build ≤ 90 s on the pinned
  machine.
- **`cargo llvm-lines` top-20 monomorphization counts tracked as the leading indicator**,
  because that moves before wall-clock does.

This is a **score-graded** lane and is sequenced as one.

---

## Consequences

- **The hot loops are readable as ordinary Rust.** An agent optimizing F4 row reduction is
  editing a function over `&mut [u32]` with a `u64` accumulator, not fighting a trait
  hierarchy. This is a significant help for agent-built code.
- **`Fp4` stays possible without being built.** The cost today is one const and one
  associated type; the cost of adding them later is a breaking change to every `impl Ring`
  in existence, including consumers'.
- **Some duplication is accepted deliberately.** The generic reference path and the
  monomorphic kernel implement the same mathematics twice. That is not waste — it is a
  free differential oracle, and ADR-012's tuning-matrix CI check forces them to agree on
  every corpus instance.
- **A consumer's foreign coefficient ring is slow, and is documented as slow.** The
  alternative (making the modular pipeline generic over an open ring set) is not
  implementable: reduction mod `p` and CRT lifting are not operations an arbitrary ring
  has.
- **`Ordered` being absent from `Ring` means generic code cannot compare coefficients.**
  That is correct — comparing elements of GF(p) is meaningless — but it will surprise
  anyone porting code that assumed it. Sign-dependent algorithms (Descartes, Sturm,
  `AlgebraicReal`) carry `C: Ordered` explicitly.

---

## Alternatives considered and why rejected

**Trait-generic everywhere, including kernels, relying on the optimizer.** Rejected. It
works for `Copy` rings and fails for `Integer`, where by-value trait methods force
allocations the optimizer cannot remove; and it multiplies compile time by the
instantiation count on exactly the largest functions.

**`feanor-math`'s `RingBase`/`RingStore` two-trait split.** The best-informed alternative,
and it exists because its author hit real Rust limitations. Rejected because it puts the
ring object in every arithmetic call, which is the ring-object shape forbidden above, and
because the resulting bounds are hard for both humans and agents to read. We accept a
narrower generic surface instead of a more expressive one.

**Runtime dispatch on a `RingTag` enum, no generics at all** (a "dynamic CAS" à la a
Python-style object model). Rejected: one branch per coefficient operation, and it makes
`UPoly<C>` impossible, which forecloses the consumer-supplies-its-own-ring path
(constraint #1).

**Const-generic modulus (`Fp<const P: u64>`).** Rejected for the same reason `ark-ff` is
rejected in ADR-003: a CAS chooses dozens of primes at runtime. Const generics *are* used —
for monomial word count `W`, where the value genuinely is known at the call site.

**Requiring `Ord` on `Ring` "because it's convenient for sorting terms".** Rejected. Terms
are sorted by *monomial*, not by coefficient. Nothing sorts coefficients. Requiring `Ord`
would buy nothing and would cost the batching door.

**Receiverless `fn zero() -> Self` (the original decision, superseded 2026-07-31).**
Rejected: unimplementable for `Fp`, `Fp4`, `Zn`, `GFpk` and `NumberFieldElem` — five of the
seven rings in this ADR's own instantiation set. The three workarounds available to a lane
agent are a poisoning sentinel, ambient state (banned by ADR-012), or changing a one-way
door after fan-out. Recorded here rather than deleted because the original shape is what an
agent reconstructs by default, and because `API.md` §3.2 still shows it.

**`Reducible::Image: Field` (the original decision, superseded 2026-07-31).** Rejected: no
prime is inert for ℚ(√2, √3), so the bound has no valid implementation for a tower geometry
actually produces. Relaxing to `CommutativeRing` with a fallible `reduce` is the minimum
honest signature; the algorithm change (multi-modular over split factors) is scoped to M8
as a lane.

**Keeping `BulkOps` "so a generic caller can get the fast path".** Rejected: a generic
caller *cannot* get the fast path, which is the entire point of Tier M. Offering a trait
method that either duplicates the kernel or forwards to it advertises a capability the
design deliberately does not have.

**`Option<Self>` for batched inversion.** Rejected: it cannot name the faulting lane, so a
batch cannot be split, so a bad prime corrupts the shared control flow instead of producing
a discardable minority. `Result<Self, LaneMask>` costs one line now and is a breaking change
after `LANES` is in use.

---

## What would reverse this

- **A measured need for an instantiation outside the closed set** (e.g. a consumer's ring
  becoming a first-class supported case). That widens the set; it does not change the
  boundary rule.
- **`Fp4` measurably not paying off** on resolvent's corpus. Then `LANES` stays at 1 for
  everything and the const is vestigial — a cost of one line, and removing it later would
  still be breaking, so it stays.
- **Compile time exceeding budget despite the controls.** Response in order: move more
  algorithms behind the inner-function trick; feature-gate more instantiations; last,
  reduce Tier G to `Fp` + `Integer` only and make everything else conversion-based. The
  boundary rule itself does not change.
