# Adversarial critique C1 — the engineering

Status: **critique, for triage.** Written 2026-07-31 against
`plans/{architecture,api-shape,roadmap,verification}.md` and `docs/decisions/ADR-001…018`,
all read in full, plus spot-verification of the consumer citations in
`/home/dev/projects/arrangements`.

The plan is unusually good. Its research is real, its citations check out (I verified
`lazy-exact/src/roots.rs:43-45`, `:317-322`, `lib.rs:82`, and
`arrangements/src/geoms/conics.rs:276-287` directly — all accurate), and it has already
found most of the obvious traps. **This document is therefore not a survey. It is a list
of the places where the plan is wrong, in descending order of what it costs to discover
late.** §11 lists the objections I expected to raise and could not, because false alarms
cost real time and the reader should know which stones have already been turned.

Three findings are fatal in the specific sense that an agent starting Wave 1 on Monday
writes code that must be thrown away. Nine are serious: they do not stop the first commit,
they stop a milestone. The rest are cheap fixes that are cheap only now.

---

## 1. FATAL — `Ring::zero()` is unimplementable for every ring the plan actually uses

**Target.** `plans/architecture.md:222-229` and `docs/decisions/ADR-006-generics-boundary.md:71-88`
— the trait vocabulary, marked *one-way door*, inherited by everything above Layer 0.

```rust
pub trait Ring: Clone + PartialEq + Send + Sync + 'static {
    const LANES: usize;
    type Scalar: Ring;
    fn zero() -> Self;  fn one() -> Self;      // ← here
    ...
}
```

`zero()` and `one()` are associated functions with no receiver and no context parameter.
They can only be implemented by a type that knows its own ring **statically**.

Now read the closed instantiation set the same ADR fixes (`ADR-006:79-82`): `Fp`, `Fp4`,
`Integer`, `Rational`, `Zn`, `GFpk`, `NumberFieldElem`. Of those, exactly **two** —
`Integer` and `Rational` — have a static zero.

- `Fp` "is `Copy` and carries `p` plus its Barrett reciprocal by value" (`ADR-006:130`,
  restated at `architecture.md:209`). `Fp::zero()` therefore has to answer "zero of *which*
  prime field?" from no information at all. It cannot.
- `Zn` carries `n`. `GFpk` carries `p`, `k`, and the modulus polynomial. `NumberFieldElem`
  carries a minimal polynomial. `Fp4` carries four moduli.

**Concrete failure scenario.** Wave 1 lane Z3 is the plan's designated best agent lane
("small, extremely well specified, exhaustively certifiable", `roadmap.md:547`). The agent
writes `impl Ring for Fp`, reaches `fn zero() -> Self`, and has three options, all bad:
(a) a `p = 0` sentinel that poisons every downstream `PartialEq` and every `is_zero`;
(b) a thread-local or `static` "current modulus", which violates INV-1
(`api-shape.md:870`) and ADR-012 outright; (c) change the trait — which is the one-way door
the freeze exists to prevent being changed after fan-out. Meanwhile Wave 2 lane U1 is
writing `UPoly<C>` and needs `C::zero()` in at least four places that have no `C` value in
hand: the zero polynomial's `lc()`, trailing-zero trimming on an empty coefficient vector,
`eval_horner` of the zero polynomial, and every `Vec::resize` in add/sub.

The plan's own draft code already knows the trait is wrong and works around it without
noticing: `api-shape.md:794` writes `vec![fp.zero(); n]` — a **ring-object** call,
`ring.zero()`, in the solverang adapter sketch. That is precisely the shape `ADR-006:128-131`
forbids ("Ring-object arithmetic — `ring.add(&a, &b)` … Never"). The document's own worked
example violates its own trait within 500 lines.

This is also the exact problem `feanor-math`'s `RingBase`/`RingStore` split exists to
solve. `ADR-006:141-146` dismisses that design as "a warning about the design space, not a
template to copy" — and then reproduces the problem it was a response to. The dismissal is
still correct about the *cure* (a ring object in every arithmetic call is unacceptable in
the inner loop); it is wrong to conclude nothing is needed.

**Adjacent, same block, same severity class:** `ADR-006:87` declares

```rust
pub trait Liftable: Ring { fn crt_lift(images: &[Self::Image], moduli: &[Modulus]) -> Result<Self>; }
```

`Self::Image` is an associated type of `Reducible`, not of `Liftable`, and `Liftable`'s
supertrait is `Ring`. **This does not compile.** Same at `architecture.md:238`. A one-way-door
trait signature that has never been typechecked is not a settled decision.

**Fix.** Split element from ring *without* putting the ring in the arithmetic path. The
minimal change that preserves everything ADR-006 actually wants:

```rust
pub trait Ring: Clone + PartialEq + Send + Sync + 'static {
    const LANES: usize;
    type Ctx: Clone + Send + Sync + 'static;   // () for Integer/Rational; FpParams for Fp
    fn zero(ctx: &Self::Ctx) -> Self;
    fn one(ctx: &Self::Ctx) -> Self;
    fn ctx(&self) -> &Self::Ctx;               // free for Fp, which carries it by value
    fn add(&self, r: &Self) -> Self;           // unchanged — element-to-element, inlineable
    ...
}
pub trait Liftable: Reducible { fn crt_lift(images: &[Self::Image], moduli: &[Modulus]) -> Result<Self>; }
```

Arithmetic stays element-to-element, so nothing enters the inner loop; only *construction*
consults a context, and construction is per-call, which is exactly ADR-006's own boundary
rule. `UPoly<C>` then stores one `C::Ctx` alongside its coefficients — which it needs
anyway, because a `UPoly<Fp>` that does not know its own `p` cannot be printed, serialized,
or compared. Typecheck the whole trait block, with one `impl` for `Fp` and one for
`Integer`, **before** the freeze. That is one afternoon and it is currently the highest-value
afternoon in the project.

---

## 2. FATAL — there are two normative API specifications and they specify different libraries

**Target.** `plans/api-shape.md` (header: "Binding on the founding architecture unless
explicitly overturned") vs `plans/architecture.md` + `docs/decisions/ADR-004…015`.

`roadmap.md:422-479` flags **two** contradictions (ADR-013 mutability, ADR-008 interning)
and treats them as the notable exceptions. They are not exceptions. Here is the actual
list, restricted to items that appear in a public signature:

| # | `architecture.md` / ADR says | `api-shape.md` says |
|---|---|---|
| 1 | Crate graph is `base/int/modular/poly/algebra/real/expr` + facade (`architecture.md:52-80`), and CI diffs `cargo tree` against a checked-in graph (gate L1, `:113`) | Crate graph is `seam/int/modular/poly/linalg/engine/alg/expr/lazy` (`api-shape.md:116-128`). `resolvent-linalg` and `resolvent-lazy` do not exist in the architecture; `resolvent-algebra`/`-real` do not exist in api-shape |
| 2 | **No scalar-seam trait in the public API.** "Adding one later is additive; removing one is breaking… the door stays closed and openable" (`architecture.md:703-709`, `:765-766`) — named as the mechanism that keeps ADR-018 option A cheap | Ships `resolvent-seam` with `Scalar`/`ScalarOrd`/`TryDiv`/`Hom` as "the single highest-leverage hook" (`api-shape.md:480-486`) and makes `Interval<f64>` implement it (`:326`) — the exact type architecture cites as the reason to refuse (`architecture.md:706-708`) |
| 3 | ADR-015: no float interval type anywhere in a published crate, enforced by grep gate | `Interval<f64>` is **core** (L0-5, `api-shape.md:156`), appears in `enclosure() -> Interval<f64>` in two adapter signatures (`:691`, `:721`) and as `Vec<Interval<Q>>` (`:632`) |
| 4 | ADR-004 (one-way): coefficients are ℤ-primitive; `AlgebraicReal`'s defining polynomial is `UPoly<Integer>` (`ADR-004:226`) | `poly: Arc<SqfrPoly<Rational>>` (`api-shape.md:93`), `sign_of(h: &UPoly<Rational>)` (`:715`) |
| 5 | ADR-014 §3 (one-way): "Multiplicity is a pair element, **never** a field"; `isolate_roots -> Vec<(AlgebraicReal, u32)>` | `mult: u32` inside the struct (`api-shape.md:95`); `L3-1` names it in the type (`:208`); the adapter calls `self.0.multiplicity()` (`:713`) |
| 6 | ADR-013: `Arc<Inner>`, `Send + Sync` | `Send + !Sync` (`api-shape.md:88-111`, INV-15) — *flagged* by the roadmap |
| 7 | ADR-008: terms are `(MonomialId, Coeff)` into a ring-owned arena | `(PackedMon, C)` inline, "no global monomial interner" (INV-13) — *flagged* by the roadmap |
| 8 | ADR-007: `MPoly` carries a `&Ring` handle and "is not `'static`-free" (`ADR-007:408`) | INV-10: **no public owned type carries a lifetime parameter** (`api-shape.md:906`), justified by consumer evidence |
| 9 | ADR-010: `Certified<T> { value, certificate }`, `ProofKind::{BoundDriven, DivisibilityAndDegree, CofactorRepresentation, ProductAndModularIrreducibility, ExhaustiveSmallCase}` | `Certificate<C: Claim>` with private fields and a claim tether, `ProofKind::{Identity, Divisibility, Cofactor, Enclosure, DegreeBound}` (`api-shape.md:374-389`). `verification.md:64-70` quotes a *third* hybrid |
| 10 | ADR-011: budgets only where no a-priori bound exists; `sign_of(&self, h) -> Sign` and `isolate_roots(p) -> Result<Vec<_>>` are budget-free (`architecture.md:606`, `:616`) | INV-6/X-1: **every** loop that can run long takes a `Budget`; `isolate_roots(&sf, Some(&window), budget)` (`api-shape.md:641`); `sign_of(h, b: Budget)` (`:715`). `verification.md:135` sides with api-shape |
| 11 | `Zn` is in the closed instantiation set (`ADR-006:79`) with a certificate row (`verification.md:179`) | L0-12: ℤ/n is **out of scope** (`api-shape.md:163`) |

Item 2 is the one that matters most. `architecture.md:764-766` names "do not put a
scalar-seam trait in resolvent's public API" as the first entry on the list of *things not
to do so that ADR-018's options stay open*, and `ADR-018`/`architecture.md:730` prices
option A ("a public generic parameter cannot be removed without a major version and a
rewrite of every consumer") as the most expensive mistake available. api-shape ships the
seam as its headline integration story. One of these two documents is wrong about the
single most expensive decision in the plan, and nothing in the freeze machinery notices.

**Concrete failure scenario.** Wave-1 lane Z7's deliverable is "the `Certificate` type,
error taxonomy, budget plumbing" (`roadmap.md:543`), and the freeze rule says a lane may
not start against a declared-but-unwritten ADR. ADR-010 and ADR-011 *are* written — the
files exist. So the gate fires green, the agent implements `Certified<T>` per ADR-010, and
every Wave-2 and Wave-3 lane that was written against `api-shape.md`'s
`Certificate<C: Claim>` / `Certainty` / `is_decline()` has to be rewritten. Same for
`Budget` on `isolate_roots`, which is called by three of the four milestone exit gates.

**Aggravating factor.** `roadmap.md:382-386` lists ADR-010, 011, 012, 015, 016 as
"**declared, unwritten**". All five files exist in `docs/decisions/`. The freeze mechanism
— "Wave 2 CI jobs do not exist until the ADRs they inherit are merged" (`roadmap.md:864`) —
keys on a status the roadmap already reports wrongly, and on "merged", which every ADR
already is.

**Fix, in order.**
1. Declare one document normative for signatures. My recommendation: `api-shape.md`'s
   *consumer-facing decisions* (budgets, error enum, no-lifetime, runtime arity) are better
   evidenced — they are derived from three real consumer evaluations — and
   `architecture.md`'s *internal decisions* (ℤ-primitive, arena, LANES) are better argued.
   Merge them item by item, and record each merge as an ADR amendment. Items 2, 3, 4, 5 and
   10 are genuine decisions, not editorial drift, and each needs a paragraph.
2. Replace "is the ADR merged" with a machine-readable gate: a `Status: Ratified` field
   parsed out of the ADR front matter, plus a checked-in `lane → required-ADR` manifest that
   CI reads. Until then the freeze is a convention.
3. Add a CI check that greps every code block in `plans/` and `docs/decisions/` for the
   headline type names and fails on divergent definitions. Cheap; it is the only thing that
   would have caught eleven of these.

---

## 3. FATAL — `groebner_certified` cannot share a reducer with `groebner`, which removes the fast mode's only internal oracle

**Target.** `ADR-010:120-131` §5, `verification.md:235-238` §2.5, `roadmap.md:419`.

ADR-010 §5 makes a specific structural promise:

> **The two modes share one reduction implementation.** The Macaulay row representation
> carries an **optional cofactor block** … reduced by the same code … Two separate reduction
> implementations would mean the certified mode is not testing the fast mode's reducer,
> which is the fast mode's *only* internal oracle.

That promise is incompatible with three other ratified decisions.

1. `architecture.md:188` puts F4 row reduction in Tier M, "concrete over `u32` payloads +
   `FpParams` by value; sparse row format". The fast reducer is a GF(p) kernel over `u32`.
2. `verification.md:235` costs the certified mode as "`|F|` normal forms **over ℚ**, with
   full coefficient blowup", and `:237` checks cofactors "by multiplication and addition"
   — over ℚ or ℤ, because a cofactor identity that holds mod `p` proves nothing about ℚ.
3. Therefore the certified mode's reducer is a ℚ/ℤ reducer and the fast mode's is a `u32`
   GF(p) kernel. They cannot be the same code. The shared "optional cofactor block" is a
   shared *row format*, not a shared reducer.

**Concrete failure scenario.** Lane G4 ships `groebner_certified` over ℚ. Lane G3 optimizes
the `u32` sparse GF(p) kernel — the plan's own hardest lane, 73–91% of an F4 run
(`roadmap.md:613`). A bug in G3's pivot selection, in its delayed-reduction cutoff, or in
its Barrett reduction is invisible to G4, because G4 never executes a line of it. The
cross-check degrades to "two implementations of Gröbner agree", which is a normal
differential test with substantial shared machinery — and `verification.md:503` already
grades that pair as "**substantial sharing; this cross-check is weaker than it looks**". The
plan therefore contains its own refutation of ADR-010 §5 and does not connect the two.

**A second, uncosted problem in the same place.** To be a certificate over ℚ, the cofactors
`h_ij` must be *reconstructed*, not just computed mod p. Cofactor coefficients are
systematically larger than basis coefficients — that is what "cofactor swell" means — so
**the number of primes needed is set by the cofactors, not by the basis.** ADR-010 §Context
quotes Cyclic-10 needing >2000 primes for the *basis*; the cofactor system for the same
instance needs more, and there are `|F| × |G|` of them. The prototype gate
(`ADR-010:129-131`) measures "memory and time multiplier of cofactor tracking on Katsura-8 /
Cyclic-7" — over GF(p), where the multiplier is a constant factor on row width. That
measurement does not answer the question that matters, which is the multiplier on the
**reconstruction**, and the reconstruction multiplier is the one that decides whether
certified mode exists at all.

**Fix.**
- Rewrite ADR-010 §5 to say what is actually true: the two modes share the *matrix
  construction*, the *symbolic preprocessing*, the *monomial layer* and the *row format*, and
  **not** the reducer. Then state honestly that the fast reducer's primary verdict is
  external differential testing (Singular/msolve), which `roadmap.md:711-717` already
  recommends for G5 — extend that inversion to G3.
- Give G3 an internal oracle it can actually have: a naive dense `u32` Gaussian elimination
  over the same `FpParams`, in the same crate, as the reference. That is a genuine
  same-arithmetic cross-check and it costs one agent-session.
- Change the cofactor prototype's success criterion from "GF(p) time and memory multiplier"
  to "**number of primes and wall time to reconstruct the cofactor system over ℚ on
  Katsura-8**", and re-run the decision against that number.

---

## 4. SERIOUS — modular methods do not apply to algebraic-extension coefficients, and the trait says they do

**Target.** `ADR-006:86` (`Reducible { type Image: Field; }`), `ADR-010:47-51`,
`ADR-014:378-388` §5, `roadmap.md` M8.

`Reducible::Image: Field` asserts that reducing a coefficient mod `p` lands in a field. For
ℤ and ℚ that is true. For `NumberFieldElem` — ℚ(α) with minimal polynomial `f` — reduction
mod `p` lands in `GF(p)[x]/(f mod p)`, which is a field **iff `f` stays irreducible mod `p`**
(the prime is inert). Otherwise it is a product ring with zero divisors, and the honest
handling is to split it by CRT into several extensions of different degrees.

Worse, the set of inert primes can be **empty**. `f` has an inert prime only if the Galois
group of its splitting field contains an `n`-cycle. For the multiquadratic fields geometry
actually produces — ℚ(√2, √3), the biquadratic tower behind
`arrangements`' `sine_radical` and `circle_segments` families — the Galois group is
`(ℤ/2)²`, which has no 4-cycle, so **no prime is inert and `Reducible` has no valid
implementation at all.** This is the same Chebotarev obstruction the plan already documents
for factorization certificates (`verification.md:227`, "Swinnerton–Dyer polynomials factor
nontrivially modulo *every* prime") — it just never connects the two, because one is about a
certificate and the other is about a trait bound.

**Concrete failure scenario.** M8 lands `UPoly<NumberField>` "as an added instantiation, not
a rewrite, because `UPoly<R>` was generic from day zero" (`roadmap.md:322-323`, restating
`ADR-014:378`). The instantiation compiles. It is then discovered that
`C: Reducible + Liftable` cannot be satisfied, so `UPoly<NumberFieldElem>` gets the Tier-G
reference path — the one the docs describe as "correctness, not speed"
(`architecture.md:179`). SMT NRA (#12), the consumer M8 exists for, does root isolation over
ℚ(α₁…α_k) in its inner loop. The "added instantiation, not a rewrite" claim survives for
correctness and fails for the only thing M8 was for.

**Fix.** Two lines of trait, one paragraph of doc, now:

```rust
pub trait Reducible: Ring {
    type Image: CommutativeRing;          // NOT Field
    /// Err(BadPrime) when the image is not a domain (split/ramified prime).
    fn reduce(&self, m: &Modulus) -> Result<Self::Image, BadPrime>;
}
```

and state in ADR-010 that for algebraic-extension coefficients the modular path is
**multi-modular over split factors** (reduce, factor `f mod p`, work in each `GF(p^{d_i})`,
CRT back), which is a different algorithm with a different bad-prime predicate, and that it
is out of scope until M8 — at which point it is a *lane*, not an instantiation. Also add
ℚ(√2, √3) to the M8 corpus specifically, because it is the instance where the naive
implementation silently divides by a zero divisor.

---

## 5. SERIOUS — the shared monomial arena forbids the parallelism the plan is counting on

**Target.** `ADR-008:75-81` (arena owned by `Ring`, `MPoly` holds `Arc<Ring>`),
`ADR-012:74-79` §4 and `:82-88` §5.

Three requirements, pairwise fine, jointly contradictory:

1. `ADR-012:76` — "`MonomialId`s are assigned in **first-encounter order** under a
   deterministic traversal, so the ids themselves are reproducible. This matters because any
   tie-break that consults id order would otherwise smuggle hash order into the result."
2. `ADR-012:85` — "Shared mutable accumulators updated from `for_each` are **banned**."
3. `ADR-008:75` — the arena is owned by the `Ring`, and `MPoly` holds `Arc<Ring>`, so the
   arena is shared, mutable, and reached by every polynomial operation that creates a
   monomial.

An interner *is* a shared mutable accumulator. Symbolic preprocessing — the phase that
discovers which monomials become matrix columns — is the natural second parallel target
after the row reduction, and it is nothing but interning. Under (2) it may not be
parallelized at all. If it is parallelized anyway, the id assigned to a monomial depends on
which thread reached it first, violating (1), and the plan has just said out loud that
tie-breaks consult id order.

**Concrete failure scenario.** An agent on lane P3 or G2 writes
`terms.par_iter().map(|t| ring.intern(t)).collect()`, which *looks* like the permitted shape
(`par_iter().map().collect()`, `ADR-012:84`) because the combination is ordered. The ids
are not. Gate 0's thread-count matrix (`verification.md:855`) catches it — good — but only
if some downstream tie-break actually consults id order on that instance, which is
data-dependent. It will pass for months and then fail on one corpus instance, and the
minimizer will not shrink it because the bug is in the schedule, not the input.

**Fix.** Say explicitly, in ADR-008 and ADR-012 both:
- Interning is **serialized** or two-phase (collect candidates in index order, then intern
  single-threaded in that order). Note the cost: it caps the parallel speedup of matrix
  construction, and the plan should stop implying that phase is parallelizable.
- **Better**: make ids order-free. Assign each monomial an id derived from its packed key by
  a deterministic content hash with a fixed collision-resolution order, so id is a pure
  function of the monomial and not of encounter order. Then parallel interning is
  deterministic and the tie-break question evaporates. This costs one design pass now and is
  unavailable after P2 freezes the term type.
- Either way, add an INV: **no tie-break anywhere may consult `MonomialId` ordering.**
  Tie-break on the key. The key is a total order by construction and is content-derived.

---

## 6. SERIOUS — shared refinement makes step budgets, declines, and telemetry schedule-dependent

**Target.** `ADR-013:65-91` (`Arc<Inner>`, shared refinement across clones),
`ADR-011:89-106` (step budgets), `ADR-012:37-38` (bit-identical output at any thread count),
`verification.md:741` (a decline counts as a failure), `api-shape.md:241` (`Telemetry`).

ADR-013's whole value proposition is that cloning shares refinement progress
(`ADR-013:76-77`) so a sort of `n` algebraic numbers does not redo bisection. That means the
number of refinement steps a given `cmp` performs is a function of **what has already been
compared** — and, under `parallel`, of what other threads have already compared.

Consequences the plan does not draw:

- **Budget exhaustion becomes history-dependent.** `ADR-011:103-105` makes the budget a real
  control-flow exit wherever no a-priori bound exists. A call that declines when run first
  succeeds when run after a warm-up comparison. `verification.md:741` grades a decline inside
  a property test as a **failure**, so property-test outcomes now depend on test execution
  order — and `proptest`-style shrinking reorders.
- **Under threads it is worse.** Two threads sorting overlapping slices of the same
  `Vec<AlgebraicReal>` refine each other's operands. At `RAYON_NUM_THREADS=8` a comparison may
  need 3 steps; at 1 it needs 40. If any budget is tight, the *verdict* (`Ok` vs
  `Err(BudgetExhausted)`) differs by thread count, which is precisely what
  `verification.md:855` asserts cannot happen.
- **`Telemetry { bisections, precision_bits }`** (`api-shape.md:459`) is explicitly returned
  to consumers as plain data. It is nondeterministic by construction under this design. So is
  `TraceEvent::BudgetTick` (`ADR-012:111`), which is inside the `Trace` whose replay CI
  asserts byte-identity (`ADR-012:116`).

**Fix.** State the invariant precisely and test it: *the refinement cache may change how
much work a call does; it may never change what the call returns, including whether it
returns a decline.* That forces one of:
- budgets on `AlgebraicReal` operations are **derived from the separation bound** and never
  from elapsed steps (i.e. ADR-011's "bound exists ⇒ budget is a bug detector" branch, always,
  for this type) — my recommendation; or
- the budget is charged against a **worst-case** step count computed from the operands'
  degrees and coefficient sizes rather than against actual work done.

And: exclude `Telemetry`, `Evidence`, and `BudgetTick` from canonical bytes explicitly
(§4.5's canonical-serialization list covers polynomials, bases and algebraic numbers only —
it never says what a `Certificate` serializes to, and the tuning-matrix byte-identity gate
at `ADR-012:129-131` cannot pass if `primes_used` is in the bytes and `N` is a tuning knob).

---

## 7. SERIOUS — `Ord` is total, infallible, and unbounded in practice, and it is the default path

**Target.** `architecture.md:612-632`, `ADR-013:92-97`, vs `api-shape.md:891` INV-6 and
`verification.md:135`.

The `Ord` argument is: a Mignotte–Davenport separation bound converts "terminates eventually"
into "terminates in a computable number of steps", therefore there is no failure to report,
therefore `Ord` is honest. The mathematics is right. The engineering conclusion does not
follow: **computable is not attainable.** For the resultants M4 produces — the plan's own
estimate is degree ~200 with ~500-bit coefficients (`ADR-004:198`) — the Davenport–Mahler
bound is on the order of `2^{-d(τ + log d)}`, i.e. tens of thousands of bits of refinement
in the worst case. An `Ord::cmp` that hits it does not return in any useful sense, and
`Ord::cmp` cannot decline: no `Result`, no budget, no way out.

`api-shape.md:854` half-notices ("`Ord` may allocate unboundedly") and adds
`cmp_exact(other, budget)` alongside. That leaves the *unbounded* one as the default, and
the default is what `sort()`, `BTreeMap`, `binary_search` and `max()` call — which is the
whole reason `Ord` was wanted. `api-shape.md:943` then lists the allocation bound under
"what this document does not settle".

This is the failure mode the plan itself calls the deadliest: `verification.md:257` —
"a hang in a library is worse than a wrong answer because it is undebuggable in production".
The plan has arranged for the flagship type's most-used entry point to be the one place a
hang is unreportable.

**Fix (pick one, and write it down).**
- **Budget-carrying values.** `AlgebraicReal` carries a construction-time step ceiling in
  `Inner`; `cmp` honours it and, on exhaustion, returns the verdict implied by the current
  enclosures if they are disjoint, or **panics with a documented, greppable message** if they
  are not — and the docs state that reaching it requires an input beyond the documented
  capability envelope. Ugly, honest, and it violates ADR-011's no-panic rule, which is why
  the alternative is probably better:
- **Keep `Ord`, but make the ceiling a diagnostic counter that fires long before the
  theoretical bound**, and add a `try_cmp(&self, &Self, Budget) -> Result<Ordering, Decline>`
  that consumers on a latency path are *documented and benchmarked* to use. Then measure the
  actual step distribution on the M4 corpus (lane Y1 can do it) and publish it. If the 99.9th
  percentile is small, `Ord` is fine and the plan can say so with evidence instead of a bound.

Either way: **do not leave this in "what this document does not settle" past M3.** It is
visible in every signature and it is the single most-called function in the consumer.

---

## 8. SERIOUS — "modular methods keep the bignum work sub-kbit" is contradicted by the plan's own numbers

**Target.** `ADR-002:44-54`, against `ADR-010:14-19` and `verification.md:371, 795`.

ADR-002 rests the entire bignum decision on this:

> Above ~1 Mbit: unknown post-NTT, and *irrelevant if the architecture is honoured*, because
> megabit integers appear in a CAS exactly when someone computes over ℤ or ℚ directly instead
> of mod several primes and reconstructing. (`ADR-002:46-48`)

That is false, and ADR-010 refutes it three pages later. The modular architecture does not
eliminate large integers; it **concentrates** them in exactly two places:

- **The CRT modulus `M = Π pᵢ`.** Cyclic-10 needs ">2000 primes of 29 bits (≈58 000 bits)"
  (`ADR-010:16`). Hexapod needs 1102 primes for a computation whose single modular run takes
  0.00 s (`verification.md:795`) — that is ≈70 kbit at 63-bit primes, and the plan
  deliberately puts Hexapod in the corpus "from the first modular milestone".
- **Rational reconstruction**, which is `gcd_ext` on integers of size `M`. This is precisely
  the operation where the plan has identified its one structural pure-Rust deficit: dashu has
  Lehmer (quadratic worst case), GMP has subquadratic half-GCD (`ADR-002:56-58`).

So the load-bearing bignum operation is a ~58–70 kbit extended GCD, performed once per
reconstructed coefficient, on an instance the plan has chosen specifically because it is
reconstruction-bound. At 70 kbit — ~1100 limbs — Lehmer is ~10⁶ word operations per
reconstruction against half-GCD's ~10⁴–10⁵. For an instance with thousands of coefficients to
reconstruct, that is the difference between seconds and minutes, and it is on the *default*
certified path.

**Aggravating.** Lane Z2's measurement ladder is "`gcd`/`gcd_ext` at 64 / 256 / 1k / 4k /
16k bits" (`roadmap.md:529`). It **stops one order of magnitude below the regime that
matters**, so the gating measurement cannot detect the problem it exists to detect.

**Fix.**
1. Extend Z2's ladder to 64k and 256k bits, and add a `rational_reconstruct` microbenchmark
   at Hexapod's modulus size, before `resolvent-int` is written. Half a day, and it is the
   difference between an informed decision and a slogan.
2. Rewrite `ADR-002:44-54` to say the truth: modular methods keep the *bulk* of arithmetic
   sub-kbit and concentrate a *small number* of very large operations in CRT and
   reconstruction. Then price those explicitly.
3. Note the cheap mitigations so they are on the record: incremental (Garner) CRT keeps the
   accumulation small-step; early-termination rational reconstruction with a doubling
   modulus avoids the full-size `gcd_ext` in the common case; and a half-GCD implemented
   *inside* `resolvent-int` is a self-contained, `rug`-certifiable lane (`ADR-002:159-162`
   already anticipates this — promote it from "what would reverse this" to a planned M1
   contingency with a trigger threshold).

---

## 9. SERIOUS — FGLM needs two orders live simultaneously, and the order-in-the-key design does not provide that

**Target.** `ADR-009:333-334` ("Cost: the key must be recomputed if the order changes.
Changing a ring's order means re-interning. That is correct … and it is what FGLM does
anyway"), `roadmap.md:617` (G7, certificate-graded, size L).

That parenthetical is wrong about what FGLM does. FGLM does not convert a basis by
re-encoding monomials. It walks monomials in **lex** order, computes the normal form of each
**modulo the drl basis** (which requires drl lead-term comparison and drl divisibility
queries against the drl divisor index), and does linear algebra over the quotient basis. Both
orders are live in the same loop, on the same monomials, for the whole computation.

Under ADR-009 the order is baked into the key at intern time and the arena belongs to the
ring. So FGLM needs two `Ring`s, two arenas, two key encodings of every monomial it touches,
and a maintained bijection between the two id spaces — plus, if the divisor index is built
per-ring (it is; it indexes the arena), two indices. None of this appears in the design, and
`roadmap.md:617` sizes G7 as a normal `L` lane with an easy certificate.

**Additional consequence not drawn.** `ADR-009:308-309` says `groebner(ideal, Order::Lex)`
is legal and internally runs drl + FGLM. The input polynomials arrive over the caller's lex
`Ring`; the computation must build a drl `Ring`, re-intern **every input monomial and every
intermediate**, compute, then map the result back into the caller's lex ring. That is a real
cost on the plan's own documented "correct" path for lex, and a real determinism surface
(two id assignments, two first-encounter orders).

**Fix.** Decide now, in ADR-009, which of these FGLM gets, and size G7 accordingly:
- a **dual-key `MonomialEntry`** (`key_a`, `key_b`, `raw`, `divmask`) for rings created as a
  conversion pair, so one arena serves both orders and ids are shared — my recommendation,
  because it costs one word per distinct monomial and deletes the bijection entirely; or
- an explicit `OrderPair` ring type with a maintained id map, documented as FGLM-only.

And correct the claim at `ADR-009:333-334`. An agent reading it will implement re-interning
and discover in week three that it does not express the algorithm.

---

## 10. SERIOUS — divisibility is an inner loop, and ADR-009 puts an order-dependent branch in it

**Target.** `ADR-009:18` and `:75-77` vs `ADR-006:66-68` (the boundary rule) and
`ADR-008:59`.

ADR-009 says order-specific work happens in exactly three places, "all O(1) and all outside
sort inner loops": encode, the constant subtract on multiply, and **the divisibility
direction**. The first two are genuinely O(1)-per-operation. The third is not "outside an
inner loop" — divisibility is *the* inner loop of symbolic preprocessing and of reducer
selection, and ADR-008's own driver ranking puts the divisor-query index at 10–20×
(`ADR-008:27-29`) precisely because it is called so often.

An order-dependent branch inside the divisibility test violates ADR-006's boundary rule
verbatim: "at most one runtime `match` per *call*, never per element" (`ADR-006:67`).

The good news is that the plan already contains the fix and does not notice: `ADR-008:59`
stores `raw: [u64; W]` — "raw packed exponents; divisibility, lcm, gcd, degree queries".
Raw exponents are **order-free**. Divisibility on `raw` is a single SWAR per-field
comparison with no order, no complement fields, and no branch.

**Fix.** One-line correction to `ADR-009:75-77`: order-specific work is **two** places
(encode, and the constant subtract on multiply). Divisibility, lcm, gcd and degree are
computed from `raw` and are order-free. Say it explicitly, because as written a lane brief
will produce an `Order`-matching divisibility routine in the hottest loop in the library.

**Related, smaller, same block.** `MonomialEntry` declares `key: [u64; W]` and
`raw: [u64; W]` with a single `W`. The field counts differ by order: lex needs `n` key
fields, grlex needs `n+1`, grevlex needs `n` (`ADR-009:20-26`), while `raw` always needs `n`.
For grlex at 8 variables and 8-bit fields, `raw` fits in one word and `key` needs two. A
single `W` either wastes a word or overflows. Split into `W_KEY` and `W_RAW`.

---

## 11. SERIOUS — three of the plan's blocking experiments require the artifact they gate

**Target.** `roadmap.md:453-479` (contradiction 2's experiment), `ADR-010:129-131` (cofactor
prototype), `ADR-013:190-197` / `roadmap.md:446-451` (the mutability experiment).

| Experiment | Gates | Requires |
|---|---|---|
| "Microbench inline packed monomials vs ids-plus-arena **on a realistic S-pair queue workload**" | P1, P2, P3 — the entire multivariate trunk | A realistic S-pair queue, i.e. a working Buchberger/F4 (lane G1/G2, Wave 4) |
| "Prototype cofactor tracking **on Katsura-8 / Cyclic-7** and measure the multiplier" | committing `groebner_certified` to the plan | An F4 or Buchberger that reaches Katsura-8 (Wave 4) |
| "Sorting `n = 10³` algebraic numbers of degree 8 with 200-bit coefficients" | A1, i.e. all of M3 | A working `AlgebraicReal` with refinement (lane A1) |

Each is correctly identified as the right question. Each is scheduled before the thing that
can answer it. As written, the freeze deadlocks: P2 waits on an experiment that waits on G2
that waits on P2.

**Fix.** Specify each experiment against a **synthetic harness**, and put the harness
definition in the ADR so the result is comparable to the real thing:

- *Monomials*: replay a **recorded** S-pair trace. Generate it once from a 200-line
  throwaway Buchberger over GF(p) with `Vec<u32>` exponents on Katsura-6/Cyclic-6 — a day of
  work, discarded afterwards — and emit `(lcm-query, divisibility-query, insert)` operation
  streams. Benchmark both term representations against the recorded stream. This measures the
  right access pattern without needing the real engine.
- *Cofactors*: measure on **Buchberger** with cofactors at Katsura-6/7 over ℚ, and report the
  multiplier as a function of instance size so it can be extrapolated, plus the
  reconstruction-prime count per §3 above. Do not wait for F4.
- *`AlgebraicReal` mutability*: the four prototypes need only `cmp`, `refine`, and a
  polynomial sign evaluation — roughly 300 lines behind one trait over `UPoly<Integer>` from
  M2, with roots generated as `Π(x−rᵢ)` products. It does **not** need the production
  isolator. Say so, or A1 blocks on itself.

---

## 12. SERIOUS — Gate 0's five-minute budget cannot survive an append-only regression corpus

**Target.** `verification.md:845-860` (Gate 0, "target < 5 minutes") vs `:651` (regression
corpus is append-only, 100% gate) and `:855` (determinism: every regression instance run
twice in-process, twice cross-process, at 1/2/8 threads, across feature combinations).

Count the executions. "Twice in-process, twice cross-process, at 1/2/8 threads, across
feature combinations" is at minimum 4 × 3 = 12 runs of the **entire regression corpus**, per
commit, plus one more for the 100% gate, plus
`:859` — "self-certification assertions enabled in the test profile (every operation checks
its own certificate on every call in tests)". Several of those certificates are `~1×` or
`>1×` by the plan's own cost column: the gcd certificate is a second gcd (`:210`), Sturm's
count is `>1×` at high degree (`:216`), the Gröbner S-pair certificate is "≈ recomputing the
basis" (`:236`).

So Gate 0 costs roughly `13 × (corpus runtime) × (1 + certificate overhead)`, against a
corpus that is contractually append-only and grows with every bug ever found.

**Concrete failure scenario.** Month three. The corpus has 400 minimized instances,
including several Mignotte and Swinnerton–Dyer entries that exist precisely because they are
slow. Gate 0 takes 40 minutes. Someone reduces the thread matrix to `{1, 8}`, then drops
cross-process, then moves the corpus to Gate 1. The determinism gate — which
`verification.md:453` calls the one that "must exist from day 1 because every other
regression artifact depends on it" — is the first thing sacrificed, because it is the most
expensive and the least often red.

**Fix.** Tier the corpus at day 1, before it has anything in it:
- **`fast` tier** (per-commit, budgeted at 90 s): every instance, at 1 and 8 threads,
  in-process only, certificates on. Instances enter `fast` by default and are *promoted out*
  when they exceed a committed per-instance time cap.
- **`full` tier** (Gate 1): everything, full determinism matrix.
- **`slow` tier** (nightly): the Mignotte/Swinnerton-Dyer/Hexapod class.
- CI prints the tier census and fails if `fast` exceeds its budget — which forces promotion
  to be a deliberate, visible act rather than a silent gate erosion.
- Make self-certification a **profile flag** (`cfg(resolvent_self_check)`), on in `full` and
  `slow`, sampled at 10% in `fast`.

---

## 13. SERIOUS — `forbid(unsafe_code)` and the "Competitive" F4 gate cannot both hold

**Target.** `api-shape.md:240` (X-7), `verification.md:854`, `roadmap.md:707-709`,
`verification.md:791` ("Competitive: Cyclic-9 < 600 s … ≈ 2× SOTA").

The plan notes that "msolve reports AVX2 halving the linear-algebra time" and that SIMD
"sits behind the `unsafe` confinement rule and needs its own audit" (`roadmap.md:708-709`).
It does not draw the arithmetic conclusion: linear algebra is 73–91% of an F4 run, so giving
up AVX2 gives up roughly a **1.6–1.8× overall factor**, against a "Competitive" target set at
2× SOTA. The gate and the policy are within noise of each other, and the plan is also
stable-Rust-only (`ADR-006:135-138`), which forecloses `portable_simd` — so the only route to
AVX2 is `core::arch` intrinsics, which are `unsafe`.

There is a second, quieter cost: auto-vectorization of a sparse GF(p) `axpy` with Barrett
reduction is unreliable in stable Rust, because the reduction step involves a widening
multiply and a conditional subtract that LLVM vectorizes inconsistently across versions. A
performance series whose level shifts on a compiler upgrade will trip the change-point
detector (`verification.md:829`) with no code change, which is the fastest way to teach
everyone to ignore it.

**Fix.** Decide now, and write it into ADR-003 or a new ADR:
- Name **one** `unsafe`-permitted leaf: `resolvent-modular::simd`, with `#![allow(unsafe_code)]`
  scoped to that module, every block carrying `SAFETY:`, runtime feature detection, and a
  **scalar fallback that CI asserts is bit-identical** (which it will be — these are exact
  integer ops, so the SIMD path is a pure speed change and cannot alter a value). That is a
  genuinely auditable exception and it preserves both the determinism story and the
  performance ceiling.
- Or keep `forbid(unsafe_code)` everywhere and **lower the published gate**, from
  "Competitive ≈ 2× SOTA" to "≈ 3–4× SOTA", stating AVX2 as the reason. Either is defensible.
  Publishing a target that the policy forbids reaching is not.
- Pin the compiler version in the benchmark record (already required, `verification.md:832`)
  and treat a compiler bump as a re-baseline event, same as a fleet version bump.

---

## 14. SERIOUS — `Fp4` / `LANES` has no story for the moment one lane goes bad

**Target.** `ADR-006:76-78` and `:120-123` (LANES on the base trait "from day one … the only
reason `Fp4` remains possible later", marked one-way), `ADR-010:141-145` §7,
`roadmap.md:541` (Z5), `:616` (G6).

Batched multi-modular arithmetic works only while all `N` primes behave identically. Two
things break that, and both are certain to happen:

1. **A pivot that is zero in one lane.** F4 row reduction needs `inv(pivot)`. With
   `Field::inv(&self) -> Option<Self>` (`ADR-006:84`), `Fp4::inv` must return `None` if *any*
   lane's component is zero — and `Option` cannot say **which** lane. The correct response is
   to split the batch, finish the good lanes, and re-run the bad prime alone. There is no way
   to express that through the trait as specified.
2. **Lead-monomial divergence.** `ADR-010:104-107` makes the Gröbner bad-prime rule "majority
   vote over lead-monomial sets" across primes. Under batching, all `N` primes share one
   matrix construction and one pair-selection path (that sharing *is* the 2.7×), so a prime
   whose lead-monomial set diverges silently corrupts the shared control flow rather than
   producing a minority vote to discard.

The plan treats Z5 as "componentwise equality with N scalar runs — a **free complete
oracle**" (`roadmap.md:541`). That oracle is complete for *arithmetic* and says nothing about
either failure above, both of which are control-flow.

**Fix.** Now, while it is one line: make the batched-lane failure *expressible*.

```rust
pub trait BatchField: Ring {
    /// Bit `i` set ⇔ lane `i` is non-invertible. 0 ⇒ all lanes fine.
    fn inv_batch(&self) -> Result<Self, LaneMask>;
}
```

and write into ADR-010 §7 that batched multi-modular Gröbner requires a **batch-split
driver** — on any lane fault or lead-monomial divergence, the batch splits and the offending
prime index is recorded in the `Trace` (which ADR-012 already provides for). Then G6's lane
brief says "implement batching **and** splitting", not "implement batching". Without this,
G6 produces a fast path that is silently wrong on exactly the instances bad primes exist for.

---

## 15. MINOR — Layer 4's scope has quietly grown a `simplify()`, three times over

**Target.** `ADR-017` §1, §4, §5 vs `api-shape.md:227-228` (L4-8, L4-9) and `roadmap.md:296`
(M7: "**No code emitter. No transcendental zero-test.**").

The source spec's named risk is "refusing to add a `simplify()` function that tries to be
clever" (`IDEAS-crates.md`, quoted at `roadmap.md:863`). Three documents currently give three
different answers:

- `api-shape.md:228` L4-9 — "A general `simplify()`: **out-of-scope**. The spec's own risk
  section says to refuse it. Both L4 consumers independently confirm."
- `api-shape.md:227` L4-8 — e-graph integration **out of scope for core**; cadabra2 "actively
  hostile" because a canonicalizing rewriter destroys its `Cos2` tether.
- `ADR-017:519-535, :615-621` — ships a `Simplifier` **trait**, a built-in bottom-up
  rewriter, `simplify(expr, rules)` as "the public entry point", named default rule sets
  (`RuleSet::polynomial_normal_form()`, `RuleSet::fem_lowering()`), and **both** `egg` and
  `egglog` adapters "later, as optional non-default features".
- `ADR-017:610-613` §4 additionally ships "exact symbolic integration of polynomials over
  reference simplices", for consumer #34 — which is not one of the three consumers evaluated,
  and whose demand is documented nowhere in `api-shape.md` §2.
- `ADR-017:604-607` §3 has `resolvent-expr` depend on `resolvent-algebra` "it wants gcd for
  **rational-function normalization**" — while `api-shape.md:181` L1-12 puts the rational
  function type out of scope with no consumer.

None of these individually is scope creep. Together they are a rewriting engine with a rule
language, two backend adapters, a symbolic integrator and rational-function normalization,
in the layer the spec calls "not the point", specified across two documents that contradict
each other about whether it exists.

**Fix.** Hold the line at what `roadmap.md` M7's exit gate actually tests: hash-consing,
`diff`/`diff_with`, constant folding, `walk_topological`, `is_polynomial_in`, canonical
bytes. Everything else in ADR-017 — `Simplifier`, `RuleSet`, the built-in rewriter, simplex
integration, rational-function normalization — moves to a "post-v1, on consumer demand"
section of the same ADR, explicitly. Keep `FuncTable` (it is genuinely the synthesis that
serves all three consumers, and `api-shape.md:851` prices it honestly as a deliberate scope
addition). Delete the `resolvent-algebra` dependency until something needs it; `ADR-017:604`
justifies it with a capability that is out of scope.

---

## 16. MINOR — the `Certificate` type has three incompatible definitions and Wave 1 is scheduled to build it

**Target.** `ADR-010:54-68`, `api-shape.md:374-389`, `verification.md:64-70`, and
`roadmap.md:543` (lane Z7).

Three shapes:

| Doc | Shape |
|---|---|
| ADR-010 | `Certified<T> { value, certificate }`; `Certificate::{Proved(ProofKind), Probable(Evidence)}`; `ProofKind::{BoundDriven, DivisibilityAndDegree, CofactorRepresentation, ProductAndModularIrreducibility, ExhaustiveSmallCase}` |
| api-shape | `Certificate<C: Claim> { claim, evidence, certainty }` with private fields, no public mint, `certifies(&C) -> bool`, `verify(Budget)`; `Certainty::{Proved(ProofKind), Probable(ProbableReason)}`; `ProofKind::{Identity, Divisibility, Cofactor, Enclosure, DegreeBound}` |
| verification | `Certified<T>` carrying a `Certainty` — a hybrid of the two, with api-shape's `ProofKind` |

`ProofKind`'s variants are disjoint between the two lists. `Certified<T>` and
`Certificate<C: Claim>` are different generic shapes with different variance and different
mint rules. Z7 is a Wave-1 `S` lane whose deliverable is "the `Certificate` type", and every
Layer-2 and Layer-3 signature depends on which one it builds.

api-shape's version is better — the claim tether (`certifies`) and the no-public-mint rule
are real requirements from a real consumer evaluation, and ADR-010's flat enum cannot express
either. Adopt it, amend ADR-010, and unify `ProofKind` by union.

**Also unsettled and load-bearing:** whether `Evidence`/`ProbableReason` (which carries
`primes_used`, `rounds`) is inside canonical bytes. `ADR-012:129-131` asserts value-equality
across a `Tuning` matrix, and the modular batch width `N` is a tuning knob
(`ADR-012:122-123`) that changes `primes_used`. If evidence is in the bytes, that CI gate
fails on the first run. §4.5's canonical-serialization list (`architecture.md:479-491`)
covers polynomials, Gröbner bases and algebraic numbers, and is silent on certificates.
Answer it in ADR-012 §9: **certificates and telemetry are excluded from canonical bytes;
only the mathematical value is serialized.**

---

## 17. MINOR — `MonomialId(u32)`, and an arena that never forgets

**Target.** `ADR-008:56-62`.

Two unhandled paths in the one-way-door type:

- **Id exhaustion.** `MonomialId(u32)` caps the arena at 2³² distinct monomials. That is
  probably enough — but `mul_monomial` returns `Result` only for the guard-bit trip
  (`ADR-008:111-112`), so exhaustion has no error path and will be an index panic, violating
  ADR-011's absolute no-panic rule. Add `Unsupported::MonomialArenaFull { capacity }`.
- **No eviction.** The arena is owned by the `Ring` and grows monotonically; monomials
  interned for S-pairs eliminated by Gebauer–Möller are never reclaimed. At 3 words per
  distinct monomial (`ADR-008:57-60`) plus hash-table overhead, a long ℚ run over 2000 primes
  with a shared ring accumulates the union of all primes' monomials. That is probably fine —
  the monomial *set* is largely prime-independent — but the plan should say so as a claim
  with a memory model, not by omission. A single `Ring::arena_stats()` and a corpus assertion
  on the largest instance settles it.

Neither is a redesign. Both are cheap now and become a public-API change later.

---

## 18. MINOR — the `rug` dev-oracle has nowhere to live that satisfies the layering gates

**Target.** `architecture.md:118` (gate L6), `ADR-002:79-81`, `verification.md:559`,
`roadmap.md:818-821` (Day 4).

Gate L6: "`publish = false` crates may depend on `publish = true` crates; **never the
reverse, including dev-dependencies**." `ADR-002:79` makes `rug` "a dev-dependency oracle
only, in `publish = false` crates". The Day-4 plan says "`rug` as a dev-dependency oracle.
Property tests: ring/field axioms, inverse-op round-trips…" for `resolvent-int`.

If those tests live in `resolvent-int/tests/`, then `resolvent-int` — a published crate —
has an LGPL-3.0+ dev-dependency. `cargo deny` is scoped to the published graph "minus
dev-only features" (`architecture.md:122`) so it will not catch it, but `cargo publish`
records dev-dependencies in the manifest, downstream `cargo test` on the published crate
pulls `gmp-mpfr-sys` and builds GMP, and L6 forbids it in words. If instead they live in
`resolvent-oracles`, they cannot reach `resolvent-int`'s private modules, and Gate 0's "unit
tests" line does not cover the bignum wall.

**Fix.** State it explicitly in ADR-002 and ADR-016: **published crates have zero
dev-dependencies.** The ℤ/ℚ differential oracle lives in `resolvent-oracles` (or a
`resolvent-int-conformance` `publish = false` crate) and tests only the *public* surface of
`resolvent-int` — which is fine, because the newtype wall means the public surface is the
whole point. Add a CI assertion that every `publish = true` crate's `[dev-dependencies]`
table is empty; it is one line of `cargo metadata` and it makes L6 real.

---

## 19. MINOR — the compile-time gate is relative, so it will be switched off in week two

**Target.** `architecture.md:255-256`, `ADR-006:159-161`.

> CI tracks `cargo build --timings` on the workspace and fails on a **>20% regression in
> total front-end time**.

In Wave 0 the workspace has no algebra. Adding `resolvent-int` is a >20% regression. Adding
`resolvent-modular` is another. Every early lane trips a relative gate measured against a
near-empty baseline, so the gate will be disabled within a fortnight — and a
compile-time budget that has been disabled once never comes back.

**Fix.** Absolute per-crate ceilings, set after M1 and revised at each milestone boundary:
e.g. `resolvent-poly` front-end ≤ 20 s, workspace clean debug build ≤ 90 s on the pinned
machine. Ratchet down, never up, and record the ratchet in the same file as the tuning
thresholds. Also record the *monomorphization count* (`cargo llvm-lines`, top 20) as the
leading indicator, because that is what actually predicts the cliff and it moves before the
wall-clock does.

---

## 20. MINOR — the `BulkOps` trait puts a generic back into the kernel it was written to exclude

**Target.** `ADR-006:88`, against `ADR-006:66-68` and `architecture.md:182-195`.

```rust
pub trait BulkOps: Ring { fn axpy(dst: &mut [Self], a: &Self, src: &[Self]); }
```

Tier M exists so that "GF(p) bulk vector ops (axpy, scale, normalize, dot)" are written
"concrete over `u32`/`u64`, `FpParams` by value" (`architecture.md:187`). `BulkOps` is that
kernel re-exposed as a trait method — which means either it is implemented once per `C`
(duplicating the kernel across the instantiation set, which is what Tier M exists to
prevent), or it is a thin forwarder to the concrete kernel, in which case the trait buys
nothing except a bound that generic call sites must carry.

It is not fatal — monomorphization means no dynamic dispatch — but it blurs the one boundary
rule the whole generics design rests on, and an agent will read `BulkOps` as licence to add
`fn row_reduce(...)` next to it.

**Fix.** Delete `BulkOps` from the trait vocabulary. The bulk kernels are free functions in
`resolvent-modular` over concrete types, selected by one `match` on the `RingTag` at the
top of each phase — which is precisely what `ADR-006:66-68` already specifies. If a generic
caller genuinely needs a bulk operation over an arbitrary `C`, it gets the naive loop, and
the doc comment says so, exactly as `architecture.md:179` already promises for Tier G.

---

## 21. Where the plan is right and an obvious objection does not apply

These are the attacks I expected to make and could not. Recording them so nobody spends a
week re-deriving them.

1. **"Packed monomials are most of your Gröbner performance."** The plan rejects this with
   measurements (`ADR-008:15-40`): packing ~15%, divisor index 10–20×, S-pair criteria four
   orders of magnitude, linear algebra 73–91%. The observation that **monomial comparison
   largely disappears inside F4** — the Macaulay columns are sorted once, after which
   comparisons are small-integer — is correct and is the kind of thing that is normally
   learned the hard way. The lane-brief consequence ("say *build the divisor index*, not
   *make compare fast*") is exactly right.

2. **"Order as runtime data will put a branch in the sort inner loop."** It will not.
   ADR-009's normalization of the order into a big-endian packed key is correct: all three
   practical orders are non-negative matrix orders, the grevlex complement proof at
   `ADR-009:262-267` is valid, and comparison becomes an order-free unsigned word compare.
   The `K(ab) = K(a) + K(b) − C` identity holds, and — I checked the SWAR arithmetic — since
   `2c ≤ 2^w − 2` for any `c` within the guarded payload, the intermediate field-wise sum
   cannot carry across a field boundary, so testing guard bits **after** the constant subtract
   is a sound overflow *and* underflow detector. This is a genuinely good design and the
   type-parameter alternative would have been worse. (The two real defects are §9 and §10,
   both local.)

3. **"The cofactor cost is assumed free."** It is not. It is flagged in three separate
   documents (`ADR-010:129-131`, `verification.md:115-120`, `roadmap.md:869`), gated behind a
   named prototype with a numeric abort threshold (~20× memory), and given a stated fallback.
   That is better discipline than most shipped CAS projects manage. My attack (§3) is on the
   *shape* of the measurement and on the shared-reducer claim, not on the omission.

4. **"Determinism and parallelism are incompatible."** Mostly they are not, and the plan's
   mechanisms are the right ones: counter-based RNG with index substreams, index-addressed
   primes with recorded rejections, ordered combination rather than completion order, a
   thread-count CI matrix. The modular loop — the natural first parallel win — is genuinely
   deterministic under this scheme. The two places it breaks (§5 interning, §6 shared
   refinement) are both *shared mutable caches*, not parallelism as such, and both are fixable
   without changing the posture.

5. **`num-bigint` was rejected for the right reason.** Not license — capability, verified by
   reading the multiplication ladder (`ADR-002:28-30`). Correct.

6. **ℤ-primitive over ℚ-primitive.** Correct, well argued, and correctly identified as the
   thing an agent reading `lazy-exact/src/roots.rs` will otherwise copy. The dyadic-interval
   consequence (`ADR-004:233-238`) is the single highest-leverage detail in the isolation lane
   and it is called out as such.

7. **Fail-at-construction, no panics, structured `Unsupported`, no tolerance parameter
   anywhere.** All correct, all consumer-evidenced, all mechanically gated. The `Verdict<T>`
   vs bare-`Sign` rule (`architecture.md:369-373`) is the right resolution of a question that
   normally gets answered badly.

8. **Sharpness gates** (`verification.md:472-489`). The observation that every soundness
   certificate in the plan is satisfied by a maximally conservative implementation, and that
   every three-valued output therefore needs a tracked rate with a committed ceiling, is the
   single most valuable paragraph in the verification spine. I have nothing to add to it.

9. **The oracle-independence table** (`verification.md:497-509`), and specifically the
   admission that the monomial layer, `UPoly` arithmetic and `Integer` are *common mode* for
   nearly every internal cross-check. Most plans never notice; this one draws the correct
   consequence (external differential testing matters most exactly where internal oracles are
   correlated).

10. **The gcd certificate's degree half**, **Sturm-as-oracle-not-product**, **build the
    oracle side first every time**, **the Swinnerton–Dyer ladder as the factorization
    adversary**, and **Hexapod as a correctness instance disguised as a performance
    instance** — all correct, all non-obvious, all worth more than they cost.

11. **The two-trunk sequencing** (`ADR-007`, `roadmap.md:39-48`): the observation that the
    first consumer touches none of the multivariate machinery, so `UPoly<C>` standalone lets
    the geometry track skip the Gröbner one-way doors entirely, is the largest schedule win in
    the plan and it is free. It is correct: I checked, and `lazy-exact`'s entire univariate
    toolkit is dense `Vec<Rational>` with hand-rolled degree-2 resultants.

12. **Losing certified-vs-uncertified Gröbner benchmarks by construction**, stated up front
    rather than discovered in a comparison table (`ADR-010:156-160`). Honest, and it protects
    the harness from an unfair comparison in both directions.

---

## 22. Triage summary

| # | Severity | Target | One-line action |
|---|---|---|---|
| 1 | fatal | `ADR-006:71-88`, `architecture.md:222-239` | Add a ring context to `zero`/`one`; fix `Liftable`'s supertrait; **typecheck the trait block before the freeze** |
| 2 | fatal | `api-shape.md` vs `architecture.md`+ADRs | Reconcile 11 signature-level contradictions; make ratification machine-readable |
| 3 | fatal | `ADR-010:120-131` | The two Gröbner modes do not share a reducer — fix the claim, give G3 its own oracle, re-scope the cofactor measurement to reconstruction |
| 4 | serious | `ADR-006:86`, M8 | `Reducible::Image: Field` is false over number fields; relax to `CommutativeRing` and scope multi-modular-over-split-primes as a lane |
| 5 | serious | `ADR-008:75`, `ADR-012:74-88` | Interning is a shared mutable accumulator; serialize it or make ids content-derived; ban id-order tie-breaks |
| 6 | serious | `ADR-013` + `ADR-011:89-106` | Shared refinement makes declines schedule-dependent; derive budgets from bounds, exclude telemetry from canonical bytes |
| 7 | serious | `architecture.md:612-632` | `Ord` is unbounded and undeclinable on the default path; measure the step distribution and pick a resolution before M3 |
| 8 | serious | `ADR-002:44-54` | Reconstruction *is* the large-integer regime; extend Z2's ladder to 64k+ bits before `resolvent-int` |
| 9 | serious | `ADR-009:333-334` | FGLM needs two orders live; adopt dual keys in one arena and re-size G7 |
| 10 | serious | `ADR-009:75-77` | Divisibility is order-free on `raw` and is an inner loop; correct the text; split `W_KEY`/`W_RAW` |
| 11 | serious | `roadmap.md:453-479`, `ADR-010:129`, `ADR-013:190` | Three blocking experiments need the artifact they gate; specify synthetic harnesses |
| 12 | serious | `verification.md:845-860` | Tier the regression corpus now, or the determinism gate is the first casualty |
| 13 | serious | `api-shape.md:240`, `verification.md:791` | Name one audited `unsafe` SIMD leaf, or lower the Competitive gate |
| 14 | serious | `ADR-006:76`, `ADR-010:141` | Batched lanes need a fault mask and a split driver, not just componentwise equality |
| 15 | minor | `ADR-017` §1/§4/§5 | L4 has grown a rewriter, two backends and an integrator; hold M7's exit gate |
| 16 | minor | `ADR-010:54-68` vs `api-shape.md:374` | Three `Certificate` shapes; adopt api-shape's, unify `ProofKind`, exclude evidence from canonical bytes |
| 17 | minor | `ADR-008:56-62` | `MonomialId` exhaustion has no error path; arena has no memory model |
| 18 | minor | `architecture.md:118`, `ADR-002:79` | Published crates must have zero dev-dependencies; assert it in CI |
| 19 | minor | `architecture.md:255` | Relative compile-time gate is unusable early; use absolute ceilings + `llvm-lines` |
| 20 | minor | `ADR-006:88` | Delete `BulkOps`; bulk kernels are free functions over concrete types |

**If only three things are done before the freeze:** typecheck the Layer-0 trait block (§1),
reconcile the two API documents (§2), and correct the cofactor/shared-reducer claim together
with what the cofactor prototype actually measures (§3). The first two are cheap and are
currently blocking-in-fact; the third decides whether the Gröbner trunk has a verdict
function at all.
