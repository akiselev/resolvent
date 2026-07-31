# The verification spine

Status: **plan, for ratification.** Lane D2 deliverable 1 of 2. Companion:
`plans/roadmap.md`.

Inputs: `docs/research/prior-art-and-licensing.md` (R1),
`docs/research/consumer-requirements.md` (R2),
`docs/research/algorithms-and-representation.md` (R3),
`docs/research/consumer-sinbad.md` (E1).

This document is not advisory. It is the specification of the harness that decides
whether any agent's work on resolvent is done. Founding constraint #3 says resolvent
will be built primarily by AI agents graded by oracles; that only works if every lane
has a verdict function that runs without a human in the loop. **This document defines
those verdict functions, names the places where no verdict function exists, and says
what stands in for one there.**

Two claims frame everything below.

- **The catalogue of what *is* self-certifying (§2) is the smaller half of the value.**
  §3 — where certificates run out — is where agents silently produce wrong code, and it
  is the section to read first if you only read one.
- **A fail-closed verdict is trivially satisfiable and therefore is not a verdict.** An
  implementation that returns `Unknown` always, or `Probable` always, or declines on
  budget always, passes every soundness certificate in §2. Every three-valued or
  two-tier output in resolvent therefore needs a *sharpness gate* alongside its
  soundness certificate (§3.13). This is the single most common way an agent-built
  library passes its own tests and is useless.

---

## 0. The verdict vocabulary

Five kinds of verdict. Every lane in `plans/roadmap.md` is tagged with exactly one
*primary* kind. The tags are load-bearing: they determine how a lane is scheduled, how
many agents can work it in parallel, and what "done" means.

| Kind | Definition | Convergence | Fan-out safety |
|---|---|---|---|
| **CERT** | The operation emits data alongside its answer that *proves* the answer, and checking that data is strictly cheaper than recomputing it. A failure is a bug in resolvent with certainty. | Days. Monotone. | Safe to fan out wide. |
| **INV** | No emitted proof, but the answer must satisfy structural invariants checkable without a second implementation (degree bounds, disjointness, parity, closure properties). Weaker than CERT: invariants can all hold on a wrong answer. | Days. | Safe, but never sufficient alone. |
| **DIFF** | Graded by disagreement with an independent implementation — either a second internal implementation, or an external system driven as a subprocess. Disagreement is a *signal*, not a verdict: it needs triage (§4.4). | Weeks. | Safe once the adapter and normalization exist. |
| **PROP** | Graded by property tests over generated inputs: algebraic laws, order axioms, round-trips, metamorphic relations. Failure is a bug; success is evidence, not proof. | Weeks; asymptotic. | Safe. |
| **SCORE** | The success criterion is *a number to optimize*, not a certificate to check. Wall time, memory, instance ceiling, Unknown rate, Proved rate, primes needed. | **Months. Non-monotone. No completion condition.** | **Unsafe to fan out.** Needs a pinned machine and a frozen baseline first. |

Two rules follow immediately and are not negotiable:

1. **Self-certification runs first and is the primary gate; oracles are secondary.** A
   self-certificate failure is a bug in resolvent with certainty. An oracle disagreement
   may be a normalization difference. Do not let a lane brief treat them as equivalent
   signals (R1 §5).
2. **A SCORE lane may not start until the CERT/INV reference implementation it is graded
   against exists and is frozen.** Do not start F4 before Buchberger passes; do not start
   ANewDsc before plain Descartes passes; do not start van Hoeij before Zassenhaus passes
   (R3 §10).

### 0.1 The certainty tag is part of the public API

R3 §3.1 forces this and it belongs in the type system, not in documentation. D1 has since
ratified the shape (ADR-010, `plans/api-shape.md` §4): every routine whose correctness
depends on a modular reconstruction, a stabilization heuristic, or a probabilistic check
returns `Certified<T>` carrying a `Certainty`:

```
enum Certainty { Proved(ProofKind), Probable(ProbableReason) }
enum ProofKind { Identity, Divisibility, Cofactor, Enclosure, DegreeBound }
```

with `Certificate<C: Claim>` as the witness object: private fields, crate-private mint,
carrying the claim it certifies, checkable via `certifies(claim) -> bool`.

`Probable` is legal — Gröbner over ℚ needs it, and every competing system defaults to it
(R3 §3.1) — but it must be *visible in the type* and the default path must be `Proved`.
D1's tiering by verification cost (cheap checks run by default with a separately-named
`*_unchecked` escape; expensive ones get a separate entry point) is the right shape and
this document's §2 columns say which tier each operation is in.

**The verification-side obligation D1's tiering creates:** the rate at which each lane
returns `Proved` on its corpus is a tracked number with a committed floor (§3.13). A
tiering scheme makes `Probable` cheap to reach, which makes the floor load-bearing rather
than decorative.

---

## 1. The three structural constraints the harness imposes on the architecture

These are places where the *verification plan* dictates an *engine* design decision.
They are flagged to D1 explicitly because retrofitting any of them is a rewrite.

### 1.1 Gröbner cofactors must be *retained*, not recomputed

The spec's Gröbner certificate is ideal membership via the stored cofactor
representation `f = Σ hᵢ gᵢ`. That is the only cheap general route to the hard half of
Gröbner correctness, `⟨G⟩ ⊆ I` (R3 §3.4): reducing the input generators to zero proves
`I ⊆ ⟨G⟩`, and checking S-pairs proves `G` is *a* Gröbner basis, but neither shows the
basis generates the *input* ideal rather than a larger one.

Retention is not an afterthought. In F4 the cofactors are extra columns carried through
every elimination, and R3 §3.4 warns of large constant factors and a real memory risk.
The consequence for the architecture:

- The reduction engine must be able to run **with or without** a cofactor block, chosen
  at call time, sharing the same matrix code. Two separate reduction implementations
  would defeat the cross-check in §4.3.
- The cofactor block's representation (dense vs sparse, over ℚ vs over ℤ with a content
  factor) is a Layer-1 decision that must be made when `MPoly` is designed.
- SMT NRA needs exactly this data for external proof production, and R2 §5.2 records
  that consumer's own doc calling proof production non-retrofittable
  (`IDEAS-crates.md:196-199`). **The same data that grades resolvent internally is the
  data that consumer emits externally.** Design it in once.

→ **D1:** cofactor retention is an architectural requirement on the Gröbner engine and
on `MPoly`'s row representation.

**Unsettled, and it gates the plan:** nothing in the literature R3 surveyed measures the
time and memory multiplier of cofactor tracking through F4. R3's open question stands.
Prototype cofactor tracking on Katsura-8 / Cyclic-7 and measure the multiplier *before*
`groebner_certified` is committed to as the regression oracle. If the multiplier is
above ~20× in memory, the certified mode cannot be the workhorse oracle and the plan
falls back to Buchberger-with-cofactors on small instances only.

### 1.2 Every entry point takes a work budget and can decline

E1 §5.4 derives this from a real consumer contract: declines are typed and distinct from
errors, and a decline always means "the next rung may succeed"
(`sinbad/crates/tiered-core/src/rung.rs:11-26`). It is also what makes the harness able
to treat a hang as a *failure* rather than a timeout, which §3.5 shows is the primary
detector for the deadliest `AlgebraicReal` bugs.

Concretely: `isolate_roots(p, interval, budget) -> Result<..., ResolventError>` where
the error enum distinguishes `BudgetExhausted { .. }` from `MalformedInput { .. }`. A
routine that can only run to completion or panic cannot be graded, cannot be fuzzed, and
cannot be a rung in a consumer's ladder.

→ **D1:** budget is a parameter on every Layer-2 and Layer-3 entry point, not a global.

### 1.3 Output must be a pure function of mathematical content

E1 §3 D1/D2/D6: bitwise reproducibility across runs *and across thread counts*; modular
methods must be deterministic or explicitly seeded (never `thread_rng()`); canonical
serialization must be independent of interning order, node ids, arena addresses, and
insertion history, because downstream artifacts are content-addressed by it.

This is a verification constraint before it is a consumer constraint: **a
non-deterministic library cannot have a regression corpus.** Every golden file, every
minimized counterexample, every change-point baseline assumes that the same input
produces the same bytes.

→ **D1:** prime selection is a deterministic sequence or takes an explicit `u64` seed.
No `HashMap` iteration in any output-affecting path. `BTreeMap` in every
ordering-visible position. Canonical bytes carry an explicit schema version.

---

## 2. The self-certification catalogue

For each operation: the certificate (what is computed alongside the answer), what it
proves, **what it does not prove**, and its cost relative to the operation. This table
is the development harness — an operation is not implemented until its certificate is
implemented and checked in the same test.

Cost column convention: `O(1)×` means a small constant fraction of the operation itself;
`~1×` means comparable to recomputing; `>1×` means the check dominates.

### 2.1 Layer 0 — coefficient rings

| Operation | Certificate / verdict | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| `Integer` add/sub/mul | Inverse op round-trip; agreement with `rug::Integer` (dev-dep oracle, precedent: dashu's own `fuzz/`, R1 §1.4) | Value correctness on the tested instance | Nothing about untested magnitudes; carry bugs cluster at word boundaries — generator must target them | `O(1)×` | CERT |
| `Integer` exact division `a/b` | `q·b == a` | Exactness | That non-exact input was rejected — needs a separate `divides` test | `O(1)×` | CERT |
| `Integer` `divrem` | `a == q·b + r`, `0 ≤ r < |b|` | Full correctness (the pair is unique) | — | `O(1)×` | CERT |
| `gcd(a,b) = g` | `g|a`, `g|b`, and `gcd(a/g, b/g) == 1` | Full correctness | — | `~1×` (the coprimality check is another gcd) | CERT |
| `gcd_ext` → `(g,u,v)` | `u·a + v·b == g`, plus the gcd certificate, plus `|u| ≤ |b/2g|`, `|v| ≤ |a/2g|` | Full correctness including minimality | — | `O(1)×` | CERT |
| `Rational` arithmetic | Canonical-form invariant: `gcd(num,den)==1`, `den > 0`; field axioms by PROP | Representation validity | Value correctness — needs the round-trip and oracle | `O(1)×` | CERT+PROP |
| `Q::from_f64` | `to_f64(from_f64(x)) == x` bit-exactly for all finite `x`; `None` for non-finite | Lossless lift, fail-closed | — | `O(1)×` | CERT |
| `Fp` arithmetic, word primes | **Exhaustive** over all `(a,b)` for every prime `p < 2^10`, against `i128` reference; then random against `i128` for `p < 2^63`; `a·a⁻¹ == 1` for all units | Complete for small `p`; strong for large | Nothing about the SIMD/bulk path — that gets its own exhaustive test | small `p`: exhaustive; large: `O(1)×` | CERT |
| `Fp` bulk/vector ops | Componentwise agreement with the scalar path on random vectors, including tails and misaligned lengths | Equivalence to the certified scalar path | — | `O(1)×` | CERT |
| Batched tuple ring `Zp4 = [u32;4]` | Componentwise agreement with 4 independent scalar `Fp` runs | Exact equivalence — a **free, complete** oracle | Nothing about the speedup, which is a SCORE | `~1×` | CERT |
| `Zn`, composite modulus | Same as `Fp` plus explicit unit/zero-divisor classification; `is_unit(a) ⇔ gcd(a,n)==1` | Correctness | — | `O(1)×` | CERT |
| `GF(p^k) = Fp[x]/(f)` | Field axioms; `x^(p^k) == x` for every element (Frobenius closure); modulus irreducibility certified by §2.4's finite-field test | Field structure | — | `~1×` | CERT |
| Prime generation | Miller–Rabin with the **known deterministic witness sets** for `n < 2^64` (a proof, not a probable-prime test) | Primality, deterministically | — | `O(1)×` | CERT |
| "Good prime" predicate | Algorithm-specific: `p ∤ lc`, degree preserved, `p ∤ disc` — each directly checkable | The stated condition only. *Unlucky* primes are a separate hazard (§3.4) | That the prime is lucky | `O(1)×` | CERT |
| CRT combine | Result `≡ rᵢ (mod pᵢ)` for every `i`; result in the symmetric range | Full correctness of the combination | Nothing about whether enough primes were used (§3.4) | `O(1)×` | CERT |
| Rational reconstruction `→ n/d` | `n ≡ d·a (mod M)`, `gcd(n,d)==1`, `|n|,|d| ≤ √(M/2)` | Uniqueness of the reconstruction *given* it exists | That `n/d` is the intended answer — that needs the modulus bound or a verification step | `O(1)×` | CERT |

### 2.2 Layer 1 — representation and polynomial arithmetic

| Operation | Certificate / verdict | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| Monomial encode/decode | `decode(encode(v)) == v` for random `v` up to the capacity bound | Round-trip fidelity | Behaviour at and past the capacity boundary — generator must sit exactly on it | `O(1)×` | CERT |
| Monomial compare | Order axioms as PROP: totality, antisymmetry, transitivity, well-ordering on ℕⁿ, and **multiplicative compatibility** `a < b ⇒ a·u < b·u`; plus agreement with a naive `Vec<u32>` comparator for each supported order | Order correctness | — | `~1×` vs naive | CERT+PROP |
| Monomial multiply | `decode(a⊗b) == decode(a) + decode(b)` elementwise; guard-bit overflow **always** detected (§3.1) | Arithmetic correctness | — | `O(1)×` | CERT |
| `divmask` filter | One-sided: `divmask` says "not divisible" ⇒ genuinely not divisible. False positives permitted and expected | Soundness of the negative answer | Nothing about filter quality — a mask that always says "maybe" is sound and useless (§3.13 sharpness gate: track the false-positive rate) | `O(1)×` | CERT+SCORE |
| Monomial interning **or** inline packed keys | Injectivity: equal exponent vectors ⇔ equal key/id; hash multiplicativity `h(u)+h(v) == h(u·v)` | Term-identity consistency | — | `O(1)×` | CERT |
| — note | **The term type is unresolved.** `ADR-008` specifies an interned arena with `(MonomialId, Coeff)` terms; `plans/api-shape.md` L1-4 specifies inline packed keys with `(PackedMon, C)` terms and no global interner. The *certificate above is the same either way*; the *type it is written against is not*. See `plans/roadmap.md` §2.5 contradiction 2 | | | | |
| `UPoly` add/sub/mul | `(a·b)/b == a`; `deg(a·b) == deg a + deg b` over an integral domain; agreement with a naive `O(n²)` reference; **evaluation homomorphism** `eval(a·b, x) == eval(a,x)·eval(b,x)` at random points in a large `GF(p)` | Strong; the evaluation check is a Schwartz–Zippel argument with failure probability `deg/p` | — | `O(1)×` | CERT |
| `MPoly` arithmetic | Same, plus representation invariants: terms strictly descending in the order, no zero coefficients, no duplicate monomials | Correctness + canonical storage | — | `O(1)×` | CERT |
| Kronecker substitution | Round-trip against direct multiplication on the same inputs | Equivalence | — | `~1×` | CERT |
| Taylor shift `p(x+a)` | `shift(shift(p,a),−a) == p`; `eval(shift(p,a), x) == eval(p, x+a)` at random `x` | Correctness | — | `O(1)×` | CERT |
| Content / primitive part | `p == content(p) · primitive(p)`; `content(primitive(p)) == 1` | Full correctness | — | `O(1)×` | CERT |
| Associate normalization | Idempotence: `norm(norm(p)) == norm(p)`; and `p ~ q ⇔ norm(p) == norm(q)` verified against an independent associate test | Canonicity — this is what lets a consumer's `eq_curves` be an `==` (R2 §1.1) | — | `~1×` | CERT |
| `divrem` | `a == q·b + r`, `deg r < deg b` | Full correctness | — | `O(1)×` | CERT |
| Pseudo-division | `lc(b)^(deg a − deg b + 1) · a == q·b + r`, `deg r < deg b` | Full correctness | — | `O(1)×` | CERT |
| `RecursiveView` | Every operation agrees with the same operation on a materialized recursive copy | View consistency | — | `~1×` | CERT |

### 2.3 Layer 2 — GCD, resultants, isolation

| Operation | Certificate / verdict | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| Univariate `gcd(A,B) = H` over ℤ | (a) `H\|A` and `H\|B` by exact division; (b) `deg H == deg gcd(A mod p, B mod p)` for one certified-good prime `p` | **Complete.** (a) gives `H \| G`; (b) with `deg gcd mod p ≥ deg G` gives `deg H ≥ deg G`; together `H = G` up to a unit (R3 §3.2) | — | `O(1)×` — two divisions + one modular gcd | CERT |
| — the half people forget | Divisibility **alone** accepts any common divisor. The degree half is mandatory. | | | | |
| `gcd_ext` / Bézout over `F[x]` | `u·A + v·B == H`; `deg u < deg B − deg H`; `deg v < deg A − deg H` | Complete, with minimality | — | `O(1)×` | CERT |
| Square-free decomposition (Yun) | `Π fᵢ^i == f`; the `fᵢ` pairwise coprime; each `gcd(fᵢ, fᵢ') == 1` | **Complete** | — | `~1×` | CERT |
| Resultant `Res(f,g)` | Four independent checks: (a) cofactors `u,v` with `u·f + v·g == Res` — the subresultant PRS produces them; (b) degree bound `deg_x Res_y ≤ deg_y(f)·deg_x(g) + deg_y(g)·deg_x(f)`; (c) `Res == 0 ⇔ deg gcd(f,g) > 0`, cross-checked against the gcd lane; (d) Poisson product `Res = lc(f)^{deg g} Π g(αᵢ)`, exact for small degree over a splitting field | Very strong. (a) proves `Res ∈ ⟨f,g⟩`; (b)+(c) bound and locate it | **(a) alone does not prove the value is *the* resultant** — any ideal element passes. The complete verdict is the two-implementation cross-check (§4.3) | (a) `O(1)×`, (b) free, (c) `~1×`, (d) expensive | CERT+INV |
| Subresultant chain | Each `Sᵢ ∈ ⟨f,g⟩` with cofactors; **specialization property**: the chain of `Ψ(f), Ψ(g)` equals `Ψ(chain)` for random good ring maps `Ψ` (random primes, random evaluation points) — a free, strong, per-instance self-check; degree sequence is a valid subresultant degree sequence; last nonzero element is a gcd up to content | Very strong; the specialization check is essentially a randomized proof | Bad specializations must be excluded first or the check is vacuous | `O(1)×` per specialization | CERT |
| Real root isolation | (a) Sturm's **exact** distinct-root count in each interval matches; (b) intervals pairwise disjoint and ordered; (c) `f(lo) ≠ 0 ≠ f(hi)`; (d) Descartes variation is exactly 1 on each returned interval; (e) all intervals inside the Cauchy bound; (f) Σ multiplicities equals the degree of the square-free-corrected input; (g) round-trip: build `f = Π(x − rᵢ)` from known values and check they come back | **Complete for correctness** at any degree Sturm can reach. (a) is the strongest single check in Layer 2 | Nothing about interval *quality* — see §3.13; nothing at degrees where Sturm is too slow (§3.14) | (a) `>1×` at high degree — this is why it is an oracle, not the production path | CERT |
| Separation bound | The bound is *valid*: for every pair in the corpus, `|α − β| ≥ bound`; and every verdict reached under the bound equals the verdict reached by unbounded refinement | Validity of the bound as used | Tightness — a bound of 0 is valid and useless (sharpness gate) | `~1×` | CERT+SCORE |
| `Bernstein` range enclosure | (a) endpoint Bernstein coefficients equal endpoint values exactly; (b) the coefficient hull contains the true range, checked by isolating the roots of `p − c` for `c` at the hull bounds; (c) a `Certain(s)` verdict never contradicts the true sign | Soundness | **Nothing about the Unknown rate** — the fail-closed direction is free (R2 R14; `arrangements/crates/lazy-exact/src/bernstein.rs:135-152` returns `Unknown` rather than guessing). Sharpness gate mandatory | `O(1)×` | CERT+SCORE |

### 2.4 Layer 2 — factorization

| Operation | Certificate / verdict | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| Factorization over `GF(p)` | (a) multiply back; (b) **complete irreducibility test** per factor: `f` of degree `d` is irreducible over `GF(p)` iff `x^(p^d) ≡ x (mod f)` and `gcd(x^(p^(d/q)) − x, f) == 1` for every prime `q \| d` | **Complete.** Both halves are decidable and cheap over a finite field | — | `O(1)×` | CERT |
| Hensel lifting `p → p^k` | `Π gᵢ ≡ f (mod p^k)` exactly; each `gᵢ ≡` its `mod p` original; the `gᵢ` pairwise coprime mod `p` | Complete for the lift | Nothing about whether `k` was large enough for recombination — that is the Landau–Mignotte bound's job and is a separate assertion | `O(1)×` | CERT |
| Factorization over ℤ — **half 1** | Multiply the factors back and compare to the input. One polynomial multiplication | The factorization *is* a factorization | **Irreducibility of the factors — not at all.** A recombination bug that merges two true factors produces `f = g·h` with `g` reducible and this check passes. An oracle that only multiplies back **silently accepts a coarse factorization** (R3 §3.3) | `O(1)×` | CERT |
| Factorization over ℤ — **half 2** | Exhibit a prime `p` with `p ∤ lc(fᵢ)`, `p ∤ disc(fᵢ)`, such that `fᵢ mod p` is irreducible of degree `deg fᵢ` | Irreducibility over ℚ, when such a `p` exists | **The certificate does not always exist.** Polynomials whose Galois group contains no `n`-cycle — Swinnerton–Dyer being canonical — factor nontrivially modulo *every* prime. See §3.2 | one modular factorization per factor | PARTIAL |
| LLL reduction | Lovász condition and size-reduction condition on the output basis, plus `det` preservation, plus "output lattice = input lattice" via unimodularity of the transform | **Complete** — LLL's output conditions are directly checkable | Nothing about the *quality* beyond the LLL guarantee | `O(1)×` | CERT |
| van Hoeij recombination | (a) half 1 above; (b) agreement with Zassenhaus for `r ≤ 20`; (c) the algorithm's own termination witness — the reduced lattice basis consists of 0/1 vectors partitioning `{1..r}` at sufficient Hensel precision | Strong but not complete | See §3.2 | (b) `>1×` and only feasible for small `r` | PARTIAL |

### 2.5 Layer 2 — Gröbner

| Operation | Certificate / verdict | Proves | Does **not** prove | Cost | Kind |
|---|---|---|---|---|---|
| `groebner_certified(F) → (G, cofactors)` — **`I ⊆ ⟨G⟩`** | Every `f ∈ F` reduces to zero modulo `G` | The easy inclusion | — | `\|F\|` normal forms over ℚ, with full coefficient blowup | CERT |
| — **`G` is a Gröbner basis** | Every S-pair of `G` reduces to zero modulo `G` (Buchberger's criterion) | The basis property | — | ≈ recomputing the basis | CERT |
| — **`⟨G⟩ ⊆ I`** | Stored cofactors: each `gⱼ = Σᵢ hᵢⱼ fᵢ`, checked by multiplication and addition | The hard inclusion — **the only cheap general route** | — | check is `O(1)×`; *producing* the cofactors is the expensive part (§1.1) | CERT |
| `groebner(F)` — fast modular mode | Graded against `groebner_certified` on every regression instance; plus lead-monomial majority vote across primes; plus reconstruction stabilization | Nothing by itself. Returns `Probable` | — | free (the cross-check is a byproduct) | SCORE + DIFF |
| FGLM change of order | The lex basis reduces every element of the drl basis to zero **and** vice versa; both ideals have the same dimension and degree | Ideal equality, hence correctness of the conversion | — | `~1×` | CERT |
| Ideal saturation | `I : f^∞ ⊇ I`; and `f` is a non-zerodivisor on the quotient; membership cross-check against a Rabinowitsch-trick computation in one extra variable — a second, independent route | Strong | — | `>1×` | DIFF (internal) |

### 2.6 Layer 3 — algebraic numbers

The property suite **is** the verdict function for this layer. R3 §8.3 specifies it and
it is reproduced here as a gate, not as a suggestion.

| Property | Statement | Why it is the canary |
|---|---|---|
| **Trichotomy** | Exactly one of `a < b`, `a == b`, `a > b` | Catches the F3 failure (returning an ordering when the equality certificate merely *failed*) |
| **Transitivity** | `a ≤ b ∧ b ≤ c ⇒ a ≤ c`. Generator must produce triples where two pairs are within `2^-1000` | The named canary. Equality-by-tolerance (F2) and failed-certificate-as-inequality (F3) are both intransitive and *only* transitivity catches them |
| **Antisymmetry** | `a ≤ b ∧ b ≤ a ⇒ a == b` | — |
| **Equality is an equivalence relation**, and equal elements agree on `sign_of(h)` for every `h` | Reflexive, symmetric, transitive | Catches representation-dependent equality (`x²−2` vs `x⁴−4`) |
| **Sort stability** | Sorting a shuffled list yields the same sequence of equality classes every time | Catches state-dependence in the refinement cache — the failure mode interior mutability introduces |
| **Enclosure consistency** | If `cmp(a,b) == Less` then the `f64` enclosures must not contradict it; disjoint enclosures must agree with the exact verdict | Catches outward-vs-nearest rounding-direction bugs, otherwise invisible (F10) |
| **Isolator consistency** | `isolate_roots(f)` returns roots strictly ascending under `cmp`, and the count matches Sturm | Cross-lane |
| **Refinement idempotence** | Refining either operand any number of times before comparing never changes the verdict | Catches non-monotone refinement |
| **Step budget** | Every property test runs under an explicit step budget. Exceeding it is a **failure**, not a timeout | **The primary detector.** F5 (`sign_of(h)` when `h(α)=0`) and F1 (non-square-free defining polynomial) manifest as *hangs*, not wrong answers, and a hang in a library is worse than a wrong answer because it is undebuggable in production |
| **No `Hash` without canonicalization** | Populating a `HashMap` with deliberately-differently-represented equal values must yield one entry, or `Hash` must not exist | F7: no unit test catches this; it shows up as nondeterministic consumer behaviour |
| **Multiplicity is not part of identity** | Two roots with equal value but different source multiplicities compare `Equal` | F9 |

Additional Layer-3 certificates:

| Operation | Certificate | Kind |
|---|---|---|
| `AlgebraicReal` construction | Defining polynomial is square-free (enforced by Yun at construction; constructor returns `Result`) and has exactly one real root in `(lo,hi)` with `poly(lo) ≠ 0 ≠ poly(hi)` — checked by Descartes variation == 1 | CERT |
| `refine` | Monotone: the new interval is contained in the old; the invariant `poly(lo) ≠ 0 ≠ poly(hi)` survives; a midpoint hit collapses the interval to a point and the number becomes exactly rational (F4) | CERT |
| `sign_of(P)` | Zero-ness settled algebraically first via `gcd(poly, P)` plus a sign-change certificate, *then* the refinement loop. Verified by: agreement with a high-precision numeric evaluation as a smoke test, and exact agreement with materializing `P(α)` as its own algebraic number | CERT+DIFF |
| Radical-tower sign `Σ cᵢ(α)√hᵢ(α)` | **Agreement with the materialized `AlgebraicReal` path.** The two routes share almost no code: one squares repeatedly over ℚ(α), the other computes a ℚ-minimal polynomial by resultant and isolates. This is a free internal differential oracle and it is a strong one (R2 D3) | DIFF (internal) |
| `SqrtExt` `a + b√r` | Sign by squaring, cross-checked against the general `AlgebraicReal` route; total-order axioms including cross-root comparison | CERT+DIFF |
| `rational_between(α, [β…])` | The returned rational compares strictly greater than `α` and strictly less than every `βᵢ`, verified with `cmp_rational` | CERT |

### 2.7 Layer 4 — expression DAG

| Operation | Certificate | Kind |
|---|---|---|
| Hash-consing | Injectivity: structurally equal terms get the same id; structurally different terms get different ids. Store is an owned value, never ambient (E1 §5.6) | CERT |
| `diff` / `diff_with` | **On the polynomial subset, the derivative must equal Layer 1's `UPoly::derivative` exactly.** That is an exact cross-layer oracle covering the chain/product/power rules. For transcendental symbols, verify by evaluating both `d/dx f` and a high-order finite-difference of `f` at random rationals — *approximate, and flagged as such* | CERT (polynomial) + DIFF (transcendental) |
| Constant folding | Value preservation at random points; idempotence | CERT |
| `is_polynomial_in(&syms)` | If it returns `Some(p)`, then `p` and the expression agree at random points; if `None`, a witness node that is not a ring op over the given symbols | CERT |
| `walk_topological` | Every node appears after its children; node ids stable across identical construction sequences | CERT |
| Canonical bytes | Byte-identical across insertion orders, across thread counts, across `--features` combinations, across two processes. Golden files, versioned by an explicit schema id (E1 §3 D6) | CERT |
| Rewrite rules (if an e-graph adapter lands) | Soundness by evaluating both sides at random points over a large `GF(p)`; **"is the simplification good" has no certificate** | CERT (soundness) + SCORE (quality) |

---

## 3. Where certificates run out

This is the section that matters. Each entry names an operation or failure mode whose
correctness is **not** self-certifying, states why, and states what grades it instead.

### 3.1 Exponent-field overflow — every certificate passes on a wrong answer

**The failure.** Wraparound in a packed exponent field silently yields a *correct Gröbner
basis of a different ideal*. Ideal membership passes. Cofactor checks pass. S-pair
reduction passes. Differential testing against Singular fails, but only if the instance
is small enough to run there. This is the single most dangerous silent failure in the
library (R3 §1.3).

**What grades it.**
1. Guard-bit SWAR detection on **every** multiply, compiled into release builds — not a
   `debug_assert`. One AND and one compare per word. Multiply returns `Result`.
2. A dedicated test mode that runs the *entire* Gröbner corpus at a deliberately narrow
   field width (4-bit fields) and asserts that every instance either completes with the
   same answer as the wide run or reports overflow. **Zero silent divergences permitted.**
3. The widen-and-restart driver is itself tested: the answer after restart must equal the
   answer from starting at the wide width.
4. A property test on the packed multiply: `decode(a ⊗ b) == decode(a) + decode(b)` OR
   the multiply returned `Err`. Never a third outcome.

Because exponents only grow, restart-at-wider-width loses bounded work — which is what
demotes field width from a one-way door to a tuning knob (R3 §1.3). The *detection* is
what makes that true, so the detection is load-bearing.

### 3.2 Factorization coarseness — multiply-back is not enough

Stated in §2.4 and repeated here because it is the exact shape of bug an agent produces:
a recombination that merges two true factors passes the product check. Over ℚ the
irreducibility certificate does not always exist.

**What grades it.**
- The modular irreducibility certificate where one exists (§2.4), with the **rate** of
  successful certification tracked as a number: if the rate falls, either the corpus got
  harder or the implementation got coarser, and both need a look.
- Degree-pattern consistency across many primes — a *necessary* condition only.
- Differential against PARI `factor` and Singular `factorize` (§4.2), compared as
  **multisets of associate-normalized factors**.
- The Swinnerton–Dyer ladder as the adversarial generator: `Π(x ± √p₁ ± … ± √p_m)` has
  degree `2^m` and `r ≈ 2^(m−1)` modular factors, is irreducible over ℚ, and has no
  modular irreducibility certificate at any prime. A coarse implementation returns a
  nontrivial factorization here; a correct one returns the input. **A single instance
  from this family is worth more than a thousand random ones.**
- Zassenhaus grades van Hoeij for `r ≤ 20`; van Hoeij grades nothing below it.

### 3.3 `⟨G⟩ ⊆ I` in the fast Gröbner mode

Without cofactors there is no cheap general certificate. Arnold's Hilbert-function
argument removes the obligation only for *homogeneous* ideals; Idrees–Pfister–Steidel
extended it to the non-homogeneous global-order case and Noro–Yokoyama then showed that
theorem needs an additional assumption (R3 §3.4).

**Unsettled and load-bearing.** R3 could not obtain the precise statement (both papers
paywalled). Until it is obtained, the plan assumes the fast mode **cannot** return
`Proved` without cofactors. Settle it by fetching Noro & Yokoyama, ICMS 2014, and
*Mathematics in Computer Science* 11(3), 2017, and recording the exact hypotheses in
`docs/research/`.

**What grades it meanwhile.** Agreement with `groebner_certified` on the entire
regression corpus; lead-monomial majority vote across primes; reconstruction
stabilization; differential against Singular and msolve. All of these are `Probable`.

### 3.4 Unlucky primes and unlucky evaluation points

A modular run can be internally consistent and wrong. CRT combination certifies the
*combination*, not the *inputs*. Rational reconstruction certifies uniqueness, not
intent.

**What grades it.**
- Where a bound exists — Landau–Mignotte for factors and GCDs, Hadamard for resultants
  and determinants — use it and be **deterministic**. Compute the bound, use enough
  primes to exceed `2·bound`, and the answer is provably right. This is a CERT and it
  should be the default path.
- Where only stabilization is available, stabilization is a heuristic and must be closed
  by a verification step. A lane brief that says "iterate until stable" without naming
  the closing check is underspecified.
- Brown's minimal-degree rule for GCD images (only images of minimal degree seen are
  kept) and the analogous rule for evaluation points; both are cheap and both must be
  present.
- A dedicated adversarial generator: instances constructed so that a *specific* small
  prime is unlucky, verifying the implementation discards it. Construct by choosing the
  cofactors first and letting `p | res(A/G, B/G)`.
- The **Hexapod** instance (R3 §9.3): 1102 primes for a computation whose single modular
  run takes 0.00 s. It is a pure reconstruction-bound instance and it is where CRT and
  rational-reconstruction bugs surface. Include it from the first modular milestone.

### 3.5 Silent hangs

R3 §8.2 F5 and F1: `sign_of(h)` never terminates when `h(α) = 0` unless zero-ness is
settled algebraically first, and refinement stalls forever on a non-square-free defining
polynomial. **A wrong implementation of `AlgebraicReal` hangs; it does not return the
wrong answer.**

**What grades it.** Step budgets on every property test, with budget exhaustion counted
as a **failure**, never as a skip or a timeout. Plus a targeted generator: for each
`AlgebraicReal α` in the corpus, call `sign_of(P)` with `P = ` the minimal polynomial of
`α` (so the answer is exactly zero), and with `P = ` a multiple of it, and with `P` sharing
a factor with `α`'s defining polynomial. These are the instances that hang.

### 3.6 Intransitivity from a *failed* certificate

The gcd sign-change test can fail spuriously — an overlap endpoint happening to be a root
of `g`. The correct response is refine-and-retry. An implementation that returns
`Less`/`Greater` on a failed certificate is intransitive in exactly the same way as
equality-by-tolerance (R3 §8.2 F3). The consumer gets this right at
`arrangements/crates/lazy-exact/src/roots.rs:578-592`.

**What grades it.** The transitivity property test with a generator that deliberately
places overlap endpoints on roots of the gcd. This will not be found by random
generation — it needs a constructive generator (§5.2).

### 3.7 `Eq`/`Hash` inconsistency

Equal values can carry different defining polynomials. A cheap `Hash` corrupts `HashMap`s
nondeterministically and no unit test catches it (R3 §8.2 F7).

**What grades it.** A property test that builds a `HashMap` keyed by algebraic numbers
generated as *deliberately different representations of the same value* and asserts the
map has one entry per distinct value. If `Hash` is not implemented, the test asserts the
absence of the impl (a compile-fail test), so the decision cannot silently regress.

### 3.8 Multiplicity semantics

The consumer today reads intersection multiplicity off a resultant root's multiplicity
and degrades to `CrossingKind::Unknown` in the ambiguous case
(`arrangements/crates/arrangements/src/geoms/conics.rs:600-618`). Whether resultant-root
multiplicity *is* intersection multiplicity in general has no certificate — it is a
mathematical claim about the generic case with known exceptions.

**What grades it.** A known-answer corpus: curve pairs with hand-computed intersection
multiplicities, including the documented ambiguous case (two distinct common points over
one abscissa with parallel gradients). This is a *human-authored* corpus and it is small;
that is acceptable because it is the only thing that can grade a definitional question.

### 3.9 Curve analysis (fiber structure, branch matching)

R2 R13/D7: the largest component with zero counterpart in the prior art. Its outputs —
per-interval branch counts, branch-index-to-root maps — have only weak invariants:
counts consistent across adjacent open intervals, Bézout-style bounds, branch matching is
a bijection where it should be.

**What grades it.** In descending order of strength:
1. **Topological consistency**: the number of branches above adjacent open intervals must
   change only at critical abscissas, and by an amount consistent with the local
   structure of the critical fiber.
2. **Differential against a second route**: compute the same fiber counts by isolating
   the roots of `f(α, y)` at a *rational* witness abscissa strictly inside each interval
   (cheap, uses only the univariate lane) and compare with the analysis' claim. **This is
   a genuinely independent route and it should be built as the oracle before the fast
   path.**
3. Differential against Sage/CGAL-class systems on small instances — expensive, Tier 2.
4. A hand-authored corpus of curves with known topology (nodal cubics, cusps, tangential
   pairs, curves with vertical asymptotes and with vertical components).

This lane's verdict is materially weaker than every other Layer-2 lane. Size it
accordingly (see `plans/roadmap.md` §5).

### 3.10 Non-determinism and thread-count dependence

No algebraic certificate exists. **What grades it:** run every corpus instance twice in
one process, twice in two processes, at 1 / 2 / 8 threads, and with each supported
feature combination, and compare canonical bytes. Any difference is a failure. This test
is cheap, catches `HashMap` iteration order, address-dependent hashing, `thread_rng`,
and work-partition-dependent accumulation order, and it must exist from day 1 because
every other regression artifact depends on it (§1.3).

### 3.11 Panics and unbounded work on adversarial input

E1 §3 D3: library code must not panic on any input-dependent path. **What grades it:** a
fuzz target per public entry point over `arbitrary`-generated inputs, run under a
panic-hook that fails the test; plus a budget test asserting that every entry point
respects its budget parameter on an instance chosen to exceed it. Degree overflow,
exponent-packing overflow, coefficient blowup, and non-finite `f64` input are all
`Result`, never abort.

### 3.12 Canonical-serialization drift

A resolvent upgrade that changes canonical form is a re-key event for every downstream
content-addressed artifact (E1 §3 D6). **What grades it:** golden files, committed,
compared byte-for-byte, with an explicit schema version. Changing a golden file requires
bumping the schema version in the same commit; CI fails a golden change without a version
bump.

### 3.13 Sharpness gates — the fail-closed trivial-satisfaction problem

**This is the most important entry in this section.** Every soundness certificate in §2
is satisfied by a maximally conservative implementation:

| API | Trivially-sound useless implementation | Sharpness gate |
|---|---|---|
| `Bernstein::sign_over -> Uncertain<Sign>` | always `Unknown` | Unknown rate on the corpus below a tracked ceiling; **zero** Unknowns on the sub-corpus of instances with a clear sign |
| `Certainty` | always `Probable` | `Proved` rate on the corpus; per-operation floors (GCD, resultant, factorization-product, isolation must be 100% `Proved`) |
| `Result<_, BudgetExhausted>` | always decline | Decline rate at the standard budget; zero declines on the "must complete" sub-corpus |
| `divmask` | always "maybe divisible" | False-positive rate; and the divisor-query benchmark (the 10–20× in R3 §1.6 comes from this filter working) |
| Separation bound | return 0 | Ratio of the returned bound to the observed separation, tracked as a distribution |
| Isolating intervals | return `(−cauchy, +cauchy)` — one interval per root is *not* required by disjointness alone if there is one root | Interval width relative to the separation bound; and the disjointness+count checks together do pin this down, but the *width* does not |
| `CrossingKind` | always `Unknown` | Unknown rate on the intersection corpus |

**Rule: any API with a "don't know" or "probably" outcome ships with a tracked rate, and
the rate is a CI-visible number with a ceiling.** Without this, an agent optimizing for
a green suite converges on an implementation that is sound and worthless.

### 3.14 Oracle independence — correlated failure between "independent" checks

Several of the strongest verdicts in §2 are two-implementation cross-checks. They are
only as strong as the code the two implementations *do not* share. This table must be
maintained and audited whenever a shared helper is introduced:

| Cross-check | Genuinely independent | **Shared, therefore correlated** |
|---|---|---|
| Descartes isolation vs Sturm count | subdivision logic, variation counting vs sign-sequence counting | `UPoly` arithmetic; `divrem`; **and Sturm's chain is a PRS, so a subresultant/PRS bug corrupts Sturm *and* the resultant lane simultaneously** |
| Ducos subresultant PRS vs modular evaluation–interpolation resultant | pseudo-division and exact division vs `GF(p)` arithmetic, CRT, interpolation | `UPoly`/`RecursiveView` storage |
| Bareiss/Bézout determinant vs both of the above | dense linear algebra | `Integer` arithmetic only — **this is the most independent of the three and is worth building for exactly that reason** |
| Radical-tower sign vs materialized `AlgebraicReal` | squaring ladders over ℚ(α) vs resultant + isolation | `UPoly` arithmetic, `Integer` |
| `groebner_certified` vs `groebner` | cofactor tracking, ℚ arithmetic vs modular + tracing | matrix construction, symbolic preprocessing, monomial layer — **substantial sharing; this cross-check is weaker than it looks** |
| Batched `Zp4` vs 4× scalar `Fp` | SIMD/tuple path vs scalar path | the reduction algorithm itself if both call the same `mulmod` |

**Consequence:** the monomial layer, `UPoly`/`UPoly` arithmetic, and `Integer` are
*common mode* for nearly every internal cross-check. They therefore need external differential
testing (§4) and exhaustive/oracle-backed testing (§2.1, §2.2) more than anything else in
the library, because a bug there is invisible to the internal oracle structure.

### 3.15 "Correct but useless"

An isolator that returns valid but enormous intervals; a Gröbner basis that is right
after 14 hours; a resultant that is correct with 500-bit coefficients computed the slow
way. No certificate detects any of these. Only §6 does.

---

## 4. Differential oracles

### 4.1 Rules

- **Nothing links.** Every external oracle is driven as a subprocess over a text
  protocol. Singular, PARI, msolve, Sage, Macaulay2 are GPL; linking them into a
  permissive library is not permitted at all (R1 §5, §6). FLINT is LGPL and *could* be
  linked, but the uniform subprocess rule has no exception process, which is what makes
  it enforceable. If FLINT's in-process speed later proves necessary, it goes in a
  `publish = false` crate behind a non-default feature that no published crate depends
  on — and `flint-sys`'s missing repo LICENSE file (R1 §5) must be resolved first.
- **The workspace has exactly two crate categories.** `publish = true` crates, whose
  dependency graph `cargo-deny` gates against a permissive allowlist; and
  `publish = false` crates (`oracles/`, `fuzz/`, `bench/`) that may carry LGPL
  dev-dependencies and shell out to GPL binaries. No third category, no per-crate
  exception.
- **A missing oracle is a counted `SKIP`, never a pass.** The harness prints a skip
  census. A CI job declares which oracle tier it requires; if an oracle in that tier is
  absent, **the job fails** rather than silently reducing coverage.
- **Text protocol only.** Each oracle adapter emits a canonical S-expression/JSON form of
  the input and parses a canonical form back. Oracle-specific parsing never appears in a
  test body.

### 4.2 Install tiers and per-operation assignment

Verified on this machine on 2026-07-31: **none of Singular, PARI, SageMath, Maxima,
FLINT, or Macaulay2 is installed**; `python3 -c "import sympy"` reports 1.14.0 via pyenv
and works today. `pacman -Si` confirms `singular` 4.4.1.p5-11 (11.38 MiB download /
59.25 MiB installed) and `pari` 2.17.4-1 (8.79 MiB / 28.69 MiB) are available in `extra`.
Macaulay2 is AUR-only.

*(Caution for future agents: `command -v gp` succeeds on this machine because `gp` is a
shell alias for `git push`. Detect PARI by invoking `gp -q -f` on a known expression and
checking the output, not by probing `$PATH`.)*

| Tier | Contents | Install cost | When it runs |
|---|---|---|---|
| **0** | `sympy` via `python3` | zero — present today | Every PR. Non-negotiable. |
| **1** | `singular`, `pari` | ~88 MiB installed, one `pacman -S` | Nightly; and locally recommended |
| **2** | `sagemath` (371 MiB, ~60 transitive deps), `msolve` (source build), `macaulay2` (AUR) | expensive, CI box only | Weekly and pre-release |
| **dev-link** | `rug` as a dev-dependency bignum oracle | crate only | Every PR, in-process (precedent: dashu's own `fuzz/` uses `rug::Integer`) |

| Operation | Primary oracle | Secondary | Note |
|---|---|---|---|
| `Integer`/`Rational` arithmetic | `rug` (dev-dep, in-process) | PARI | The only oracle worth linking, and only as a dev-dep |
| Univariate gcd, square-free | sympy (`gcd`, `sqf_list`) | PARI, Singular | Tier 0 covers it |
| Resultant, **subresultant chain** | sympy `subresultants` | PARI `polresultant`, Singular `resultant` | sympy gives the **whole PRS chain**, which is the intermediate data the lane must match — more useful than the final resultant alone |
| Univariate factorization over ℤ/ℚ/GF(p) | PARI `factor` | sympy `factor_list`, Singular | PARI is the number-theory specialist |
| Multivariate factorization | Singular `factorize` | sympy, PARI | |
| Real root isolation | PARI `polrootsreal` | sympy `real_roots`, `CRootOf` | PARI returns certified intervals |
| Algebraic-number comparison | sympy `CRootOf`, `minimal_polynomial` | PARI `nfinit`/`polredabs`, Sage `QQbar` | Sage's `QQbar` is richest and is the 371 MiB dependency |
| Gröbner (drl, lex, elimination) | Singular `std`/`groebner` | msolve, Macaulay2 | Singular is the literature's reference and needs no Python layer |
| FGLM / change of order | msolve | Singular `fglm` | msolve is the pipeline resolvent copies architecturally |
| Everything, fallback | SageMath | — | Install once on the CI box; never a gate for local dev |

**Unsettled:** whether Singular or msolve is the better Gröbner oracle in practice
(R1 §8.5). Start with Singular because it is one command away; build msolve once and
compare on a shared corpus before committing the CI box to it.

### 4.3 Normalization — the comparison must be semantic, not textual

Two valid Gröbner bases are still two valid Gröbner bases. Every comparison below is
specified as an equivalence test, not a string or structure comparison.

| Operation | How disagreement is defined |
|---|---|
| **Gröbner basis** | *Never* compare generator lists. Compare semantically, in this order: (1) the two lead-monomial ideals are equal; (2) every element of `A` reduces to zero modulo `B` and vice versa — this is ideal equality and it is the real test; (3) as a fast pre-filter only, the reduced+monic+order-sorted bases are byte-equal. Additionally compare the Hilbert series when both systems can produce it, which catches "same ideal, different order convention". **Order convention is the number-one false disagreement**: pin the variable order and the term order explicitly in the emitted input, and assert the oracle echoes them back. |
| **Factorization** | Compare **multisets** of associate-normalized factors (content removed, positive leading coefficient), with multiplicities. Unit factors are stripped and compared separately. |
| **GCD** | Normalize both to primitive with positive leading coefficient, then `==`. |
| **Resultant** | Sign conventions differ under argument swap by `(−1)^(mn)`. Emit arguments in a pinned order and accept `±Res` only with the sign rule applied explicitly — never accept "up to sign" unconditionally, because that hides genuine sign bugs. Compare after content normalization. |
| **Subresultant chain** | Compare the *degree sequence* first (that is the structural content), then each `Sᵢ` up to the known scalar convention. Conventions genuinely differ between sources; pin resolvent's and document the conversion in the adapter, not in the test. |
| **Root isolation** | *Never* compare interval endpoints — different algorithms produce different intervals. Compare: (1) the count of real roots in the query interval; (2) a bijection where each of ours overlaps exactly one of theirs; (3) refine both sides by `k` steps and assert the intervals stay nested. |
| **Algebraic number comparison** | Compare *sign verdicts* and *orderings*, never representations. `x²−2` and `x⁴−4` are the same number. |
| **Minimal polynomial** | Compare after monic normalization over ℚ, or primitive-with-positive-lc over ℤ. Degrees must match exactly — a degree mismatch is a real bug (a non-minimal polynomial), not a convention. |

**Free internal differential oracles** (no external system, no install cost, strongest
signal-to-noise because both sides are ours and both self-certify):

| Pair | Covers |
|---|---|
| Ducos subresultant PRS ↔ modular evaluation–interpolation ↔ Bareiss/Bézout determinant | Resultants — three routes, and R3 §6.3 notes they share almost no code |
| plain Descartes ↔ Sturm ↔ ANewDsc | Root isolation; Sturm gives the *exact* count so it grades the others automatically |
| Zassenhaus ↔ van Hoeij (`r ≤ 20`) | Factorization recombination |
| Buchberger ↔ F4 ↔ `groebner_certified` ↔ `groebner` | Gröbner |
| radical-tower sign ↔ materialized `AlgebraicReal` | Layer 3's hot path |
| batched `Zp4` ↔ 4× scalar `Fp` | Modular batching |
| narrow-field-width run ↔ wide-field-width run | Exponent packing (§3.1) |
| `UPoly` fast paths ↔ naive `O(n²)` reference | Layer 1 |

**Build the oracle side first, every time.** Sturm exists to grade Descartes; Buchberger
exists to grade F4; Zassenhaus exists to grade van Hoeij; the naive `O(n²)` multiply
exists to grade the fast one. None of them will ever be the production algorithm. R3 §10
states the sequencing rule and it is a CI-enforced rule here: **a SCORE lane's CI job
does not exist until its oracle lane is green and frozen.**

### 4.4 Triage and minimization

Every disagreement runs through this pipeline automatically before a human sees it.

1. **Classify by self-certificate.** Re-run resolvent's own certificate on the instance.
   - *Self-certificate also fails* → **Class A: resolvent bug, certain.** Highest
     severity. Goes straight to the regression corpus.
   - *Self-certificate passes, oracle disagrees* → **Class B: normalization,
     convention, or oracle limitation.** Proceed to step 2.
2. **Minimize.** Delta-debug while the disagreement persists, in this order (cheapest
   structural reduction first): drop terms → reduce coefficient bit-length by halving →
   reduce degree → reduce variable count → reduce the number of generators → shrink the
   query interval. Minimization is fully automatic and its result is what gets recorded;
   an unminimized counterexample is not accepted into the corpus.
3. **Re-classify the minimized instance.**
   - Both sides self-certify and the answers are genuinely different mathematical objects
     → **normalization bug in the adapter.** Fix the adapter, add the instance as an
     adapter test.
   - Both sides self-certify and the answers are the same object under a convention we
     had not pinned → **unspecified convention.** Write it into an ADR and into the
     normalization table above. This is the outcome that improves the plan.
   - The oracle is wrong or out of range → record, report upstream if warranted, and mark
     the instance `oracle-limitation` so it stops being reported.
4. **Record.** Every triage outcome — including "not a bug" — is appended to the corpus
   with its class, its minimized form, the oracle version, and the resolvent commit. The
   corpus is the institutional memory; nothing is triaged twice.

---

## 5. The property-test corpus and the score

### 5.1 Corpus structure

Three layers, with different lifecycles and different gate semantics.

| Layer | Contents | Lifecycle | Gate |
|---|---|---|---|
| **Regression corpus** | Every minimized counterexample ever found, plus every hand-authored known-answer instance | **Append-only.** Deletion requires a recorded justification and is counted in CI output | **100% pass, always.** A gate, not a score |
| **Generator fleet** | Versioned, seeded generators (§5.2) | Grows; each addition bumps the fleet version | Feeds the score |
| **Benchmark corpus** | Pinned, degree-checked instances of the standard families (§6) | Frozen per benchmark generation | Feeds the performance scoreboard, never the correctness gate |

The regression corpus and the generator fleet live in the repository. Benchmark instances
are *generated by committed generators with committed assertions on their invariants*
(§6.1), not committed as data, because Gröbner instances are large.

### 5.2 The generator fleet

Random generation finds shallow bugs. The deep bugs in this library all have structure,
so most of the fleet is *constructive*.

**Random / statistical**

| Generator | Parameters | Targets |
|---|---|---|
| Random dense `UPoly` over ℤ | degree, coefficient bit-length, sparsity | Layer-1 arithmetic, gcd, isolation |
| Random `MPoly` | vars, degree, term count, coefficient size | Layer-1 multivariate |
| Random systems | vars, generators, degree | Gröbner |
| Random `Fp` elements and vectors | `p` near `2^63`, `2^31`, `2^27`, and tiny | Layer 0; word-boundary carries |
| Random rationals | numerator/denominator bit-length, including 1-word and just-over-1-word | `Integer` carry bugs cluster at word boundaries |

**Constructive — known answer by construction**

| Generator | Construction | Targets |
|---|---|---|
| Known-gcd pairs | `A = G·A'`, `B = G·B'` with `gcd(A',B') = 1` enforced | gcd correctness *and* the degree half of its certificate |
| Known-factorization | `f = Π fᵢ^eᵢ` from a pool of certified-irreducible factors | Factorization, including multiplicity |
| Known-roots | `f = Π(x − rᵢ)` over rationals and small algebraic numbers, with controlled spacing | Isolation round-trip |
| Known-ideal | `I = ⟨g₁…⟩` for a precomputed Gröbner basis `G`, then generators formed as random combinations `Σ hᵢgᵢ` | Gröbner: the answer is known and the cofactors are known |
| Equal-value/different-representation algebraic pairs | `√2` as a root of `x²−2`, of `x⁴−4`, of `x⁴−4x²+4`… | F7 `Eq`/`Hash`, equality-by-gcd |
| Deliberately-close triples | three algebraic numbers with pairwise separations spanning `2^-10` to `2^-1000` | Transitivity (F2/F3) |
| Overlap-endpoint-on-a-root | pairs constructed so the isolating-interval overlap endpoint is a root of the gcd | F3 specifically. Random generation will not find this |
| `sign_of` at zero | `sign_of(P)` where `P` is the minimal polynomial of the argument, or a multiple, or shares a factor | F5 hangs |

**Adversarial families — the ones that separate implementations**

| Family | Definition | Cliff it triggers |
|---|---|---|
| **Mignotte** | `x^n − ((2^(τ/2)−1)x − 1)²` | Clustered roots. Plain Descartes falls off a cliff here (RS >600 s at `n=1025`; ANewDsc 0.7 s). Near-tangential curve contact produces exactly this |
| Nested Mignotte | — | Worse |
| **Swinnerton–Dyer** | `Π(x ± √p₁ ± … ± √p_m)`, degree `2^m`, `r ≈ 2^(m−1)` | Zassenhaus `2^r` recombination; and no modular irreducibility certificate exists at any prime (§3.2) |
| Gaussian-coefficient squares | `f² − 1` | Many multiplicity-two clusters |
| Wilkinson, Chebyshev, Legendre | classical | Well-separated: catches *regressions on the easy case* from an accelerated path |
| **Hexapod** | R3 §9.3 | Reconstruction-bound: 1102 primes for a 0.00 s modular run. Finds CRT and rational-reconstruction bugs |
| Coincident / shared-component curve pairs | `f = h·f'`, `g = h·g'` | Identically-vanishing resultant. Must be *distinguishable*, never a silently-empty root list (consumer fails closed here: `conics.rs:565-566`, `sine_radical.rs:1158-1161`) |
| Degree-drop specializations | leading coefficient vanishing at the chosen evaluation point | Bad-specialization detection |
| Exactly-rational algebraic numbers | roots that are exactly rational, arriving where an interval is expected | F4 interval collapse |
| Capacity-boundary monomials | total degree exactly `D`, exactly `D+1`, exponents exactly at the field max | §3.1 overflow detection |
| Empty / degenerate | zero polynomial, constants, degree-0 systems, `⟨1⟩`, `⟨0⟩`, single-variable systems | Every layer's edge handling |

**Pinning rule for benchmark families:** conventions differ by an index shift and that
silently changes which instance is being benchmarked. Katsura-`n` has a checkable
invariant — ideal degree `2^(n−1)` under msolve's naming — and the generator must
**assert** it. Cyclic-`n` is pinned by its explicit formula. Eco-`n`, Noon-`n`, and
Reimer-`n` have no such published invariant, so pin them to a specific generator source
and commit the SHA-256 of the generated system (R3 §9.2).

### 5.3 The score

> **The Score is the *falsification budget*: the number of CPU-seconds of adversarial
> generation that resolvent survives with zero invariant violations, on a fixed machine,
> against a fixed, versioned generator fleet, with a fixed seed schedule.**
>
> Reported always as the pair **`(fleet_version, seconds_survived)`**. Higher is better.

Why this shape and not "percentage of tests passing":

- **A pass-rate is gamed by weakening tests.** A survival time cannot be gamed by
  weakening a test, because weakening a generator is a *fleet version bump* and shows up
  in the reported pair. A silent weakening of an existing generator is a diff in a
  committed file.
- **It never saturates dishonestly.** When resolvent survives the whole budget, the
  correct response is to raise the ceiling or add a generator. Both are recorded as an
  explicit **re-baseline event**: fleet version increments and the score legitimately
  drops. A dropping score after a re-baseline is progress and is labelled as such. A
  dropping score without a re-baseline is a regression.
- **It matches how the bugs actually arrive.** The deadly failures in this library
  (§3.1–§3.7) are found by generation, not by inspection.

**Anti-gaming rules, enforced in CI:**

1. The regression corpus is a **gate at 100%**, evaluated *outside* the budget. It can
   never be traded against the score.
2. Generator deletions and generator parameter-range reductions are counted and printed
   in CI output on every run.
3. The seed schedule is committed. Two runs of the same `(fleet_version, resolvent
   commit)` produce identical results — which §3.10 already requires anyway.
4. A budget-exhausted (`Decline`) outcome inside a property test counts as a **failure**,
   not as a survived instance. Otherwise declining everything maximizes the score.
5. Sharpness rates (§3.13) are reported alongside the score. A run whose score improved
   while its Unknown rate rose is flagged.

**The score does not measure performance.** That is §6, and it is a different scoreboard
with different convergence properties. Never combine them into one number.

---

## 6. Performance gates

**These are optimization targets, not certificates.** They converge over months, not
days; they are non-monotone; they require a pinned machine; and they cannot be fanned out
to parallel agents without a shared frozen baseline. Constraint #3 asks that this be said
explicitly, so it is said here and again in `plans/roadmap.md` §3.

### 6.1 Honesty rules

- **Compare like with like.** msolve, Maple/FGb and Groebner.jl all default to
  *uncertified* Gröbner over ℚ; Groebner.jl says so plainly ("no out of the box guarantee
  that the reconstructed basis is correct"). A certified resolvent will lose those
  benchmarks *by construction*. The harness records the certification mode of both sides
  and refuses to print a comparison across modes without labelling it.
- **Do not invent numbers.** Every threshold below is a published figure from R3 §9.3 or
  a derived multiple of one. Any threshold not traceable to a citation is marked TBD and
  is set by measurement before it becomes a gate.
- **Every tuning threshold in resolvent is re-derived by measurement on resolvent's own
  corpus, and the measurement is committed.** Copying a threshold from a reference
  implementation is simultaneously a licensing hazard and a correctness hazard: it is
  someone else's measurement on someone else's machine (R1 §6, Tier B).
- **Single-threaded numbers are the primary series.** Parallel numbers are a separate
  series; a parallel speedup that changes results violates §3.10 and is a bug, not a win.

### 6.2 The ladders

**Bignum (settles R1's open question 1, and must run before Layer 0 is written).**

| Instance | Baseline | Target |
|---|---|---|
| `tczajka/bigint-benchmark-rs` with `dashu` 0.5.2 pinned | The published table used `dashu` 0.4.2 — **one release before NTT landed in 0.4.3** — so the widely-cited figure is stale | Measure. A negative result strengthens the case for an optional GMP feature flag, which is cheap to design now and expensive later |
| `gcd` / `gcd_ext` at 64, 256, 1k, 4k, 16k bits vs `rug` | dashu has Lehmer; GMP has subquadratic half-GCD — the one identified structural pure-Rust deficit | Measure. This sets how aggressive the ℤ-primitive discipline must be |

**Gröbner over `GF(p ≈ 2^30)`, single-threaded, drl** (published context: Groebner.jl,
Maple/FGb and msolve are within ~1.5× of each other; OpenF4 is 4–21× off; Singular's
Buchberger is ~150× off on Katsura-11):

| Milestone | Gate |
|---|---|
| **Correct** | Cyclic-7, Katsura-8, Eco-10 complete and agree with an external system |
| **Working** | Cyclic-8 < 60 s, Katsura-11 < 500 s, Eco-13 < 500 s (≈ Singular-Buchberger class) |
| **Competitive** | Cyclic-9 < 600 s, Katsura-13 < 900 s, Eco-14 < 600 s (≈ 2× SOTA) |
| **State of the art** | within 1.5× of msolve/Maple/Groebner.jl. **Do not plan for this.** |

**Gröbner over ℚ:** Katsura-10 (54 primes), Katsura-11 (78), Cyclic-8 (54),
Chandra-13 (166), Reimer-8 (78), **Hexapod (1102 primes for a 0.00 s modular run)**.
Hexapod is a correctness instance disguised as a performance instance — include it from
the first modular milestone.

**Real root isolation:**

| Milestone | Gate |
|---|---|
| **Correct** | degree ≤ 20 random and Mignotte instances verified against Sturm counts |
| **Working** | random dense `n=1024, τ=1024` < 30 s; Mignotte `n=257, τ=14` < 60 s |
| **Competitive** | random dense `n=8192, τ=8192` < 200 s; Mignotte `n=1025, τ=14` < 5 s — i.e. Newton acceleration is present and working |

**Resultants:** modular bivariate must beat the Ducos implementation by **≥ 100×** on
ℤ[x,y] inputs of degree ~20 (the published figure for the technique is 400×).

**Factorization:** Swinnerton–Dyer degree 32 (`r ≈ 16`) must complete — Zassenhaus can do
this. Degree 64 (`r ≈ 32`) separates van Hoeij from Zassenhaus. Degree 256 is the "van
Hoeij is really working" mark.

**Consumer-shaped workload — currently unmeasured and the most important gap.** R2's open
question 1: nobody knows what degree and coefficient bit-size an arbitrary-degree
arrangement engine actually produces. Every performance requirement on the geometry path
hinges on it. Settle it early by generating degree 3–8 curve pairs, computing `Res_y`
with the existing `QPoly`, and recording resultant degree, coefficient bit-length, and
the wall time of `isolate_roots` plus a `sign_of` sweep. That gives the actual target
rather than a guess, and it also answers R2's questions 4 (does modular gcd pay at these
sizes?) and 6 (does `CrossingKind::Unknown` stay rare at arbitrary degree?).

### 6.3 Regression tracking

- Fixed machine. Benchmark results from any other machine are recorded but never gate.
- Report medians of `k` runs with the interquartile range; never a single sample.
- **Change-point detection over the time series, not per-run thresholds.** A per-run
  threshold either flaps or is set so loose it detects nothing. Track each series and
  alert on a sustained level shift (compare the median of the last `k` runs against the
  median of the preceding `w`, with the alert threshold calibrated against the observed
  run-to-run noise of that specific series — calibration is per-series and committed).
- Every benchmark run records: resolvent commit, fleet/benchmark generation, machine id,
  compiler version, feature flags, thread count, certification mode.
- A performance regression **does not block a correctness PR.** It opens an issue against
  the score-graded lane that owns the series. Blocking correctness on a noisy number is
  how a project stops merging.

---

## 7. The CI gate

Four gates. A lane's work is "done" when the gate its milestone declares is green **and**
the lane checklist in §7.5 is satisfied.

### 7.1 Gate 0 — every commit, target < 5 minutes

| Check | Fails on |
|---|---|
| `cargo build --workspace --all-targets` | any error |
| `cargo clippy --workspace --all-targets -- -D warnings` | any warning |
| `cargo fmt --check` | any diff |
| **`cargo deny check licenses bans sources advisories`** over the **published** dependency graph (`--all-features` minus dev-only features), with an explicit `[licenses] allow` list and every copyleft SPDX id denied | any non-permissive crate anywhere in the published tree |
| **License-gate regression corpus**: the gate must *fail* on each of three planted cases — `malachite` (LGPL behind a permissive-looking pure-Rust crate), `polynomen` (GPL-3.0-only with an innocuous name), and a synthetic Apache-only crate depending on `rug` (the shipped-today `alkahest-cas` shape). If it does not fail on all three, the gate is not working | a gate that passes what it must reject |
| `unsafe` inventory: `#![forbid(unsafe_code)]` on every crate except a named allowlist; every `unsafe` block in the allowlist carries a `SAFETY:` comment | new `unsafe` outside the allowlist |
| **Determinism**: every regression instance run twice in-process, twice cross-process, at 1/2/8 threads, across feature combinations — canonical bytes compared | any difference |
| **Golden canonical-serialization files** byte-compared; a golden change without a schema-version bump in the same commit | drift |
| Unit tests + doc tests | any failure |
| **The full regression corpus at 100%** | any failure |
| Self-certification assertions enabled in the test profile (every operation checks its own certificate on every call in tests) | any failure |
| No-panic fuzz smoke: 30 s per public entry point | any panic |

### 7.2 Gate 1 — every PR, target < 25 minutes

Gate 0, plus:

| Check | Fails on |
|---|---|
| Property suite, full fleet, fixed seeds, fixed budget | any falsification |
| **Differential Tier 0 (sympy)** across every operation with an assignment in §4.2 | any Class A disagreement; any Class B disagreement not already recorded |
| **Oracle skip census** printed; job fails if a Tier-0 oracle is absent | absence |
| Sharpness rates (§3.13) computed and compared against committed ceilings | any ceiling exceeded |
| `--no-default-features` and each feature individually build + test | any failure |
| MSRV build | any failure |
| Semver check against the last published version (once published) | an unintended break |
| **Score reported** as `(fleet_version, seconds_survived)`; a drop without a re-baseline marker | regression |

### 7.3 Gate 2 — nightly

Gate 1, plus:

| Check |
|---|
| **Differential Tier 1** (Singular, PARI); job declares Tier 1 and fails if either is absent |
| Long adversarial budget (hours), full generator fleet |
| Narrow-field-width Gröbner sweep (§3.1): entire corpus at 4-bit exponent fields, zero silent divergences |
| Benchmark ladder (§6.2) with change-point report; regressions open issues, do not block |
| Miri on the monomial arena and packing crate, if it can be made to terminate |
| Fuzz targets, extended duration, corpus minimized and promoted |

### 7.4 Gate 3 — weekly and pre-release

Gate 2, plus Tier 2 oracles (Sage, msolve, Macaulay2); the full benchmark ladder
including the SOTA-comparison instances; `cargo about` attribution file regenerated and
diffed; a manual read of the §3.13 sharpness table and the §3.14 oracle-independence
table.

### 7.5 The lane completion checklist

A lane is not done until **all** of these hold. This list is what an agent is graded
against, and it is deliberately mechanical.

1. The operation's row in §2 exists, is implemented, and its certificate is checked in
   the same test that exercises the operation.
2. Every "does not prove" cell in that row has a corresponding entry in §3 or is
   explicitly discharged in the PR description.
3. Generators for the operation are in the fleet, including at least one *constructive*
   and one *adversarial* generator.
4. If the operation has a "don't know" or "probably" outcome, a sharpness rate is
   computed, has a committed ceiling, and is in Gate 1.
5. If the operation has an external oracle assignment in §4.2, the adapter exists, the
   normalization rule is in §4.3, and Tier 0 runs in Gate 1.
6. If the lane is SCORE-graded, its CERT/INV oracle lane is green and **frozen** first,
   and a baseline exists on the pinned machine.
7. Determinism holds (§3.10): the operation's output is byte-identical across runs,
   processes, thread counts, and feature combinations.
8. The operation takes a budget and returns a typed decline rather than hanging or
   panicking (§1.2, §3.11).
9. `docs/` carries a `Derivation:` line in the module doc-comment citing the **paper**,
   not a reference implementation. A module that cannot cite a paper is a signal it was
   written from a source tree and review must catch it (R1 §6).
10. Gate 1 is green.

---

## 8. Bootstrapping — the harness exists before the algebra

The order below is the only order in which this plan is self-consistent, because every
later item depends on an earlier one being able to grade it. It is expanded day-by-day in
`plans/roadmap.md` §6.

1. **License gate + workspace two-category rule + Gate 0.** Fully automatic verdict,
   zero algebra required, cheap now and expensive to retrofit.
2. **Determinism and canonical-bytes harness.** Everything downstream is an artifact that
   assumes it.
3. **Corpus and score harness**, with the generator interface, the seed schedule, the
   regression-corpus format, and the minimizer — before there is anything to generate for.
4. **Tier-0 oracle adapter (sympy)** with the S-expression protocol and the triage
   classifier.
5. **`resolvent-int` + `rug` oracle** — the first thing with a real verdict.
6. **`Fp`** — exhaustively certifiable, the ideal first agent lane.
7. **`UPoly` over ℤ + naive reference.**
8. **Sturm + Descartes.** At this point two independent implementations grade each other
   and the oracle loop is closed. Everything after this is filling in the table in §2.

---

## 9. What this document cannot settle

Stated as what would settle them, not guessed at.

1. **The cost multiplier of cofactor tracking through F4** (§1.1). Prototype on
   Katsura-8 / Cyclic-7 and measure time and memory before `groebner_certified` is
   committed to as the regression oracle. If memory exceeds ~20×, the plan's Gröbner
   verification story changes materially.
2. **The exact hypotheses of the Idrees–Pfister–Steidel theorem after Noro–Yokoyama's
   correction** (§3.3). Fetch Noro & Yokoyama (ICMS 2014) and *Mathematics in Computer
   Science* 11(3), 2017. This decides whether the fast Gröbner path can ever return
   `Proved` without cofactors.
3. **The real degree and coefficient profile of a geometry consumer's resultants**
   (§6.2). Everything on the geometry performance path is currently a guess. Instrument a
   lifted-degree prototype against the existing `QPoly`.
4. **Whether certified/fast Gröbner cross-checks compare bases or hashes of them.** A
   hash only works if normalization — ordering, monic-ness, term-order tie-breaks — is
   byte-identical across implementations. Fix a canonical serialization *before* any
   cross-implementation oracle is written; §3.12's golden files depend on the same
   decision.
5. **Whether Sturm remains a usable oracle at the degrees the corpus reaches.** Sturm is
   `Õ_B(d⁴τ²)` and that bound is tight. There will be a degree above which the strongest
   isolation certificate is unaffordable, and above it the lane's verdict degrades from
   CERT to DIFF. Measure where that is and record it, rather than discovering it as a
   mysteriously slow CI job.
6. **Whether `lll-rs` (MIT) is usable at van Hoeij precision.** The lattice has dimension
   ~`r` with entries of size `p^k` exceeding twice the Landau–Mignotte bound —
   potentially thousands of bits. Run it on a Swinnerton–Dyer degree-64 lattice and check
   both correctness and time. If it fails, LLL becomes its own lane (and §2.4 shows LLL
   is at least fully self-certifying, which makes it a good lane).
7. **Whether the interned-monomial design defeats its own comparison key** — a random
   arena load per comparison may cost more than the `u64` compare it enables. This is no
   longer only an open question: ADR-008 and `plans/api-shape.md` L1-4 currently specify
   *different term types*. Microbench inline-packed against id-plus-arena on a realistic
   S-pair queue, and measure the divisor-query index's speedup under each, before the
   multivariate trunk starts. See `plans/roadmap.md` §2.5.
8. **Where the fail-closed verdict vocabulary lives in the type system.** If Bernstein
   range-bounding returns `Uncertain<Sign>` while `AlgebraicReal` returns `Sign`,
   consumers get two verdict vocabularies. This needs a decision doc before the API is
   written, not an experiment (R2 open question 8).

---

## Sources

Research inputs: `docs/research/prior-art-and-licensing.md`,
`docs/research/consumer-requirements.md`,
`docs/research/algorithms-and-representation.md`,
`docs/research/consumer-sinbad.md`, `docs/research/consumer-cadabra2.md`,
`docs/research/consumer-solverang.md`, and `/home/dev/projects/IDEAS-crates.md` §4.
Architecture inputs: `docs/decisions/ADR-001…009`, `plans/architecture.md`,
`plans/api-shape.md`.
Every external citation in this document is carried from those, where it is sourced.

Consumer code read directly for grounding (context only; resolvent does not depend on it):
`/home/dev/projects/arrangements/crates/lazy-exact/src/roots.rs:1-16, 41-45, 317-322`,
`/home/dev/projects/arrangements/crates/lazy-exact/src/bernstein.rs:135-152`,
`/home/dev/projects/arrangements/crates/arrangements/src/geoms/conics.rs:32-46`.

Machine state verified 2026-07-31: no CAS oracle installed; `sympy` 1.14.0 importable via
pyenv `python3`; `singular` 4.4.1.p5-11 and `pari` 2.17.4-1 available in Arch `extra`;
`cargo` 1.96.0.
