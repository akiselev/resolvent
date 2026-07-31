# ADR-010 — Modular methods everywhere, with the certificate in the return type

**Status:** Ratified 2026-07-31
**Reversibility:** one-way
**Amended:** 2026-07-31 — four corrections: the certificate type adopts `API.md` §5's shape
and `ProofKind` is unified by union; §4 gains the algebraic-extension bad-prime predicate;
§5's shared-reducer claim is **retracted** and the cofactor prototype is respecified; §7
gains the batch-split driver (critique-engineering §3, §4, §14, §16).
**Gates lanes:** Z0, Z6, U3, T2, G3, G4, G5, G6.
**Evidence:** `docs/research/algorithms-and-representation.md` §3, §6.2, §9.1;
`docs/research/prior-art-and-licensing.md` §1.3;
`docs/research/critique-engineering.md` §3, §4, §14, §16.

---

## Context

The source spec calls this "*the* structural decision" and says naive rational arithmetic
gives coefficient explosion and a dead project. The research confirms the diagnosis and
sharpens it in three ways that change the design.

**1. The numbers are not marginal.** Cyclic-10 over ℚ (Giac): 5690 basis elements, ~20
million monomials, coefficients needing **more than 2000 primes of 29 bits** (≈58 000 bits)
to reconstruct, **>50 GB of RAM**, 14 hours. Katsura-`n` primes needed by msolve:
`n=9` → 83, `n=11` → 388, `n=13` → 1772, `n=14` → 3847 — roughly doubling per step. The
same effect at geometry scale: two degree-10 bivariate curves with 32-bit coefficients give
a resultant of degree ~200 with ~500-bit coefficients.

**2. It is also a licensing decision.** GMP/FLINT-class bignum speed is unavailable
permissively (ADR-001, ADR-002). Modular methods are what keep nearly all bignum work in
the sub-kbit regime, which is exactly where the permissive bignum is competitive or better.
The bignum choice and this one are the same decision viewed from two sides.

**3. "Verification is cheap" is true for two of three algorithms and false for the third.**
The spec asserts self-certification uniformly. It is not uniform:

| | Certificate | Cost |
|---|---|---|
| **GCD** | `H \| A`, `H \| B` (proves `H \| G`) **plus** `deg H = deg gcd(A mod p, B mod p)` for one good prime (proves `deg H ≥ deg G`). Together `H = G` up to a unit. | Two exact divisions and one modular gcd. Genuinely cheap. |
| **Factorization** | Multiply back — certifies the *product*, **not irreducibility**. A buggy recombination merging two true factors produces `f = g·h` with `g` reducible and the product check passes. | Cheap, and incomplete. The irreducibility certificate (exhibit a prime where `f_i mod p` is irreducible of full degree) **does not always exist** — Swinnerton-Dyer polynomials factor nontrivially modulo every prime. |
| **Gröbner** | Three separate claims. `I ⊆ ⟨G⟩`: reduce each generator to zero over ℚ — moderate. `G` is a Gröbner basis: all S-pairs reduce to zero over ℚ — ≈ recomputing the basis. **`⟨G⟩ ⊆ I`: no cheap general certificate exists.** Arnold's Hilbert-function argument covers homogeneous ideals only; Idrees–Pfister–Steidel extended it to the non-homogeneous global-order case, and Noro–Yokoyama showed that theorem needs an extra assumption. | The direct route to `⟨G⟩ ⊆ I` is cofactor tracking (`f = Σ hᵢgᵢ`), which is a genuine certificate — and its cost is **cofactor swell** through F4's linear algebra, with a real memory risk. |

And a fact that the benchmark harness must be built around: **msolve, Maple/FGb and
Groebner.jl all default to UNCERTIFIED over ℚ.** Groebner.jl states it plainly: "there is
no out of the box guarantee that the reconstructed basis is correct over the rational
numbers." A certified resolvent loses those benchmarks by construction.

---

## Decision

### 1. The loop, uniformly

Every computation over ℤ or ℚ that admits it goes: **reduce mod several primes → compute in
GF(p) → CRT + rational reconstruction → verify.** Concretely bounded in the type system by
`C: Reducible + Liftable` (ADR-006). A coefficient ring that cannot be reduced mod `p`
cannot reach this path and gets the generic reference implementation.

### 2. The certificate is in the return type, not in a log line

*Amended 2026-07-31.* This ADR originally specified a flat `Certificate` enum. Three
documents then specified three incompatible shapes, with **disjoint** `ProofKind` variant
sets, while lane Z0/Z7's deliverable is "the `Certificate` type" and every Layer-2 and
Layer-3 signature depends on which one gets built. The shape below is `API.md` §5's,
adopted here as normative, with `ProofKind` unified by union. `API.md`'s shape wins because
the **claim tether** and the **absence of a public mint** are requirements from a real
consumer evaluation that a flat enum cannot express.

```rust
pub struct Certified<T> { pub value: T, pub certainty: Certainty }

pub struct Certificate<C: Claim> {
    claim:     C,             // private
    evidence:  C::Evidence,   // private
    certainty: Certainty,     // private
}
impl<C: Claim> Certificate<C> {
    pub fn claim(&self)     -> &C;                  // public READ
    pub fn evidence(&self)  -> &C::Evidence;        // public READ
    pub fn certainty(&self) -> Certainty;
    pub fn certifies(&self, claim: &C) -> bool;     // structural tether
    pub fn verify(&self, budget: Budget) -> Result<(), Error>;   // public ops only
}
// No public constructor on any certificate type. Mints are pub(crate).

pub enum Certainty { Proved(ProofKind), Probable(ProbableReason) }

pub enum ProofKind {          // the union of the three earlier lists
    Identity,                 // u·f + v·g == r, cofactor identities, multiply-back
    Divisibility,             // exact division witnesses
    DivisibilityAndDegree,    // the GCD certificate: both halves
    Cofactor,                 // Gröbner: g_j = Σ h_ij f_i
    Enclosure,                // Bernstein / interval-certified sign
    DegreeBound,              // Hadamard / Bézout degree envelopes
    BoundDriven { bound_bits: u64, primes_used: u32 },   // Landau–Mignotte / Hadamard
    ProductAndModularIrreducibility { primes: SmallVec<[u32; 4]> },
    ExhaustiveSmallCase,
}
pub enum ProbableReason {
    Stabilized      { rounds: u32, primes_used: u32 },
    MajorityVote    { agreeing: u32, total: u32 },
    RandomizedCheck { failure_prob_log2: i32 },
}
```

**Unforgeable means no public mint; checkable means public read.** Both, not either. A
consumer that trusts resolvent calls `verify()`; a consumer with its own trusted computing
base reads `evidence()` and checks it with its own arithmetic. `certifies()` is structural
equality against the tethered claim, so a transplanted certificate fails the comparison
instead of riding along.

**`Probable` is allowed to exist — Gröbner over ℚ needs it — but it must be visible in the
type, and the default path must be `Proved`.** A caller who wants speed asks for it by
name; a caller who does nothing gets a proof.

**Certificates and telemetry are excluded from canonical bytes.** *Added 2026-07-31, and it
is load-bearing rather than editorial.* ADR-012 §8 asserts value-equality across a `Tuning`
matrix; the modular batch width `N` is a tuning knob and it changes `primes_used`. If
`ProbableReason` were serialized, that CI gate would fail on its first run. Only the
mathematical value is serialized (ADR-012 §9). The same exclusion covers `Telemetry
{ tier_reached, bisections, precision_bits, primes_used }`, which is returned alongside a
certificate as **plain proof-free data** so a consumer can cache warm-start hints without
laundering them into evidence.

### 3. Stopping rules, and which is which

- **Bound-driven** (deterministic): compute an a-priori bound on the answer's coefficients —
  Landau–Mignotte for factors and gcds, Hadamard for determinants and resultants — use
  enough primes to exceed `2 × bound`, and the CRT result is provably correct. Sometimes
  wildly pessimistic; always `Proved`.
- **Stabilization-driven** (heuristic): add primes until the reconstruction stops changing.
  Cheaper. **Stabilization alone is not a proof** and yields `Probable(Stabilized)` unless
  closed by a verification step that upgrades it to `Proved`.

For resultants specifically the bound exists and is cheap
(`O((m+n)τ + (m+n)log(m+n))` bits from the Sylvester determinant), so the resultant lane is
`Proved` by default with no argument.

### 4. Bad-prime and bad-point detection is per-algorithm and explicit

- **GCD**: `p` is bad if `p | lc(A)` or `p | lc(B)` (degree drop, directly detectable) or
  *unlucky* if `deg gcd(A mod p, B mod p) > deg gcd(A,B)`. The mod-`p` gcd can only be
  bigger, never smaller, which gives Brown's rule: **keep only images of the minimal degree
  seen, discard the rest.** The same argument one level down governs unlucky evaluation
  points in the Brown/Zippel recursion.
- **Factorization**: `p | lc(f)` or `p | disc(f)` are bad. Beyond that a prime is merely
  *unhelpful* (many small modular factors makes recombination harder without making the
  answer wrong); try several small primes and keep the one minimizing the modular factor
  count `r`.
- **Subresultants / resultants**: a specialization is bad when it drops a leading
  coefficient — evaluate `lc(a)` and `lc(b)` at the point and reject if either vanishes.
  Points where the gcd degree jumps corrupt the low subresultants and are caught by the same
  minimal-degree-wins rule.
- **Gröbner**: `p` is good iff the lead-monomial set of the mod-`p` basis agrees with the
  one over ℚ, which cannot be checked directly. Practical rule: compute mod several primes,
  **majority vote over lead-monomial sets**, discard the minority, run a stabilization test
  on the reconstruction.
- **Algebraic-extension coefficients (ℚ(α), M8)** *(added 2026-07-31)*: `p` is bad if it
  divides the denominator of any coefficient or the discriminant of the minimal polynomial
  `f`. Beyond that, **`p` is essentially never inert**, so the modular path here is *not*
  "reduce into a field": it is **multi-modular over split factors** — factor `f mod p` into
  `Π f_i` of degrees `d_i`, work independently in each `GF(p^{d_i})`, CRT the images back.
  ADR-006 §Context defect (3) has the argument: for ℚ(√2, √3) the Galois group is `(ℤ/2)²`,
  has no 4-cycle, and therefore **no prime is inert at all**. The bad-prime predicate for
  this path is "`f mod p` is not squarefree", which is checkable, plus the usual leading-
  coefficient conditions in each factor.

  **This is a lane, not an instantiation.** Writing `UPoly<NumberFieldElem>` compiles; it
  does not get the fast path for free, and the plan previously implied it did. Lane
  brief M8-N carries the split-factor driver; ℚ(√2, √3) is in the M8 corpus specifically
  because it is where a naive implementation divides by a zero divisor.

Every accepted and every rejected prime is recorded **by index** in the `Trace` (ADR-012),
so a Las Vegas run is replayable.

### 5. Two Gröbner modes, cross-checking each other

- **`groebner_certified`** — tracks cofactors, returns `Proved(CofactorRepresentation)`, is
  the differential-test and regression workhorse, **is not expected to be competitive.**
- **`groebner`** — modular + tracing + majority vote + stabilization + a randomized check,
  returns `Probable`, is the performance lane.
- **The certified mode's output must equal the fast mode's output on every regression
  instance.** That is a free oracle for the fast path and exactly the structure an
  agent-graded build wants.

**Retraction, 2026-07-31.** This section previously claimed: "The two modes share one
reduction implementation … Two separate reduction implementations would mean the certified
mode is not testing the fast mode's reducer, which is the fast mode's *only* internal
oracle." **The premise is false and the conclusion therefore stands unmet.** The two modes
*cannot* share a reducer:

- `plans/architecture.md` §2.1 puts F4 row reduction in Tier M — concrete over `u32`
  payloads with `FpParams` by value, sparse row format. The fast reducer is a GF(p) kernel.
- `plans/verification.md` §2.5 costs the certified mode as `|F|` normal forms **over ℚ**
  with full coefficient blowup, and checks cofactors by multiplication and addition over
  ℚ/ℤ — because **a cofactor identity that holds mod `p` certifies nothing over ℚ**.

A `u32` GF(p) kernel and a ℚ/ℤ reducer are not the same code. The plan already contained
its own refutation and did not connect it: `plans/verification.md` §3.14 grades
`groebner_certified` vs `groebner` as "substantial sharing; this cross-check is weaker than
it looks."

**What is actually shared, stated correctly:** matrix construction, symbolic preprocessing,
the monomial layer, pair selection, and the row format — including the optional cofactor
block, which remains a genuine design requirement so that the certified mode is not a
second engine. **The reducer is not shared.**

**Consequences that must be written into the lane briefs rather than discovered:**

1. **Lane G3 (sparse GF(p) row reduction) gets its own internal oracle**, because G4 cannot
   be one: **a naive dense `u32` Gaussian elimination over the same `FpParams`, in the same
   crate, as the committed reference.** Same arithmetic, different control flow — a genuine
   cross-check, and one agent-session. Without it, a bug in G3's pivot selection, its
   delayed-reduction cutoff, or its Barrett reduction is invisible to every internal check
   in the library.
2. **G3's *primary* verdict is external differential testing** (Singular, msolve), the same
   inversion of the normal rule already written into G5's brief. This is stated in the brief,
   not left to be inferred.

### 5a. The cofactor gate, respecified

**Gate:** `groebner_certified` is not committed to the plan until experiment **E-COFACTOR**
returns. The original gate — "prototype cofactor tracking on Katsura-8 / Cyclic-7 and
measure the memory and time multiplier" — is wrong twice over.

*It measures the wrong number.* The multiplier it names is the GF(p) time/memory cost of
carrying extra columns, which is a constant factor on row width. To be a certificate **over
ℚ**, the cofactors must be *reconstructed*, and cofactor coefficients are systematically
larger than basis coefficients — that is what "cofactor swell" means — so **the prime count
is set by the cofactors, not by the basis**, and there are `|F| × |G|` of them. Cyclic-10
needs >2000 primes for the basis alone (§Context); the cofactor system needs more. The
reconstruction multiplier is the number that decides whether certified mode can exist.

*It needs the artifact it gates.* Katsura-8 requires an engine that reaches Katsura-8 —
lanes G1/G2 in Wave 4 — while gating whether that engine's certified mode exists.

> **E-COFACTOR.** Implement **Buchberger with cofactors** (not F4) over ℚ, on Katsura-6 and
> Katsura-7 and Cyclic-6. Report, per instance: (a) the number of primes needed to
> reconstruct the **cofactor system** over ℚ, against the number needed for the basis;
> (b) wall time and peak RSS of the reconstruction; (c) both as a function of instance size,
> so they can be extrapolated to Katsura-8/Cyclic-7. Buchberger-with-cofactors is lane G1's
> deliverable anyway, so this is sequencing rather than throwaway work.
>
> **Abort criterion, committed before the run:** if reconstructing the cofactor system needs
> more than **5× the basis's prime count**, or if extrapolated peak memory at Katsura-8
> exceeds **20×** the uncertified run, then `groebner_certified` becomes
> *small-instance-only*, is documented as an oracle rather than an API, and the fast mode's
> primary verdict becomes external differential testing at every size.

### 6. The bivariate modular path is not an optimization

Modular subresultant chains are up to **10× faster in ℤ[y] and 400× in ℤ[x,y]** than
Ducos, with a further 7×/2× from Half-GCD speculation. The bivariate case is exactly what a
geometry consumer generates. Both implementations exist — Ducos as the reference and the
oracle, modular as the production path — and they share almost no code, which makes them a
strong differential pair (ADR-016).

### 7. Batching stays possible

Groebner.jl computes over `Z/p₁ × … × Z/p_N` as tuples, sharing all non-arithmetic work and
exposing SIMD, for up to ~2.7× amortized (`N = 4` in production; 8/16/32 gave no further
gain). ADR-006's `LANES`/`Scalar` on the base trait is what keeps this reachable.

**Batching requires a split driver, and lane G6's brief says "batching *and* splitting".**
*Added 2026-07-31.* Batched multi-modular arithmetic works only while all `N` primes behave
identically, and two events certain to occur break that:

1. **A pivot zero in one lane.** F4 needs `inv(pivot)`. ADR-006 §Decision now provides
   `BatchField::inv_batch(&self) -> Result<Self, LaneMask>`; the driver reads the mask,
   splits the batch, finishes the good lanes, and re-runs the faulting prime alone.
2. **Lead-monomial divergence.** §4's Gröbner rule is a majority vote over lead-monomial
   sets — but under batching, all `N` primes share one matrix construction and one
   pair-selection path (that sharing *is* the 2.7×), so a diverging prime **corrupts shared
   control flow** rather than producing a minority to discard. The driver therefore compares
   lead-monomial sets **per lane after each matrix**, and splits on divergence.

On either fault the batch splits and the offending prime index is recorded in the `Trace`
(ADR-012 §7), so the run stays replayable. Lane Z5's "componentwise equality with `N` scalar
runs" is a complete oracle for **arithmetic** and is silent on both of these, which are
control-flow failures; G6's verdict function must include a planted-fault test that forces
each split path.

---

## Consequences

- **The permissive bignum becomes affordable.** This is the load-bearing consequence and it
  is why ADR-002 and this ADR must be ratified together.
- **Every modular routine needs a bad-prime predicate, a stopping rule, and a verification
  step.** Three things per algorithm, not one. That is more design work per lane and it is
  the work that makes the lane self-certifying.
- **resolvent will lose published Gröbner-over-ℚ benchmarks to msolve/Maple/Groebner.jl by
  construction**, because they default to uncertified and resolvent defaults to certified.
  The harness must compare like with like — `groebner` (Probable) against their default,
  and report both. Hiding the difference would be dishonest; defaulting to Probable would
  violate fail-closed.
- **`Certified<T>` appears in a lot of signatures.** Accepted. A caller that does not care
  writes `.value`, and the type is what stops "probably right" from silently becoming
  "right".
- **Retrofitting is not possible.** Building the algorithms over ℚ first and adding modular
  methods later is a rewrite, not an optimization. Sequencing consequence: modular gcd and
  squarefree decomposition land **before** `Res_y` and curve analysis.

---

## Alternatives considered and why rejected

**Compute directly over ℚ, add modular methods when profiling demands.** Rejected — this is
the failure the source spec names, the research quantifies (§Context 1), and the sequencing
consequence above forbids.

**Monte Carlo by default (match the competition).** Rejected. It contradicts fail-closed and
it removes the mechanism (`certified` vs `fast` agreement) that gives the fast lane an
automatic verdict. The competitive comparison is preserved by *offering* the probable mode,
not by defaulting to it.

**Return a bare value and log the certificate.** Rejected. A log line is not checkable by a
consumer and is invisible to a type-level audit. The one thing that reliably survives agent
fan-out is a type.

**A flat `Certificate` enum with no claim tether and a public mint (this ADR's original
§2).** Superseded 2026-07-31. It cannot express two requirements that came out of consumer
evaluation: a certificate must not be forgeable (no public constructor) and it must fail
when transplanted onto a different claim (`certifies`). Recorded rather than deleted because
three documents carried three shapes and an agent may find any of them.

**Keeping the shared-reducer claim and calling the difference an implementation detail
(this ADR's original §5).** Rejected. The claim was the sole justification offered for the
fast mode having an internal oracle at all; leaving it standing means lane G3 ships the
library's hottest code with no verdict function, which founding constraint #3 forbids
outright. Retracting it costs one naive dense reference and an inverted verdict order in
one lane brief.

**Bound-driven stopping only, never stabilization.** Rejected as universal: Landau–Mignotte
is often wildly pessimistic and would make factorization pay for Hensel precision it does
not need. Kept as the default *where the bound is tight enough to be cheap*, which includes
resultants.

**A single `groebner` with a `certified: bool` parameter.** Rejected. Two modes with
different performance characteristics, different memory profiles, and different return
certificates are two functions. A boolean parameter hides that from the type system and from
the benchmark harness.

---

## What would reverse this

- **The R2 §8 Q1 corpus showing the geometry workload never leaves the regime where a
  well-implemented ℚ subresultant PRS wins.** That would change *sequencing* — modular
  methods would land later on the geometry path — but not the structure, because consumers
  #12 (SMT NRA) and #27 (medial axis) provably exceed that regime, and because retrofitting
  is a rewrite.
- **Cofactor tracking measuring catastrophically** (say, >20× memory on Katsura-8). Response:
  `groebner_certified` becomes small-instance-only and is documented as an oracle rather
  than an API, and the fast mode's `Probable` becomes the only path at scale. The
  `Certificate` type does not change; what changes is which variants are reachable at which
  sizes.
- **Obtaining the precise Idrees–Pfister–Steidel statement with Noro–Yokoyama's correction
  and finding it applies to resolvent's cases.** That would let the fast path return
  `Proved` without cofactors, which is a strict improvement and does not reverse anything —
  it adds a `ProofKind` variant.
