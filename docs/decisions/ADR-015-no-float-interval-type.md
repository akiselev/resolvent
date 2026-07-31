# ADR-015 — resolvent exposes no float interval type

**Status:** Ratified 2026-07-31
**Reversibility:** cheap (adding a type is additive)
**Amended:** 2026-07-31 — the enclosure contract ships as a **committed conformance-vector
file**, not as prose (critique-plan C14).
**Gates lanes:** A1, A5, and every consumer adapter.
**Resolves:** contradiction-register item 3 — `Interval<f64>` is not core and appears in no
published signature. `API.md` §3.5 concurs.
**Evidence:** `docs/research/consumer-requirements.md` §3, §7 D1, §8 Q8;
`docs/research/algorithms-and-representation.md` §8.2 F10;
`docs/research/critique-plan.md` C14.

---

## Context

Exact-geometry consumers all have a filtered-arithmetic layer: compute cheaply in floating
point with rigorous outward-rounded bounds, and fall back to exact arithmetic only when the
filter cannot decide. The first consumer has a carefully built one —
`/home/dev/projects/arrangements/crates/lazy-exact/src/interval.rs`, 431 LOC, with **no
global rounding-mode state** (it widens outward via `next_up`/`next_down` rather than
setting the FPU mode, which is the design that survives compiler optimization and threading).

resolvent will inevitably want to return *some* float information — an `AlgebraicReal`'s
approximate location is useful for printing, for plotting, for a consumer's own filter, and
for the F10 consistency property test.

The trap: if resolvent ships its own `Interval`, then at every adapter boundary there are
**two interval implementations with two enclosure semantics**, and they can silently
disagree. Outward-vs-nearest rounding, half-open vs closed, whether the enclosure is
guaranteed to contain the value or only to be "close" — these differ subtly between
implementations, and a disagreement at a filter boundary produces a wrong *verdict*, not a
wrong *number*, which is much harder to detect.

resolvent is also **exact-only by construction**. Its internal bounds are rationals; it has
no filtered-arithmetic layer of its own and does not want one, because filtering *arithmetic*
is an orthogonal axis to filtering *algebra*.

---

## Decision

**resolvent exposes no float interval type. Bounds are rationals; the float information is a
plain pair.**

```rust
impl AlgebraicReal {
    /// Exact current enclosure. Monotone: narrows on refinement, never widens.
    pub fn bounds(&self) -> (Rational, Rational);

    /// Outward-correct f64 enclosure: lo is rounded DOWN, hi is rounded UP,
    /// and the true value is guaranteed to lie in [lo, hi]. Never NaN.
    /// Infinities are permitted and mean "no finite bound on that side".
    pub fn enclosure_f64(&self) -> (f64, f64);
}
```

Four supporting rules:

1. **No `Interval` type, no `interval` module, no `IntervalArithmetic` trait** in any
   published crate. Grep gate.
2. **`enclosure_f64` is outward-correct and says so in its name and its docs.** The
   contract is exactly one sentence: the true value lies in the returned closed interval.
   No claim about tightness beyond "as tight as the current rational bounds allow".
3. **The enclosure is never a decision input.** No resolvent API consumes an `(f64, f64)`
   to decide anything. The floats are an *output* for consumers and diagnostics only. This
   is what keeps ADR-012's "no floating point in a decision path" true.
4. **F10 is a property test, not a docs claim.** If `a.cmp(&b) == Less`, then disjoint
   enclosures must agree with that ordering — i.e. `a.enclosure_f64().1 < b.enclosure_f64().0`
   whenever the enclosures are disjoint. This catches outward-vs-nearest rounding-direction
   bugs, which are otherwise invisible.
5. **The enclosure contract ships as a committed conformance-vector file**, not as a
   sentence. *Added 2026-07-31.* ADR-018 item 4 correctly identifies that two enclosure
   semantics disagreeing at a filter boundary produce a wrong *verdict* rather than a wrong
   number — and then files it as something a future measurement would settle. It is not a
   measurement; it is a specification, and writing it is an afternoon:

   > `resolvent-oracles/vectors/enclosure_f64.toml` — a few hundred
   > `(exact rational, expected (lo, hi))` triples covering subnormals, values at and
   > adjacent to powers of two, exact halves, values requiring rounding in both directions,
   > the largest finite double, and rationals whose nearest double is `±inf`. Committed in a
   > `publish = false` crate so **any** consumer can run its own interval type against it.

   That artifact makes ADR-018 option C's hardest item checkable *before* anyone commits to
   option C, and makes option B's adapter testable. Nothing else on the deferred list has
   that property.

The one place a *rational* interval is a public concept is `sign_over(p, lo, hi)`
(certified Bernstein range enclosure), and there `lo`/`hi` are two `Rational`s, not an
interval type. That keeps the vocabulary at one level: resolvent speaks in rational
endpoints, consumers assemble whatever interval type they already have.

---

## Consequences

- **The adapter owns exactly one interval implementation — its own.** This is the cheapest
  single thing that keeps ADR-018's deferred integration decision cheap, because there is
  no semantics negotiation at the boundary: the consumer builds its `Interval` from two
  `f64`s (or from two `Rational`s if it wants tighter), using its own constructor and its
  own contract.
- **No duplication of a 431-LOC carefully-tuned component**, and no risk of resolvent's
  copy drifting from the consumer's.
- **Consumers who want tighter enclosures than `f64` can build them** from `bounds()`, which
  returns exact rationals. Nothing is hidden.
- **Diagnostics and printing are slightly more awkward** — a `Debug` impl must format from
  the float pair rather than from a nice `Interval` type. Trivial.
- **A consumer without a filtered layer must write two lines** to make an interval. Also
  trivial, and it is the right two lines to write, because the consumer knows its own
  rounding contract.
- **resolvent gives up the ability to accelerate its own predicates with a float filter.**
  This is the real cost and it is accepted with a caveat: the *internal* dyadic-approximation
  fast path in sign-variation counting (R3 §7.3 — "taking appropriate dyadic approximations
  of these coefficients is sufficient to decide the sign unless some unexpected cancellations
  occur") is permitted and is not an interval type. It is a private filter whose declining
  case falls through to exact arithmetic, and no verdict depends on which branch ran.

---

## Alternatives considered and why rejected

**Ship a resolvent `Interval` with the same outward-widening design as the consumer's.**
Rejected. Two implementations of the same contract in one process is a maintenance and
correctness liability, and "the same design" is exactly the claim that is expensive to
verify and easy to break. It also makes ADR-018 option C (eventual merge) harder rather
than easier, because two `Interval` types must then be reconciled.

**Depend on an existing interval-arithmetic crate.** Rejected on ADR-001 grounds first
(most are Apache-only or worse) and on architecture grounds second: it would put a
third-party type in resolvent's public signature, which ADR-002 and gate L3 forbid.

**Return `Option<(f64, f64)>` to signal "no useful enclosure".** Rejected. There is always a
useful enclosure — `(-inf, +inf)` is one — and an `Option` invites `unwrap` at exactly the
boundary where a wrong assumption is silent.

**Expose only rational bounds and no float at all.** Considered seriously; it is the purest
version. Rejected because every consumer would then write the same rational→f64
outward-rounding conversion, which is genuinely subtle (correctly rounding a big rational to
a float in a specified direction is not one line), and getting it wrong reintroduces exactly
the enclosure-direction bug this ADR is trying to prevent. resolvent writes it once,
correctly, and property-tests it.

**Expose a `Verdict`-returning float comparison** (`try_cmp_f64`) as a consumer-facing
filter. Rejected for now: it is a filter API, and filtering arithmetic is the consumer's
axis, not resolvent's. Adding it later is additive if a consumer asks.

---

## What would reverse this

- **A consumer with no filtered-arithmetic layer asking for one.** Response: a separate,
  optional `resolvent-interval` crate that is not a dependency of anything in the core
  graph. That is additive and does not put an interval type into resolvent's algebra
  signatures.
- **resolvent needing an internal interval type for its own filtering** (e.g. an ANewDsc
  implementation with interval Newton steps). Response: a *private* type, not exported, not
  in any public signature. The decision here is about the public API, and it should be
  restated that way if that happens: "no float interval in the public API".
