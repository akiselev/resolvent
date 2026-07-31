# ADR-023 — Certificates are adversarially validated: mutant sets, non-circularity, fleet seeds

**Status:** Ratified 2026-07-31
**Reversibility:** cheap to adopt, one-way in practice — a fifty-row certificate catalogue
retrofitted with mutants after the fact is a rewrite of fifty test modules
**Gates lanes:** every certificate-graded lane, i.e. most of them.
**Evidence:** `docs/research/critique-plan.md` C1 (fatal), C3, C5, C6, C10, C12, C13, C20;
`plans/verification.md` §2, §7.5.

---

## Context

The verification spine's central claim is self-certification: the operation emits data
alongside its answer that *proves* the answer. Roughly fifty such certificates are
catalogued. The lane checklist says: "the operation's row exists, is implemented, and its
certificate is checked in the same test that exercises the operation."

**A certificate is code, and the failure mode of certificate code is not that it rejects a
correct answer — that is loud. It is that it accepts everything — that is silent.** All of
the following compile, go green, and grade nothing:

- a `certifies()` that iterates an empty evidence vector and returns `true`;
- `assert!(prod == f)` where `prod` was computed from the buffer the factorizer left the
  factors in;
- a degree check written as `deg(H) == deg(H)`;
- a specialization check that specializes at a point where the polynomial is constant;
- an "exhaustive over all `(a,b)` for `p < 2^10`" loop whose inner bound is `p` instead of
  `p·p`, silently testing `p` pairs instead of `p²`.

None is exotic. All are what a model writes when it is optimizing for a green suite, which
founding constraint #3 says is exactly the population building this library.

**The plan already knows the rule and applies it to one gate out of fifty.** The license
gate "must *fail* on each of three planted cases … If it does not fail on all three, the
gate is not working", and M0's exit gate restates it: "a gate that has never been observed
to fail is not a gate." That epistemology is correct and it was applied only to licensing.

**Second-order, and why this is fatal rather than untidy.** The triage classifier is defined
as: re-run resolvent's own certificate; if it *also* fails → Class A, resolvent bug, certain;
if it *passes* and the oracle disagrees → Class B, normalization or convention. A vacuous
certificate does not merely fail to catch bugs — it **routes every real bug into Class B**,
where the prescribed response is to fix the adapter or record a convention. The pipeline
metabolizes bugs into documentation.

Three further defects in the catalogue share one root and are fixed by one rule.

**Circular certificates.** The Layer-0 gcd row is `g|a`, `g|b`, `gcd(a/g, b/g) == 1`, marked
**Complete** with an empty "does not prove" cell. `fn gcd(_a,_b) -> Integer { ONE }` passes
all three, the third by calling itself. The Layer-2 row is worse because the document warns
about it specifically: `H|A`, `H|B`, and `deg H == deg gcd(A mod p, B mod p)` — "the degree
half is mandatory" — and the degree half is computed by the routine under test. `H = 1`
gives `deg H = 0` and the modular gcd returns 1. The mathematical argument in the row is
sound; its second premise is a fact about the *true* modular gcd, not the computed one.

**Randomized certificates at a fixed seed.** Several of the strongest rows are
Schwartz–Zippel arguments: "`eval(a·b,x) == eval(a,x)·eval(b,x)` at random points in a large
`GF(p)`", the subresultant specialization property "for random good ring maps", L4 rewrite
soundness. Determinism then requires the default seed to be a fixed checked-in constant and
`prime(i)` to be a pure function of `i`. Both requirements are right; together they mean the
"random points" are **the same points on every run forever**. A failure probability of
`deg/p` is a statement about a draw. When the draw is committed, there is no probability
left — what remains is a golden test at one point, and an error that happens to vanish at
`prime(0)` and `prime(1)` is certified forever.

**Gates whose expected outcome is universal refusal.** The 4-bit-field overflow sweep asserts
that every instance "either completes with the same answer as the wide run or reports
overflow". At 4-bit fields with a guard bit, the total-degree bound is **7**, and every
instance in the Gröbner corpus exceeds it. So the second disjunct is satisfied universally,
the test is green, and it never once exercises a monomial multiply that *succeeds* near the
boundary — which is where a guard-bit off-by-one lives.

---

## Decision

### 1. Every certificate row ships a mutant set

> **Every operation with a certificate ships at least one deliberately wrong implementation,
> committed under `#[cfg(test)]` in the same module, and a test asserting that the
> certificate *rejects* each mutant.** A mutant rejected by the *type system* rather than by
> the certificate does not count: the mutant must compile and must produce a plausible wrong
> value.

This is lane-checklist **item 0**, before everything else. The mutant classes are prescribed
so they are not chosen to be easy — one per failure family the plan already names:

| Mutant class | Applies to | Must be rejected by |
|---|---|---|
| **Coarsening** — merge two outputs into one | factorization, square-free decomposition, root isolation, ideal decomposition | the irreducibility / disjointness / count half |
| **Refining** — split one output into two | factorization, isolation | multiply-back, Sturm count |
| **Off-by-one in a bound** | Landau–Mignotte, Cauchy, separation, Hadamard, arena capacity | the bound's own validity check |
| **Identity** — return an input unchanged | gcd, normalization, reduction | divisibility + degree, idempotence |
| **Trivial constant** — return `1`, `0`, `Unknown`, `Probable`, `Decline` | **everything** | the certificate **and** the sharpness rate (ADR-024) |
| **Sign flip** | resultant, Sturm variation counting, `sign_of` | the cofactor identity, cross-route agreement |
| **Silent wrap** | monomial multiply, `Fp` reduction | guard bits, exhaustive small-`p` |
| **Criterion over-elimination** — drop one extra S-pair class | Gebauer–Möller | the exhaustive S-pair certificate (§4) |
| **Witness-table corruption** — one wrong Miller–Rabin witness | prime registry | the independent sieve cross-check (§5) |

The trivial-constant row is the machine-checkable form of the sharpness argument, and making
it a per-operation obligation is what turns "sharpness gate" from a policy into a test.

### 2. A certificate may not invoke the operation it certifies

> **A certificate may not call the operation it certifies, nor any routine on that
> operation's call graph. Where it must, the row is INV, not CERT.**

Applied to the two gcd rows, this restores completeness *non-circularly*, using the Bézout
witness the adjacent `gcd_ext` row already has:

- **Over ℤ:** `g | a`, `g | b`, **and** `(u, v)` with `u·a + v·b == g`. Complete — any common
  divisor dividing a Bézout combination equal to `g` forces `g` to be *the* gcd — costs one
  multiply-add, and shares no control flow with the gcd routine. The recursive coprimality
  clause is deleted.
- **Over `F[x]`:** identically. Complete over a field.
- **For the ℤ[x] two-part certificate:** the per-prime modular gcd **returns its GF(p)
  Bézout cofactors** — the extended Euclid that computes it already has them, so retaining
  them is free. Then `deg gcd(A mod p, B mod p)` is itself certified, and the ℤ-level
  completeness argument holds against the *computed* modular gcd rather than the ideal one.

The same circularity in smaller doses appears in the `Rational` canonical-form row
(`gcd(num,den)==1` by the gcd under test) and the `UPoly` mul row (`(a·b)/b == a` by the
division under test); the second is mitigated by the independent naive reference and the
evaluation homomorphism, and both rows carry the mitigation explicitly rather than by luck.

### 3. Randomized certificates are graded across the fleet seed schedule

The two uses of randomness are different uses, and separating them resolves the tension
between determinism and Schwartz–Zippel:

1. **Inside the library, at the default seed:** deterministic, as ADR-012 specifies.
   Unchanged.
2. **In the harness:** a certificate whose soundness argument is probabilistic is evaluated
   **across the committed fleet seed schedule**, never at the default seed alone.
3. **Grading rule:** *a row whose "Proves" column relies on a randomized argument is **CERT**
   only when evaluated over the fleet seed schedule; at a single fixed seed it is a golden
   test and is graded **INV**.*
4. **The number of distinct seeds at which each randomized certificate was checked is
   reported alongside the score**, for the same reason generator deletions are reported: a
   silent reduction from 64 seeds to 1 is otherwise invisible and improves every number.

### 4. Gates whose expected outcome is universal refusal are failed sweeps

The overflow sweep becomes a **distribution assertion**, not a disjunction. The wide run
knows each instance's true maximum total degree `D_max`, so the expected partition is
computable:

> For each width `w ∈ {4, 8, 16}` and each corpus instance, the narrow run **must complete
> and match** iff `D_max ≤ 2^(w−1) − 1`, and **must report overflow** otherwise. An instance
> that overflows when it should have completed is a false positive and fails; one that
> completes when it should have overflowed is a silent wrap and fails. CI prints the
> completed/overflowed counts per width, and **a width at which zero instances complete is a
> failed sweep, not a passed one.**

Plus the boundary sub-corpus the generator fleet already specifies — total degree exactly
`D` and exactly `D+1`, exponents exactly at the field max — with the requirement that at
each width both are present and land on **opposite sides**.

The same rule generalizes: **any gate whose green condition can be met by refusing
everything is not a gate.** It is the sharpness argument applied to gates rather than to
APIs, and it is why the S-pair certificate below is specified the way it is.

### 5. Six specific certificate repairs, because they are the ones a lane brief is written from

| Row | Defect | Repair |
|---|---|---|
| **S-pair / Buchberger's criterion** | Vacuous if the verifier reuses Gebauer–Möller. Those criteria are the library's largest single speedup (four orders of magnitude), so they are exactly what an agent reaches for when verification is slow — and then a criteria bug is invisible to the certificate that exists to catch it | The certificate **enumerates all `C(|G|,2)` S-pairs and may not consult any pair-elimination criterion**. A criteria-aware verifier is a separate, explicitly named `*_fast_recheck` used only as a pre-filter. Mutant: a Gebauer–Möller that drops one extra pair class |
| **FGLM** | "Reduces the drl basis to zero and vice versa" proves generation, not the *lex Gröbner basis* property FGLM exists to produce — and reduction modulo a non-Gröbner generating set is not even a well-defined normal form | Add: the lex output satisfies **Buchberger's criterion in the lex order** (per the row above), **and** the lex staircase has exactly `dim_ℚ ℚ[x]/I` standard monomials — a number FGLM already computes and can assert for free |
| **Prime generation** | Deterministic Miller–Rabin witness sets are a Tier-A published result; a transcription error in the table, or a witness set valid to a smaller bound than queried, declares composites prime on a *sparse* set. Every downstream certificate keeps passing: CRT certifies `r ≡ rᵢ (mod pᵢ)` regardless of primality, and rational reconstruction certifies statements about `M`, not about `M`'s factorization. **This is the modular architecture's root of trust and the one assumption with no downstream detector** | Cross-check against an **independent segmented sieve** over a committed window — all primes below `2^24`, plus the first `N` registry entries at each magnitude used (near `2^27`, `2^31`, `2^63`). The registry is index-addressed, so the entries used are known and finite. Commit the count and a hash of the accepted set |
| **CRT combine** | Uniqueness needs pairwise-coprime moduli. A duplicate — an off-by-one in an index-advance loop, a real bug class in an index-addressed registry — passes the congruence check trivially while the effective modulus is smaller than `Π pᵢ`, so the reconstruction bound is wrong and every certificate is green | Assert the moduli multiset is **pairwise distinct** (they are prime by the row above, so distinctness gives coprimality) and that `M = Π pᵢ` is `≥` the bound the caller sized against |
| **Landau–Mignotte / Hadamard / Cauchy bounds** | Referenced by four rows as the thing that makes a modular answer "provably right", and graded nowhere. A too-small bound with Zassenhaus usually yields no valid recombination (benign); with **van Hoeij** it yields a lattice that has not stabilized, spurious 0/1 vectors accepted by the algorithm's own termination witness, and a **coarse factorization that multiplies back correctly** — the named failure, reached through an uncertified input, in the hardest lane | Add a row: for every instance from the **known-factorization generator**, the computed bound is `≥` the true maximum coefficient of any true factor, and `bound / actual` is tracked as a distribution (an astronomically loose bound is valid and makes Hensel lifting unaffordable). The generator already exists, so the true factors are known by construction and the check is free |
| **Factorization over ℤ** | Half 1 proves the product, half 2 proves each factor irreducible; **neither rejects `f = g·g` returned as two multiplicity-1 factors instead of `g²`**. Both pass, the multiplicity data is wrong, and multiplicity is what M5 exists to give the consumer | Assert factors are **pairwise non-associate** after canonical normalization, and assert the exponent multiset against the input degree |

Two smaller items, recorded because a lane brief is written from the table: the resultant
row's Poisson-product check "over a splitting field" requires `GF(p^k)` at best and number
fields at worst, so it is marked **M8; not available at M4** rather than left unlabelled in a
Layer-2 table; and the oracle-independence table gets a mechanical gate (§6).

### 6. Oracle independence is enforced, not audited

The independence table's closing instruction is "this table must be maintained and audited
whenever a shared helper is introduced" — audited by whom, in a project whose premise is
that agents build it and oracles grade it? Two named failures are predictable:

- **Sturm.** Built naive over ℚ on day 7, sharing only `divrem`. Between M2 and M4 someone
  notices Sturm's coefficient growth and "fixes" it by routing it through the Ducos PRS lane
  T1 just landed. At that moment the strongest certificate in Layer 2 becomes a check of a
  component against itself and **no test changes colour**.
- **Buchberger.** G1 exists to grade G2/G3, and F4's symbolic preprocessing, normal-form
  routine and monomial handling are exactly the code an agent factors out and shares.

So: **each oracle module declares, in `lanes.toml` (ADR-021 §3), the set of modules it may
reach transitively. CI walks the module graph and fails on any edge into the lane the oracle
grades.** Same shape as gate L1, one level finer. Additionally, an oracle module carries
`#![doc = "ORACLE: graded lane = U5"]`, and a PR touching both it and the graded lane fails
without an explicit `oracle-independence-reviewed:` trailer naming what was checked — which
catches "I made both sides agree" edits that the import gate cannot.

Lane-checklist **item 11**: if the lane is an oracle for another lane, its permitted-import
set is committed and enforced.

---

## Consequences

- **Every certificate has been observed rejecting a wrong answer.** That is the entire point
  and it is the difference between a self-certifying library and a library that says it is
  one.
- **The triage classifier becomes trustworthy**, which matters more than the direct effect:
  Class A/Class B routing is only as good as the certificate it consults.
- **Cost: roughly one extra test module per operation**, containing a handful of small wrong
  implementations. Real, bounded, and cheapest at the moment the operation is written —
  the mutant is the wrong version the author already had in their head.
- **Some rows are demoted from CERT to INV**, honestly. The separation-bound row, the
  fixed-seed randomized rows before the fleet-seed wiring, the resultant's Poisson check at
  M4. A smaller set of stronger claims is worth more than a larger set of overstated ones.
- **The overflow sweep gets slower**, because it now runs at three widths and must complete
  a real fraction of instances rather than refusing all of them. That cost is paid in the
  nightly tier (ADR-024).
- **A mutant set is itself code that can be wrong** — a mutant that is *not* actually wrong
  makes the test unfalsifiable in the other direction. Mitigation: each mutant carries a
  one-line comment naming the failure class it belongs to and the certificate clause it must
  trip, and a mutant no certificate rejects fails CI as loudly as a bug.

---

## Alternatives considered and why rejected

**Rely on the external differential oracles to catch vacuous certificates.** Rejected. They
catch a wrong *answer*, not a vacuous *check*, and only within the instance range the oracle
can reach — while the triage pipeline routes exactly those disagreements into Class B on the
strength of the vacuous certificate. The failure is self-concealing.

**Mutation-test the whole library with a generic mutation-testing tool.** Attractive, and
rejected as a substitute. A generic mutator flips operators uniformly and mostly produces
mutants that fail loudly for uninteresting reasons; it does not produce *coarsening*,
*trivial-constant* or *criterion-over-elimination* mutants, which are the classes that
matter here and are domain-specific. A generic tool is welcome later as an additional signal;
the prescribed class table is the requirement.

**Require every certificate to be non-circular, full stop, with no INV escape.** Rejected as
unachievable: some invariants genuinely cannot be checked without the machinery they check —
`Rational`'s canonical form needs a gcd. The escape is to *label* the row INV and let the
independent reference and the oracle carry it, not to pretend.

**Vary the seed inside the library so randomized certificates are genuinely probabilistic.**
Rejected outright. It breaks reproducibility, breaks the regression corpus, breaks the
replay guarantee, and breaks the tuning-matrix oracle. The harness varies the seed; the
library does not.

**Keep the overflow sweep as a disjunction and add a note that it should exercise the
success path.** Rejected — that is the shape of every gate that quietly stops testing
anything. A gate whose green condition is computable from the wide run should assert the
computed partition, not a disjunction that one side satisfies universally.

---

## What would reverse this

- **Mutant maintenance becoming a drag on lane throughput** — measurable as mutants updated
  per operation change. Response: reduce the required set to the two classes that apply to a
  given row rather than the whole table; do not drop the requirement, because the first row
  to lose its mutant is the row that was hardest to certify.
- **A row where no plausible compiling mutant exists.** Response: that is evidence the
  certificate is a type-level guarantee rather than a runtime check, which is a *stronger*
  position — record it in the row and move on.
- **The import gate proving unworkable at module granularity** (Rust's module graph is not
  as cleanly extractable as its crate graph). Response: fall back to crate granularity plus
  the `ORACLE:` doc marker and the PR trailer, and accept the weaker guarantee explicitly
  rather than silently.
