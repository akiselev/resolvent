# ADR-026 — Layer-3 entry-point signatures: `isolate_roots`, `SqfrPoly`, `rational_between`

**Status:** Proposed (2026-07-31)
**Reversibility:** one-way — these are public signatures on the layer `API.md` §4 calls
resolvent's headline differentiator, and every L3 consumer call site inherits them
**Amends:** ADR-014 §2 and §3. ADR-013 §5's `Inner.poly` field type is unaffected.
**Gates lanes:** A1, A4, U5.
**Evidence:** `docs/decisions/RECONCILIATION.md` §2.1, §2.2; `DESIGN.md` §5.3, §8.4;
`ADR-014` §2, §3; `API.md` §5.3, §7.5, L2-1, L3-4, INV-6;
`docs/research/challenge-generality.md` §1.1; `docs/research/critique-plan.md` C14.

---

## Context

Three normative documents give `isolate_roots` three different signatures, and two give
`rational_between` two. Neither track flagged it, because each wrote one half and
`ADR-021` §4's grep gate does not yet list `SqfrPoly`.

| Document | `isolate_roots` |
|---|---|
| `ADR-014` §3 (ratified, one-way) | `(p: &UPoly<Integer>, b: Budget) -> Result<Vec<IsolatedRoot>>` |
| `DESIGN.md` §5.3, repeated at §8.4 | `(p: &SqfrPoly, window: Option<(&Rational, &Rational)>, b: Budget) -> Result<Certified<Vec<IsolatedRoot>>>` |
| `API.md` §7.1(c), before 2026-07-31 | a separate `isolate_roots_in(p, lo, hi, budget)` over `Vec<(AlgebraicReal, u32)>` |

`ADR-021` §1 gives an ADR the last word on a signature, so as the register stands ADR-014's
form is binding — and it is the form that is wrong on two of four axes.

**The decisive defect is not a preference.** `DESIGN.md` §5.3 defines `SqfrPoly` as
"squarefree, primitive, lc > 0" and `square_free` as returning `Vec<(SqfrPoly, u32)>`. A
square-free polynomial has every root at multiplicity 1. So `isolate_roots(p: &SqfrPoly)`
returning `Vec<IsolatedRoot>` returns a struct whose `multiplicity` field is the constant
`1` — and `IsolatedRoot` exists only because ADR-014 was amended, on `critique-plan.md`
C14's argument, to keep multiplicity available to a consumer that *stores* a root. The
argument type makes the field vacuous.

Two further axes:

- **The window.** `API.md` L2-1 requires isolation over an optional window, because a DAE
  event detector isolates only within `[t_n, t_n + h]`. ADR-014's form omits it entirely; the
  earlier `API.md` sketch made it a second entry point, which doubles the surface for an
  `Option`.
- **The `Certified` wrapper.** `API.md` §5.3 and `DESIGN.md` §5.4 both adopt
  `challenge-generality.md` §1.1's correction that **an isolating interval certifies
  nothing** — the claim "`f` has exactly one root in `[a,b]`" is established by a
  Descartes/VCA sign-variation count, and the interval is the conclusion. The witness is
  returned under `ProofKind::RootCount` and the item moves from evidence tier F to tier C.
  A bare `Vec<IsolatedRoot>` has nowhere to put it. ADR-014's signature predates that
  correction.

Separately, `ADR-014` §2 has `rational_between(a, uppers) -> Rational` (total) and
`DESIGN.md` §8.4 has `(a, uppers, b: Budget) -> Result<Rational, Decline>` (fallible).

---

## Decision

### 1. `isolate_roots` takes an unrestricted `UPoly<Integer>`, a window, and returns `Certified`

```rust
pub fn isolate_roots(
    p:      &UPoly<Integer>,
    window: Option<(&Rational, &Rational)>,
    b:      Budget,
) -> Result<Certified<Vec<IsolatedRoot>>>;
```

There is **one** entry point, not two. `window: None` means the full Cauchy box. The square-
free reduction happens inside, which is what makes `IsolatedRoot::multiplicity` meaningful:
the routine runs Yun, isolates the roots of each square-free factor, and reports the factor's
exponent as the multiplicity. `Certainty` is `Proved(ProofKind::RootCount)` carrying the
sign-variation witness per interval.

A `UPoly<Rational>` handed in at the boundary is converted by `clear_denominators()` on
ingress — ℚ is a transport type, not a working type (ADR-004).

### 2. `SqfrPoly` is public, and is on construction, not on isolation

```rust
pub struct SqfrPoly(/* UPoly<Integer>, squarefree, primitive, lc > 0 */);
impl SqfrPoly { pub fn new(p: &UPoly<Integer>) -> Result<SqfrPoly>; }  // Err(NotSquarefree)

pub fn square_free(p: &UPoly<Integer>) -> Certified<Vec<(SqfrPoly, u32)>>;   // Yun

impl AlgebraicReal {
    pub fn new(poly: SqfrPoly, lo: Rational, hi: Rational) -> Result<AlgebraicReal>;
    pub fn defining_poly(&self) -> &SqfrPoly;
}
```

`DESIGN.md` §5.3's fail-closed-by-type argument is correct **for construction**: with a
double root the sign never changes across the interval, bisection cannot decide which half to
keep, and every downstream guarantee collapses — so square-freeness is a type rather than a
`Result` the caller always pre-checks. It is wrong for isolation, per §1. `API.md`'s previous
claim that `SqfrPoly` is not caller-visible is withdrawn: `square_free` is where a caller
obtains one, and `AlgebraicReal::new` and `defining_poly` are where it is used.

### 3. `rational_between` ships as a total function plus a budgeted sibling

```rust
pub fn rational_between(a: &AlgebraicReal, uppers: &[AlgebraicReal]) -> Rational;
pub fn try_rational_between(a: &AlgebraicReal, uppers: &[AlgebraicReal], b: Budget)
    -> Result<Rational, Decline>;
```

Both, not either. Two *distinct* algebraic reals are separated by a computable bound, so the
refinement loop is bound-derived: `API.md` INV-6 regime 1, exhaustion proven impossible, the
budget a bug detector, and the query total. INV-6's own last clause then requires the
budgeted sibling for every total query that can allocate unboundedly, which this one can —
the same shape already settled for `cmp`/`try_cmp` in ADR-013 §5b, and the shape a consumer
that hand-rolled this primitive **twice in two crates** already chose, both times with a hard
256-step budget.

`rational_between(a, &[])` — a witness strictly above `a` with no upper constraint — is
total by the same argument and is not a special case.

---

## Consequences

- **`IsolatedRoot::multiplicity` becomes meaningful**, which is what ADR-014's amendment was
  for. Under `&SqfrPoly` it would have been a constant, and a consumer reading it would have
  concluded resolvent had no multiplicity information at all.
- **One isolation entry point, one `Option`.** The DAE-event adapter passes
  `Some((&lo, &hi))`; the geometry adapter passes `None`. Neither needs a second function
  name in scope.
- **A consumer can check an isolation result.** The `RootCount` witness makes the operation
  auditable by a proof-assistant or SMT consumer that trusts nothing, which
  `challenge-generality.md` §1.1 identified as the one defect invisible from inside all three
  surveyed consumers — all three consume intervals as *data*.
- **Retaining the witness is not free**, and that is why the item is evidence tier C rather
  than tier F. The constant factor is documented at the entry point, per `API.md` §5.2's rule
  that no certificate may add more than a documented constant factor to the answer path.
- **`square_free` returning `Certified<Vec<(SqfrPoly, u32)>>` is the only public minter of
  `SqfrPoly` besides `SqfrPoly::new`.** A consumer that wants the roots does not touch either.
- **Two names for `rational_between` where one existed.** Accepted, because the alternative
  is a consumer that cannot bound a latency-path call, which is exactly the position the
  incumbent's twice-duplicated 256-step loop was working around.

---

## Alternatives considered and why rejected

**Keep `ADR-014` §3's signature unchanged, since ADR-021 §1 makes it binding.** Rejected on
the merits, which is what an amendment is for. Precedence decides who arbitrates, not what is
true; and the vacuous-multiplicity defect is a fact about the two type definitions, not a
matter of taste.

**Keep `&SqfrPoly` and drop `multiplicity` from `IsolatedRoot`.** Rejected. It would reverse
ADR-014's amendment and reinstate exactly the collision `critique-plan.md` C14 found: the
nearest prior art has `RealRoot::multiplicity(&self) -> u32` as a method on a stored value,
and a consumer that stores roots and later asks a multiplicity would have to thread a
parallel structure. It would also delete a capability with a real consumer — a double
radicand root *is* a sheet-junction signature.

**Keep both `isolate_roots(&SqfrPoly, ..)` and `isolate_roots_full(&UPoly<Integer>, ..)`.**
Rejected. Two entry points where the second calls Yun and then the first; the square-free
one saves nothing a caller who already ran `square_free` cannot get by isolating each factor
itself, and it invites the wrong one to be reached for.

**Make `SqfrPoly` crate-private and have `AlgebraicReal::new` take a `UPoly<Integer>` with a
`Result`.** Rejected. That is the signature smell `DESIGN.md` §5.3 diagnoses — a
precondition the caller must always pre-check, expressed as a runtime error — and it deletes
the type-level guarantee that the refinement loop depends on.

**Make `rational_between` fallible only.** Rejected. It forces every consumer to invent an
error path for an outcome the mathematics proves cannot occur, and the surveyed consumer's
exact families declare `type Error = Infallible` (ADR-011 §Context), so a fallible-only form
would push a `Result` into predicates that have nowhere to put it.

**Make `rational_between` total only.** Rejected. `API.md` §6.2 measures the consumer's
latency classes and the arrangement sweep cannot afford an unbounded call; the incumbent's
two hand-rolled versions both carry a budget, which is evidence rather than preference.

---

## What would reverse this

- **A measured `RootCount` witness cost above a small constant factor** on the M2 corpus.
  Response: keep the witness behind the same tier-C rule — `isolate_roots_unchecked`
  returning `Certainty::Probable` — not remove it from the default path.
- **`try_rational_between` never being called by any consumer after M3.** Response: keep it,
  because deleting a public function is breaking and its cost is one function; but stop
  citing it as evidence for the pair rule.
- **A measured case where isolating a non-square-free polynomial through the internal Yun
  pass is materially slower than the caller doing it.** Response: expose
  `isolate_roots_sqfr(&SqfrPoly, ..)` **additively**, with every multiplicity documented as
  1, rather than changing this signature.
