# resolvent — verification

**Status:** canonical. **Supersedes `plans/verification.md`**, which remains readable as the
working notes it was.

**Inputs.** `plans/verification.md` (the superseded draft), `plans/architecture.md`,
`plans/roadmap.md`, `API.md` (canonical for public signatures),
`docs/decisions/ADR-001…020`, `docs/research/{prior-art-and-licensing,
consumer-requirements,algorithms-and-representation,consumer-sinbad,consumer-cadabra2,
consumer-solverang}.md`, and the two adversarial critiques
`docs/research/critique-engineering.md` (C1) and `docs/research/critique-plan.md` (C2).

**The critiques are authoritative.** Where the superseded draft and a critique disagree,
this document states the corrected position and says so in the row. §14 is the register of
what changed and why, so nobody has to diff two long documents to find out.

**Precedence, stated once so it is not relitigated.** For a *public signature*, `API.md`
wins. For an *internal decision* (representation, coefficient domain, arena ownership,
generics boundary), the ADR wins. For a *verdict function* — what proves what, what
grades what, what is green — **this document wins**, and an ADR that asserts a
verification property this document contradicts is a proposed amendment to this document,
not a binding statement. C1 §2 and C2 §2 both found that two documents each claiming
supremacy produced eleven live signature-level contradictions; the fix is a precedence
rule plus the mechanical gate in §11.1 (`docs-consistency`), not more prose.

---

## 1. The thesis

resolvent is built primarily by AI agents graded by oracles, not by a human team reading
diffs (founding constraint #3). That only works if correctness is **certified rather than
reviewed**: every operation emits, alongside its answer, data that makes the answer
checkable by code that did not compute it — a factorization multiplies back and each
factor carries an irreducibility witness where one exists; a gcd carries a Bézout pair; a
Gröbner basis carries the cofactors that express each element in terms of the input
generators; an isolation carries the sign-variation count that established "exactly one
root here". The check is cheaper than the computation and it is run in the same test that
exercises the operation. **But a certificate is code**, written by the same agent, in the
same commit, as the thing it grades — and the failure mode of certificate code is not
rejecting a correct answer (loud), it is accepting everything (silent). So the thesis has
a second half without which the first is decoration: **every certificate is itself
falsified before it is trusted**, by a committed set of deliberately wrong implementations
it must reject (§2.1). The catalogue of what is certified (§3) is the smaller half of this
document's value. The larger half is §4 and §5 — where certificates run out, and where
code passes every gate and is still wrong.

---

## 2. The four rules that make a certificate a certificate

These are prior to the catalogue. A row in §3 that violates any of them is not a CERT row
regardless of what its "Proves" column says.

### 2.1 Rule M — every certificate ships a mutant set

> **Every row in §3 ships a *mutant set*: at least one deliberately wrong implementation of
> the operation, committed under `#[cfg(test)]` in the same module, plus a test asserting
> that the certificate **rejects** each mutant. A mutant that fails to compile, or that is
> rejected by the type system rather than by the certificate, does not count. The mutant
> must compile and must produce a plausible wrong value.**

This is C2 §1, and it is the single largest hole the critiques found. `plans/verification.md`
§7.1 already required the *license* gate to be observed failing on three planted cases —
"a gate that passes what it must reject is not a gate" — and applied that epistemology to
exactly one gate out of roughly fifty. All of the following pass CI and grade nothing: a
`certifies()` that iterates an empty evidence vector and returns `true`; `assert!(prod == f)`
where `prod` was computed by multiplying the factors in the buffer the factorizer left them
in; a degree check written `deg(H) == deg(H)`; a specialization check evaluated at a point
where the polynomial is constant; an "exhaustive over all `(a,b)` for `p < 2^10`" loop whose
inner bound is `p` instead of `p²`.

**Second-order consequence, and it is why this is prior to everything else.** §6.5's triage
classifier routes a disagreement by re-running resolvent's own certificate: certificate also
fails → Class A (resolvent bug, certain); certificate passes → Class B (normalization,
convention, oracle limitation). A vacuous certificate does not merely fail to catch bugs, it
**routes every real bug into Class B**, where the prescribed response is to fix the adapter
or record a convention. Without rule M the triage pipeline metabolizes bugs into ADRs.

**Prescribed mutant classes**, so they are not chosen to be easy. One per failure family
this document names elsewhere:

| Mutant class | Applies to | Must be rejected by |
|---|---|---|
| **Coarsening** — merge two outputs into one | factorization, square-free decomposition, isolation, ideal decomposition, primary decomposition | the irreducibility / disjointness / count half |
| **Refining** — split one output into two | factorization, isolation, square-free | multiply-back / Sturm count / `Π fᵢ^i == f` |
| **Off-by-one in a bound** | Landau–Mignotte, Cauchy, Hadamard, Mignotte–Davenport separation, monomial capacity | the bound's own validity check (§3.4 bound row) |
| **Identity** — return an input unchanged | gcd, normalization, reduction, refinement | divisibility + degree + Bézout, idempotence |
| **Trivial constant** — return `1`, `0`, `Unknown`, `Probable`, `Decline`, the empty list | *everything* | the certificate **and** the §4.16 sharpness rate |
| **Sign flip** | resultant, Sturm variation counting, `sign_of`, Descartes | the cofactor identity, cross-route agreement |
| **Silent wrap** | monomial multiply, `Fp` reduction, exponent packing | guard bits, exhaustive small-`p` |
| **Criterion over-elimination** | Gebauer–Möller pair elimination, divmask filter, bad-prime rejection | the criteria-free verifier (§3.5), the one-sided filter law |
| **Schedule** — assign ids/accumulate in completion order | interning, parallel reduction, CRT accumulation | the thread-count determinism matrix (§11) |

The trivial-constant row is the machine-checkable form of the sharpness argument (§4.16),
and making it a per-operation obligation is what turns "sharpness gate" from a policy into
a test.

### 2.2 Rule C — a certificate may not invoke what it certifies

> **A certificate may not call the operation it certifies, nor any routine on that
> operation's call graph. Where it must, the row is INV, not CERT.**

This is C2 §3, and it demolished two rows the superseded draft marked "Complete" with an
empty "does not prove" cell. The Layer-0 gcd certificate was `g|a`, `g|b`, and
`gcd(a/g, b/g) == 1` — and `fn gcd(_a,_b) -> Integer { Integer::ONE }` satisfies all three,
because the coprimality clause is evaluated by the function under test. The Layer-2
univariate gcd certificate was worse, because the draft specifically warned that the degree
half is the one people forget: `H = 1` gives `deg H = 0`, and the GF(p) gcd — the same
modular gcd whose ℤ lift is being certified — returns `1`, giving
`deg gcd(A mod p, B mod p) = 0`. Both green. Both marked Complete. The mathematical
argument in the row was correct; its second premise was a fact about the *true* modular gcd,
not the computed one.

Both are repaired in §3 with Bézout witnesses, which share no control flow with the gcd
routine's search and cost one multiply-add to check.

### 2.3 Rule S — a randomized certificate is graded across seeds, never at one

Several of the strongest rows are Schwartz–Zippel arguments: `eval(a·b, x) == eval(a,x)·eval(b,x)`
at random points in a large `GF(p)`; the subresultant specialization property at random
good ring maps; L4 rewrite soundness at random points. Meanwhile ADR-012 §2–§3 requires the
library to be deterministic — the default seed is a fixed checked-in constant and
`prime(i)` is a pure function of `i`. Both requirements are right. Together they mean the
"random points" are the same points on every run of every CI job forever, and a failure
probability of `deg/p` is a statement about a *draw*. When the draw is fixed and committed
there is no probability left; what remains is a golden test at one point (C2 §5).

> **A row whose "Proves" column rests on a randomized argument is CERT only when evaluated
> over the fleet seed schedule (§9). At a single fixed seed it is a golden test and is
> graded INV.** The number of distinct seeds at which each randomized certificate was
> checked is reported alongside the score, for the same reason generator deletions are
> reported: a silent reduction from 64 seeds to 1 is otherwise invisible and improves every
> number.

Inside the library, at the default seed, nothing changes: determinism is unaffected. The
harness draws from a different seed source than `Session::default()`.

### 2.4 Rule F — a fail-closed verdict is trivially satisfiable and therefore is not a verdict

An implementation that returns `Unknown` always, or `Probable` always, or declines on budget
always, passes every soundness certificate in §3. Every three-valued or two-tier output
therefore ships a **sharpness rate** with a committed numeric ceiling, ratcheted per §4.16.
The superseded draft stated this and it was the best paragraph in it; what it did not do is
make any ceiling a number, so Gate 1's "compared against committed ceilings" and M3's exit
gate "the Unknown-rate ceiling met" were evaluated against ceilings that did not exist
(C2 §8). §4.16 supplies the ratchet mechanism.

### 2.5 The verdict vocabulary

Every lane in `plans/roadmap.md` carries exactly one *primary* tag. The tags decide
scheduling, fan-out width, and what "done" means.

| Kind | Definition | Convergence | Fan-out |
|---|---|---|---|
| **CERT** | Emits data that *proves* the answer; checking is strictly cheaper than recomputing; obeys rules M and C. A failure is a resolvent bug with certainty. | Days. Monotone. | Wide. |
| **INV** | No emitted proof; the answer must satisfy structural invariants checkable without a second implementation. All invariants can hold on a wrong answer. | Days. | Safe, never sufficient alone. |
| **DIFF** | Graded by disagreement with an independent implementation, internal or external. Disagreement is a *signal* needing triage (§6.5), not a verdict. | Weeks. | Safe once adapter + normalization exist. |
| **PROP** | Graded by property tests over generated inputs. Failure is a bug; success is evidence. | Weeks; asymptotic. | Safe. |
| **SCORE** | The criterion is *a number to optimize*, not a certificate to check. | **Months. Non-monotone. No completion condition.** | **Unsafe.** Needs a pinned machine and a frozen baseline first. |

Two non-negotiable consequences:

1. **Self-certification runs first and is the primary gate; oracles are secondary**
   (ADR-016 §3). A self-certificate failure is a resolvent bug with certainty; an oracle
   disagreement may be a convention.
2. **A SCORE lane may not start until the CERT/INV reference it is graded against exists
   and is frozen.** Sturm before Descartes; Descartes before ANewDsc; Buchberger before F4;
   Zassenhaus before van Hoeij; Ducos before the modular resultant; the rational-witness
   fiber oracle before curve analysis. CI-enforced: **a SCORE lane's CI job does not exist
   until its oracle lane is green and frozen.**

### 2.6 Certainty is in the type

Per `API.md` §5 (canonical; ADR-010 §2 is amended to match — the three incompatible
`Certificate` shapes C1 §16 found are resolved in favour of the claim-tethered one):

```rust
pub struct Certified<T> { pub value: T, pub certainty: Certainty }
pub enum Certainty { Proved(ProofKind), Probable(ProbableReason) }
pub enum ProofKind {
    BoundDriven { bound_bits: u64, primes_used: u32 },
    DivisibilityAndDegree, CofactorRepresentation,
    Identity, Enclosure, RootCount,
}
pub struct Certificate<C: Claim> { /* private fields; no public mint */ }
// pub fn claim(&self) -> &C; pub fn evidence(&self) -> &C::Evidence;
// pub fn certifies(&self, claim: &C) -> bool;   // structural tether
// pub fn verify(&self, budget: Budget) -> Result<(), Error>;
```

Three verification obligations follow, all of them tests:

- **Unforgeable.** A compile-fail test asserts no public constructor exists on any
  certificate type. This is a decision that can silently regress under a refactor.
- **Tethered.** A transplanted certificate must fail `certifies` rather than riding along.
  Test: mint a certificate for claim `A`, present it against claim `B`, assert rejection.
  (Shape lifted from the consumer's own mechanism at
  `cadabra2/crates/cadabra-check/src/certificate.rs:41-42`.)
- **Excluded from canonical bytes.** `Certificate`, `ProbableReason` (which carries
  `primes_used`, `rounds`), `Telemetry`, and `TraceEvent::BudgetTick` are **not** part of
  the canonical serialization. Only the mathematical value is. ADR-012 §8 asserts
  value-equality across a `Tuning` matrix while the modular batch width `N` is a tuning knob
  that changes `primes_used`; if evidence were serialized that gate would fail on its first
  run (C1 §6, §16). ADR-012 §9's list covers polynomials, bases and algebraic numbers and is
  silent on certificates; this sentence closes it.

The rate at which each lane returns `Proved` on its corpus is a tracked number with a
committed floor (§4.16). Tiering makes `Probable` cheap to reach, which is what makes the
floor load-bearing rather than decorative.

---

## 3. The certificate catalogue

For each operation: the certificate, what checking it proves, **what it does not prove**,
the cost of checking, and the kind. The fourth column is the reason this table exists.

Cost convention: `O(1)×` = a small constant fraction of the operation; `~1×` = comparable to
recomputing; `>1×` = the check dominates.

### 3.1 Layer 0 — coefficient rings

| Operation | Certificate | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| `Integer` add/sub/mul | Inverse-op round-trip; differential vs `rug::Integer` in a `publish = false` crate (§6.1) | Value correctness on the tested instance | Nothing about untested magnitudes. Carry bugs cluster at word boundaries; the generator must target 1-word, just-over-1-word, and limb-boundary operands or this row is vacuous | `O(1)×` | CERT |
| `Integer` exact division `a/b` | `q·b == a` | Exactness | That non-exact input was *rejected* — needs a separate `divides` negative test | `O(1)×` | CERT |
| `Integer` `divrem` | `a == q·b + r`, `0 ≤ r < \|b\|` | Full correctness (the pair is unique) | — | `O(1)×` | CERT |
| **`gcd(a,b) = g`** | `g\|a`, `g\|b`, **and a Bézout pair `(u,v)` with `u·a + v·b == g`** | **Complete, non-circularly**: any common divisor divides `u·a+v·b = g`, forcing `g` to be *the* gcd | — | `O(1)×` (two divisions, one multiply-add) | CERT |
| — correction | The superseded row's third clause was `gcd(a/g, b/g) == 1`, evaluated by the function under test. `fn gcd(_,_) { ONE }` passed it and was marked "Complete" (C2 §3). Rule C now forbids that shape everywhere | | | | |
| `gcd_ext → (g,u,v)` | The row above, plus minimality `\|u\| ≤ \|b/2g\|`, `\|v\| ≤ \|a/2g\|` | Full correctness including minimality | — | `O(1)×` | CERT |
| `Rational` arithmetic | Canonical-form invariant `gcd(num,den)==1` (checked by the **certified** gcd of the row above, not by an independent reimplementation), `den > 0`; field axioms by PROP | Representation validity | Value correctness — needs round-trip + oracle | `O(1)×` | CERT+PROP |
| `Rational::from_f64` | `to_f64(from_f64(x)) == x` bit-exactly for every finite `x`; `None` for non-finite | Lossless lift, fail-closed | — | `O(1)×` | CERT |
| `Fp` arithmetic, word primes | **Exhaustive** over all `p²` pairs `(a,b)` for every prime `p < 2^10` against an `i128` reference; random against `i128` for `p < 2^63`; `a·a⁻¹ == 1` for every unit | Complete for small `p`; strong for large | Nothing about the bulk/SIMD path, which gets its own row. **Mutant required:** the inner loop bounded by `p` instead of `p²` must be rejected by a committed pair-count assertion | small `p`: exhaustive; large: `O(1)×` | CERT |
| `Fp` bulk/vector ops | Componentwise agreement with the certified scalar path on random vectors including tails, misaligned lengths, and length 0/1 | Equivalence to the certified scalar path | Nothing about a shared `mulmod` bug — both sides call it (§4.17 common mode) | `O(1)×` | CERT |
| Batched tuple ring `Fp4` — arithmetic | Componentwise agreement with 4 independent scalar `Fp` runs | Exact arithmetic equivalence — a free, complete oracle **for arithmetic** | **Nothing about lane divergence.** A zero pivot in one lane and lead-monomial divergence across lanes are *control-flow* failures this oracle cannot see (C1 §14; §4.14) | `~1×` | CERT |
| Batched tuple ring — lane faults | `inv_batch(&self) -> Result<Self, LaneMask>` where bit `i` means lane `i` is non-invertible; corpus instances constructed so exactly one lane's pivot vanishes, asserting the batch splits and the offending prime index reaches the `Trace` | Lane-fault detection and the split path | — | `O(1)×` | CERT |
| `Zn`, composite modulus | `Fp` row plus explicit unit/zero-divisor classification: `is_unit(a) ⇔ gcd(a,n)==1` | Correctness | — | `O(1)×` | CERT |
| — scope note | ℤ/n for composite `n` is **in** scope, contra the superseded api-shape L0-12. Hensel lifting to `p^k` (M5, lane K2) *is* arithmetic modulo a composite, and M1's exit gate requires `Zn` (C2 §2) | | | | |
| `GF(p^k) = Fp[x]/(f)` | Field axioms; Frobenius closure `x^(p^k) == x` for every element; modulus irreducibility certified by §3.4's finite-field test | Field structure | — | `~1×` | CERT |
| **Prime registry** | Miller–Rabin with the published deterministic witness sets for `n < 2^64`, **cross-checked against an independent segmented sieve** over a committed window (all primes below `2^24`, plus the first `N` entries at every magnitude class actually used: near `2^27`, `2^31`, `2^63`), with the accepted set's count and SHA-256 committed as a golden file | Primality of the entries actually consumed | Nothing outside the committed window. **Mutant required:** one corrupted witness-table entry, which the sieve cross-check must reject | one-time; `O(1)×` per run | CERT |
| — why this row grew | A composite in the registry is invisible to every downstream certificate: CRT's congruence check and rational reconstruction's bound check are statements about `M`, not about `M`'s factorization, and both stay green (C2 §12). This is the modular architecture's root of trust and it had no detector | | | | |
| "Good prime" predicate | Algorithm-specific and directly checkable: `p ∤ lc`, degree preserved, `p ∤ disc` | The stated condition only | That the prime is **lucky**. Unluckiness is a separate hazard (§4.5) | `O(1)×` | CERT |
| **CRT combine** | Result `≡ rᵢ (mod pᵢ)` for every `i`; result in the symmetric range; **moduli asserted pairwise distinct**; **`M = Π pᵢ` asserted `≥` the bound the caller sized against** | Full correctness of the combination | Nothing about whether enough primes were used *for the mathematical claim* (§4.5) | `O(1)×` | CERT |
| — correction | Uniqueness requires pairwise-coprime moduli. A duplicated index — a real bug class in an index-addressed registry — passes the congruence check trivially while the effective modulus is smaller than `Π pᵢ`, so the reconstruction bound is wrong with every certificate green (C2 §20a) | | | | |
| Rational reconstruction `→ n/d` | `n ≡ d·a (mod M)`, `gcd(n,d)==1` (certified gcd), `\|n\|,\|d\| ≤ √(M/2)` | Uniqueness of the reconstruction *given that one exists* | That `n/d` is the **intended** answer — that needs the modulus bound or a verification step | `O(1)×` | CERT |
| `Reducible::reduce` | `reduce(&self, m) -> Result<Self::Image, BadPrime>`; round-trip on the residue class; `BadPrime` returned rather than a silent zero-divisor | Reduction correctness and honest refusal | — | `O(1)×` | CERT |
| — correction | `Reducible::Image: Field` is false. Over ℚ(α) reduction lands in `GF(p)[x]/(f mod p)`, a field only at inert primes — and for the multiquadratic towers geometry produces (ℚ(√2,√3), Galois group `(ℤ/2)²`, no 4-cycle) **no prime is inert**, so the bound has no valid implementation at all. `Image: CommutativeRing` and a `BadPrime` error are the corrected shape (C1 §4; §4.13) | | | | |

### 3.2 Layer 1 — representation and polynomial arithmetic

| Operation | Certificate | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| Monomial encode/decode | `decode(encode(v)) == v` for random `v` **and for `v` exactly at, one below, and one above the capacity bound** | Round-trip fidelity including the boundary | — | `O(1)×` | CERT |
| Monomial compare | Order axioms as PROP — totality, antisymmetry, transitivity, well-ordering on ℕⁿ, and **multiplicative compatibility** `a < b ⇒ a·u < b·u` — plus agreement with a naive `Vec<u32>` comparator per supported order | Order correctness | — | `~1×` vs naive | CERT+PROP |
| Monomial multiply | `decode(a ⊗ b) == decode(a) + decode(b)` elementwise **or** the multiply returned `Err`; never a third outcome. Guard-bit detection compiled into release, not `debug_assert` | Arithmetic correctness and total overflow detection | — | `O(1)×` | CERT |
| **Divisibility / lcm / gcd / degree** | Computed from `raw` (raw packed exponents, order-free) and agreeing with the naive `Vec<u32>` routine; asserted **identical across every `Order`** on the same monomials | Order-independence, which is the property that keeps the branch out of the hottest loop in the library | — | `O(1)×` | CERT |
| — correction | ADR-009 lists "an order-specific divisibility direction" as one of three O(1) order-specific sites "all outside sort inner loops". Divisibility is *the* inner loop of symbolic preprocessing and reducer selection — ADR-008 ranks the divisor-query index at 10–20× precisely because of call volume — and an order-dependent branch there violates ADR-006's "at most one runtime `match` per call, never per element" verbatim. The fix is already in the design: `raw` is order-free (C1 §10). Order-specific work is **two** places: encode, and the constant subtract on multiply | | | | |
| `divmask` filter | One-sided law: mask says "not divisible" ⇒ genuinely not divisible, verified against exact divisibility on generated pairs. False positives permitted | Soundness of the negative answer | **Nothing about filter quality.** A mask that always says "maybe" is sound and useless — sharpness gate: false-positive rate with a committed ceiling | `O(1)×` | CERT+SCORE |
| Monomial identity (interned ids **or** inline packed keys) | Injectivity: equal exponent vectors ⇔ equal key/id; hash multiplicativity `h(u) + h(v) == h(u·v)`; **ids are a pure function of content**, asserted by building the same ring's arena at `RAYON_NUM_THREADS ∈ {1,8}` and comparing the full id assignment | Term-identity consistency and schedule-independence | — | `O(1)×` | CERT |
| — correction | An interner is a shared mutable accumulator, which ADR-012 §5 bans, and symbolic preprocessing is nothing but interning. First-encounter-order ids under parallel interning depend on thread arrival, and ADR-012 §4 says out loud that tie-breaks consult id order. Corrected: **ids are content-derived** (deterministic hash of the packed key with a fixed collision-resolution order), so parallel interning is deterministic; **and no tie-break anywhere may consult `MonomialId` ordering** — tie-break on the key, which is content-derived and totally ordered. A mutant that tie-breaks on id must be rejected by the thread matrix (C1 §5) | | | | |
| Monomial arena capacity | `MonomialId` exhaustion returns `Unsupported::MonomialArenaFull { capacity }`, never an index panic; `Ring::arena_stats()` asserted against a committed ceiling on the largest corpus instance | No-panic conformance and a measured memory model | That the arena's monotone growth is affordable at instances larger than the corpus | `O(1)×` | CERT |
| `UPoly` add/sub/mul | `(a·b)/b == a`; `deg(a·b) == deg a + deg b` over an integral domain; agreement with a naive `O(n²)` reference **in the same crate**; evaluation homomorphism `eval(a·b, x) == eval(a,x)·eval(b,x)` at points drawn from the **fleet seed schedule** | Strong. The naive reference carries the weight; the evaluation check is Schwartz–Zippel with failure probability `deg/p` per seed | The `(a·b)/b == a` clause alone is circular under rule C (it invokes division). It is retained as INV; the CERT strength comes from the naive reference and the multi-seed evaluation | `O(1)×`; naive reference `~1×` at small degree | CERT+INV |
| `MPoly` arithmetic | The `UPoly` row, plus representation invariants: terms strictly descending in the ring's order, no zero coefficients, no duplicate monomials | Correctness + canonical storage | — | `O(1)×` | CERT |
| Kronecker substitution | Round-trip against direct multiplication on the same inputs | Equivalence | — | `~1×` | CERT |
| Taylor shift `p(x+a)` | `shift(shift(p,a), −a) == p`; `eval(shift(p,a), x) == eval(p, x+a)` at multi-seed random `x` | Correctness | — | `O(1)×` | CERT |
| Content / primitive part | `p == content(p) · primitive(p)`; `content(primitive(p)) == 1` | Full correctness | — | `O(1)×` | CERT |
| Associate normalization | Idempotence `norm(norm(p)) == norm(p)`; and `p ~ q ⇔ norm(p) == norm(q)` against an independent associate test | Canonicity — this is what lets a consumer's curve-equality test be an `==`, replacing the hand-rolled all-2×2-minors check at `arrangements/crates/arrangements/src/geoms/conics.rs:259-270` | — | `~1×` | CERT |
| `divrem` | `a == q·b + r`, `deg r < deg b` | Full correctness | — | `O(1)×` | CERT |
| Pseudo-division | `lc(b)^(deg a − deg b + 1) · a == q·b + r`, `deg r < deg b` | Full correctness | — | `O(1)×` | CERT |
| `RecursiveView` | Every operation agrees with the same operation on a materialized recursive copy | View consistency | — | `~1×` | CERT |
| `map_coefficients` (ring hom) | `φ(a) ⊕ φ(b) == φ(a + b)` and the multiplicative analogue on generated pairs; `eval` is same-ring by type | Homomorphism laws | — | `O(1)×` | CERT |

### 3.3 Layer 2 — gcd, resultants, isolation

| Operation | Certificate | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| **`gcd(A,B) = H` over `F[x]`** | `H\|A`, `H\|B` by exact division, **and `(u,v)` with `u·A + v·B == H`**, `deg u < deg B − deg H`, `deg v < deg A − deg H` | **Complete over a field**, non-circularly | — | `O(1)×` | CERT |
| **`gcd(A,B) = H` over ℤ[x]** | (a) `H\|A`, `H\|B` by exact division; (b) `content(H) == gcd(content A, content B)` by the Layer-0 certified gcd; (c) `deg H == deg gcd(A mod p, B mod p)` for a certified-good prime `p`, **where the modular gcd returns its GF(p) Bézout cofactors and is therefore itself certified by the row above** | **Complete.** (a)+(b) give `H \| G`; (c) against a *certified* modular gcd gives `deg H ≥ deg G` | Nothing, once (c)'s input is certified. Bézout does not exist in ℤ[x], which is why the modular route carries the completeness argument here | `O(1)×` — two divisions plus one certified modular gcd | CERT |
| — correction | The superseded row's premise "`deg gcd(A mod p, B mod p) ≥ deg G`" was a fact about the *true* modular gcd. Retaining the extended-Euclid cofactors that the modular gcd already computes is free and makes the argument hold against the *computed* one (C2 §3) | | | | |
| Square-free decomposition (Yun) | `Π fᵢ^i == f`; the `fᵢ` pairwise coprime; each `gcd(fᵢ, fᵢ') == 1`; **the `fᵢ` pairwise non-associate** | **Complete** | — | `~1×` | CERT |
| **Resultant `Res(f,g)`** | (a) cofactors `u,v` with `u·f + v·g == Res` from the subresultant PRS; (b) degree bound `deg_x Res_y ≤ deg_y(f)·deg_x(g) + deg_y(g)·deg_x(f)`; (c) `Res == 0 ⇔ deg gcd(f,g) > 0`, cross-checked against the gcd lane; (d) three-route agreement (§6.4) | Very strong. (a) proves `Res ∈ ⟨f,g⟩`; (b)+(c) bound and locate it; (d) is the completeness argument | **(a) alone does not prove the value is *the* resultant** — any ideal element passes. The Poisson-product route `Res = lc(f)^{deg g} Π g(αᵢ)` is **M8, not available at M4**: it needs a splitting field, i.e. `GF(p^k)` at best and number-field arithmetic at worst (C2 §20c) | (a) `O(1)×`, (b) free, (c) `~1×`, (d) `~2×` | CERT+INV |
| Resultant degenerate inputs | The pinned convention for vanishing leading coefficient, degree drop, constant argument, and zero argument, asserted identically across all three routes | Convention agreement | Nothing mathematical. **Pin the convention in an ADR before T1/T2/T3 start** — sources genuinely differ (0, `lc^k`, 1), the fleet contains degree-drop adversarials, and an unpinned convention produces a permanent triage queue (C2 §20) | free | INV |
| Subresultant chain | Each `Sᵢ ∈ ⟨f,g⟩` with cofactors; **specialization property** — the chain of `Ψ(f), Ψ(g)` equals `Ψ(chain)` for good ring maps `Ψ` drawn from the fleet seed schedule; valid subresultant degree sequence; last nonzero element a gcd up to content | Very strong; the specialization check is a randomized proof **across seeds** | Bad specializations must be excluded first or the check is vacuous. **Mutant required:** a specialization chosen where the polynomial is constant, which must be rejected as a degenerate witness | `O(1)×` per specialization | CERT |
| **Real root isolation** | Per returned interval: the **retained Descartes/VCA sign-variation count** (exactly 1) as the witness, `ProofKind::RootCount`; intervals pairwise disjoint and ordered; `f(lo) ≠ 0 ≠ f(hi)`; all intervals inside the Cauchy bound; Σ multiplicities equals the degree of the square-free-corrected input; round-trip from `f = Π(x − rᵢ)`; and, at oracle tier, Sturm's **exact** distinct-root count per interval | Complete for correctness at any degree Sturm can reach; Sturm's count is the strongest single check in Layer 2 | **The interval is the conclusion, not the evidence.** A consumer handed `Vec<IsolatedRoot>` and nothing else cannot check an isolation at all — it must redo it. Retaining the variation witness is *not free* and moves this row from tier F to tier C with the constant documented (`API.md` §5.3). Nothing about interval *quality* (§4.16); nothing at degrees where Sturm is unaffordable (§4.18) | witness `O(1)×`; Sturm `>1×` at high degree, which is why it is an oracle and not the production path | CERT |
| Separation bound | Validity: for every corpus pair `\|α − β\| ≥ bound`; plus a symbolic unit test against brute-force certified separations at degree ≤ 6, where the true separation is computable; plus the citation in the module's `Derivation:` line | Validity on the corpus and on the small-degree class | **Not validity in general** — a finite check of a universally quantified claim, and the corpus is least likely to contain by chance the near-degenerate inputs where an off-by-one bites. Tightness is a separate sharpness number: a bound of 0 is valid and useless | `~1×` | **INV+PROP** (downgraded from CERT) |
| — invariant | **No `Equal` verdict is ever produced by exhausting the separation bound.** `Equal` comes only from the gcd-plus-sign-change certificate (ADR-013 §5). The bound's sole role is to bound the refinement rounds before the *inequality* branch is guaranteed to have separated; reaching it with neither a certificate nor a separation is an internal-invariant failure, not an answer. The other reading is silently wrong and *transitively* wrong — a systematically over-large bound collapses equality consistently, so the transitivity property cannot catch it (C2 §11) | | | | |
| Bernstein range enclosure | (a) endpoint Bernstein coefficients equal endpoint values exactly; (b) the coefficient hull contains the true range, checked by isolating the roots of `p − c` at the hull bounds; (c) a `Certain(s)` verdict never contradicts the true sign | Soundness | **Nothing about the Unknown rate.** The fail-closed direction is free — the consumer's own implementation returns `Unknown` rather than guessing (`arrangements/crates/lazy-exact/src/bernstein.rs:135-152`). Sharpness gate mandatory, with `0` Unknowns on the clear-sign sub-corpus | `O(1)×` | CERT+SCORE |

### 3.4 Layer 2 — factorization

| Operation | Certificate | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| Factorization over `GF(p)` | (a) multiply back; (b) **complete irreducibility test** per factor: `f` of degree `d` is irreducible over `GF(p)` iff `x^(p^d) ≡ x (mod f)` and `gcd(x^(p^(d/q)) − x, f) == 1` for every prime `q \| d`; (c) factors pairwise non-associate | **Complete.** Both halves decidable and cheap over a finite field | — | `O(1)×` | CERT |
| **Landau–Mignotte / Hadamard / Cauchy bounds** | For every instance from the **known-factorization** generator, the computed bound is `≥` the true maximum coefficient of any true factor; the ratio `bound / actual` tracked as a distribution | Validity on the generated class, and sharpness | **Validity outside that class** — the derivation must be cited and unit-tested symbolically. **Mutant required:** off-by-one in the exponent | `O(1)×` (the true factors are known by construction) | CERT+SCORE |
| — why this row is new | The superseded draft deferred bound sufficiency to "a separate assertion" and then never graded it anywhere: not in the catalogue, not in the danger list, not in M5's exit gate, not in the lane checklist — while §3.4 leaned on bounds for its strongest determinism claim. The failure direction matters: a too-small bound with **van Hoeij** produces a lattice that has not stabilized, spurious 0/1 vectors are accepted by the algorithm's own termination witness, and the output is a coarse factorization that multiplies back correctly (C2 §13) | | | | |
| Hensel lifting `p → p^k` | `Π gᵢ ≡ f (mod p^k)` exactly; each `gᵢ ≡` its `mod p` original; the `gᵢ` pairwise coprime mod `p` | Complete for the lift | Whether `k` was large enough for recombination — that is the bound row above, and it is now graded | `O(1)×` | CERT |
| Factorization over ℤ — **half 1** | Multiply the factors back and compare | The factorization *is* a factorization | **Irreducibility — not at all.** A recombination bug merging two true factors produces `f = g·h` with `g` reducible and this passes. An oracle that only multiplies back silently accepts a coarse factorization | `O(1)×` | CERT |
| Factorization over ℤ — **half 2** | Exhibit `p` with `p ∤ lc(fᵢ)`, `p ∤ disc(fᵢ)` such that `fᵢ mod p` is irreducible of degree `deg fᵢ` | Irreducibility over ℚ, when such a `p` exists | **The certificate does not always exist.** Polynomials whose Galois group contains no `n`-cycle — Swinnerton–Dyer canonically — factor nontrivially modulo *every* prime (§4.2) | one modular factorization per factor | PARTIAL |
| Factorization over ℤ — **half 3** | Factors pairwise **non-associate** after canonical normalization; the exponent multiset asserted against the input degree | Multiplicity correctness | — | `O(1)×` | CERT |
| — why this row is new | Halves 1 and 2 both pass on `f = g·g` returned as two multiplicity-1 factors instead of `g²`. Multiplicity is precisely what M5 exists to give the consumer (C2 §20b) | | | | |
| LLL reduction | Lovász condition and size-reduction condition on the output basis; `det` preservation; "output lattice == input lattice" via unimodularity of the transform | **Complete** — LLL's output conditions are directly checkable, which makes it a much better agent lane than it looks | Nothing about quality beyond the LLL guarantee | `O(1)×` | CERT |
| van Hoeij recombination | (a) half 1; (b) agreement with Zassenhaus for `r ≤ 20`; (c) the termination witness — the reduced basis consists of 0/1 vectors partitioning `{1..r}` **at a Hensel precision that is itself certified by the bound row above** | Strong, not complete | See §4.2. (c) is conditional on the precision, which was the uncertified input (C2 §13) | (b) `>1×`, feasible only for small `r` | PARTIAL |

### 3.5 Layer 2 — Gröbner

| Operation | Certificate | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| `groebner_certified(F) → (G, cofactors)` — **`I ⊆ ⟨G⟩`** | Every `f ∈ F` reduces to zero modulo `G` | The easy inclusion | — | `\|F\|` normal forms over ℚ with full coefficient blowup | CERT |
| — **`G` is a Gröbner basis** | Every S-pair of `G` reduces to zero modulo `G`. **The verifier enumerates all `C(\|G\|,2)` pairs and may not consult any pair-elimination criterion.** A criteria-aware verifier is a separate, explicitly named `*_fast_recheck` usable only as a pre-filter | The basis property | — | ≈ recomputing the basis | CERT |
| — why the clause | Gebauer–Möller is the largest single speedup in the library (ADR-008 reports `yang1`: 1,998,099,720 pairs generated, 148,812 surviving) and is exactly the code an agent reaches for when verification is slow. If the certificate applies the criteria, a bug that drops a non-redundant pair is invisible and the object is not a Gröbner basis while passing "Buchberger's criterion". **Mutant required:** a Gebauer–Möller variant that drops one extra pair class (C2 §10a) | | | | |
| — **`⟨G⟩ ⊆ I`** | Stored cofactors: each `gⱼ = Σᵢ hᵢⱼ fᵢ`, checked by multiplication and addition **over ℚ** | The hard inclusion — the only cheap general route | — | check `O(1)×`; *producing and reconstructing* the cofactors is the expensive part (§4.4) | CERT |
| `groebner(F)` — fast modular mode | Agreement with `groebner_certified` on every regression instance; lead-monomial majority vote across primes; reconstruction stabilization; **external differential (Singular/msolve) as the primary verdict, not the secondary one** | Nothing by itself. Returns `Probable` | See §4.4: the two modes do **not** share a reducer, so this pair does not grade the fast reducer at all | the internal cross-check is a byproduct; external is subprocess-bounded | SCORE + DIFF |
| Sparse `GF(p)` row reduction (F4 kernel) | **A naive dense `u32` Gaussian elimination over the same `FpParams`, in the same crate**, as the reference; agreement on every matrix the corpus produces at sizes the dense route can reach | Same-arithmetic equivalence — a genuine internal oracle for the lane that is 73–91% of an F4 run | Nothing at matrix sizes the dense route cannot reach; there the verdict degrades to DIFF | `>1×`, size-capped | CERT (capped) |
| — why this row is new | ADR-010 §5 claims the certified and fast modes "share one reduction implementation" and calls that the fast mode's only internal oracle. They cannot: `plans/architecture.md` §2.1 puts F4 row reduction in Tier M concrete over `u32` payloads, while the certified mode's cofactor identity must hold over ℚ (mod `p` proves nothing about ℚ). They share matrix construction, symbolic preprocessing, the monomial layer and the row format — **not the reducer**. So a bug in pivot selection, the delayed-reduction cutoff, or Barrett reduction was invisible to the certified mode (C1 §3) | | | | |
| FGLM change of order | (a) the lex basis reduces every element of the drl basis to zero and vice versa; (b) **the lex output satisfies Buchberger's criterion in the lex order** (all pairs, per the clause above); (c) **the lex staircase has exactly `dim_ℚ ℚ[x]/I` standard monomials**, a number FGLM already computes as the dimension of the multiplication-matrix space | Ideal equality *and* the lex Gröbner-basis property, which is what FGLM exists to produce | (a) alone proves generation, not the GB property — and reduction modulo a non-Gröbner generating set is not a well-defined normal form, so "reduces to zero" is weaker than it reads. A generating set that is not a lex GB passed the superseded row and then silently broke elimination, RUR, and all of M8 (C2 §10b) | (a) `~1×`, (b) ≈ recomputing, (c) free | CERT |
| Ideal saturation | `I : f^∞ ⊇ I`; `f` a non-zerodivisor on the quotient; membership cross-checked against a Rabinowitsch computation in one extra variable | Strong | — | `>1×` | DIFF (internal) |

### 3.6 Layer 3 — algebraic numbers

The property suite **is** the verdict function for this layer (§7.4). Additional certificates:

| Operation | Certificate | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| `AlgebraicReal::new` | Defining polynomial square-free (enforced by Yun at construction; constructor returns `Result`), primitive, `lc > 0`, exactly one real root in `(lo,hi)` with `poly(lo) ≠ 0 ≠ poly(hi)`, checked by Descartes variation `== 1` | Well-formedness, which is what makes every query total (ADR-011: fail at construction) | — | `O(1)×` | CERT |
| `refine_to` | Monotone: the new interval is contained in the old; `poly(lo) ≠ 0 ≠ poly(hi)` survives; a midpoint hit collapses to a point and the number becomes exactly rational; **the verdict of every query is identical with and without any number of refinements** | Monotonicity and verdict-invariance | — | `O(1)×` | CERT |
| `cmp` / `Ord` | Equality by `g = gcd(a.poly, b.poly)` plus a **sign change of `g` across the overlap**; a *failed* certificate is never evidence of inequality — the response is refine-and-retry (the consumer gets this right at `arrangements/crates/lazy-exact/src/roots.rs:576-586`) | Trichotomy soundness | **Nothing about attainability.** `Ord::cmp` has no `Result`, no budget, and no way out; on a pathological pair it hangs inside an infallible function — the failure this document calls deadliest (§4.7) | `O(1)×` per step | CERT + open (§4.7) |
| **Budget invariance** | The refinement cache may change *how much work* a call does; it may never change *what the call returns*, including whether it declines. Test: run the property suite twice on the same values, the second time after a warm-up comparison, and at `RAYON_NUM_THREADS ∈ {1,8}`; every verdict, including every `Ok`/`Err(BudgetExhausted)`, must be identical | Schedule- and history-independence of results | — | `~2×` on the L3 sub-corpus | CERT |
| — why this row is new | ADR-013's value proposition is that clones share refinement progress, so the step count of a given `cmp` depends on what has already been compared — and under `parallel`, on what other threads did. Budget exhaustion therefore became history- and schedule-dependent: a call that declines when run first succeeds after a warm-up, and `Ok` vs `Err` could differ by thread count, which is exactly what the determinism gate asserts cannot happen. Corrected: `AlgebraicReal` budgets are **derived from the separation bound** (ADR-011's bound-exists branch, always, for this type) or charged against a worst-case step count computed from operand degree and coefficient size — never against work actually done (C1 §6) | | | | |
| `sign_of(P)` | Zero-ness settled **algebraically first** via `gcd(poly, P)` plus a sign-change certificate, then the refinement loop; cross-checked against materializing `P(α)` as its own algebraic number | Correctness, and termination at `P(α) = 0` | — | `~1×` for the cross-check | CERT+DIFF |
| Radical-tower sign `Σ cᵢ(α)√hᵢ(α)` | Agreement with the materialized `AlgebraicReal` route | A strong, free internal differential oracle: one route squares repeatedly over ℚ(α), the other computes a ℚ-minimal polynomial by resultant and isolates | The two share `UPoly` arithmetic and `Integer` (§4.17) | `~1×` | DIFF (internal) |
| `SqrtExt` `a + b√r` | Sign by squaring, cross-checked against the general `AlgebraicReal` route; total-order axioms including cross-root comparison | Correctness of the fast path that must not be subsumed — `arrangements/crates/arrangements/src/geoms/circle_segments.rs` is 931 LOC using `SqrtExt` exclusively | — | `~1×` | CERT+DIFF |
| `rational_between(α, [β…])` | The returned rational compares strictly greater than `α` and strictly less than every `βᵢ`, via `cmp_rational` | Full correctness | — | `O(1)×` | CERT |
| `IsolatedRoot { value, multiplicity }` | Multiplicity does not participate in `Eq`, `Ord`, `Hash`, `sign_of`, or canonical bytes — asserted by a property test over equal-value/different-multiplicity pairs | That multiplicity cannot leak into identity | — | `O(1)×` | CERT |
| — shape note | A named struct rather than a tuple. It preserves ADR-014's actual safety property in full while keeping the consumer's call-site shape (`root.multiplicity`) intact — the prior art stores it as a field and reads it as a method (`arrangements/crates/lazy-exact/src/roots.rs:319, 437-439`), and a bare tuple forces every storing consumer to thread a parallel structure (C2 §14) | | | | |

### 3.7 Layer 4 — expression DAG

Scope note: M7's exit gate tests hash-consing, `diff`/`diff_with`, constant folding,
`walk_topological`, `is_polynomial_in`, and canonical bytes. `Simplifier`, `RuleSet`, the
built-in rewriter, simplex integration, and rational-function normalization are **post-v1,
on consumer demand**, and have no rows here (C1 §15). The source spec's named risk is
refusing a clever `simplify()`; three documents currently give three answers, and this one
holds M7's line.

| Operation | Certificate | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| Hash-consing | Injectivity: structurally equal terms get the same id, structurally different terms different ids. `Store` is a caller-owned value, never ambient (ADR-020) | Sharing correctness | — | `O(1)×` | CERT |
| `diff` / `diff_with` | **On the polynomial subset the derivative equals `UPoly::derivative` exactly** — an exact cross-layer oracle covering chain, product, and power rules. For transcendental symbols: both `d/dx f` and a high-order finite difference of `f` at random rationals, *approximate and flagged as such* | Exactness on the polynomial subset | Nothing exact about transcendental leaves. The finite-difference check is a smoke test and is graded DIFF, never CERT | `~1×` | CERT (poly) + DIFF (transcendental) |
| Constant folding | Value preservation at multi-seed random points; idempotence | Correctness | — | `O(1)×` | CERT |
| `is_polynomial_in(&syms)` | `Some(p)` ⇒ `p` and the expression agree at multi-seed random points; `None` ⇒ a witness node that is not a ring op over the given symbols | Soundness in both directions | — | `O(1)×` | CERT |
| `walk_topological` | Every node appears after its children; ids stable across identical construction sequences | Ordering and stability | — | `O(1)×` | CERT |
| `Store::rebuild_from` | Round-trip: rebuilding an expression into a second store and back yields byte-identical canonical bytes | Cross-store portability | Nothing about the residual in-range foreign-handle hazard, which is documented and closed only by the optional `store-tags` feature (ADR-020) | `~1×` | CERT |
| Canonical bytes | Byte-identical across insertion orders, thread counts, processes, and `--features` combinations; golden files versioned by explicit `SCHEMA_VERSION` | Content-addressability | — | `O(1)×` | CERT |

---

## 4. Where certificates run out

The danger list. Each entry names an operation or failure mode whose correctness is **not**
self-certifying, says why, and says what grades it instead.

### 4.1 Exponent-field overflow

Wraparound in a packed exponent field silently yields a *correct Gröbner basis of a
different ideal*. Ideal membership passes. Cofactor checks pass. S-pair reduction passes.
Differential testing catches it only where the instance is small enough to run elsewhere.

**What grades it.**
1. Guard-bit SWAR detection on **every** multiply, compiled into release builds, not a
   `debug_assert`. One AND and one compare per word; multiply returns `Result`.
2. **The narrow-width sweep, as a distribution assertion — not a disjunction.** The
   superseded specification ("run the corpus at 4-bit fields; every instance either matches
   the wide run or reports overflow; zero silent divergences") is **trivially satisfied**:
   ADR-008 §4 gives 4-bit fields with one guard bit a total-degree bound of 7, and every
   Katsura/Cyclic/Eco instance passes degree 7, so *every* instance reports overflow, the
   second disjunct holds universally, and the test is green while never once exercising a
   multiply that succeeds near the boundary — which is where a guard-bit off-by-one lives
   (C2 §6). Corrected:

   > For each width `w ∈ {4, 8, 16}` and each corpus instance, let `D_max` be the maximum
   > total degree observed in the **wide** run. The narrow run **must complete and match**
   > iff `D_max ≤ 2^(w−1) − 1`, and **must report overflow** otherwise. An instance that
   > overflows when it should have completed is a false positive and fails; one that
   > completes when it should have overflowed is a silent wrap and fails. CI prints the
   > completed/overflowed counts per width, and **a width at which zero instances complete
   > is a failed sweep, not a passed one.**

   Additionally, the capacity-boundary sub-corpus (total degree exactly `D` and exactly
   `D+1`) must be present at each width and must land on opposite sides.
3. The widen-and-restart driver is itself tested: the answer after restart equals the answer
   from starting wide.
4. Property: `decode(a ⊗ b) == decode(a) + decode(b)` **or** the multiply returned `Err`.
   Never a third outcome.

Because exponents only grow, restart-at-wider-width loses bounded work, which is what
demotes field width from a one-way door to a tuning knob. The *detection* is what makes that
true, so the detection is load-bearing.

### 4.2 Factorization coarseness

A recombination that merges two true factors passes the product check, and over ℚ the
irreducibility certificate does not always exist.

**What grades it.** The modular irreducibility certificate where one exists, with the
**rate** of successful certification tracked as a sharpness number; degree-pattern
consistency across many primes (a *necessary* condition only); differential against PARI
`factor` and Singular `factorize` compared as multisets of associate-normalized factors; the
pairwise-non-associate clause (§3.4 half 3); and the **Swinnerton–Dyer ladder** as the
adversarial generator: `Π(x ± √p₁ ± … ± √p_m)` has degree `2^m`, `r ≈ 2^(m−1)` modular
factors, is irreducible over ℚ, and has no modular irreducibility certificate at any prime.
A coarse implementation returns a nontrivial factorization here; a correct one returns the
input. **One instance from this family is worth more than a thousand random ones.**
Zassenhaus grades van Hoeij for `r ≤ 20`; van Hoeij grades nothing below it.

### 4.3 `⟨G⟩ ⊆ I` in the fast Gröbner mode

Without cofactors there is no cheap general certificate. Arnold's Hilbert-function argument
removes the obligation only for *homogeneous* ideals; Idrees–Pfister–Steidel extended it to
the non-homogeneous global-order case and Noro–Yokoyama then showed that theorem needs an
additional assumption.

**Unsettled and load-bearing.** The research could not obtain the precise statement (both
papers paywalled). Until it is obtained the plan assumes the fast mode **cannot** return
`Proved` without cofactors. *What would settle it:* fetch Noro & Yokoyama, ICMS 2014, and
*Mathematics in Computer Science* 11(3), 2017, and record the exact hypotheses in
`docs/research/`.

### 4.4 The certified Gröbner mode does not grade the fast reducer

This is the correction C1 §3 forced and it changes what the Gröbner trunk's verdict *is*.

- ADR-010 §5's claim that the two modes "share one reduction implementation" is false and is
  amended: they share **matrix construction, symbolic preprocessing, the monomial layer, and
  the row format**. The certified mode reduces over ℚ/ℤ with a cofactor block; the fast mode
  is a `u32` GF(p) kernel. A cofactor identity that holds mod `p` proves nothing over ℚ.
- **Consequence:** a bug in pivot selection, the delayed-reduction cutoff, or Barrett
  reduction is invisible to the certified mode. The cross-check degrades to "two
  implementations of Gröbner agree" with substantial shared machinery — which §4.17 already
  grades as weaker than it looks.
- **What grades it instead:** the naive dense `u32` Gaussian-elimination reference in the
  same crate (§3.5), plus external differential testing promoted to the *primary* verdict
  for both G3 and G5.
- **The cofactor prototype measures the wrong thing.** ADR-010's gate is "memory and time
  multiplier of cofactor tracking on Katsura-8 / Cyclic-7", measured over GF(p), where the
  multiplier is a constant factor on row width. To be a certificate over ℚ the cofactors must
  be **reconstructed**, and cofactor coefficients are systematically larger than basis
  coefficients, so the prime count is set by the cofactors — of which there are `|F| × |G|`.
  Corrected criterion: **"number of primes and wall time to reconstruct the cofactor system
  over ℚ on Katsura-8"**, and the decision re-run against that number.
- **The prototype must not require the artifact it gates** (C1 §11). Measure on
  **Buchberger** with cofactors at Katsura-6/7 over ℚ, reporting the multiplier as a
  function of instance size so it extrapolates, plus the reconstruction prime count. Do not
  wait for F4.

### 4.5 Unlucky primes and unlucky evaluation points

A modular run can be internally consistent and wrong. CRT certifies the *combination*, not
the inputs. Rational reconstruction certifies uniqueness, not intent.

**What grades it.** Where a bound exists — Landau–Mignotte for factors and gcds, Hadamard
for resultants and determinants — use it, be deterministic, exceed `2 × bound`, and the
answer is provably right; this is the default path and the bound itself is now graded
(§3.4). Where only stabilization exists, stabilization is a heuristic and must be closed by
a verification step; a lane brief saying "iterate until stable" without naming the closing
check is underspecified. Brown's minimal-degree rule for gcd images and its analogue for
evaluation points, both cheap and both mandatory. A dedicated adversarial generator:
instances built so a *specific* small prime is unlucky (choose the cofactors first and let
`p | res(A/G, B/G)`), verifying the implementation discards it. And **Hexapod** — 1102
primes for a computation whose single modular run takes 0.00 s — a pure reconstruction-bound
instance, in the corpus from the first modular milestone.

### 4.6 The prime registry is the modular architecture's root of trust

Covered as a catalogue row (§3.1) because it now has a certificate, and named here because
its *absence* was invisible: a composite in the registry breaks `Fp` silently while CRT and
rational reconstruction keep passing, since their checks are statements about `M` and not
about `M`'s factorization (C2 §12). This is a different failure from §4.5's unlucky primes
and only one of the two was previously covered.

### 4.7 `Ord` is total, infallible, and unbounded in practice — on the default path

The mathematics is right: a Mignotte–Davenport separation bound converts "terminates
eventually" into "terminates in a computable number of steps". The engineering conclusion
does not follow, because **computable is not attainable**. For the resultants M4 produces —
ADR-004's own estimate is degree ~200 with ~500-bit coefficients — the bound implies a very
large number of refinement steps, and `Ord::cmp` has no `Result`, no budget, and no exit. A
pathological pair hangs inside an infallible function, which is the failure mode this
document calls deadliest: a hang is worse than a wrong answer because it is undebuggable in
production. Adding `cmp_exact(budget)` alongside leaves the unbounded one as the default,
and the default is what `sort()`, `BTreeMap`, `binary_search` and `max()` call (C1 §7).

**What grades it, and what must be settled by M3.**
1. **Measure the step distribution** on the M4 corpus (lane Y1 can do it before
   `AlgebraicReal` exists, using `Π(x − rᵢ)`-constructed roots) and publish it. If the
   99.9th percentile is small, `Ord` is defensible with evidence rather than with a bound.
2. **Keep `Ord`, with a diagnostic step ceiling far below the theoretical bound**, counted
   on the diagnostics hook, plus a documented and benchmarked
   `try_cmp(&self, &Self, Budget) -> Result<Ordering, Decline>` that latency-path consumers
   are directed to by name in the docs.
3. The step-count 99.9th percentile is a committed sharpness number with a ratchet.

**Do not leave this in "unsettled" past M3.** It is in every signature and it is the
consumer's most-called function.

### 4.8 Silent hangs

`sign_of(h)` never terminates when `h(α) = 0` unless zero-ness is settled algebraically
first, and refinement stalls forever on a non-square-free defining polynomial. **A wrong
implementation of `AlgebraicReal` hangs; it does not return the wrong answer.**

**What grades it.** Step budgets on every property test, with exhaustion counted as a
**failure** for this sub-corpus (see §9's decline classification), never as a skip or a
timeout. Plus a targeted generator: for each `AlgebraicReal α` in the corpus, call
`sign_of(P)` with `P` = the minimal polynomial of `α` (answer exactly zero), with `P` a
multiple of it, and with `P` sharing a factor with `α`'s defining polynomial. These are the
instances that hang.

### 4.9 Intransitivity from a *failed* certificate

The gcd sign-change test can fail spuriously when an overlap endpoint happens to be a root
of `g`. The correct response is refine-and-retry; returning `Less`/`Greater` on a failed
certificate is intransitive in exactly the same way as equality-by-tolerance.

**What grades it.** The transitivity property with a **constructive** generator that
deliberately places overlap endpoints on roots of the gcd. Random generation will not find
this.

### 4.10 `Eq`/`Hash` inconsistency

Equal values can carry different defining polynomials. A cheap `Hash` corrupts `HashMap`s
nondeterministically and no unit test catches it.

**What grades it.** A property test building a map keyed by algebraic numbers generated as
*deliberately different representations of the same value* (`√2` as a root of `x²−2`, of
`x⁴−4`, of `x⁴−4x²+4`), asserting one entry per distinct value. If `Hash` is not implemented
— it is not, on the un-canonicalized type — the test asserts the *absence* of the impl as a
compile-fail test, so the decision cannot silently regress.

### 4.11 Multiplicity semantics

Whether resultant-root multiplicity *is* intersection multiplicity in general has no
certificate; it is a mathematical claim about the generic case with known exceptions. The
consumer today reads it off a resultant root and degrades to an `Unknown` crossing kind in
the ambiguous case (`arrangements/crates/arrangements/src/geoms/conics.rs:600-618`).

**What grades it.** A hand-authored known-answer corpus of curve pairs with computed
intersection multiplicities, including the documented ambiguous case (two distinct common
points over one abscissa with parallel gradients). This is human-authored and small; that is
acceptable because it is the only thing that can grade a definitional question. Every entry
carries `provenance = hand-computed(author, method)` (§8.4).

### 4.12 Curve analysis (fiber structure, branch matching)

The largest component with zero counterpart in the prior art. Its outputs — per-interval
branch counts, branch-index-to-root maps — have only weak invariants.

**What grades it**, in descending order of strength:
1. **Topological consistency:** branch counts over adjacent open intervals change only at
   critical abscissas, and by an amount consistent with the local structure of the critical
   fiber.
2. **A second, genuinely independent route:** compute the same fiber counts by isolating the
   roots of `f(α, y)` at a *rational* witness abscissa strictly inside each interval — cheap,
   uses only the univariate lane. **Build this as the oracle before the fast path.**
3. Differential against Sage/CGAL-class systems on small instances (Tier 2, expensive).
4. A hand-authored corpus of curves with known topology: nodal cubics, cusps, tangential
   pairs, vertical components, vertical asymptotes.

This lane's verdict is materially weaker than every other Layer-2 lane. Split it: (a)
critical-abscissa extraction is `Res_y(f, ∂f/∂y)` plus isolation and is well graded; (b)
branch matching across critical fibers is the genuinely hard part and is a human-specified,
agent-implemented lane.

### 4.13 Modular methods over algebraic-extension coefficients

`Reducible::Image: Field` asserts reduction mod `p` lands in a field. Over ℚ(α) it lands in
`GF(p)[x]/(f mod p)`, a field only at inert primes — and for the multiquadratic towers
geometry actually produces (ℚ(√2,√3), Galois group `(ℤ/2)²`, no 4-cycle) **no prime is
inert**, so the bound has no valid implementation at all. This is the same Chebotarev
obstruction the plan documents for Swinnerton–Dyer factorization certificates, never
connected to the trait bound (C1 §4).

**Corrected position.** `Image: CommutativeRing`; `reduce -> Result<Self::Image, BadPrime>`.
The modular path over algebraic extensions is **multi-modular over split factors** — factor
`f mod p`, work in each `GF(p^{d_i})`, CRT back — which is a different algorithm with its own
bad-prime predicate, and it is a **lane**, not an instantiation. `UPoly<NumberFieldElem>`
compiles as an added instantiation and gets the Tier-G reference path (correctness, not
speed) until that lane exists; the docs say so in those words, because SMT NRA is the
consumer M8 exists for and it does root isolation over ℚ(α₁…α_k) in its inner loop.

**What grades it.** ℚ(√2,√3) is in the M8 corpus specifically, because it is the instance
where a naive implementation silently divides by a zero divisor.

### 4.14 Batched multi-modular lanes: arithmetic is certified, control flow is not

Batching works only while all `N` primes behave identically, and two things break that.
(1) A pivot zero in one lane: `Field::inv -> Option<Self>` must return `None` if any lane is
zero, and `Option` cannot say *which*, so the batch cannot be split. (2) Lead-monomial
divergence: the Gröbner bad-prime rule is a majority vote over lead-monomial sets, but under
batching all `N` primes share one matrix construction and one pair-selection path — that
sharing *is* the ~2.7× — so a diverging prime corrupts shared control flow instead of
producing a minority to discard. Componentwise equality with `N` scalar runs is a complete
oracle **for arithmetic** and silent on both (C1 §14).

**What grades it.** `inv_batch -> Result<Self, LaneMask>`; a batch-split driver that on any
lane fault or lead-monomial divergence splits and records the offending prime index in the
`Trace`; corpus instances constructed to trigger each; and the lane brief that says
"batching **and** splitting", not "batching".

### 4.15 Non-determinism and thread-count dependence

No algebraic certificate exists. **What grades it:** run every corpus instance twice in one
process, twice in two processes, at 1/2/8 threads, and under each supported feature
combination, comparing canonical bytes. Any difference is a failure. This catches `HashMap`
iteration order, address-dependent hashing, `thread_rng`, work-partition-dependent
accumulation order, and — with the corrected `AlgebraicReal` budget rule (§3.6) — cache-warm
verdict flips. It must exist from day 1 because every other regression artifact assumes it,
and §11.2's tiering exists specifically so this gate is not the first thing sacrificed for
CI time.

### 4.16 Sharpness gates, and the ratchet that makes them real

Every soundness certificate in §3 is satisfied by a maximally conservative implementation:

| API | Trivially-sound useless implementation | Sharpness gate |
|---|---|---|
| `sign_over -> Verdict<Sign>` | always `Unknown` | Unknown rate below a committed ceiling; **zero** Unknowns on the clear-sign sub-corpus |
| `Certainty` | always `Probable` | `Proved` rate, with per-operation floors: gcd, resultant, factorization-product, and isolation are **`1.0`, committed on day one, never ratcheted** |
| `Result<_, BudgetExhausted>` | always decline | Decline rate at the standard budget; zero declines on the must-complete sub-corpus |
| `divmask` | always "maybe divisible" | False-positive rate, plus the divisor-query benchmark (the 10–20× comes from this filter working) |
| Separation bound | return 0 | Ratio of returned bound to observed separation, as a distribution |
| Isolating intervals | return `(−cauchy, +cauchy)` | Interval width relative to the separation bound. Disjointness and count pin the *set*; they do not pin the *width* |
| `Ord` step count | correct but unattainable | 99.9th-percentile step count on the M4 corpus (§4.7) |
| Curve crossing classification | always `Unknown` | Unknown rate on the intersection corpus |
| Modular irreducibility | never certify | Certification rate on the factorization corpus |

**The ratchet** (this is what the superseded draft lacked — not one of its ceilings was a
number, while Gate 1 and M3's exit gate both keyed on "committed ceilings", C2 §8):

> Every sharpness rate is established by measurement in the first PR that lands the API it
> guards. That PR commits the measured value, rounded outward by a stated margin, to
> `sharpness-ceilings.toml`. Thereafter:
> - CI fails if a measured rate exceeds its committed ceiling.
> - **A PR may lower a ceiling freely.** Lowering is progress and needs no justification.
> - **A PR may not raise a ceiling** without a recorded justification *in the file* and a
>   line in CI output, counted the same way generator deletions are counted (§9).
> - A rate with **no** committed ceiling fails Gate 1. `TBD` is not a ceiling.

### 4.17 Oracle independence — correlated failure between "independent" checks, and its gate

Several of the strongest verdicts are two-implementation cross-checks. They are only as
strong as the code the two implementations do **not** share.

| Cross-check | Genuinely independent | **Shared, therefore correlated** |
|---|---|---|
| Descartes isolation vs Sturm count | subdivision + variation counting vs sign-sequence counting | `UPoly` arithmetic; `divrem`; **and if Sturm's chain is ever routed through a subresultant PRS, a PRS bug corrupts Sturm *and* the resultant lane simultaneously** |
| Ducos PRS vs modular evaluation–interpolation | pseudo-division and exact division vs GF(p) arithmetic, CRT, interpolation | `UPoly`/`RecursiveView` storage |
| Bareiss/Bézout determinant vs both | dense linear algebra | `Integer` arithmetic only — **the most independent of the three, and worth building for exactly that reason** |
| Radical-tower sign vs materialized `AlgebraicReal` | squaring ladders over ℚ(α) vs resultant + isolation | `UPoly`, `Integer` |
| `groebner_certified` vs `groebner` | cofactor tracking, ℚ arithmetic vs modular + tracing | matrix construction, symbolic preprocessing, monomial layer, row format — **and not the reducer at all** (§4.4). Substantial sharing; this cross-check is weaker than it looks |
| F4 kernel vs naive dense `u32` GE | sparse pivoting/scheduling vs textbook elimination | `FpParams`, `mulmod` |
| Batched `Fp4` vs 4× scalar `Fp` | tuple/SIMD path vs scalar path | the reduction algorithm itself if both call the same `mulmod`; and all control flow (§4.14) |

**Consequence:** the monomial layer, `UPoly` arithmetic, and `Integer` are *common mode* for
nearly every internal cross-check, so they need external differential testing and
exhaustive/oracle-backed testing more than anything else in the library.

**The gate**, which the superseded draft left as "this table must be maintained and audited"
— audited by whom, in a project whose founding constraint is that agents build it (C2 §4):

1. **Module-import manifest.** Each oracle module declares, in a committed manifest, the set
   of modules it may reach transitively. CI walks the module graph (or the crate graph where
   the split is crate-level) and **fails on any edge into the lane the oracle grades.** Same
   shape as gate L1 (`cargo tree` diffed against a checked-in graph), one level finer.
2. **Frozen-oracle marker.** An oracle module carries `#![doc = "ORACLE: graded lane = U5"]`,
   and any PR touching both it and the graded lane fails CI without an explicit
   `oracle-independence-reviewed:` trailer naming what was checked. Weaker than (1), worth
   having as well, because it catches "I made both sides agree" edits.

The predictable failure this closes: between M2 and M4 someone notices Sturm over ℚ has
catastrophic coefficient growth and "fixes" it by routing it through the Ducos PRS that lane
T1 just landed. At that moment the strongest certificate in Layer 2 silently becomes a check
of a component against itself and **no test changes colour**.

### 4.18 Sturm's affordability ceiling

Sturm is `Õ_B(d⁴τ²)` and that bound is tight. There is a degree above which the strongest
isolation certificate is unaffordable, and above it the lane's verdict degrades from CERT to
DIFF. **What grades it:** measure `d*` — the largest degree at which Sturm's median runtime
on the pinned machine is ≤ a committed `T` — commit `d*`, and record the degradation in the
lane's status. Below `d*` the isolation lane is CERT (Sturm-graded); above it, DIFF
(Descartes ↔ ANewDsc ↔ external oracle). Discovering this as a mysteriously slow CI job
instead is the failure mode.

### 4.19 Panics and unbounded work on adversarial input

Library code must not panic on any input-dependent path (ADR-011; `API.md` INV-4). **What
grades it:** a fuzz target per public entry point over `arbitrary`-generated inputs under a
panic hook that fails the test; plus a budget test asserting every entry point respects its
budget on an instance chosen to exceed it. Degree overflow, exponent-packing overflow,
coefficient blowup, arena exhaustion, and non-finite `f64` ingress are all `Result`, never
abort.

### 4.20 Canonical-serialization drift

A resolvent upgrade that changes canonical form is a re-key event for every downstream
content-addressed artifact. **What grades it:** committed golden files compared byte-for-byte
with an explicit `SCHEMA_VERSION`; changing a golden requires bumping the version in the same
commit, and CI fails a golden change without a bump.

### 4.21 "Correct but useless"

An isolator returning valid but enormous intervals; a Gröbner basis right after 14 hours; a
resultant correct with 500-bit coefficients computed the slow way. No certificate detects any
of these. Only §10 does, and it converges over months.

---

## 5. The silent-wrongness surface

Concrete scenarios where the code passes **every** gate in this document as it would
naturally be written, and is still wrong. Each row names the additional check that closes
it; every one of those checks is specified elsewhere in this document, and this table is the
index into them.

| # | Scenario | Why every gate is green | What closes it |
|---|---|---|---|
| 1 | `certifies()` iterates an empty evidence vector and returns `true` | The certificate is written by the same agent in the same PR; it never rejects anything | **Mutant sets** (§2.1) — a plausible wrong value the certificate must reject |
| 2 | `fn gcd(_,_) -> Integer { ONE }` | Divisibility holds; the coprimality clause is evaluated by the function under test | **Bézout witness** (§3.1, §3.3) and **rule C** (§2.2) |
| 3 | A recombination merges two true factors | Multiply-back passes; the merged factor is reducible | Modular irreducibility half, **pairwise non-associate** clause, Swinnerton–Dyer ladder (§3.4, §4.2) |
| 4 | A packed exponent field wraps | The result is a correct Gröbner basis of a *different* ideal; membership, cofactors and S-pairs all pass | Guard bits in release + the **distribution-asserting** narrow-width sweep (§4.1) |
| 5 | A composite enters the prime registry | `Fp` stops being field arithmetic; CRT and rational reconstruction still certify, because their checks concern `M`, not its factorization | Independent segmented-sieve cross-check + golden hash of the accepted set (§3.1, §4.6) |
| 6 | The S-pair verifier applies Gebauer–Möller | A pair-elimination bug drops a non-redundant pair; the object is not a Gröbner basis and passes "Buchberger's criterion" | Criteria-free verifier over all `C(\|G\|,2)` pairs; criteria mutant (§3.5) |
| 7 | FGLM returns a generating set that is not a lex Gröbner basis | Two-way reduction to zero passes; dimension and degree match | Buchberger's criterion in lex + staircase `== dim_ℚ ℚ[x]/I` (§3.5) |
| 8 | The separation bound is systematically over-large (off-by-one exponent, `bit_length` vs `ceil(log2)`) and `Equal` is concluded by exhausting it | Distinct numbers compare `Equal` — **consistently**, so the collapse is transitive and the transitivity property cannot see it | The invariant "no `Equal` from exhaustion" + brute-force separations at degree ≤ 6 + the bound's own validity row (§3.3, §3.4) |
| 9 | A Schwartz–Zippel certificate runs at the fixed default seed forever | An error vanishing at `prime(0)` and `prime(1)` — not a contrived class in modular arithmetic — is certified permanently | **Rule S** (§2.3): grade across the fleet seed schedule; report the seed count |
| 10 | Interning is parallelized: `terms.par_iter().map(\|t\| ring.intern(t)).collect()` | The collection is ordered — the permitted shape — but the ids are not; only tie-breaks that consult id order diverge, which is data-dependent | Content-derived ids + the ban on id-order tie-breaks + the id-assignment thread matrix (§3.2) |
| 11 | A warm refinement cache flips a decline into a success | Property outcomes depend on execution order, which shrinking reorders; `Ok` vs `Err` differs by thread count | Budget invariance test (§3.6); budgets derived from the bound, never from work done |
| 12 | One lane of an `Fp4` batch has a zero pivot, or one prime's lead-monomial set diverges | Componentwise arithmetic equality with 4 scalar runs still holds; the failure is in shared control flow | `inv_batch -> Result<_, LaneMask>` + batch-split driver + constructed corpus instances (§4.14) |
| 13 | Sturm is "fixed" by routing it through the Ducos PRS | The strongest Layer-2 certificate becomes a check of a component against itself, and no test changes colour | Module-import manifest + `ORACLE:` marker (§4.17) |
| 14 | A mis-triaged Class-B disagreement is frozen into the append-only corpus as an expected answer | The corpus gates at 100% forever, so a *correct* future implementation fails | `provenance` field per entry; `oracle-consensus` entries are re-derivable and drift-checked nightly (§8.4) |
| 15 | An oracle adapter emits variables in the wrong order | The S-expression round-trips perfectly through resolvent's own encoder/decoder; the oracle answers confidently about a different object | Per-adapter **calibration corpus** of hand-computed answers (§6.3) |
| 16 | `primes_used` lands inside canonical bytes | The tuning-matrix byte-identity gate fails on the first run, and the natural "fix" is to disable the gate | Certificates, evidence and telemetry are excluded from canonical bytes (§2.6) |
| 17 | Gate 0 grows to 40 minutes, so the determinism matrix is trimmed to `{1,8}`, then cross-process is dropped, then the corpus moves to Gate 1 | Every step is individually reasonable; the gate that "must exist from day 1" is the most expensive and the least often red | Day-1 corpus tiering with a printed census and a hard `fast`-tier budget (§11.2) |
| 18 | A published crate carries an LGPL dev-dependency (`rug` tests inside `resolvent-int/tests/`) | `cargo-deny` is scoped to the published graph minus dev-only features, so it does not fire; `cargo publish` records it and downstream `cargo test` builds GMP | **Published crates have zero dev-dependencies**, asserted by one `cargo metadata` check (§6.1) |
| 19 | An Eco-`n` / Noon-`n` / Reimer-`n` generator is transcribed from a Singular `.lib`, an msolve test directory, or a Groebner.jl benchmark file | All three are GPL-2.0; the verification plan literally instructed "pin them to a specific generator source", in the one lane nobody looks for a licensing problem | Tier-A citation required as a benchmark-generator metadata field, checked by the same CI rule as `Derivation:`; otherwise the family is dropped (§8.6) |
| 20 | A `Derivation:` line cites a paper the author never opened | CI checks the line exists; it cannot check it is true, so the gate detects only the laziest violation | `Derivation:` cites **both** the paper and a resolvable path into `docs/research/`; CI resolves the path and requires a `Sources:` block with a tier tag (§11.4) |

---

## 6. Differential oracles

### 6.1 Rules

- **Nothing links.** Every external oracle is driven as a subprocess over a text protocol.
  Singular, PARI, msolve, Sage, Macaulay2 are GPL; linking them into a permissive library is
  not permitted at all. FLINT is LGPL and *could* be linked, but the uniform subprocess rule
  has no exception process, which is what makes it enforceable. If in-process FLINT speed
  ever proves necessary it goes in a `publish = false` crate behind a non-default feature —
  and `flint-sys`'s missing repository LICENSE file must be resolved upstream first.
- **Exactly two crate categories.** `publish = true`, gated by `cargo-deny` over the
  published graph; `publish = false` (`resolvent-oracles`, `resolvent-bench`,
  `resolvent-fuzz`) which may carry LGPL dev-dependencies and shell out to GPL binaries. No
  third category, no per-crate exception.
- **Published crates have zero dev-dependencies**, asserted by a one-line `cargo metadata`
  check. The ℤ/ℚ differential oracle against `rug` lives in a `publish = false` crate and
  tests only `resolvent-int`'s *public* surface — which is sufficient by design, because the
  newtype wall means the public surface is the whole point. Without this rule the `rug`
  oracle has nowhere to live that satisfies gate L6: inside `resolvent-int/tests/` it makes a
  published crate carry an LGPL dev-dependency that `cargo-deny` does not see and
  `cargo publish` records (C1 §18).
- **A missing oracle is a counted `SKIP`, never a pass.** The harness prints a skip census.
  A CI job declares which oracle tier it requires; if an oracle in that tier is absent, **the
  job fails** rather than silently reducing coverage.
- **Text protocol only**, through the canonical serializer in `resolvent-base` (ADR-012 §9),
  shared by every crate and every adapter. Oracle-specific parsing never appears in a test
  body.

### 6.2 Install tiers and per-operation assignment

Machine state verified 2026-07-31: **none of Singular, PARI, SageMath, Maxima, FLINT, or
Macaulay2 is installed**; `python3 -c "import sympy"` reports 1.14.0 via pyenv and works
today. `pacman -Si` confirms `singular` 4.4.1.p5-11 (11.38 MiB download / 59.25 MiB
installed) and `pari` 2.17.4-1 (8.79 MiB / 28.69 MiB) available in `extra`. Macaulay2 is
AUR-only; msolve is not packaged.

*Caution for future agents:* `command -v gp` succeeds on this machine because `gp` is a shell
alias for `git push`. Detect PARI by invoking `gp -q -f` on a known expression and checking
the output, never by probing `$PATH`.

| Tier | Contents | Install cost | When it runs |
|---|---|---|---|
| **0** | `sympy` via `python3` | zero — present today | Every PR. Non-negotiable. |
| **1** | `singular`, `pari` | ~88 MiB installed, one `pacman -S` | Nightly; locally recommended |
| **2** | `sagemath` (371 MiB, ~60 transitive deps), `msolve` (source build), `macaulay2` (AUR) | expensive, CI box only | Weekly and pre-release |
| **dev-link** | `rug` in a `publish = false` crate (precedent: `dashu`'s own `fuzz/` uses `rug::Integer`) | crate only | Every PR, in-process |

| Operation | Primary | Secondary | Note |
|---|---|---|---|
| `Integer`/`Rational` arithmetic | `rug` (in-process, unpublished crate) | PARI | The only oracle worth linking, and only as a dev-dependency |
| Univariate gcd, square-free | sympy (`gcd`, `sqf_list`) | PARI, Singular | Tier 0 covers it |
| Resultant, **subresultant chain** | sympy `subresultants` | PARI `polresultant`, Singular `resultant` | sympy gives the **whole PRS chain**, which is the intermediate data the lane must match |
| Univariate factorization over ℤ/ℚ/GF(p) | PARI `factor` | sympy `factor_list`, Singular | PARI is the number-theory specialist |
| Multivariate factorization | Singular `factorize` | sympy, PARI | |
| Real root isolation | PARI `polrootsreal` | sympy `real_roots`, `CRootOf` | PARI returns certified intervals |
| Algebraic-number comparison | sympy `CRootOf`, `minimal_polynomial` | PARI `nfinit`/`polredabs`, Sage `QQbar` | Sage's `QQbar` is richest and is the 371 MiB dependency |
| Gröbner (drl, lex, elimination) | Singular `std`/`groebner` | msolve, Macaulay2 | Singular is the literature's reference and needs no Python layer |
| FGLM / change of order | msolve | Singular `fglm` | msolve is the pipeline resolvent copies architecturally |
| Everything, fallback | SageMath | — | CI box only; never a gate for local dev |

**Unsettled, with what would settle it.** (a) Whether Singular or msolve is the better
Gröbner oracle in practice — start with Singular because it is one command away, build msolve
once, and compare on a shared corpus before committing the CI box. (b) Whether
`feanor-math` (MIT) is usable as an **in-process, `publish = false` dev-oracle** for
Cantor–Zassenhaus, Hensel-lifted ℤ factorization, LLL, and `zn_64` Barrett reduction. Its
license permits linking where every other capable system's does not, and the prior-art survey
evaluated it only as a *dependency* and as a *fork*, never as an oracle (C2 §17). Settle it by
counting and classifying its `#![feature(...)]` uses — specifically whether any is inside
`factorization`/`lll`/`zn_64` or only in the ring-framework glue — and by filing the upstream
missing-LICENSE-file issue immediately regardless, since that blocker takes weeks of latency
to clear later.

### 6.3 Adapter calibration — the check that grades the oracle, not the round-trip

An adapter graded by round-trip (`resolvent → S-expression → resolvent`) exercises
resolvent's own encoder and decoder and never establishes that *the oracle read the same
polynomial*. An adapter that emits variables in the wrong order, or emits a polynomial in `x`
where resolvent meant `y`, round-trips perfectly and then produces confident agreement or
confident disagreement about the wrong object (C2 §22).

> **Each adapter ships an oracle calibration corpus:** a dozen instances per operation whose
> answers are hand-computed and committed, with the *oracle's* answer asserted against them.
> `Res(x²−2, x²−3)`; `gcd(x²−1, x³−1)`; `factor(x⁴+1)` over ℚ and over `GF(3)`;
> `isolate_roots` of a Chebyshev polynomial; a two-variable Gröbner basis with a
> deliberately non-alphabetical variable order. If the oracle's answer to a known-answer
> instance is wrong, the adapter is wrong — and this is the only test that can say so. It is
> also the test that catches an oracle version bump changing a convention.

### 6.4 Normalization — the comparison is semantic, never textual

| Operation | How disagreement is defined |
|---|---|
| **Gröbner basis** | *Never* compare generator lists. In order: (1) the two lead-monomial ideals are equal; (2) every element of `A` reduces to zero modulo `B` and vice versa — this is ideal equality and it is the real test; (3) as a fast pre-filter only, reduced+monic+order-sorted bases byte-equal. Compare Hilbert series where both systems produce it, which catches "same ideal, different order convention". **Order convention is the number-one false disagreement:** pin the variable order and the term order explicitly in the emitted input and assert the oracle echoes them back |
| **Factorization** | Multisets of associate-normalized factors (content removed, positive leading coefficient) with multiplicities. Unit factors stripped and compared separately |
| **GCD** | Normalize both to primitive with positive leading coefficient, then `==` |
| **Resultant** | Sign conventions differ under argument swap by `(−1)^(mn)`. Emit arguments in a pinned order and accept `±Res` **only with the sign rule applied explicitly** — never "up to sign" unconditionally, because that hides genuine sign bugs. Compare after content normalization. Degenerate inputs follow the pinned convention (§3.3) |
| **Subresultant chain** | Compare the *degree sequence* first — that is the structural content — then each `Sᵢ` up to the known scalar convention. Conventions genuinely differ between sources; pin resolvent's and document the conversion in the adapter, never in a test body |
| **Root isolation** | *Never* compare endpoints. Compare: (1) the count of real roots in the query interval; (2) a bijection where each of ours overlaps exactly one of theirs; (3) refine both sides by `k` steps and assert the intervals stay nested |
| **Algebraic-number comparison** | Compare *sign verdicts* and *orderings*, never representations. `x²−2` and `x⁴−4` are the same number |
| **Minimal polynomial** | Compare after monic normalization over ℚ, or primitive-with-positive-lc over ℤ. Degrees must match exactly — a mismatch is a real bug (a non-minimal polynomial), not a convention |

**Free internal differential oracles** — no external system, no install cost, best
signal-to-noise because both sides self-certify:

| Pair | Covers |
|---|---|
| Ducos PRS ↔ modular evaluation–interpolation ↔ Bareiss/Bézout determinant | Resultants — three routes sharing almost no code |
| plain Descartes ↔ Sturm ↔ ANewDsc | Isolation; Sturm gives the *exact* count and grades the others automatically |
| Zassenhaus ↔ van Hoeij (`r ≤ 20`) | Factorization recombination |
| Buchberger ↔ F4 ↔ `groebner_certified` ↔ `groebner` | Gröbner — but see §4.4 for what this pair does **not** cover |
| F4 sparse kernel ↔ naive dense `u32` Gaussian elimination | The GF(p) reducer, which nothing else grades |
| radical-tower sign ↔ materialized `AlgebraicReal` | Layer 3's hot path |
| batched `Fp4` ↔ 4× scalar `Fp` | Modular batching arithmetic only (§4.14) |
| narrow-field-width run ↔ wide-field-width run | Exponent packing (§4.1) |
| `UPoly` fast paths ↔ naive `O(n²)` reference | Layer 1 |
| `Tuning` matrix: naive path ↔ fast path at every crossover | Every threshold in ADR-012 §8, for free |

**Build the oracle side first, every time.** Sturm exists to grade Descartes; Buchberger
exists to grade F4; Zassenhaus exists to grade van Hoeij; the naive `O(n²)` multiply exists to
grade the fast one; the dense `u32` elimination exists to grade the sparse kernel. None of
them will ever be the production algorithm.

### 6.5 Triage and minimization

Every disagreement runs this pipeline automatically before a human sees it.

1. **Classify by self-certificate.** Re-run resolvent's own certificate on the instance.
   - *Self-certificate also fails* → **Class A: resolvent bug, certain.** Highest severity;
     straight into the regression corpus.
   - *Self-certificate passes, oracle disagrees* → **Class B: normalization, convention, or
     oracle limitation.** Go to step 2. **This classifier is only as good as the
     certificates, which is why rule M (§2.1) is prior to it.**
2. **Minimize.** Delta-debug while the disagreement persists, cheapest structural reduction
   first: drop terms → halve coefficient bit-length → reduce degree → reduce variable count →
   reduce generator count → shrink the query interval. Minimization is fully automatic; an
   unminimized counterexample is not accepted into the corpus. Target: **1-minimal** (no
   single further reduction step preserves the disagreement) within a committed time bound.
3. **Re-classify the minimized instance.** Both sides self-certify and the answers are
   genuinely different objects → adapter normalization bug; fix the adapter and add an adapter
   test. Both self-certify and the answers agree under a convention not yet pinned →
   **unspecified convention**; write it into an ADR and into §6.4. The oracle is wrong or out
   of range → record, report upstream if warranted, mark `oracle-limitation`.
4. **Record.** Every outcome — including "not a bug" — is appended to the corpus with its
   class, minimized form, provenance (§8.4), oracle version, and resolvent commit. Nothing is
   triaged twice.

---

## 7. Property tests

Cheap to write, and they catch most of what the certificates cannot. Every law below is a
gate, not a suggestion. Every suite runs under an explicit **step budget**, and every suite
is quantified over the generator fleet (§8), not over hand-picked examples.

### 7.1 Layer 0 — rings, fields, and the trait tower

**Trait laws are quantified over `(Ctx, elements)` pairs, not over elements alone.** `Ring`
carries `type Ctx` with `fn zero(ctx: &Self::Ctx)`, `fn one(ctx: &Self::Ctx)`, and
`fn ctx(&self) -> &Self::Ctx`; element-to-element arithmetic is unchanged and nothing enters
the inner loop. A law suite that only exercises `Integer` and `Rational` — the only two
members of the closed instantiation set with a static zero — silently passes a trait whose
`zero()` cannot answer "zero of *which* prime field" for `Fp`, `Zn`, `GFpk`, `Fp4`, or
`NumberFieldElem` (C1 §1). The suite therefore instantiates **every member of the closed
set**, each with a `Ctx` generator, and the whole trait block is typechecked with real impls
for `Fp` and `Integer` before the freeze.

| Law | Statement |
|---|---|
| Additive group | associativity, commutativity, `a + zero(ctx) == a`, `a + (−a) == zero(ctx)` |
| Multiplicative monoid | associativity, `a · one(ctx) == a` |
| Distributivity | `a·(b + c) == a·b + a·c` |
| Commutativity | on `CommutativeRing` only; asserted absent-by-type elsewhere |
| Field inverse | `a ≠ 0 ⇒ a · a⁻¹ == one(ctx)`; `inv(zero) == None` |
| Euclidean | `a == q·d + r` with `r` smaller in the ring's norm; `div_rem(_, zero) == None` |
| `Ordered` | totality, transitivity, `sign(a·b) == sign(a)·sign(b)`, `sign(a) == 0 ⇔ a.is_zero()` — for `Integer`/`Rational` only, and **asserted absent for `Fp`/`Fp4`**, because requiring `Ord` on the coefficient ring would permanently close the batching door |
| `Reducible` | `reduce` is a ring homomorphism onto its image where it succeeds; returns `Err(BadPrime)` and never a silent zero-divisor otherwise |
| `Liftable: Reducible` | `crt_lift(reduce(x, mᵢ), mᵢ) == x` whenever `Π mᵢ` exceeds the operand's bound. (The supertrait is `Reducible`, not `Ring` — `Self::Image` is `Reducible`'s associated type, and the draft's declaration did not compile) |
| `Ctx` coherence | `x.ctx() == y.ctx()` for every pair produced by an operation on `x`, `y`; mixing contexts is a `Result`, never a silent wrong answer |
| Frobenius | over `GF(p^k)`: `x^(p^k) == x` for every element |
| Batch coherence | `Fp4` componentwise equals four scalar `Fp` runs, **and** lane faults surface as `LaneMask` rather than as a bare `None` |

### 7.2 Layer 1 — polynomials and monomials

| Law | Statement |
|---|---|
| Ring axioms on `UPoly<C>`/`MPoly` | inherited from §7.1, quantified over the same closed set |
| Degree | `deg(a·b) == deg a + deg b` over an integral domain; `deg(a+b) ≤ max(deg a, deg b)`; the zero polynomial has degree `None` and every operation handles it |
| Evaluation homomorphism | `eval(a+b, x) == eval(a,x)+eval(b,x)` and the multiplicative analogue, at multi-seed points |
| Division | `a == q·b + r ∧ deg r < deg b`; pseudo-division's `lc(b)^δ` identity |
| Content | `p == content(p)·primitive(p)`; `content(primitive(p)) == 1`; Gauss: `content(p·q) == content(p)·content(q)` |
| Associate normalization | idempotent; `p ~ q ⇔ norm(p) == norm(q)` |
| Reciprocal transform | `reverse(reverse(p)) == p` for `p` with nonzero constant term; `eval(reverse(p), 1/x) == x^{−n}·eval(p, x)` symbolically |
| Taylor shift | `shift(shift(p,a),−a) == p`; composition with `scale_pow2` commutes as expected |
| Monomial order | totality, antisymmetry, transitivity, well-ordering, **multiplicative compatibility** `a < b ⇒ a·u < b·u`; agreement with a naive `Vec<u32>` comparator per order |
| Monomial arithmetic | `decode(a ⊗ b) == decode(a) + decode(b)` **xor** `Err`; divisibility/lcm/gcd/degree **identical across all orders** |
| Interning | `key(u) == key(v) ⇔ u == v`; `h(u) + h(v) == h(u·v)`; ids are a pure function of content, identical at 1 and 8 threads |
| `map_coefficients` | homomorphism laws; `map(id) == id`; `map(f ∘ g) == map(f) ∘ map(g)` |

### 7.3 Layer 2

| Law | Statement |
|---|---|
| gcd | `gcd(a,0) == normalize(a)`; `gcd(a,b) == gcd(b,a)`; `gcd(ka,kb) == k·gcd(a,b)` up to units; `gcd(a,b)·lcm(a,b) ~ a·b`; Bézout identity holds |
| Square-free | `Π fᵢ^i == f`; each `fᵢ` square-free; pairwise coprime and pairwise non-associate; idempotent on square-free input |
| Resultant | `Res(f,g) == (−1)^{mn} Res(g,f)`; `Res(f, g·h) == Res(f,g)·Res(f,h)`; `Res(f,g) == 0 ⇔ deg gcd > 0`; multiplicativity under the pinned degenerate convention |
| Subresultant | valid degree sequence; specialization commutes at good ring maps; last nonzero element is a gcd up to content |
| Isolation | counts match Sturm; intervals disjoint, ascending, inside the Cauchy bound; variation exactly 1; Σ multiplicities `== deg` of the square-free-corrected input; round-trip from `Π(x − rᵢ)` |
| Factorization | `Π fᵢ^{eᵢ} == f`; factors non-associate; each irreducible where a certificate exists; idempotent — refactoring a factor returns it unchanged |
| Gröbner | `G` reduces every `f ∈ F` to zero; all S-pairs reduce to zero; cofactors reconstruct each `gⱼ`; normal form is unique and order-independent given the same order; `⟨G⟩ == ⟨F⟩` by two-way reduction; the reduced basis is unique for a fixed order |
| FGLM | lex output is a lex Gröbner basis; staircase size `== dim_ℚ ℚ[x]/I`; round-trip drl → lex → drl is the identity on the reduced basis |
| Bound validity | every computed Landau–Mignotte / Hadamard / Cauchy / separation bound is `≥` the true quantity on the known-answer generators |

### 7.4 Layer 3 — the suite that *is* the verdict

| Property | Statement | Why it is the canary |
|---|---|---|
| **Trichotomy** | Exactly one of `a < b`, `a == b`, `a > b` | Catches returning an ordering when the equality certificate merely *failed* |
| **Transitivity** | `a ≤ b ∧ b ≤ c ⇒ a ≤ c`. Generator must produce triples with two pairs within `2^-1000` | The named canary. Equality-by-tolerance and failed-certificate-as-inequality are both intransitive and *only* transitivity catches them |
| **Antisymmetry** | `a ≤ b ∧ b ≤ a ⇒ a == b` | — |
| **Equality is an equivalence relation**, and equal elements agree on `sign_of(h)` for every `h` | reflexive, symmetric, transitive | Catches representation-dependent equality (`x²−2` vs `x⁴−4`) |
| **Sort stability** | Sorting a shuffled list yields the same sequence of equality classes every time | Catches state-dependence in the refinement cache |
| **Budget invariance** | Every verdict — including `Ok` vs `Err(BudgetExhausted)` — is identical with a cold cache, with a warm cache, and at 1 vs 8 threads | The corrected form of the step-budget property. Without it, shared refinement makes declines schedule-dependent (§3.6) |
| **Enclosure consistency** | If `cmp(a,b) == Less` the `f64` enclosures must not contradict it; disjoint enclosures agree with the exact verdict | Catches outward-vs-nearest rounding-direction bugs, otherwise invisible |
| **Isolator consistency** | `isolate_roots(f)` returns roots strictly ascending under `cmp`, and the count matches Sturm | Cross-lane |
| **Refinement idempotence** | Refining either operand any number of times before comparing never changes the verdict | Catches non-monotone refinement; also the reason `refine_to(width)` is not a tolerance |
| **Step budget** | Every case runs under an explicit step budget; exhaustion on the must-complete sub-corpus is a **failure**, never a timeout | The primary detector: `sign_of(h)` at `h(α)=0` and a non-square-free defining polynomial manifest as *hangs*, not wrong answers |
| **No `Hash` without canonicalization** | Populating a map with deliberately-differently-represented equal values yields one entry, or `Hash` must not exist (compile-fail test) | No unit test catches this; it shows up as nondeterministic consumer behaviour |
| **Multiplicity is not identity** | Two roots with equal value and different source multiplicities compare `Equal`, hash identically after canonicalization, and agree on every `sign_of` | — |
| **`SqrtExt` totality** | Cross-root comparison is a total order across different radicands, agreeing with the general route | The fast path that must not be subsumed |

### 7.5 Layer 4

| Law | Statement |
|---|---|
| Hash-consing | structural equality ⇔ id equality, within one `Store` |
| Differentiation | linearity; product rule; chain rule; `diff(const) == 0`; **equals `UPoly::derivative` exactly on the polynomial subset** |
| Constant folding | value-preserving at multi-seed points; idempotent; `fold(fold(e)) == fold(e)` |
| Topological walk | children before parents; ids stable across identical construction sequences |
| Canonical bytes | byte-identical across insertion orders, thread counts, processes, feature combinations; `SCHEMA_VERSION` present |
| `rebuild_from` | round-trips to byte-identical canonical bytes in a second store |
| `is_polynomial_in` | sound both directions; `Some(p)` agrees with the expression at multi-seed points |

---

## 8. The corpus

### 8.1 Structure and lifecycle

| Layer | Contents | Lifecycle | Gate |
|---|---|---|---|
| **Regression corpus** | Every minimized counterexample ever found, plus every hand-authored known-answer instance | **Append-only.** Deletion requires a recorded justification and is counted in CI output | **100% pass, always.** A gate, not a score |
| **Generator fleet** | Versioned, seeded generators (§8.2–§8.3) | Grows; each addition bumps the fleet version | Feeds the score |
| **Benchmark corpus** | Pinned, invariant-asserted instances of the standard families | Frozen per benchmark generation | Feeds the performance scoreboard, never the correctness gate |

The regression corpus and the generator fleet live in the repository. Benchmark instances are
*generated by committed generators with committed invariant assertions* (§8.6), not committed
as data, because Gröbner instances are large.

### 8.2 Random / statistical generators

| Generator | Parameters | Targets |
|---|---|---|
| Random dense `UPoly` over ℤ | degree, coefficient bit-length, sparsity | Layer-1 arithmetic, gcd, isolation |
| Random `MPoly` | vars, degree, term count, coefficient size | Layer-1 multivariate |
| Random systems | vars, generators, degree | Gröbner |
| Random `Fp` elements and vectors | `p` near `2^63`, `2^31`, `2^27`, and tiny | Layer 0; word-boundary carries |
| Random rationals | numerator/denominator bit-length, including 1-word and just-over-1-word | `Integer` carry bugs cluster at word boundaries |
| Random `Ctx` values per ring | modulus, extension degree, defining polynomial, variable count, order | The trait-law suite (§7.1), which is vacuous without them |

### 8.3 Constructive generators — known answer by construction

| Generator | Construction | Targets |
|---|---|---|
| Known-gcd pairs | `A = G·A'`, `B = G·B'` with `gcd(A',B') = 1` enforced | gcd correctness *and* the degree half of its certificate |
| Known-factorization | `f = Π fᵢ^{eᵢ}` from a pool of certified-irreducible factors | Factorization, multiplicity, **and the Landau–Mignotte bound row** (the true factor coefficients are known, so bound validity is free) |
| Known-roots | `f = Π(x − rᵢ)` over rationals and small algebraic numbers, controlled spacing | Isolation round-trip; also the `Ord` step-distribution measurement (§4.7) before `AlgebraicReal` exists |
| Known-ideal | precompute a Gröbner basis `G`, then form generators as random combinations `Σ hᵢgᵢ` | Gröbner: the answer *and* the cofactors are known |
| Equal-value/different-representation algebraic pairs | `√2` as a root of `x²−2`, `x⁴−4`, `x⁴−4x²+4`… | `Eq`/`Hash`, equality-by-gcd |
| Deliberately-close triples | pairwise separations spanning `2^-10` to `2^-1000` | Transitivity |
| Overlap-endpoint-on-a-root | pairs where the isolating-interval overlap endpoint is a root of the gcd | §4.9 specifically. **Random generation will not find this** |
| `sign_of` at zero | `P` = the minimal polynomial of the argument, a multiple of it, or a shared-factor polynomial | §4.8 hangs |
| Single-lane-bad batches | four primes of which exactly one gives a zero pivot or a divergent lead-monomial set | §4.14 batch splitting |
| Planted-unlucky-prime instances | choose cofactors first, let `p \| res(A/G, B/G)` | §4.5 bad-prime rejection |
| Certificate transplants | a certificate minted for claim `A`, presented against claim `B` | §2.6 tether |

### 8.4 Provenance — every entry carries one

The append-only 100% gate is right for counterexamples, whose expected outcome is "does not
crash / self-certifies". It is dangerous for hand-authored known-answer instances: an expected
answer that entered from a mis-triaged Class-B disagreement, or from an oracle that was itself
wrong, becomes a permanent gate a *correct* future implementation fails, and append-only means
the corpus can only accumulate such entries (C2 §21).

> Every entry carries `provenance ∈ { constructive-generator, oracle-consensus(k systems),
> hand-computed(author, method), minimized-counterexample }`. `oracle-consensus` entries name
> the systems and versions and are **re-derivable**: a nightly job re-asks the oracles and
> flags drift. `hand-computed` entries carry the derivation. One field, and it is the
> difference between institutional memory and institutional debt.

### 8.5 Adversarial families — the ones that separate implementations

| Family | Definition | Cliff it triggers |
|---|---|---|
| **Mignotte** | `x^n − ((2^(τ/2)−1)x − 1)²` | Clustered roots. Plain Descartes falls off a cliff here; near-tangential curve contact produces exactly this |
| Nested Mignotte | — | Worse |
| **Swinnerton–Dyer** | `Π(x ± √p₁ ± … ± √p_m)`, degree `2^m`, `r ≈ 2^(m−1)` | Zassenhaus `2^r` recombination; and **no modular irreducibility certificate exists at any prime** (§4.2) |
| Gaussian-coefficient squares | `f² − 1` | Many multiplicity-two clusters |
| Wilkinson, Chebyshev, Legendre | classical | Well-separated: catches *regressions on the easy case* from an accelerated path |
| **Hexapod** | 1102 primes for a 0.00 s modular run | Reconstruction-bound. Finds CRT and rational-reconstruction bugs. In the corpus from the first modular milestone |
| **ℚ(√2, √3)** | the biquadratic tower | §4.13: no inert prime exists, so a naive `Reducible` divides by a zero divisor |
| Coincident / shared-component curve pairs | `f = h·f'`, `g = h·g'` | Identically-vanishing resultant. Must be *distinguishable*, never a silently-empty root list — the consumer fails closed here at `conics.rs:565-566` |
| Degree-drop specializations | leading coefficient vanishing at the evaluation point | Bad-specialization detection; also the pinned degenerate-resultant convention |
| Exactly-rational algebraic numbers | roots that are exactly rational arriving where an interval is expected | Interval collapse |
| Capacity-boundary monomials | total degree exactly `D`, exactly `D+1`, exponents exactly at the field max | §4.1 overflow detection at every width |
| Empty / degenerate | zero polynomial, constants, degree-0 systems, `⟨1⟩`, `⟨0⟩`, single-variable systems, one-element bases | Every layer's edge handling |

### 8.6 Benchmark-family provenance — a licensing hazard inside the verification plan

Katsura-`n` has a checkable invariant — ideal degree `2^(n−1)` under msolve's naming — and the
generator must **assert** it. Cyclic-`n` is pinned by its explicit formula. Eco-`n`, Noon-`n`
and Reimer-`n` have no such published invariant, and the superseded instruction was "pin them
to a specific generator source" — which in practice means a Singular `.lib`, an msolve test
directory, or a Groebner.jl benchmark file, all GPL-2.0. Following the verification plan
literally transcribes a generator out of a GPL test suite into an MIT repository, in the one
lane nobody looks for a licensing problem (C2 §16).

> **Every benchmark family carries a Tier-A citation** — the original paper — as a required
> field of the generator's metadata, checked by the same CI rule as `Derivation:`. Where only
> a system's test file states the system, treat the *system itself* as the published
> mathematical object, transcribe it from the paper, and assert an invariant that pins it; the
> defining recurrences for Eco-`n` and Noon-`n` are published and short. **A family with no
> Tier-A source is dropped, not pinned to a GPL file.** Commit the SHA-256 of each generated
> system so an index-convention shift is loud rather than silent.

### 8.7 Corpus tiering — decided on day 1, before the corpus has entries

Count the executions Gate 0 was specified to perform: the determinism matrix ("twice
in-process, twice cross-process, at 1/2/8 threads, across feature combinations") is at minimum
12 full-corpus runs, plus the 100% gate, plus self-certification on every call in tests — where
this document's own cost column marks the gcd certificate `O(1)×` but Sturm `>1×` at high
degree and the S-pair certificate ≈ recomputing the basis. That is roughly
`13 × corpus × (1 + certificate overhead)` per commit, against a corpus that is contractually
append-only and stocked with Mignotte, Swinnerton–Dyer and Hexapod instances that exist
*because* they are slow. By month three Gate 0 takes forty minutes and the determinism matrix
— the gate that must exist from day 1 because every other artifact depends on it — is the
first thing cut, because it is the most expensive and the least often red (C1 §12).

| Tier | Membership | Runs in | Budget |
|---|---|---|---|
| **`fast`** | Every instance by default; promoted out when it exceeds a committed per-instance time cap | Gate 0, every commit, 1 and 8 threads, in-process only, certificates on | **90 s, hard** |
| **`full`** | Everything | Gate 1, every PR; complete determinism matrix | Gate 1's budget |
| **`slow`** | Mignotte / Swinnerton–Dyer / Hexapod class, and anything promoted out of `fast` | Gate 2, nightly | unbounded |

CI **prints the tier census** and fails if `fast` exceeds its budget, so promotion is a
deliberate and visible act rather than silent gate erosion. Self-certification is a profile
flag (`cfg(resolvent_self_check)`): on in `full` and `slow`, **sampled at 10%** in `fast`.

---

## 9. The score

> **The Score is the *falsification budget*: the number of CPU-seconds of adversarial
> generation that resolvent survives with zero invariant violations, on a fixed machine,
> against a fixed, versioned generator fleet, with a fixed seed schedule.**
>
> Reported always as the pair **`(fleet_version, seconds_survived)`**. Higher is better.

Why this shape and not "percentage of tests passing":

- **A pass-rate is gamed by weakening tests.** A survival time cannot be, because weakening a
  generator is a *fleet version bump* and shows up in the reported pair; a silent weakening is
  a diff in a committed file.
- **It never saturates dishonestly.** When resolvent survives the whole budget, the correct
  response is to raise the ceiling or add a generator, recorded as an explicit **re-baseline
  event**: the fleet version increments and the score legitimately drops. A drop after a
  re-baseline is progress and is labelled as such; a drop without one is a regression.
- **It matches how the bugs arrive.** The deadly failures in §4 and §5 are found by
  generation, not by inspection.

**Anti-gaming rules, enforced in CI:**

1. The regression corpus is a **gate at 100%**, evaluated *outside* the budget. It can never
   be traded against the score.
2. Generator deletions and generator parameter-range reductions are counted and printed on
   every run.
3. The seed schedule is committed. Two runs of the same `(fleet_version, commit)` produce
   identical results — which §4.15 requires anyway.
4. **Declines are classified before they are scored.** A decline is a **failure** if (a) the
   instance is in the must-complete sub-corpus, or (b) the operation's budget was derived from
   a *proven* bound (Landau–Mignotte, Mignotte–Davenport, Hadamard, Cauchy), in which case
   exhaustion is impossible for a correct implementation and the decline is a bug. Otherwise a
   decline is a **survived instance** and is counted in the decline rate, which is a §4.16
   sharpness number with a committed ceiling. **Budget defaults are committed values; raising
   one is a diff, is counted in CI output, and requires a recorded justification** — the same
   discipline as a generator parameter reduction.
   *Why this replaces "any decline is a failure":* that rule contradicted the decline-rate
   sharpness gate, contradicted the design intent that every entry point can decline, and —
   worst — made the cheapest fix "raise the default budget until nothing declines", which
   converts declines into long runs and sanctions the hang that §4.8 calls the deadliest
   failure mode (C2 §7).
5. Sharpness rates (§4.16) are reported alongside the score. A run whose score improved while
   its Unknown rate rose is flagged.
6. **The number of distinct seeds at which each randomized certificate was checked is
   reported** (§2.3). A silent reduction from 64 seeds to 1 is otherwise invisible and
   improves every number.
7. **The mutant-rejection census is reported**: total mutants committed, mutants rejected,
   mutants *not* rejected. A row whose mutants all pass is a red gate, not a warning.

**The score does not measure performance.** That is §10, a different scoreboard with different
convergence properties. Never combine them into one number.

---

## 10. Performance gates

**These are optimization targets, not certificates.** They converge over months, not days;
they are non-monotone; they require a pinned machine; and they cannot be fanned out to
parallel agents without a shared frozen baseline. Founding constraint #3 asks that this be
said explicitly, so it is said here, in `plans/roadmap.md` §3, and in every SCORE lane brief.

### 10.1 Honesty rules

- **Compare like with like.** msolve, Maple/FGb and Groebner.jl all default to *uncertified*
  Gröbner over ℚ — Groebner.jl says so plainly ("no out of the box guarantee that the
  reconstructed basis is correct"). A certified resolvent loses those benchmarks *by
  construction*. The harness records the certification mode of both sides and **refuses to
  print a cross-mode comparison without labelling it**.
- **Do not invent numbers.** Every threshold below is a published figure carried from the
  research or a derived multiple of one. Any threshold not traceable to a citation is marked
  TBD and is set by measurement before it becomes a gate.
- **Every tuning threshold is re-derived by measurement on resolvent's own corpus, and the
  measurement is committed.** This is simultaneously a licensing rule and a correctness rule:
  a threshold lifted from a GPL source tree is both a transcription hazard and *wrong for our
  machine*.
- **Single-threaded numbers are the primary series.** Parallel numbers are a separate series;
  a parallel speedup that changes results violates §4.15 and is a bug, not a win.

### 10.2 The ladders

**Bignum — must run before Layer 0 is written, and the ladder is longer than it was.**

| Instance | Why | Target |
|---|---|---|
| `tczajka/bigint-benchmark-rs` with `dashu` 0.5.2 pinned | Every published figure used 0.4.2, one release before NTT landed in 0.4.3, so the widely-cited number is stale | Measure. A negative result strengthens the case for an optional non-default GMP feature, cheap to design now and expensive later |
| `gcd` / `gcd_ext` at 64, 256, 1k, 4k, 16k, **64k, and 256k** bits vs `rug` | dashu has Lehmer (quadratic worst case); GMP has subquadratic half-GCD — the one identified structural pure-Rust deficit | Measure. Commit `(dashu_ns, rug_ns, ratio)` medians with IQR, plus a committed decision threshold `R`: if `ratio > R` at 4k-bit `gcd_ext`, ADR-002 gains an amendment specifying the optional `backend-gmp` feature's shape |
| **`rational_reconstruct` at Hexapod's modulus size** | This is the operation that actually matters and it was not on the ladder | Measure |

**Why the ladder grew.** ADR-002 rested the bignum decision on "megabit integers appear
exactly when someone computes over ℤ or ℚ directly instead of mod several primes and
reconstructing". That is refuted three pages later by ADR-010's own numbers: Cyclic-10 needs
>2000 primes of 29 bits (≈58 000 bits), and Hexapod needs 1102 primes for a 0.00 s modular
run. Modular methods do not eliminate large integers — they **concentrate** them in the CRT
modulus and in rational reconstruction, which *is* `gcd_ext` at that size, on the *default
certified path*. The old ladder stopped at 16k bits, an order of magnitude below the regime it
existed to measure (C1 §8). Record the mitigations so they are on the record: incremental
(Garner) CRT keeps accumulation small-step; early-termination reconstruction with a doubling
modulus avoids the full-size `gcd_ext` in the common case; and a half-GCD implemented *inside*
`resolvent-int` is a self-contained, `rug`-certifiable lane — promoted from "what would
reverse this" to a planned M1 contingency with a numeric trigger.

**Gröbner over `GF(p ≈ 2^30)`, single-threaded, drl** (published context: Groebner.jl,
Maple/FGb and msolve within ~1.5× of each other; OpenF4 4–21× off; Singular's Buchberger
~150× off on Katsura-11):

| Milestone | Gate |
|---|---|
| **Correct** | Cyclic-7, Katsura-8, Eco-10 complete and agree with an external system |
| **Working** | Cyclic-8 < 60 s, Katsura-11 < 500 s, Eco-13 < 500 s (≈ Singular-Buchberger class) |
| **Competitive** | Cyclic-9 < 600 s, Katsura-13 < 900 s, Eco-14 < 600 s — **conditional on §10.3** |
| **State of the art** | within 1.5× of msolve/Maple/Groebner.jl. **Do not plan for this.** |

**Gröbner over ℚ:** Katsura-10 (54 primes), Katsura-11 (78), Cyclic-8 (54), Chandra-13 (166),
Reimer-8 (78), **Hexapod (1102 primes for a 0.00 s modular run)** — a correctness instance
disguised as a performance instance, included from the first modular milestone.

**Real root isolation:** *Correct* — degree ≤ 20 random and Mignotte instances verified against
Sturm counts. *Working* — random dense `n=1024, τ=1024` < 30 s; Mignotte `n=257, τ=14` < 60 s.
*Competitive* — random dense `n=8192, τ=8192` < 200 s; Mignotte `n=1025, τ=14` < 5 s, i.e.
Newton acceleration present and working.

**Resultants:** modular bivariate must beat the Ducos implementation by ≥ 100× on ℤ[x,y]
inputs of degree ~20 (the published figure for the technique is 400×).

**Factorization:** Swinnerton–Dyer degree 32 (`r ≈ 16`) must complete — Zassenhaus can do this.
Degree 64 (`r ≈ 32`) separates van Hoeij from Zassenhaus. Degree 256 is the "van Hoeij is
really working" mark.

**Consumer-shaped workload — the most important gap, and it is unmeasured.** Nobody knows what
degree and coefficient bit-size an arbitrary-degree arrangement engine actually produces, and
every performance requirement on the geometry path hinges on it. Settle it early (lane Y1):
generate degree 3–8 curve pairs, compute `Res_y` with the existing `QPoly`, record resultant
degree, coefficient bit-length, and the wall time of `isolate_roots` plus a `sign_of` sweep.
The same run produces the `Ord` step distribution §4.7 needs.

### 10.3 The SIMD decision, which the Competitive gate depends on

Linear algebra is 73–91% of an F4 run and msolve reports AVX2 halving it, so forgoing AVX2
forgoes roughly a 1.6–1.8× overall factor — against a "Competitive" gate set at ≈2× SOTA. The
policy (`forbid(unsafe_code)`) and the published target are within noise of each other, and
pinning stable Rust forecloses `portable_simd`, leaving `core::arch` intrinsics (unsafe) as the
only route (C1 §13). **Decide, and record the decision in an ADR:**

- **Either** name exactly one `unsafe`-permitted leaf — `resolvent-modular::simd`, with
  `#![allow(unsafe_code)]` scoped to that module, `SAFETY:` on every block, runtime feature
  detection, and a **CI-asserted bit-identical scalar fallback** (it will be bit-identical:
  these are exact integer operations, so the SIMD path is a pure speed change and cannot alter
  a value);
- **or** keep `forbid(unsafe_code)` everywhere and lower the published gate from ≈2× SOTA to
  ≈3–4×, naming AVX2 as the reason.

Publishing a target the policy forbids reaching is the one option that is not defensible.
Secondary: auto-vectorization of a sparse GF(p) `axpy` with Barrett reduction (widening
multiply plus conditional subtract) is inconsistent across LLVM versions, so the series will
level-shift on a compiler upgrade with no code change. **Treat a compiler bump as a re-baseline
event**, the same as a fleet version bump.

### 10.4 Compile time

The superseded gate — "fails on a >20% regression in total front-end time" — is measured
against the previous workspace. In Wave 0 the workspace has no algebra, so adding
`resolvent-int` is a >20% regression, and so is adding `resolvent-modular`. Every early lane
trips it against a near-empty baseline, so it is disabled within a fortnight, and a
compile-time budget disabled once never returns — which is exactly how monomorphization
explosions arrive unannounced (C1 §19).

**Corrected:** absolute per-crate ceilings, set after M1 and revised at each milestone
boundary (e.g. `resolvent-poly` front-end ≤ 20 s, workspace clean debug ≤ 90 s on the pinned
machine), **ratcheting down only**, recorded alongside the tuning thresholds. Track
`cargo llvm-lines` top-20 monomorphization counts as the leading indicator, because that moves
before wall-clock does.

### 10.5 Regression tracking

Fixed machine; results from any other machine are recorded and never gate. Report medians of
`k` runs with the interquartile range, never a single sample. **Change-point detection over the
time series, not per-run thresholds** — a per-run threshold either flaps or is set so loose it
detects nothing; compare the median of the last `k` runs against the median of the preceding
`w`, with the alert threshold calibrated per-series against that series' observed run-to-run
noise, and commit the calibration. Every run records: resolvent commit, fleet/benchmark
generation, machine id, compiler version, feature flags, thread count, certification mode. **A
performance regression does not block a correctness PR** — it opens an issue against the
SCORE lane that owns the series. Blocking correctness on a noisy number is how a project stops
merging.

---

## 11. The CI gate

### 11.1 Gate 0 — every commit, hard budget **5 minutes**

| Check | Fails on |
|---|---|
| `cargo build --workspace --all-targets` | any error |
| `cargo clippy --workspace --all-targets -- -D warnings` | any warning |
| `cargo fmt --check` | any diff |
| `cargo deny check licenses bans sources advisories` over the **published** graph, with an explicit `[licenses] allow` list and every copyleft SPDX id denied | any non-permissive crate in the published tree |
| **License-gate regression corpus**: the gate must *fail* on three planted cases — `malachite` (LGPL behind a permissive-looking pure-Rust crate), `polynomen` (GPL-3.0-only with an innocuous name), and a synthetic Apache-only crate depending on `rug` | a gate that passes what it must reject |
| **Zero dev-dependencies on every `publish = true` crate**, by `cargo metadata` | any dev-dependency on a published crate |
| **Crate-graph diff**: `cargo tree --edges normal` against the checked-in expected graph | any unexpected edge |
| **`docs-consistency`**: every fenced code block in `plans/`, `docs/decisions/`, `API.md` and this file is scanned for the headline type names (`Ring`, `Certified`, `Certificate`, `ProofKind`, `AlgebraicReal`, `MPoly`, `IsolatedRoot`, `Budget`, `Error`, `Unsupported`) and the job fails on divergent definitions | two documents defining one type differently |
| **ADR status**: every lane's `lane.toml` names its gating ADRs; a lane's test target is absent from the workspace while any gating ADR's status line does not match `^\*\*Status:\*\* Ratified` | a lane running against an unratified one-way door |
| `unsafe` inventory: `#![forbid(unsafe_code)]` on every crate except the single named allowlist entry (§10.3); every `unsafe` block carries `SAFETY:` and a bit-identical scalar fallback test | new `unsafe` outside the allowlist |
| **Determinism** on the `fast` tier: every instance twice in-process, at 1 and 8 threads | any difference in canonical bytes |
| **Golden canonical-serialization files** byte-compared; a golden change without a `SCHEMA_VERSION` bump in the same commit | drift |
| Unit tests + doc tests | any failure |
| **The `fast` regression tier at 100%**, with the tier census printed | any failure, or `fast` exceeding its 90 s budget |
| **Mutant sets** for every operation touched by the commit | any mutant the certificate fails to reject |
| Self-certification sampled at 10% (`cfg(resolvent_self_check)`) | any failure |
| No-panic fuzz smoke: 30 s per public entry point | any panic |

### 11.2 Gate 1 — every PR, target 25 minutes

Gate 0, plus:

| Check | Fails on |
|---|---|
| **The `full` regression tier at 100%** | any failure |
| **Full determinism matrix**: twice in-process, twice cross-process, at 1/2/8 threads, across feature combinations | any difference |
| **`Tuning`-matrix value equality** (ADR-012 §8) — the free naive-vs-fast agreement oracle | any value difference |
| **Trace replay equality**: `op_replay(input, &trace)` byte-identical to `op_with_trace(input)` | any difference |
| Property suite, full fleet, fixed seed schedule, fixed budget | any falsification |
| **Randomized certificates evaluated over the seed schedule** (§2.3), seed count reported | a single-seed evaluation |
| **Differential Tier 0 (sympy)** across every operation with an assignment in §6.2, **including each adapter's calibration corpus** | any Class A disagreement; any Class B not already recorded; any calibration mismatch |
| **Oracle skip census** printed; the job declares Tier 0 and fails if it is absent | absence |
| **Oracle-independence import manifest** walked | any edge from an oracle module into the lane it grades |
| Sharpness rates computed and compared against `sharpness-ceilings.toml`; a rate with no ceiling | any ceiling exceeded; any `TBD` |
| Mutant-rejection census, complete for every row in §3 | any unrejected mutant |
| `--no-default-features` and each feature individually build + test | any failure |
| MSRV build | any failure |
| Semver check against the last published version (once published) | an unintended break |
| **Score reported** as `(fleet_version, seconds_survived)` | a drop without a re-baseline marker |

### 11.3 Gate 2 — nightly

Gate 1, plus: the **`slow` tier**; **Differential Tier 1** (Singular, PARI), declared and
failing if absent; long adversarial budget (hours) on the full fleet; the **narrow-field-width
sweep** at widths `{4, 8, 16}` with the §4.1 distribution assertion and per-width
completed/overflowed counts; **oracle-consensus corpus re-derivation** with drift flagged
(§8.4); the benchmark ladder with change-point report (regressions open issues, do not block);
Miri on the monomial arena and packing crate if it can be made to terminate; fuzz targets at
extended duration with the corpus minimized and promoted.

### 11.4 Gate 3 — weekly and pre-release

Gate 2, plus Tier 2 oracles (Sage, msolve, Macaulay2); the full benchmark ladder including
SOTA-comparison instances; `cargo about` attribution regenerated and diffed; and a manual read
of §4.16's sharpness table and §4.17's oracle-independence table — the two tables whose *rows*
a machine cannot invent.

**The `Derivation:` gate, corrected.** Every non-obvious module carries a `Derivation:` line
citing **both the paper and a resolvable path into `docs/research/`**, e.g.
`//! Derivation: van Hoeij, J. Symbolic Comput. 33(5):425-445, 2002, §3; see
docs/research/notes-van-hoeij-recombination.md §2.` CI resolves the path, fails if it does not
exist, and fails if the note lacks a `Sources:` block with a tier tag per reference. A note may
serve many modules; a module may not exist without one. The previous form — cite a paper —
was satisfied by pasting a citation for a paper the author never opened, which is exactly what
an agent that worked from a source tree would do, and it was *weaker* than the posture it
claimed to mirror (C2 §15).

### 11.5 The lane completion checklist

A lane is not done until **all** of these hold. This is what an agent is graded against, and it
is deliberately mechanical.

0. **Every operation the lane lands has a committed mutant set, and the certificate rejects
   every mutant.** A mutant rejected by the type system does not count (§2.1).
1. The operation's row in §3 exists, is implemented, and its certificate is checked in the same
   test that exercises the operation.
2. The certificate does not invoke the operation it certifies, nor anything on that
   operation's call graph. Where it must, the row says INV, not CERT (§2.2).
3. Every "does not prove" cell in that row has a corresponding entry in §4 or §5, or is
   explicitly discharged in the PR description.
4. Generators for the operation are in the fleet, including at least one **constructive** and
   one **adversarial** generator.
5. If the operation has a "don't know" or "probably" outcome, its sharpness rate is measured in
   this PR, committed to `sharpness-ceilings.toml`, and is in Gate 1 (§4.16).
6. If the operation's certificate rests on a randomized argument, it is wired to the fleet seed
   schedule and its seed count is reported (§2.3).
7. If the operation has an external oracle assignment in §6.2, the adapter exists, its
   calibration corpus is committed, the normalization rule is in §6.4, and Tier 0 runs in
   Gate 1.
8. If the lane is an **oracle** for another lane, its permitted-import set is committed and
   enforced, and the module carries the `ORACLE:` marker (§4.17).
9. If the lane is SCORE-graded, its CERT/INV oracle lane is green and **frozen** first, and a
   baseline exists on the pinned machine.
10. Determinism holds (§4.15): output is byte-identical across runs, processes, thread counts,
    and feature combinations.
11. The operation takes a budget where §2.6/INV-6 says it should, and returns a typed decline
    rather than hanging or panicking (§4.19).
12. The module carries a `Derivation:` line citing a paper **and** a resolvable research note
    (§11.4).
13. Gate 1 is green.

### 11.6 The honest-verification rule

> **`cargo build` does not compile test targets, and a green result from before a signature
> change is not evidence about the code after it. Re-run the actual gate after the last edit,
> and never report a pass you did not just observe.**

This is not a style note. In an agent-built library the most common false green is a verdict
carried forward across an edit: an agent runs Gate 0, then fixes a clippy warning, then reports
Gate 0 green. Three consequences are mechanical: (a) CI is the only authority — a local run is
evidence, not a verdict; (b) `cargo test --workspace --all-targets` (or the named gate script)
is the command, never `cargo build`; (c) a PR description that claims a gate is green records
the commit hash the gate ran against, and CI fails the claim if the hash is not the PR head.

---

## 12. Bootstrapping — the harness exists before the algebra

The only order in which this document is self-consistent, because every later item depends on
an earlier one being able to grade it.

1. **License gate + two-category workspace + Gate 0 skeleton**, with the three planted cases
   *observed failing*. Fully automatic verdict, zero algebra, cheap now and expensive later.
2. **The canonical serializer and its `SCHEMA_VERSION`** — a *blocking* deliverable, not a
   parallel one. The determinism harness, the corpus format, and every oracle adapter all
   serialize polynomials, and the architecture already says the serializer lives in
   `resolvent-base` "so every crate and every oracle adapter shares one implementation". Three
   agents fanning out here write three serializers and two get rewritten (C2 §9a).
3. **`resolvent-base`** — the trait tower with `Ctx`, `Sign`, `Verdict`, `Certified`,
   `Certainty`, `ProofKind`, `Error`/`Unsupported`/`Budget`, and the serializer from (2). It is
   the most-inherited artifact in the project, it is marked a one-way door, and in the
   superseded plan it appeared in **no lane, no wave, and no milestone's "Lands" list** (C2
   §9d). It is a single blocking lane, and the whole trait block is typechecked with real
   impls for `Fp` and `Integer` before anything above it starts (C1 §1).
4. **Determinism and canonical-bytes harness**, over (2).
5. **Corpus, generator interface, seed schedule, minimizer, score reporter, provenance field,
   tier census** — before there is anything to generate for.
6. **Tier-0 oracle adapter (sympy)** with the text protocol, the calibration corpus, and the
   triage classifier.
7. **`resolvent-int` + the `rug` oracle in a `publish = false` crate** — the first thing with a
   real verdict. The extended bignum ladder (§10.2) runs here.
8. **`Fp`** — exhaustively certifiable, the ideal first agent lane.
9. **`UPoly` over ℤ + the naive `O(n²)` reference**, in the same crate.
10. **Sturm + Descartes.** At this point two independently written implementations grade each
    other and the oracle loop is closed. Everything after this is filling in §3.

**The real critical path**, stated plainly: not "the ADR freeze" alone, but *the ADR freeze,
**and** the canonical serializer, **and** `resolvent-base`* — in that order, all three global,
all three previously either unstaffed or mislabelled as parallel.

**Blocking experiments must not require the artifact they gate** (C1 §11). Three did:

| Experiment | Gates | Synthetic harness that replaces it |
|---|---|---|
| Inline packed monomials vs ids-plus-arena "on a realistic S-pair queue workload" | P1/P2/P3 — the whole multivariate trunk | Record an S-pair operation trace (`lcm-query`, `divisibility-query`, `insert`) from a ~200-line throwaway Buchberger over GF(p) with `Vec<u32>` exponents on Katsura-6/Cyclic-6, discarded afterwards, and replay it against both term representations |
| Cofactor multiplier "on Katsura-8 / Cyclic-7" | whether `groebner_certified` exists | Buchberger-with-cofactors at Katsura-6/7 **over ℚ**, reporting the multiplier as a function of instance size plus the **reconstruction prime count** (§4.4) |
| `AlgebraicReal` mutability: "sort 10³ degree-8 algebraic numbers" | A1, i.e. all of M3 | ~300 lines behind one trait over M2's `UPoly<Integer>`, with roots built as `Π(x − rᵢ)`. It needs only `cmp`, `refine`, and polynomial sign evaluation — **not** the production isolator |

---

## 13. What this document does not settle

Stated as what would settle them, not guessed at.

1. **The cofactor reconstruction multiplier** (§4.4). Measure primes and wall time to
   reconstruct the cofactor system over ℚ on Katsura-6/7 with Buchberger, before
   `groebner_certified` is committed to as the regression oracle.
2. **The exact hypotheses of the Idrees–Pfister–Steidel theorem after Noro–Yokoyama's
   correction** (§4.3). Fetch Noro & Yokoyama (ICMS 2014) and *Math. Comp. Sci.* 11(3), 2017.
   This decides whether the fast Gröbner path can ever return `Proved` without cofactors.
3. **The `Ord` step distribution on the M4 corpus** (§4.7). Measurable now with
   `Π(x − rᵢ)`-constructed roots; must be settled by M3 because it is in every signature.
4. **Where Sturm stops being affordable** (§4.18). Measure `d*`, commit it, and record the
   CERT → DIFF degradation in the lane's status rather than discovering it as a slow CI job.
5. **Whether `lll-rs` (MIT) is usable at van Hoeij precision.** The lattice has dimension ~`r`
   with entries of size `p^k` exceeding twice the Landau–Mignotte bound — potentially thousands
   of bits. Run it on a Swinnerton–Dyer degree-64 lattice; if it fails, LLL becomes its own
   lane and van Hoeij's schedule doubles. Note LLL is fully self-certifying (§3.4), which makes
   it a good lane either way.
6. **Whether terms are `(MonomialId, C)` into a ring-owned arena or `(PackedMon, C)` inline.**
   The ownership rule is settled (ADR-020); the term type is not, and it is decided by the
   recorded-trace microbenchmark in §12 before the multivariate trunk starts. **The certificate
   in §3.2 is the same either way; the type it is written against is not.**
7. **Whether `BulkOps` survives in the trait tower.** `API.md` INV-14 retains it; C1 §20 argues
   it re-exposes a Tier-M kernel as a trait method and should be deleted in favour of free
   functions over concrete types in `resolvent-modular`. **No certificate in §3 depends on the
   answer** — this is recorded here so the divergence is visible to the `docs-consistency` gate
   rather than resolved silently in code.
8. **Whether `feanor-math` (MIT) is usable as an in-process dev-oracle** (§6.2). Classify its
   `#![feature(...)]` uses; file the upstream missing-LICENSE issue regardless.
9. **Whether the `f64` enclosure semantics can be pinned as a committed conformance vector
   file now.** ADR-018 lists it as a future measurement; it is not a measurement, it is a
   specification — a few hundred `(exact value, expected (lo, hi))` pairs including subnormals,
   powers of two, exact halves, and the largest finite double, in a `publish = false` crate any
   consumer can run against its own interval type. Writing it costs an afternoon and it makes
   the hardest item of the deferred integration decision checkable before anyone commits to it
   (C2 §14).
10. **Whether `SqrtExt` carries a public generic parameter.** `plans/architecture.md` §5.4 has
    `impl<T: Ordered + Field> SqrtExt<T>`; the API notes had it monomorphic. ADR-018 forbids a
    generic parameter on `AlgebraicReal` by name and is silent about `SqrtExt`, which is the
    type it also requires stay first-class. It is the same one-way door for the same reason;
    decide it in the same place.

---

## 14. Register of corrections applied

What changed from `plans/verification.md`, and the finding that forced it. Nothing in this
list is editorial.

| # | Correction | Source |
|---|---|---|
| 1 | **Mutant sets are mandatory** for every certificate; the mutant-class table; the mutant census in CI and in the score | C2 §1 |
| 2 | **Rule C**: a certificate may not invoke what it certifies. Both gcd certificates rewritten around Bézout witnesses; the ℤ[x] modular gcd's degree half now runs against a *certified* modular gcd | C2 §3 |
| 3 | **Rule S**: randomized certificates are graded over the fleet seed schedule; at one fixed seed they are golden tests and grade INV. Seed count reported | C2 §5 |
| 4 | **Sharpness ceilings get a ratchet** (`sharpness-ceilings.toml`, lower freely, raise with justification, `TBD` fails Gate 1). Per-operation `Proved` floors committed as `1.0` on day one | C2 §8 |
| 5 | **Oracle independence gets a gate**: module-import manifest + `ORACLE:` marker + lane-checklist item | C2 §4 |
| 6 | **The narrow-width overflow sweep becomes a distribution assertion**; a width at which zero instances complete is a failed sweep | C2 §6 |
| 7 | **Decline classification replaces "any decline is a failure"**, which pushed agents to inflate budgets until declines became hangs | C2 §7 |
| 8 | **S-pair verifier must enumerate all pairs** and may not consult a pair-elimination criterion; criteria mutant required | C2 §10a |
| 9 | **FGLM certificate gains the lex Buchberger criterion and the staircase-dimension count**; two-way reduction alone proves generation, not the GB property | C2 §10b |
| 10 | **No `Equal` from exhausting the separation bound**; the bound row downgraded to INV+PROP with a degree-≤6 brute-force test | C2 §11 |
| 11 | **The prime registry gets a certificate** (independent sieve + golden hash) and a danger-list entry; it was the modular architecture's undetected root of trust | C2 §12 |
| 12 | **Landau–Mignotte / Hadamard / Cauchy bounds get a catalogue row and a generator**; a too-small bound feeds van Hoeij's coarse-factorization failure through an uncertified input | C2 §13 |
| 13 | **CRT gains pairwise-distinct moduli and a modulus-≥-bound assertion**; factorization gains a pairwise-non-associate clause; the Poisson-product resultant route is marked M8, not M4 | C2 §20 |
| 14 | **Corpus entries carry provenance**; `oracle-consensus` entries are re-derivable and drift-checked | C2 §21 |
| 15 | **Adapters ship calibration corpora**; round-trip proves nothing about whether the oracle read the same object | C2 §22 |
| 16 | **`Derivation:` cites a paper *and* a resolvable research note**; the previous form was trivially satisfiable and weaker than the posture it mirrored | C2 §15 |
| 17 | **Benchmark families require a Tier-A citation**; "pin them to a specific generator source" instructed transcription from GPL test suites | C2 §16 |
| 18 | **Trait laws are quantified over `Ctx`**; `Liftable: Reducible`; the trait block is typechecked with real impls before the freeze | C1 §1 |
| 19 | **Precedence rule + `docs-consistency` CI gate + machine-readable `Status: Ratified`**, replacing "is the ADR merged" | C1 §2, C2 §2/§19 |
| 20 | **The two Gröbner modes do not share a reducer.** G3 gets a naive dense `u32` Gaussian-elimination oracle; the cofactor prototype's criterion becomes reconstruction primes and wall time | C1 §3 |
| 21 | **`Reducible::Image: CommutativeRing` with `Result<_, BadPrime>`**; multi-modular over split factors is a lane, not an instantiation; ℚ(√2,√3) added to the corpus | C1 §4 |
| 22 | **Monomial ids are content-derived**; no tie-break may consult id order; id assignment asserted identical at 1 and 8 threads | C1 §5 |
| 23 | **`AlgebraicReal` budget invariance**: the refinement cache may change work done, never the result including a decline; budgets derived from the separation bound. Telemetry, evidence and `BudgetTick` excluded from canonical bytes | C1 §6, §16 |
| 24 | **`Ord` attainability is a measured, committed number** with a diagnostic ceiling and a benchmarked `try_cmp` sibling; settled by M3, not left open | C1 §7 |
| 25 | **The bignum ladder extends to 64k and 256k bits** and gains a `rational_reconstruct` microbenchmark at Hexapod's modulus size; reconstruction *is* the large-integer regime | C1 §8 |
| 26 | **Divisibility/lcm/gcd/degree are order-free on `raw`** and are asserted identical across orders; they are the hottest loop in the library | C1 §10 |
| 27 | **Three blocking experiments get synthetic harnesses** so the freeze does not deadlock | C1 §11 |
| 28 | **The corpus is tiered `fast`/`full`/`slow` on day 1**, with a printed census and a hard `fast` budget; self-certification becomes a profile flag sampled at 10% in `fast` | C1 §12 |
| 29 | **The SIMD decision is forced**: one audited `unsafe` leaf with a bit-identical fallback, or a lowered Competitive gate. A compiler bump is a re-baseline event | C1 §13 |
| 30 | **Batched lanes get `inv_batch -> Result<_, LaneMask>`, a split driver, and constructed corpus instances**; componentwise equality is complete for arithmetic and silent on control flow | C1 §14 |
| 31 | **L4's catalogue holds M7's exit-gate line**; `Simplifier`, `RuleSet`, the rewriter, simplex integration and rational-function normalization are post-v1 | C1 §15 |
| 32 | **Certificate shape unified** on `API.md`'s claim-tethered form; ProofKind unified by union; unforgeability and tether are tests | C1 §16 |
| 33 | **Monomial arena capacity and memory model become a certificate** (`Unsupported::MonomialArenaFull`, `arena_stats()` with a committed ceiling) | C1 §17 |
| 34 | **Published crates have zero dev-dependencies**, asserted in CI; the `rug` oracle lives in a `publish = false` crate | C1 §18 |
| 35 | **Compile-time gate becomes absolute per-crate ceilings** with `cargo llvm-lines` as the leading indicator | C1 §19 |
| 36 | **Isolation's evidence is the sign-variation witness, not the interval**; `ProofKind::RootCount`; tier F → tier C with the constant documented | `API.md` §5.3 (X1 §1.1) |
| 37 | **`IsolatedRoot { value, multiplicity }`** as a named struct rather than a bare tuple, preserving ADR-014's safety property without forcing consumers to thread a parallel structure | C2 §14 |
| 38 | **`resolvent-base` and the canonical serializer are named as blocking bootstrapping items**; the real critical path is stated | C2 §9 |
| 39 | **ℤ/n for composite `n` is in scope** — Hensel lifting to `p^k` is arithmetic modulo a composite and M1's exit gate requires it | C2 §2 |
| 40 | **The honest-verification rule is a section**, with the three mechanical consequences | founding constraint #4; house rule |

---

## Sources

Plan documents: `plans/verification.md` (superseded by this file), `plans/architecture.md`,
`plans/roadmap.md`, `plans/api-shape.md` (superseded by `API.md`), `API.md`.
Decisions: `docs/decisions/ADR-001…020`.
Research: `docs/research/{prior-art-and-licensing,consumer-requirements,
algorithms-and-representation,consumer-sinbad,consumer-cadabra2,consumer-solverang,
challenge-generality,challenge-evidence}.md`.
Critiques, authoritative: `docs/research/critique-engineering.md` (C1),
`docs/research/critique-plan.md` (C2).
Source specification: `/home/dev/projects/IDEAS-crates.md` §4.

Every external citation — benchmark figures, prime counts, published thresholds, algorithm
complexities — is carried from those documents, where it is sourced. **No number in this file
is new.**

Consumer code read directly for grounding (context only; resolvent depends on nothing local):
`/home/dev/projects/arrangements/crates/lazy-exact/src/roots.rs:316-322, 437-439, 576-586`,
`/home/dev/projects/arrangements/crates/lazy-exact/src/bernstein.rs:135-152`,
`/home/dev/projects/arrangements/crates/arrangements/src/geoms/conics.rs:259-270, 565-566,
600-618`.

Machine state verified 2026-07-31: no CAS oracle installed; `sympy` 1.14.0 importable via
pyenv `python3`; `singular` 4.4.1.p5-11 and `pari` 2.17.4-1 available in Arch `extra`;
`cargo` 1.96.0.
