# ADR-011 — Error model: fail at construction, not at query

**Status:** Ratified 2026-07-31
**Reversibility:** one-way (it shapes every signature)
**Amended:** 2026-07-31 — the budget rule is unified with `API.md` INV-6 (**every** looping
entry point takes a `Budget`; where a bound exists the *default* budget is derived from it
and a decline is a bug); declines are classified before they are scored; `AlgebraicReal`
budgets are bound-derived so that a shared refinement cache can never change a verdict
(critique-engineering §6, §7; critique-plan C7).
**Gates lanes:** Z0, and every Layer-2/Layer-3 lane.
**Evidence:** `docs/research/algorithms-and-representation.md` §8.2 (F1–F5);
`docs/research/consumer-requirements.md` §4, §6, §8 Q8;
`docs/research/critique-engineering.md` §6, §7; `docs/research/critique-plan.md` C7.

---

## Context

Three facts about this problem domain make a conventional Rust error model wrong.

**1. The characteristic failure is a silent hang, not a wrong answer.** `sign_of(h)` where
`h(α) = 0` never terminates unless zero-ness is settled algebraically first; refinement
stalls forever on a non-squarefree defining polynomial; comparison of two equal algebraic
numbers loops forever if equality is not decided by gcd. A hang is worse than a wrong
answer in a library, because it is undebuggable in production and invisible to a test suite
that grades on assertions.

**2. The consumer's exact code paths are infallible by construction.** `arrangements`'
exact geometry families declare `type Error = Infallible` precisely because no configuration
escapes the exactness ladder, and where a case genuinely is not handled they fail closed
with a *structured* value — `SphError::Unsupported`, detected by exact algebraic conditions
(`crates/arrangements/src/geoms/spherical_circle.rs:45-60`). A resolvent whose comparison
returns `Result` would force that consumer to invent an error path it does not have.

**3. Tolerance parameters are permanently out.** The same consumer's design excludes snap
rounding and automatic tolerance modes by construction. An API taking an epsilon would be
unusable by the consumer resolvent exists for.

---

## Decision

> **Fail at construction, not at query.**
> Every invariant is checked when a value is built. Construction returns `Result`. Every
> method on a well-formed value that is *mathematically* total is total *in the type system*
> too.

### 1. What returns `Result`

- **All constructors and parsers.** `AlgebraicReal::new(poly, lo, hi)` returns `Err` if
  `poly` is not squarefree, if `(lo, hi)` does not isolate exactly one root, or if
  `poly(lo) == 0` or `poly(hi) == 0` (F1, F4). `Ring::new` returns `Err` on a matrix order
  with negative entries or a variable count exceeding the arena width.
- **Operations whose input domain is narrower than their input type**: `div_rem` by zero,
  `inv` of a non-unit, a zero-dimensional routine given a positive-dimensional ideal.
- **Operations that can hit a documented capability limit**: monomial multiply on exponent
  overflow (ADR-008), degree beyond a packed bound.
- **Operations whose termination argument is a budget rather than a theorem** (§4).

### 2. What panics: nothing

No published crate panics outside `#[cfg(test)]`. Enforced by
`#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::indexing_slicing,
clippy::arithmetic_side_effects)]` and `cargo clippy -- -D warnings` in CI.

- `debug_assert!` is encouraged; it compiles out.
- A violated **internal** invariant returns `Error::Internal { invariant: &'static str }`.
  It does not panic. Two reasons: an embedding kernel may sit behind an `extern "C"`
  boundary where unwinding is UB, and callers under `panic = "abort"` cannot recover. More
  fundamentally, to a user of an exact kernel a panic and a hang are the same event — an
  operation that produced no answer.
- Allocation failure keeps Rust's default (abort). resolvent does not pretend to handle OOM
  and says so in the crate docs.

### 3. "Unsupported" is a structured value, never a string

```rust
#[non_exhaustive]
pub enum Unsupported {
    CoefficientRing    { got: RingTag, required: &'static [RingTag] },
    Characteristic     { got: u64, required: CharacteristicClass },
    VariableCount      { got: u32, max: u32 },
    TotalDegree        { got: u64, max: u64 },
    MonomialOrder      { got: OrderTag, required: &'static [OrderTag] },
    NotSquarefree,
    NotZeroDimensional { dimension: u32 },
    PositiveDimensionalRealSolve,
    TranscendentalSymbol  { name: SymbolId },   // L4 symbol reached an exact algebraic context
    NoDerivativeRule      { func: FuncId },     // ADR-017: differentiate an opaque Apply
    NoLeafRule            { symbol: SymbolId }, // ADR-017: diff_with, Refuse default
}
```

A consumer's fail-closed path matches on variants. A string forces string-matching and
breaks silently on rewording.

### 4. Budgets, and why they are not timeouts

**Every entry point that can loop takes a `Budget`**, counted in **steps** — never
wall-clock, because wall-clock is nondeterministic and ADR-012 forbids that.

*Amended 2026-07-31.* This ADR originally said "every loop **without an a-priori
termination proof** takes a `Budget`", which put `isolate_roots` and `sign_of` in one shape
here and a different shape in `API.md` (X-1, INV-6) and `plans/verification.md` §1.2 — three
signatures for the plan's most-called functions. The unification keeps both halves of what
each document was protecting: **the budget parameter is universal; the two regimes below
govern what exhaustion *means*, not whether the parameter exists.** Two consumers
independently derived the universal-parameter requirement, and a routine that can only run
to completion or panic cannot be a rung in a tiered ladder.

Two regimes, and the distinction is load-bearing:

- **A proven bound exists** — Mignotte–Davenport root separation for comparison,
  Landau–Mignotte for factor coefficient size, Hadamard for resultant coefficient size. The
  **default** budget is *derived from the bound*, so for a correct implementation exhaustion
  is **impossible**, and a decline on the default budget is therefore **a bug, not an
  answer**. It is a `debug_assert!` in debug, a diagnostics counter in release, and a
  **failure** in every grading context. A caller may still pass a *tighter* budget than the
  default, and then a decline is an honest "not at this price" — which is what a
  latency-bounded consumer wants and is why the parameter exists at all.
- **No proven bound exists** — van Hoeij lattice iteration, stabilization-driven modular
  reconstruction. The budget *is* the exit; exhaustion returns
  `Err(Error::BudgetExhausted { consumed, partial })` carrying enough state to resume, and a
  decline is a legitimate outcome counted against a committed decline-rate ceiling.

**Declines are classified before they are scored.** A decline inside a property test is a
**failure** iff (a) the instance is in the must-complete sub-corpus, or (b) the operation's
budget came from a proven bound. Otherwise it is a **survived instance** counted in the
decline rate. The blanket rule "any decline is a failure" is rejected: facing it, the
cheapest fix available to an agent is to raise the default budget until nothing declines,
which converts declines into long runs — i.e. into the sanctioned hang that §Context calls
the deadliest failure mode. **Budget defaults are committed values; raising one is a diff,
is counted in CI output, and requires a recorded justification** (ADR-024).

**`AlgebraicReal`'s budgets are always bound-derived.** Not "usually" — always, and this is
a correctness requirement rather than a performance choice. ADR-013's shared refinement
cache means the number of steps a given `cmp` performs depends on what has already been
compared, and under `parallel` on what other threads have compared. If a budget were charged
against *work actually done*, then `Ok` vs `Err(BudgetExhausted)` would differ by execution
order and by thread count — which ADR-012 asserts cannot happen, and which would make
property-test outcomes depend on shrinking order. The invariant, stated so it can be tested:

> **INV-AR1. The refinement cache may change how much work a call does. It may never change
> what the call returns, including whether it declines.**

Where a bound is unavailable for a given operand pair, the budget is charged against a
**worst-case** step count computed from the operands' degrees and coefficient sizes — a pure
function of the inputs — never against elapsed steps.

The `AlgebraicReal` property-test harness runs every case under a step budget and grades
"did not finish" as **wrong**, not as "timeout". Given that hangs are the expected failure
mode of a wrong implementation, this is the primary detector, not boilerplate.

### 5. One verdict vocabulary

> A function returns a **bare `Sign`** iff it is total and exact.
> A function that can be indeterminate returns **`Verdict<Sign>`** and never `Sign`.

```rust
pub enum Sign { Negative, Zero, Positive }
pub enum Verdict<T> { Certain(T), Unknown }
```

`Verdict` is produced **only** by enclosure and filter APIs — Bernstein `sign_over`, f64
enclosure comparison — and **never** by an algebraic-decision API. `Unknown` means "this
cheap rung declined to decide"; the caller's response is to climb to the exact rung, never
to guess. This maps 1:1 onto the consumer's existing `Uncertain<T>` without resolvent
importing anything.

### 6. The query surface is total, and here is why that is honest

Because construction enforces squarefree-ness and isolation (F1, F4), and because the
separation bound converts "terminates eventually" into "terminates in a computable number
of steps", the following are **infallible**:

```rust
impl AlgebraicReal {
    // Total surface: the default budget is derived from the separation bound, so a
    // correct implementation cannot exhaust it. Exhaustion is Error::Internal, not a
    // reportable outcome, and it is counted by the diagnostics hook.
    pub fn sign_of(&self, h: &UPoly<Integer>) -> Sign;      // not Result<Sign>
    pub fn is_root_of(&self, h: &UPoly<Integer>) -> bool;
    pub fn cmp_rational(&self, q: &Rational) -> Ordering;
    pub fn refine_to(&self, width: &Rational);

    // Budgeted siblings, for latency-bounded callers. Same verdicts, earlier exits.
    pub fn try_cmp(&self, o: &Self, b: Budget) -> Result<Ordering, Decline>;
    pub fn try_sign_of(&self, h: &UPoly<Integer>, b: Budget) -> Result<Sign, Decline>;
}
impl Ord for AlgebraicReal { /* total; see ADR-013 §5 for the ceiling and the measurement */ }
```

The prior art is explicit that it lacks this: "no separation-bound machinery is included —
comparisons terminate because distinct algebraic numbers are eventually separated by
bisection" (`crates/lazy-exact/src/roots.rs:8-11`). True, correct, and unbounded. Adding
the bound is what buys the total surface — and therefore the `Ord` impl (ADR-013).

### 7. Partial results are values

`AlgebraicReal` refinement is **monotone**: any observation of its bounds is a valid
enclosure, including a partially-completed one. There is no unsound intermediate state.
Probabilistic results are typed as `Certified<T>` (ADR-010), not annotated in prose.

### 8. No tolerance parameter, anywhere

No API in any published crate takes a tolerance, epsilon, or "close enough" argument, under
any name, at any layer. A grep gate enforces it (`plans/architecture.md` §1.3, L4).
`refine_to(width)` is not a tolerance — it is a *request for more precision*, it never
affects a verdict, and the verdict is identical whether or not it is called (property-tested
as "idempotence under refinement").

---

## Consequences

- **The consumer's `type Error = Infallible` survives.** All fallibility is concentrated at
  conversion/construction, where the adapter already has an error path.
- **Construction does real work.** `AlgebraicReal::new` must verify squarefree-ness and
  isolation, which is not free. Mitigated by an internal `new_unchecked` used by
  `isolate_roots`, which has just established both invariants — private, with the proof at
  the call site.
- **`Error::Internal` instead of a panic means a bug can be *returned*.** Callers may
  silently discard it. Mitigated by making `Error` `#[must_use]` through `Result`, by a
  diagnostics hook that counts internal errors, and by CI failing any test run with a
  nonzero internal-error count.
- **Two verdict types (`Sign` and `Verdict<Sign>`) rather than one.** Accepted: the
  alternative is one type where half the values are impossible for half the functions,
  which is worse. The rule for which is which is mechanical.
- **`BudgetExhausted` carrying `partial` state makes some error variants large.** Boxed.

---

## Alternatives considered and why rejected

**Return `Result` from every query, including `cmp` and `sign_of`.** Rejected. It is
*dishonest* once the separation bound exists — there is no failure to report — and it
forces the consumer to invent an impossible error case. It also makes `Ord` impossible,
which costs `sort`, `BTreeMap`, and every ordered collection.

**Panic on internal invariant violation (the usual Rust idiom).** Rejected for the
FFI/`panic = "abort"` reasons in §2, and because in this domain a panic is not more visible
than a returned error — both mean "no answer".

**Panic on exponent overflow / degree limits.** Rejected. Those are *legitimate inputs* at
the boundary of a capability, not programmer errors. A library that panics on a legitimate
input is unusable inside a kernel with its own error discipline.

**A single `Error::Unsupported(String)`.** Rejected. Not matchable, not stable, not
testable.

**Wall-clock timeouts instead of step budgets.** Rejected. Nondeterministic, which breaks
ADR-012 and makes a failing test irreproducible on a different machine.

**A `tolerance` parameter "for callers who want approximate answers".** Rejected
permanently. It is the F2 failure — equality by tolerance is *intransitive*
(`α = β`, `β = γ`, `α ≠ γ`), a sort then produces garbage, and a geometry consumer produces
a topologically inconsistent arrangement. No epsilon exists anywhere in any type.

**One verdict type everywhere (`Verdict<Sign>` from all sign APIs).** Rejected: it forces
every exact call site to handle an `Unknown` that cannot occur, which trains callers to
write `unwrap`-shaped code at exactly the boundary where correctness matters most.

---

## What would reverse this

- **A total operation turning out not to be total** — e.g. ADR-013's separation-bound
  argument failing for some class of inputs. Response: fix the bound, or move that specific
  method to `Result`. That is a targeted signature change, not a model change, and it is
  precisely the kind of thing the step-budget diagnostics counter exists to surface *before*
  it ships.
- **A consumer needing `no_std`.** The error model is compatible; ADR-013's `Arc` is what is
  not. Unrelated to this ADR.
