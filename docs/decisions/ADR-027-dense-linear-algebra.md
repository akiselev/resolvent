# ADR-027 — Dense linear algebra is public, context-taking, and scheduled

**Status:** Proposed (2026-07-31)
**Reversibility:** costly — the signatures are public and the scheduling change moves a lane
onto the shortest path to consumer value, where it is expensive to discover missing
**Amends:** ADR-005 §Decision (the sentence placing `row_echelon` and `bareiss_det` in
`resolvent-algebra`); proposes lane `LA` and an M2 Lands entry to `ROADMAP.md`.
**Gates lanes:** LA (new), G4, T3.
**Evidence:** `docs/decisions/RECONCILIATION.md` §2.3, §4 A1, §5.1; `DESIGN.md` §1.2, §3.3,
§3.4, §8.3; `ADR-006` §Decision (the `Ctx` amendment); `API.md` L2-6, L2-7, §6.3, §7.3;
`ROADMAP.md` §5 M1 row, Wave 2, Wave 3 lane T3.

---

## Context

Two defects, one in a signature and one in the schedule, both on the same capability.

**The signature does not compile.** `DESIGN.md` §8.3 declares

```rust
pub fn row_echelon<C: Field>(rows: Vec<Vec<C>>) -> Result<Echelon<C>>;
pub fn bareiss_det<C: CommutativeRing>(m: &Matrix<C>) -> Result<C>;
```

Under `ADR-006`'s amended tower there is no `C::zero()`. `Ring::zero` takes `&Self::Ctx`,
because `Fp` carries its modulus and Barrett reciprocal **by value** and cannot answer "zero
of which prime field" from no information at all — that is the defect that made the
pre-amendment tower unimplementable for five of the seven rings in its own instantiation set.
Row reduction must produce a zero for an all-zero pivot column, and Bareiss must be able to
return a zero determinant. Both therefore need the context.

This is a **knock-on the architecture track applied and then missed**: `DESIGN.md` §5.1
detail 1 states the consequence for polynomial types in so many words — "`UPoly<C>` and
`MPoly<C>` store one `C::Ctx` alongside their coefficients" — and §8.3 was not revisited.
`API.md` §7.3's adapter sketch already passes `&fp` as the first argument and is right.

**Nothing schedules it.** `DESIGN.md` §1.2 lists dense linear algebra in Layer 2's in-scope
set, §3.3 puts `linalg` in `resolvent-algebra`, §3.4 argues at length about which crate it
belongs in and records the compile cost of the answer, and §8.3 gives its signatures.
`API.md` places `row_echelon` (L2-6) and `bareiss_det` (L2-7) in **core** on two independent
consumers. **`ROADMAP.md` has no lane and no milestone for either.** Lane `T3` is
"Bareiss / Bézout determinant route", scoped inside M4 as the third independent *resultant*
route — a different deliverable with a different verdict function.

The scheduling gap has a visible symptom in the roadmap's own text: `ROADMAP.md` §5's
consumer value ladder says CAD constraint solving becomes possible at **M1**, and attributes
it to "`resolvent-modular` plus a row-echelon returning rank, pivot rows, dependent rows
**and the transform**" — while M1's Lands list contains no linear algebra at all.

**Why the gap matters more than its size.** `row_echelon` over a field is the *whole* of one
consumer's demand above Layer 0 (`API.md` §6.3), and `bareiss_det` over an integral domain is
what the strongest consumer asks for to replace a measured 2.448 ms recursive Laplace
determinant. Neither depends on anything above `Field` and `resolvent-modular`: not `UPoly`,
not the monomial layer, not `MPoly`. It is the cheapest consumer-visible capability in the
plan and it is the one with no owner.

---

## Decision

### 1. The signatures take the coefficient context explicitly

```rust
pub mod linalg {
    pub fn row_echelon<C: Field>(ctx: &C::Ctx, rows: Vec<Vec<C>>) -> Result<Echelon<C>>;

    pub struct Echelon<C: Field> { /* private */ }
    impl<C: Field> Echelon<C> {
        pub fn rank(&self)            -> usize;
        pub fn pivot_rows(&self)      -> &[usize];
        pub fn pivot_cols(&self)      -> &[usize];
        pub fn dependent_rows(&self)  -> &[usize];
        /// The transform `T` with `T · A₀ = A`. Evidence tier F: it *is* the answer's shape.
        pub fn transform(&self)       -> &[Vec<C>];
    }

    pub fn bareiss_det<C: CommutativeRing>(ctx: &C::Ctx, m: &Matrix<C>) -> Result<C>;
}
```

**No prime, modulus, or `Budget` appears in either signature.** That is `API.md` L2-7's
constraint and it is a boundary rule, not an omission: a consumer asks for a fast exact
determinant, and modular arithmetic is *how* you give it one, not *what* it asked for. A
caller that wants the GF(p) path calls with `C = FpElem` and its `FpParams` context; a caller
that wants ℤ or ℚ calls with those, and the implementation may go modular internally and
reconstruct. Termination is by dimension, so neither is a budgeted entry point (`API.md`
INV-6 regime 1).

### 2. The transform is returned unconditionally, not on request

It is the same object as a Gröbner cofactor representation one layer down, it costs `O(n²)`
storage against the `O(n³)` the elimination already spends, and one consumer ships an
`implied_by` certificate as an unconditionally empty vector at two sites for want of it. A
`with_transform: bool` parameter would be the boolean flag `API.md` §5.2 rejects for
certificates, in the one case where the evidence genuinely is free.

### 3. Lane `LA`, in Wave 2, landing in M2

| Field | Value |
|---|---|
| Lane | `LA` — dense linear algebra over a field and over an integral domain |
| Wave | 2 |
| Gates | ADR-005, ADR-006, ADR-011, ADR-021, ADR-027 |
| Depends on | Z0, Z1, Z3. **Not** on U1, P1–P3, or the monomial layer |
| Grade | certificate |
| Size | S–M |
| Oracle | naive Gaussian elimination over the same `C`, in the same crate |

Verdict function, all four required:

- **`T · A₀ == A`** by direct multiplication, for every instance — the transform certifies
  itself and shares no control flow with the elimination that produced it.
- **Rank agreement** with an independent minor computation at `n ≤ 6`, and with the naive
  reference at every size.
- **Bareiss against naive Laplace expansion at `n ≤ 6`**, and against `row_echelon`'s pivot
  product over a field.
- **Mutants rejected** (ADR-023 §1): the *identity* mutant (return the input unchanged with
  `rank = n`), the *coarsening* mutant (report one dependent row where there are two), and
  the *trivial constant* mutant (`rank = 0`, empty transform), the last of which must also be
  caught by a committed sharpness floor — `rank` is exact, so the floor is `1.0` and is never
  ratcheted.

`ROADMAP.md` §5's M1 consumer row moves to **M2** in the same edit.

---

## Consequences

- **One consumer's entire demand above Layer 0 acquires an owner**, three milestones earlier
  than the resultant lane it was being carried by implication.
- **G4 inherits a worked precedent.** Cofactor tracking in the certified Gröbner mode is the
  same object at a harder layer; building the easy case first, with its self-multiplying
  certificate, is the "build the oracle side first" rule applied one layer down.
- **`resolvent-algebra` gains a lane that touches none of the polynomial layer**, which is a
  good fan-out property: `LA` can run concurrently with the whole univariate trunk with
  disjoint files and a disjoint test suite.
- **The compile cost recorded in `DESIGN.md` §3.4 stands**: a consumer wanting only generic
  rank compiles the polynomial layer it does not use. That is a compile cost, not a
  correctness or API cost, and the asymmetry argument (merging two published crates strands a
  name; splitting one later is mechanical) still says start merged.
- **`Echelon<C>` becomes a public type and must be added to ADR-021 §4's grep-gate list**,
  alongside `SqfrPoly`.
- **Do not size this lane on a speedup.** `challenge-evidence.md` refuted the "18× / 40×
  against LAPACK" headline against the adjacent column of its own source table: against a
  single-pass float baseline the unoptimized modular echelon is 4.4× faster at n=200, 1.5× at
  n=400, and **2.9× slower at n=800**, so the claim is false at the largest size measured, and
  column-pivoted float QR delivers one pass plus rank, pivots, dependent rows and a
  dependency certificate. The durable wins are **exactness at near-degenerate configurations
  and the transform**, not wall-clock, and substituting generic rank for numerical rank is a
  semantics change rather than a drop-in speedup.

---

## Alternatives considered and why rejected

**Leave `linalg` unscheduled and let it fall out of T3.** Rejected. T3's deliverable is a
determinant *route* for resultants, graded by agreement with two other resultant routes; it
has no `row_echelon`, no transform, no rank, and no public module. A capability that three
documents call core and no lane builds is how a milestone ships without the thing its own
consumer ladder promised.

**Keep `DESIGN.md` §8.3's context-free signatures and give `Field` a `zero()` associated
function.** Rejected — that is the pre-amendment tower, and `ADR-006` §Context documents why
it has no valid implementation for `Fp`, `Fp4`, `Zn`, `GFpk` or `NumberFieldElem`.

**Pass a `&Ring` context object and call `ring.add(&a, &b)` inside the kernel.** Rejected in
ADR-006 and unchanged: an indirect call per coefficient operation. The context is consulted
at construction only, which is per-call.

**Take `ctx` from the first element of `rows`.** Rejected. An empty matrix, and a matrix
whose rows are all empty, are both legal inputs with well-defined answers (`rank = 0`), and
neither has an element to read a context from — the same argument that puts the target `ctx`
on `map_coefficients`.

**Split `resolvent-linalg` out as its own crate now.** Rejected, per `DESIGN.md` §3.4 and
ADR-005: crate names on crates.io are sticky, merging two published crates strands a name,
and one consumer's compile cost does not justify a tenth crate. If a second
linear-algebra-only consumer appears, splitting later is mechanical.

**Make the transform opt-in behind a parameter.** Rejected, §2.

---

## What would reverse this

- **A second consumer wanting linear algebra without polynomials.** Response: split
  `resolvent-linalg` out — the same ownership rule, one more crate.
- **`Echelon`'s transform measuring as a material cost on the F4 row-reduction path.**
  It will not, because that path is Tier M and concrete and does not call this module; if
  something upstream starts calling it, the response is a Tier-M kernel next to it, not an
  opt-out parameter here.
- **Lane `LA` proving to need `UPoly` after all** — for example if the ℚ path is implemented
  by clearing denominators through a polynomial type. Response: keep the lane in Wave 2 and
  gate only its ℚ instantiation on U1; the `Fp` and `Integer` instantiations, which are the
  consumer-facing ones, still do not.
