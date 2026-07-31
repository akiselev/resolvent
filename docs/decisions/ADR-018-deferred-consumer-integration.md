# ADR-018 — Deferred: how resolvent and `arrangements` eventually fit together

**Status:** Ratified 2026-07-31
**Reversibility:** deliberately deferred, and cheap to keep deferred — which is the whole
content of this ADR
**Amended:** 2026-07-31 — three items added to the "what to avoid" list and one collision
that was not on it is named and fixed (critique-plan C14).
**Gates lanes:** none directly; constrains every public signature.
**Evidence:** `docs/research/consumer-requirements.md` (whole document);
`docs/research/algorithms-and-representation.md` §2.5, §8.2 F6;
`docs/research/critique-plan.md` C14; founding constraint #1.

---

## Context

`/home/dev/projects/arrangements` is an exact 2-D arrangement engine (~17k LOC in
`crates/arrangements/src`) built on its own `lazy-exact` kernel (3,602 LOC). Its geometry
families are capped at degree 4, and lifting that ceiling is what resolvent eventually
unlocks.

`lazy-exact/src/roots.rs` (927 LOC) is resolvent's L1+L2+L3 in miniature, by the same
author, built the way resolvent must not build it: dense univariate over ℚ
(`roots.rs:43-45`), monic-normalized Euclid gcd (`roots.rs:169-182`), univariate only —
callers hand-eliminate — and explicitly no separation bounds (`roots.rs:8-11`:
"comparisons terminate because distinct algebraic numbers are eventually separated by
bisection").

So the two codebases will do overlapping work. **Founding constraint #1 says resolvent is
independent and that whether to refactor the two to fit each other is an explicitly deferred
decision.** This ADR ratifies the deferral, names the options, names the evidence that would
settle it, and — most importantly — names what must *not* be done now so that all three
options stay open.

---

## Decision

**Defer. Build toward option B (the consumer writes an adapter) as the default, and pay the
small, enumerated costs that keep options A and C reachable.**

### The three options

| | Option | Shape | Cost if chosen | Cost if it turns out wrong |
|---|---|---|---|---|
| **A** | resolvent adopts a scalar-seam trait | resolvent's polynomial and algebraic-number types become generic over a consumer-supplied scalar | A public generic parameter on the headline types; monomorphization over an *open* instantiation set (ADR-006 §2.4); the modular fast path becomes conditional on a consumer trait impl | **Very high.** A public generic parameter cannot be removed without a major version and a rewrite of every consumer. |
| **B** | the consumer writes an adapter | a small `arrangements`-side crate maps `resolvent::{Integer, Rational, UPoly, AlgebraicReal, SqrtExt}` onto `lazy-exact`'s vocabulary | conversion at the boundary; two enclosure implementations; two `Sign` types, trivially mapped | **Low.** Delete the adapter. |
| **C** | eventual merge | `lazy-exact`'s `roots.rs` and `sqrt_ext.rs` are deleted; `arrangements` depends on `resolvent` directly | one number vocabulary — but resolvent inherits geometry's latency requirements, and `lazy-exact`'s `Real`/`Interval` *arithmetic*-filtering layer stays behind, so the seam moves rather than disappearing | **Medium.** Reverting means reinstating deleted code. |

**B is the default and the only one being built toward.** A and C are kept reachable.

Note that C is less of a merge than it sounds: `lazy-exact` filters *arithmetic* (lazy-exact
`Real<E>`, `Interval`, expansions, error-free transforms) while resolvent does *algebra*.
Those are orthogonal axes. Even under C, `lazy-exact` survives as the filtered-arithmetic
layer and resolvent supplies the algebra — the merge is of `roots.rs` and `sqrt_ext.rs`
only, not of the two kernels.

---

## What would settle it

Each item is a measurement or an event, not an opinion.

1. **The degree/coefficient profile of the real workload** (R2 §8 Q1). Generate degree 3–8
   curve pairs with realistic rational coefficients; record `Res_y` degree and coefficient
   bit-length, and the wall time of `isolate_roots` plus a `sign_of` sweep, against the
   existing `QPoly`. **If resolvent's ℤ + modular pipeline wins by a large factor, C becomes
   attractive. If the crossover sits above the workload, B is correct indefinitely and A
   never pays for itself.** This is the single most informative measurement available and it
   is cheap — it runs against the existing crate.
2. **Whether resolvent's `SqrtExt` matches `sqrt_ext.rs`'s cross-root comparison** on the
   `circle_segments.rs` path. That file is 931 LOC that uses `SqrtExt` exclusively and never
   imports `RealRoot` or `QPoly`. **If resolvent's version is slower, C is off the table** —
   the cheapest and most common case must not regress.
3. **Whether a second consumer with a different number type materializes** (#12 SMT NRA,
   #27 medial axis, #34 FEM). Two consumers with different scalars argue for A or B and
   against C, because C is a merge with *one* consumer.
4. **Whether the f64 enclosure semantics can be made to agree exactly** with `lazy-exact`'s
   outward-widening `Interval` (431 LOC, no global rounding-mode state). Two enclosure
   semantics that silently disagree at a filter boundary produce a wrong *verdict*, not a
   wrong number — the specific failure ADR-015 exists to prevent.
5. **Whether `AlgebraicReal`'s `Arc` + `&self`-refinement model actually removes the
   `Rc<RefCell<_>>` tax.** The consumer has four independent copies of
   `type SharedRoot = Rc<RefCell<RealRoot>>` plus four copies of the `Rc::ptr_eq`
   self-deadlock guard (`conics.rs:32-46`, `sine_waves.rs:31-42`, `sine_radical.rs:71-84`,
   `spherical_circle.rs:70-88`). **If the adapter still needs a wrapper, ADR-013's model is
   wrong and must be revisited before C, not after.**
6. **Whether `sign_radical_tower` at arbitrary depth actually beats materialization**
   (R2 §8 Q2). If it does not, the consumer's ~150-lines-per-family ladders can be deleted in
   favour of the general path, which makes C substantially smaller.

---

## What to avoid doing now

Each item closes an option if violated. These are enforced by grep gates
(`plans/architecture.md` §1.3) where possible.

- **Do not put a scalar-seam trait in resolvent's public API.** No trait mirroring
  `lazy-exact`'s `RingOps` / `ExactRing` / `ExactField`
  (`crates/lazy-exact/src/exact/mod.rs:16-29`) — which is explicitly an *ops surface* and
  "not an algebraic claim", since `Interval` implements it too. resolvent's traits are
  algebraic claims. Adding a seam later is additive; removing one is breaking.
- **Do not add a generic parameter to `AlgebraicReal`.** It is `AlgebraicReal`, not
  `AlgebraicReal<S>`. If A is ever chosen, the generic type is a **new** type and the
  monomorphic one stays. (This is the single most expensive thing to get wrong.)
- **Do not expose a float interval type** (ADR-015). One of the two enclosure semantics must
  be the adapter's, and it must be the consumer's.
- **Do not name `arrangements` or `lazy-exact` anywhere in a published crate** — not in a
  feature flag, not in a doc example, not in a comment (gate L5). A `lazy-exact` feature
  would be option B smuggled into resolvent, which is the one place it must not live.
- **Do not copy `lazy-exact`'s trait or type names** (`RingOps`, `ExactRing`, `ExactField`,
  `Scalar`, `Uncertain`, `USign`, `UOrd`). Identical names with different contracts across an
  adapter boundary is a bug generator. resolvent uses `Ring`/`Field`/`Verdict`/`Sign`
  deliberately differently.
- **Do not subsume `SqrtExt` into `AlgebraicReal`** (ADR-014 context). Keeping degree-2
  radicals first-class is what keeps C from being a regression on 931 LOC.
- **Do not accept a tolerance parameter anywhere**, at any layer, under any name (ADR-011
  §8). The consumer's exact families declare `type Error = Infallible` and its design
  permanently excludes snap rounding; a tolerance argument would make resolvent unusable by
  the consumer it was designed for.
- **Do not let resolvent's error surface force a fallible query path.** ADR-011's "fail at
  construction, not at query" exists partly for this: the adapter converts once, where it
  already has an error path, and its `Geometry` predicates stay infallible.

- **Do not put a public generic parameter on `SqrtExt` for a *consumer's* scalar.** *Added
  2026-07-31.* This list forbade one on `AlgebraicReal` by name and was silent about
  `SqrtExt`, which is the type it also requires stay first-class — the same one-way door for
  the same reason. ADR-014 §4 decides it: the parameter exists and is instantiated **only**
  over resolvent's own base fields, with `SqrtExtQ = SqrtExt<Rational>` as the alias every
  consumer-facing signature uses.
- **Do not let multiplicity move onto the number, and do not make the caller carry a bare
  tuple.** *Added 2026-07-31.* The nearest prior art has `RealRoot::multiplicity(&self)` as a
  method on a stored value (`arrangements/crates/lazy-exact/src/roots.rs:438`). ADR-014 §3's
  `IsolatedRoot { value, multiplicity }` keeps the safety property *and* the call-site shape.
  A bare `(AlgebraicReal, u32)` would have forced the adapter to define its own pair type,
  which falsifies the "a merge is a rename plus `&mut → &self`" claim this deferral rests on.

And two things to do:

- **Commit the `enclosure_f64` conformance vectors now** (ADR-015 §5). Item 4 in *What would
  settle it* is written as a future measurement; it is not a measurement, it is a
  specification, and it is the only item on the list that becomes *checkable by both sides*
  the moment it is written. An afternoon now; an archaeology project later.

- **Write `resolvent-oracles` so that `lazy-exact` can be *added* as a differential oracle**
  — subprocess, or a `publish = false` dev-only path. That is how option C's evidence
  (items 1, 2, 5, 6 above) gets collected without option C's coupling. It is also the only
  place in the whole workspace where the two codebases may meet before the decision is made,
  and it is `publish = false`, so it cannot leak into a consumer's graph (ADR-016 §2).

---

## Consequences

- **The deferral is cheap and stays cheap**, which is the requirement. The enumerated costs
  are: one conversion layer at the adapter boundary, two `Sign` types, two interval
  implementations, and a slightly larger public conversion surface on `Integer`/`Rational`.
  All small, all local to the adapter.
- **resolvent is designed against a real consumer without depending on one.** Every design
  decision in ADRs 004, 007, 011, 013, 014, 015 cites concrete consumer code, which is what
  makes them requirements rather than speculation — while gate L5 keeps the dependency at
  zero.
- **Option A is kept open at almost no cost**, because it is *additive*: a future
  `AlgebraicRealGeneric<S>` alongside the monomorphic type is a new type, not a change to an
  existing one. The only thing that would foreclose it is putting a generic parameter on the
  existing type prematurely, which is explicitly forbidden above.
- **Option C is kept open at moderate cost** — mainly the obligation to keep `SqrtExt`
  first-class and to make the enclosure semantics reconcilable.
- **There is a risk of building for a consumer that never adopts.** Mitigated by the fact
  that the requirements derived from it (arbitrary-degree `Res_y`, real root isolation,
  exact comparison, radical towers, curve analysis) are exactly what #12, #27, and #34 also
  need. No requirement in Tier A is geometry-specific.

---

## Alternatives considered and why rejected

**Decide now, in favour of A.** Rejected. It is the most expensive option to reverse, it
would be decided on zero measurements, and R2's finding that geometry *never* does
algebraic-number arithmetic means the generic machinery would be paid for on a path that
does not use it.

**Decide now, in favour of C.** Rejected. The evidence that would justify it (items 1, 2, 5)
does not exist yet, and C's central claim — that resolvent's ℤ + modular pipeline beats the
existing ℚ implementation *at the consumer's actual sizes* — is precisely the thing R2 §8
Q4 flags as unknown and possibly false at degree ≤ 8.

**Decide now, in favour of B, and stop tracking A and C.** Nearly right, and rejected only
because "stop tracking" would mean dropping the constraints in §What to avoid, several of
which (no generic parameter on `AlgebraicReal`; keep `SqrtExt` first-class; no float
interval) are *also* justified on their own merits. Keeping A and C open costs nothing
beyond what the other ADRs already require.

**Formally rule out C to simplify the design space.** Rejected. C is the option that would
delete the most code and remove a real duplication, and the measurement that would support
it is cheap. Ruling it out on aesthetics would be premature.

---

## What would reverse this

This ADR *is* a deferral, so "reversal" means making the decision. Triggers:

- **Measurement 1 completes** and shows a large win for resolvent's pipeline at the
  consumer's real sizes, **and** measurements 2 and 5 come back clean. → Choose C, and write
  a new ADR that supersedes this one.
- **Measurement 1 completes** and shows the crossover is above the workload. → Choose B
  permanently, write it down, and stop paying for C's constraints (specifically, the
  obligation to keep enclosure semantics reconcilable).
- **A second consumer with an incompatible scalar type adopts resolvent.** → B is confirmed
  and C is foreclosed; A becomes a live question again, to be answered by whether *both*
  consumers would use the generic path.
- **The adapter, once written, turns out to be more than ~500 lines or to need a wrapper
  around `AlgebraicReal`.** → That is evidence a design decision in ADR-013 or ADR-015 is
  wrong, and it should be fixed there before the integration question is answered here.
