# ADR-013 — `AlgebraicReal`: `Arc<Inner>`, `&self` monotone refinement, `Send + Sync`, total `Ord`

**Status:** Ratified 2026-07-31
**Reversibility:** one-way — visible in every signature that mentions an algebraic number
**Amended:** 2026-07-31 — three corrections: **no `Equal` verdict is ever produced by
exhausting the separation bound**; `Ord` gains a diagnostic ceiling plus a documented
budgeted sibling, with the step distribution to be measured rather than asserted; the
deciding experiment is respecified so it does not require the artifact it gates
(critique-engineering §6, §7, §11; critique-plan C11).
**Gates lanes:** A1, A2, A3, A4, U8.
**Evidence:** `docs/research/algorithms-and-representation.md` §8.2 (F1–F6, F9, F10);
`docs/research/consumer-requirements.md` §3.2, §7 D4, §6 R4/R5/R7;
`docs/research/critique-engineering.md` §6, §7; `docs/research/critique-plan.md` C11.

---

## Context

`AlgebraicReal` is resolvent's headline type — the bridge to computational geometry. Its
API shape is decided by one tension:

**Comparison must refine, refinement mutates, and `Ord::cmp` takes `&self`.**

Refinement state is not incidental. It is what makes sorting `n` algebraic numbers
affordable: without a cache, every `cmp` redoes the bisection work from scratch, and an
`O(n log n)` sort becomes `O(n log n × precision)` bignum operations.

R3 §8.2 F6 enumerates the four exits and their costs:

| Exit | Cost |
|---|---|
| Interior mutability (`Cell`/`RefCell`) | `!Sync`. Self-comparison re-borrows and **panics** unless guarded. |
| Interior mutability with a lock | `Sync`, but self-comparison **deadlocks** instead of panicking; every comparison pays an atomic. |
| Pure recompute, no cache | `Ord` works, `Send + Sync`, no aliasing hazard — but quadratic-ish blowup in practice. |
| Explicit context `ctx.cmp(&a, &b)` | Correct, `Send`, aliasing-safe, cache-preserving. **No `Ord`.** Consumers thread a context. |

Both realistic prior arts picked differently, and both documented why:

- **CGAL chose pure recompute**, stating: "there is no way to directly ask for the
  refinement of the current isolating interval since this would impose a state to every
  object of an Algebraic kernel."
- **`arrangements` chose shared interior mutability**, and pays for it in the open:
  `type SharedRoot = Rc<RefCell<RealRoot>>` appears **four independent times** —
  `conics.rs:32-46`, `sine_waves.rs:31-42`, `sine_radical.rs:71-84`,
  `spherical_circle.rs:70-88` — each with its own copy of the self-deadlock guard:

  ```rust
  fn cmp_roots(p: &SharedRoot, q: &SharedRoot) -> Ordering {
      if Rc::ptr_eq(p, q) { return Ordering::Equal; }   // load-bearing, easy to forget
      p.borrow_mut().cmp_root(&mut q.borrow_mut())
  }
  ```

  The cause is exactly `RealRoot::refine` taking `&mut self` (`roots.rs:450`) while every
  `Geometry` predicate takes `&self`. The whole type is also `!Send`.

There is a fifth option the enumeration does not name, and it is the one the same codebase
already built for a *different* problem: **`Arc` + a monotone atomic cache where even a torn
read is a valid answer.** `crates/lazy-exact/src/real.rs:1-15` documents that protocol for
lazy-exact numbers — eager interval, lazy exact, per-node lock, iterative evaluation, at
most one node lock held at a time so no waits-for cycle can form.

Refinement of an isolating interval has exactly the property that makes this work: it is
**monotone**. The interval only shrinks, and it always contains the root. Any observation —
including one interleaved with a concurrent refinement — is a *valid enclosure*. There is no
unsound intermediate state.

---

## Decision

```rust
#[derive(Clone)]
pub struct AlgebraicReal(Arc<Inner>);

struct Inner {
    poly:  UPoly<Integer>,     // squarefree, primitive, lc > 0. Immutable.
    state: Mutex<Bounds>,      // (lo, hi) rationals; MONOTONE: only ever shrinks
    hint:  AtomicU64,          // f64-pair enclosure cache; torn read is still valid
}
```

1. **`Arc<Inner>`, `&self` methods, `Send + Sync`.** Cloning shares refinement progress —
   which is what the consumer wants and currently hand-builds four times.
2. **Refinement is monotone.** `refine_to(&self, width)` narrows and never widens; every
   observation is a valid enclosure; `refine_to` is idempotent and never changes a verdict
   (property-tested: "idempotence under refinement").

   The stronger form, because sharing refinement across clones is this ADR's whole value
   proposition and it has a cost the original text did not draw:

   > **INV-AR1. The refinement cache may change how much work a call does. It may never
   > change what the call returns, including whether it declines.**

   Without INV-AR1, budget exhaustion is history-dependent — a call that declines when run
   first succeeds after a warm-up — and under `parallel` it is *schedule*-dependent, so
   `Ok` vs `Err(BudgetExhausted)` could differ between `RAYON_NUM_THREADS=1` and `=8`, which
   ADR-012 asserts cannot happen. It would also make property-test outcomes depend on
   execution order, which shrinking reorders. ADR-011 §4 discharges it: `AlgebraicReal`
   budgets are **always** derived from the separation bound or from a worst-case step count
   computed from operand degree and coefficient size — a pure function of the inputs — never
   from steps actually taken. `Telemetry { bisections, precision_bits }` and
   `TraceEvent::BudgetTick` remain honest *measurements of work* and are therefore excluded
   from canonical bytes and from replay comparison (ADR-012 §9).
3. **The defining polynomial is immutable** and its invariants (squarefree, primitive,
   `lc > 0`, exactly one real root in the isolating interval, nonzero at both endpoints) are
   established at construction and never revisited (ADR-011: fail at construction).
4. **Self-comparison is safe without a caller-side guard.** `cmp` checks `Arc::ptr_eq`
   first — internally, once, in resolvent — and never takes two locks. Where two distinct
   values are compared, locks are acquired in a fixed order derived from the `Arc` pointer
   value — **no.** Pointer-ordered locking is address-dependent and therefore forbidden by
   ADR-012. Instead, **no two locks are ever held simultaneously**: `cmp` reads a snapshot of
   each operand's bounds, releases, refines each independently, and re-reads. Refinement is
   monotone, so a snapshot is never stale in a way that matters — it is merely less precise
   than the current state, and the loop simply runs one more round.
5. **`Ord`, `PartialOrd`, `Eq`, `PartialEq` are implemented, and they are total.**
   Equality is decided **algebraically** — `g = gcd(a.poly, b.poly)`; if `deg g = 0` they
   cannot be equal and refinement is guaranteed to separate them; if `deg g > 0`, a **sign
   change of `g` across the overlap** certifies equality.

   **5a. No `Equal` verdict is ever produced by exhausting the separation bound.** *Added
   2026-07-31, and this is the difference between a loud failure and a silent wrong answer.*
   `Equal` comes **only** from the gcd-plus-sign-change certificate. The bound's sole role is
   to bound the number of refinement rounds before the *inequality* branch is guaranteed to
   have separated. Reaching the bound with neither a certificate nor a separation is an
   **internal-invariant failure**, not an answer.

   The reason this must be stated: `plans/verification.md` §2.3's phrase "every verdict
   reached under the bound" and `plans/architecture.md` §5.3's "there is no failure to
   report" both read as though verdicts are *produced* by exhausting the bound. If they ever
   were, then an over-large bound — an off-by-one in the Mignotte–Davenport exponent, a
   `bit_length`-vs-`ceil(log2)` confusion, a dropped leading-coefficient factor — returns
   **`Equal` for distinct numbers**. That is the F2/F3 failure by a new route, and the
   transitivity property test **will not catch it**, because a systematically over-large
   bound is over-large for all pairs consistently and consistent equality-collapse is
   transitive. Under 5a the same bug is a loud internal error caught by the step-budget
   diagnostics counter.

   Grading consequence: the separation-bound row is **INV+PROP, not CERT** — "for every pair
   in the corpus, `|α − β| ≥ bound`" is a finite check of a universally quantified claim, and
   the corpus is exactly where the near-degenerate inputs that expose an off-by-one are least
   likely to arise by chance. It is additionally graded *by derivation*: the implementation
   carries its citation and a symbolic unit test against brute-force certified separations at
   degree ≤ 6, where the true separation is computable.

   **5b. `Ord` is total, and "computable" is not "attainable".** The mathematics of the
   bound is right and the engineering conclusion does not follow from it alone. For the
   resultants M4 produces — this plan's own estimate is degree ~200 with ~500-bit
   coefficients (ADR-004 §Context) — the Davenport–Mahler bound implies tens of thousands of
   bits of refinement in the worst case, and `Ord::cmp` has no `Result`, no budget, and no
   way out. A pathological pair would hang inside an infallible function, which is the
   failure this plan calls the deadliest. So:

   - **`try_cmp(&self, &Self, Budget) -> Result<Ordering, Decline>` ships alongside `Ord`,
     is documented as the latency-path entry point, and is benchmarked.** Consumers on a
     latency path are directed to it in the type's own doc comment, not in a guide.
   - **`Ord::cmp` carries a diagnostic ceiling set far below the theoretical bound.**
     Reaching it increments the diagnostics counter and is a CI failure in every grading
     context (ADR-011 §4); it does not change the returned verdict, because by 5a the
     verdict at that point is an internal-invariant failure either way.
   - **The step distribution is measured, not asserted.** Lane Y1 measures `cmp` step counts
     over the M4 corpus and publishes the distribution; the ceiling is then set from the
     measured 99.9th percentile with a stated outward margin, and ratcheted per ADR-024.
     **This must be settled by M3, not carried as an open question** — it is in every
     signature and it is the consumer's most-called function.

   The `Ord` impl is honest under 5a + 5b: total, terminating, with the one pathological
   regime detected and reported rather than hung on.
6. **A failed equality certificate is never evidence of inequality.** If an overlap endpoint
   happens to be a root of `g`, the sign-change test sees a zero and **cannot conclude**;
   the correct response is refine-and-retry, not "return Less". Returning an ordering there
   is intransitive in exactly the same way as equality-by-tolerance. This is subtle enough
   that it is an explicit property test, not a review item. (The prior art gets this right at
   `roots.rs:578-592`.)
7. **`Hash` is not implemented.** See ADR-014.
8. **Multiplicity is not a field.** `isolate_roots` returns
   `Vec<IsolatedRoot { value, multiplicity }>` — a named struct, so multiplicity stays off
   the number *and* the caller does not have to carry two values. See ADR-014 §3.
9. **`no_std` is feature-gated off by this decision.** `Arc` requires `alloc` and the atomic
   hint requires `core::sync::atomic` (or `portable-atomic`). No prospective consumer
   (#12, #24, #27, #28, #34) is embedded, so the cost is accepted.

---

## Consequences

- **The four-times-duplicated `Rc<RefCell<_>>` boilerplate in the shipping consumer
  disappears**, including the four copies of the self-deadlock guard. That is the single
  most concrete API win available and it is measurable: the adapter either needs a wrapper
  or it does not (ADR-018 §6.3.5).
- **resolvent's headline type is `Send + Sync`**, so a consumer can parallelize predicate
  evaluation across an arrangement without a per-point lock of its own.
- **Comparison pays a mutex acquire per operand per round.** Uncontended `Mutex` is a few
  nanoseconds; the surrounding work is bignum arithmetic on rationals. The atomic `hint`
  short-circuits the common case (already-disjoint enclosures) without touching the lock at
  all.
- **The snapshot-and-retry protocol is subtly different from "lock both operands"** and must
  be written down where the code is, not just here — a future edit that "simplifies" it into
  two simultaneous locks reintroduces a deadlock class that ADR-012 also forbids for a
  second reason (address-dependent lock ordering is nondeterministic).
- **`Ord` means `AlgebraicReal` works in `BTreeMap`, `sort`, `binary_search`, `max`** —
  everything a consumer expects — which is a large ergonomic win over the explicit-context
  alternative.
- **`Ord` is expensive and does not look it.** A sort of `n` algebraic numbers is
  `O(n log n)` comparisons each of which may refine. Mitigated by shared refinement progress
  (the whole point of `Arc<Inner>`) and documented loudly.
- **Open, and it must be settled before ANewDsc lands:** ANewDsc's speedup comes from
  Newton steps that *jump*, so the isolating interval does not shrink monotonically by
  halving. The monotonicity invariant here is "the interval only shrinks and always contains
  the root", which a Newton step still satisfies — but the F4 endpoint invariant
  (`poly(lo) ≠ 0 ≠ poly(hi)`, and collapse-to-a-point on an exact hit) was derived for
  bisection and must be re-derived for the Newton path **before** implementing.

---

## Alternatives considered and why rejected

**Explicit context — `ctx.cmp(&a, &b)`** (R3's recommendation). The most defensible
alternative: correct, `Send`, aliasing-safe, cache-preserving, and it makes the cost
visible. Rejected because it costs `Ord`, and losing `Ord` costs `sort`, `BTreeMap`,
`binary_search`, `Iterator::max`, and every generic algorithm that orders things — in a type
whose entire purpose is to be an ordered number. The consumer would thread a context through
every `Geometry` predicate, which is a *worse* version of the `Rc<RefCell<_>>` tax this ADR
exists to remove. The `Arc` + monotone-cache protocol gets the same aliasing safety without
the threading, because monotonicity makes stale reads harmless.

**Pure recompute (CGAL's choice).** Rejected on cost. CGAL's reason — "this would impose a
state to every object of an Algebraic kernel" — is a real concern, and the `Arc<Inner>`
answer to it is that the state is *monotone and semantically invisible*: no observable
result depends on how much refinement has happened. That is exactly the property CGAL's
design could not assume for a mutable-interval API.

**`Rc<RefCell<Inner>>` (the consumer's current shape).** Rejected. `!Send`, and it requires
every caller to carry the pointer-equality guard. The evidence that this is a real tax and
not a theoretical one is that it appears four times in one codebase.

**`Arc<UPoly>` plus an *inline* `RefCell<Isolation>` cache — `Send + !Sync`, cheap `Clone`
for per-thread copies, self-comparison guarded by `std::ptr::eq` inside resolvent.** This is
the closest competitor and its central argument is genuinely good: with the cache *inline*
rather than shared, address equality means *value identity*, so the guard is exactly correct
rather than a heuristic, and there is no atomic on any path. **Rejected on two counts.**
First, it gives up `Sync` on the headline type, which means a consumer cannot evaluate
predicates over an arrangement in parallel without wrapping — reintroducing at the consumer
level precisely the boilerplate R2 D4's evidence was about. Second, "cheap `Clone` for
per-thread copies" is only cheap in the `Arc<UPoly>`; the *refinement progress* is not
shared across clones, so a parallel sweep re-does bisection work per thread, which is the
cost the cache exists to avoid. The `Arc<Inner>` design keeps the same self-comparison
safety without either cost: `Arc::ptr_eq` is a *fast path*, not the equality test, and
correctness does not depend on it — two distinct `Arc`s holding equal values still compare
`Equal` via the gcd certificate.

**Fully lock-free `Arc<Inner>` with an atomically swapped bounds pair.** Attractive: readers
never block and a stale read is merely a wider valid enclosure, so there is no
self-comparison hazard and no atomic on the compare path at all. Not chosen *as the
mechanism* only because rational bounds do not fit in a word and swapping them atomically
needs either a third-party `ArcSwap` (a dependency in a hot type) or hand-rolled
`AtomicPtr` reclamation. **The decision here is the contract — monotone, `&self`,
`Send + Sync`, total `Ord` — not the mechanism**, and moving from an uncontended `Mutex` to
a lock-free swap later changes no signature. If the deciding experiment below shows lock
contention matters, take this option.

**The deciding experiment — E-MUT — and it must run before `AlgebraicReal` ships.**
Prototype all four behind one trait — pure recompute; `Arc<Inner>` + uncontended `Mutex`
(this ADR); lock-free `Arc<Inner>`; inline `RefCell` + `ptr_eq`. Grade on: `cmp(&a, &a)`;
sort stability under shuffling; the transitivity suite; whether `Send`/`Sync` compile; and
the number that actually decides it — **sorting `n = 10³` algebraic numbers of degree 8 with
200-bit coefficients**, once single-threaded and once across 8 threads. Record the
measurement in this ADR. The *contract* above does not change with the result; only the
mechanism does, and only if the lock-free variant wins.

**E-MUT does not need lane A1, and the original wording implied it did** — which deadlocked
the schedule, since E-MUT gates A1 and A1 builds `AlgebraicReal`. *Clarified 2026-07-31:*
the four prototypes need exactly three things — `cmp`, `refine`, and polynomial sign
evaluation — which is roughly **300 lines over M2's `UPoly<Integer>`**, with test roots
built as `Π(x − rᵢ)` from known rationals and small radicals so the true ordering is known
by construction. **It does not need the production isolator, the separation bound, or the
gcd equality certificate** (equal-value pairs are constructed, not detected). E-MUT is
therefore an M2-tail experiment, not an M3 one.

**`Arc<RwLock<Inner>>` with both operands locked for comparison.** Rejected: self-comparison
deadlocks, and any fixed lock order must be derived from something, and the only available
something is the address — which ADR-012 forbids because it is nondeterministic.

**Lock-free with a `Mutex`-free monotone `AtomicU64` bounds representation.** Attractive
(this is what `real.rs` does for the f64 interval) but rationals do not fit in a word and
the bounds must be swapped atomically as a pair. Kept for the f64 `hint` only, where it
does fit and where a torn read is genuinely still a valid enclosure.

**Making refinement `&mut self` and telling consumers to wrap it.** Rejected — that is the
status quo the research identified as a hard API requirement to fix, not a nit.

---

## What would reverse this

- **Mutex contention measuring badly** on a parallel predicate workload. Response: sharded
  or seqlock-style bounds, still monotone, still `&self`. The public signature does not
  change, which is the point of deciding the *contract* (monotone, `&self`, `Send + Sync`)
  rather than the mechanism.
- **A consumer requiring `no_std`.** Response: a `portable-atomic` + `alloc` build, or a
  feature-gated `AlgebraicRealLocal` with `Rc`. Both are additive. None of the five
  prospective consumers is embedded, so this is not planned for.
- **The separation-bound argument failing for some input class**, making `Ord` dishonest.
  Response: fix the bound. If it truly cannot be fixed, `Ord` must go and the explicit
  context returns — which is why the step-budget diagnostics counter (ADR-011 §4) exists: it
  surfaces that condition *before* it ships rather than after.
