# C2 — Adversarial critique of the plan and the verification story

**Status:** challenge deliverable. Attacks `plans/verification.md` (D2-1),
`plans/roadmap.md` (D2-2), and the parts of `plans/architecture.md`,
`plans/api-shape.md` and `docs/decisions/ADR-001…018` that those two documents depend on.
**Method:** read all four plans (3,710 lines), all eighteen ADRs, `prior-art-and-licensing.md`,
and the two existing challenge documents (X1 `challenge-generality.md`, X2
`challenge-evidence.md`) so as not to re-file their findings. Consumer claims were re-checked
against `/home/dev/projects/arrangements` directly.
**Companion critiques:** X1 attacks the API's generality; X2 attacks the consumer evidence.
This document attacks the *executability* of the plan and the *strength of the oracles*.

**Bottom line.** The verification document is the best thing in this repository and it is
better than most published test plans. It anticipates its own most famous objection
(multiply-back does not prove irreducibility) in the row where it appears, and §3.13's
sharpness gates are a genuinely original contribution that most CAS test suites lack. So the
attack has to go somewhere else, and it lands in three places:

1. **The certificates are unverified code, and the document's own strongest rule is never
   applied to them.** §7.1 requires the *license* gate to be observed failing on three
   planted cases — "a gate that passes what it must reject is not a gate". That rule is not
   applied to any of the ~50 certificates in §2, which are written by the same agent, in the
   same PR, as the operation they grade. This is the single largest hole.
2. **Two of the four plans specify materially different libraries**, on four one-way doors,
   and the roadmap's contradiction census (§2.5) names two contradictions — both of which
   have since been *settled* by ADR files that now exist — while naming none of the six that
   are live.
3. **Two certificates the document marks "Complete" with an empty "does not prove" cell are
   passed by a trivially wrong implementation.** `gcd(a,b) ≡ 1` satisfies the Layer-0 gcd
   certificate *and* the Layer-2 univariate gcd certificate, including the "half people
   forget".

Ranked findings follow. §9 lists what I checked and found sound, by name, so this can be read
as a coverage report rather than only a complaint list.

---

## 0. Severity index

| # | Sev | Target | One line |
|---|---|---|---|
| **C1** | fatal | `verification.md` §2 (all), §7.5 | No certificate is ever observed rejecting a wrong answer; the license gate's own discipline is not applied to the certificates |
| **C2** | fatal | `roadmap.md` §2.5 vs `api-shape.md` §1.3/§1.4/§3.3, ADR-004/005/013/014/015/018 | Six live contradictions across two plans, four on one-way doors; the roadmap flags two, both now stale |
| **C3** | serious | `verification.md` §2.1 gcd row, §2.3 gcd row | Both gcd certificates are circular; `gcd ≡ 1` passes both, including the "mandatory" degree half |
| **C4** | serious | `verification.md` §3.14 | Oracle independence is asserted, audited by hand, and has no mechanical gate; Sturm and Buchberger will both absorb the code they grade |
| **C5** | serious | `verification.md` §2.2, §2.3, §2.7 vs ADR-012 | Every Schwartz–Zippel certificate is run at a fixed committed seed, which makes it a golden test, not a probabilistic proof |
| **C6** | serious | `verification.md` §3.1 item 2, `roadmap.md` M6 overflow sweep | The 4-bit-field sweep is trivially satisfied by *universal* overflow; as specified it detects nothing |
| **C7** | serious | `verification.md` §5.3 rule 4 vs §3.13 row 3 | "Any decline is a failure" contradicts the decline-rate gate and pushes agents to inflate budgets until declines become hangs |
| **C8** | serious | `verification.md` §3.13, `roadmap.md` M3 exit gate | Not one sharpness ceiling is a number; M3 exits on "the Unknown-rate ceiling met", against a ceiling that does not exist |
| **C9** | serious | `roadmap.md` §3, Wave 0/1/2; §1 M7 | Fan-out independence is overstated at three specific points, and `resolvent-base` — "must land first" — has no lane at all |
| **C10** | serious | `verification.md` §2.5 Buchberger row, FGLM row | The S-pair certificate is vacuous if it reuses Gebauer–Möller; the FGLM certificate proves generation, not the lex-GB property FGLM exists to produce |
| **C11** | serious | `verification.md` §2.3 separation-bound row; ADR-013 §5 | Unstated whether an `Equal` verdict can be reached by exhausting the bound. One reading is silently wrong, the other fails loudly |
| **C12** | serious | `verification.md` §2.1 prime-generation row | A composite in the deterministic prime table breaks GF(p) silently while CRT and rational-reconstruction certificates keep passing |
| **C13** | serious | `verification.md` §2.4 Hensel row | The Landau–Mignotte bound has no certificate and feeds precisely the coarse-factorization failure of §3.2 |
| **C14** | serious | ADR-018, ADR-014 vs `arrangements/crates/lazy-exact/src/roots.rs:438` | The deferred merge has one unflagged collision — `RealRoot::multiplicity()` is a method on a stored value — and the fix costs nothing now |
| **C15** | serious | ADR-001 gate 4; `arrangements/DESIGN.md` §1 | The `Derivation:` gate is trivially satisfiable and is *weaker* than the posture it claims to mirror; provenance is not licensing paperwork and cannot be deferred |
| **C16** | serious | `verification.md` §5.2 pinning rule | Eco-`n`/Noon-`n`/Reimer-`n` have no Tier-A source; "pin them to a specific generator source" instructs transcription from a GPL test suite |
| **C17** | serious | `prior-art-and-licensing.md` §3.2, §3.5 | The prior-art answer is sound, but *forking* `feanor-math` (MIT) is never evaluated, and it already ships four of the lanes the roadmap sizes at ~80 sessions |
| **C18** | minor | `roadmap.md` M0/M1/M2/M3/M5 exit gates | Five exit criteria are vibes; rewritten as gates in §7 |
| **C19** | minor | `roadmap.md` §2, all ADRs | "Ratified" has no mechanical definition and every ADR says `Proposed`, so the freeze is unenforceable as written |
| **C20** | minor | `verification.md` §2.1 CRT row, §2.4 factorization row, §2.3 resultant row (d) | Three concrete missing clauses: moduli distinctness, factor pairwise non-associateness, and a splitting-field dependency inversion |
| **C21** | minor | `verification.md` §5.1 | The regression corpus is append-only and gates at 100% with no provenance field; a mis-triaged expected answer is frozen forever |
| **C22** | minor | `roadmap.md` H4, `verification.md` §4.1 | Oracle adapters are graded by round-trip, which is self-consistent and says nothing about whether the oracle understood the input |

---

## 1. C1 — the certificates are unverified code (fatal)

**The claim under attack.** §7.5 item 1: "The operation's row in §2 exists, is implemented,
and its certificate is checked in the same test that exercises the operation." That is the
whole harness. The agent writes the operation, writes the certificate check, and both go
green together.

**Why it fails.** A certificate is code. The failure mode of certificate code is not "it
rejects a correct answer" (that is loud) — it is "it accepts everything" (that is silent).
Concretely, all of these pass CI and grade nothing:

- a `certifies()` that iterates an empty evidence vector and returns `true`;
- `assert!(prod == f)` where `prod` was computed by multiplying the factors using the
  *same* buffer the factorizer left them in;
- a degree check written as `deg(H) == deg(H)`;
- a specialization check (§2.3 subresultant row) that specializes at a point where the
  polynomial is constant, so both sides are trivially equal;
- an "exhaustive over all `(a,b)` for `p < 2^10`" loop whose inner bound is `p` instead of
  `p*p`, silently testing `p` pairs instead of `p²`.

None of these is exotic. All of them are what a model writes when it is optimizing for a
green suite, which constraint #3 says is exactly the population building this library.

**The document already knows the rule and applies it exactly once.** §7.1: the license gate
"must *fail* on each of three planted cases … If it does not fail on all three, the gate is
not working." M0's exit gate repeats it: "A gate that has never been observed to fail is not
a gate." That epistemology is correct and it is applied to one gate out of fifty.

**Fix, and it is cheap.** Add to §7.5 as item 0, before everything else:

> **Every row in §2 ships with a *mutant set*: at least one deliberately wrong implementation
> of the operation, committed under `#[cfg(test)]` in the same module, and a test asserting
> that the certificate **rejects** each mutant.** A row whose mutants are all rejected by the
> *type system* rather than by the certificate does not count; the mutant must compile and
> must produce a plausible wrong value.

Prescribe the mutant classes so they are not chosen to be easy — one per failure family the
document already names:

| Mutant class | Applies to | Must be rejected by |
|---|---|---|
| **Coarsening** — merge two outputs into one | factorization, square-free decomposition, root isolation, ideal decomposition | the irreducibility / disjointness / count half |
| **Refining** — split one output into two | factorization, isolation | multiply-back / Sturm count |
| **Off-by-one in a bound** | Landau–Mignotte, Cauchy, separation, Hadamard, capacity | the bound's own validity check |
| **Identity** — return an input unchanged | gcd, normalization, reduction | divisibility + degree, idempotence |
| **Trivial constant** — return `1`, `0`, `Unknown`, `Probable`, `Decline` | *everything* | the certificate **and** the §3.13 sharpness rate |
| **Sign flip** | resultant, Sturm variation counting, `sign_of` | the cofactor identity, cross-route agreement |
| **Silent wrap** | monomial multiply, `Fp` reduction | guard bits, exhaustive small-`p` |

Note the fifth row. The trivial-constant mutant is the machine-checkable form of §3.13's
argument, and making it a per-operation obligation is what turns "sharpness gate" from a
policy into a test.

**Second-order consequence, and it is why this is fatal rather than serious.** §4.4's triage
classifier is defined as: "Re-run resolvent's own certificate on the instance. *Self-certificate
also fails* → Class A: resolvent bug, certain. *Self-certificate passes, oracle disagrees* →
Class B: normalization, convention, or oracle limitation." A vacuous certificate does not
merely fail to catch bugs — it **routes every real bug into Class B**, where the prescribed
response is to fix the adapter or record a convention. The triage pipeline converts a broken
certificate into a stream of spurious "unspecified convention" ADRs. That is a plan that
actively metabolizes its own bugs into documentation.

---

## 2. C2 — two plans specify two different libraries (fatal)

`plans/roadmap.md` §2.5 is titled "Two live contradictions inside D1's own output" and calls
flagging them "the highest-value thing this section does". It is right about the value and
wrong about the census.

**Both flagged contradictions are now stale.** The roadmap says (§2, opening) "ADR-010…018 are
declared in `plans/architecture.md`'s ADR table but not yet written" and that "a lane may not
start against a declared-but-unwritten ADR". All nine files now exist in `docs/decisions/`.
Worse:

- **Contradiction 1 (`AlgebraicReal` mutability) is decided.** `ADR-013-algebraic-real-mutability.md`
  picks `Arc<Inner>` + `Mutex<Bounds>`, `Send + Sync`, and explicitly rejects api-shape's
  inline-`RefCell` variant in its Alternatives section (:165-178), on two named grounds.
- **Contradiction 2 (interning) is reconciled.** `ADR-008` :74-86 states, in the words the
  roadmap asked for, that "no global interner" and "terms are interned ids" are both true and
  not in conflict: the arena is owned by the `Ring` and reached through an `Arc<Ring>` held by
  the `MPoly`.

So an agent reading the roadmap blocks lanes A1 and P1/P2/P3 on experiments the ADRs have
pre-empted; an agent reading the ADRs starts them. Both are following the plan.

**Six contradictions that are live and unflagged.** Four are on one-way doors.

| # | `architecture.md` + ADRs say | `api-shape.md` says | Door |
|---|---|---|---|
| 1 | `resolvent-{base,int,modular,poly,algebra,real,expr}` + 3 unpublished (ADR-005, §1.1) | `resolvent-{seam,int,modular,poly,linalg,engine,alg,expr,lazy}` (§1.4) | **costly** — crates.io names are sticky, and lane H1 creates the workspace on day 1 |
| 2 | "**Do not** put a scalar-seam trait in resolvent's public API" (ADR-018 :95-99); "Not supported: resolvent becoming generic over a consumer-shaped scalar seam" (§5.6) | `Scalar`, `ScalarOrd`, `TryDiv`, `Hom` in a public zero-dependency `resolvent-seam`, pitched as "the single highest-leverage hook" (§3.3, §5.1) | **one-way** — removing a public trait is breaking |
| 3 | "**Do not expose a float interval type**" (ADR-015, §6.4) | `Interval<f64>` is **core** (L0-5) and appears in adapter signatures (§6.1c, §6.4 #10) | cheap to add, one-way to remove |
| 4 | `poly: UPoly<Integer>`; coefficients are ℤ-primitive, ℚ is a boundary façade (ADR-013 :70, ADR-004) | `poly: Arc<SqfrPoly<Rational>>` (§1.3); `SqfrPoly::new(&p)` over ℚ in the adapter sketch (§6.1c) | **one-way** (ADR-004 is marked one-way) |
| 5 | "Multiplicity is not a field"; `isolate_roots -> Vec<(AlgebraicReal, u32)>` (ADR-014, §5.3) | `AlgebraicReal { defining_poly, isolating_interval, multiplicity }`, `mult: u32` inline; "Multiplicity must be on the returned root" (L3-1, L2-1, §1.3) | **one-way** |
| 6 | `MPoly` holds an `Arc<Ring>`; the `Ring` owns the monomial arena (ADR-008 :74-81) | `Ring { nvars, order, packing }` is **`Copy` and ~12 bytes, carried inline**; INV-13 (L1-3, §7) | **one-way** — a 12-byte `Copy` value cannot own an arena; this is `MPoly`'s memory layout |

Contradictions 4 and 6 are the interesting ones, because they are not stylistic. #4 decides
whether the univariate trunk's headline type is over ℤ or ℚ, and ADR-004's entire argument is
that the nearest prior art chose ℚ and was wrong to. #6 is *not* the interning question the
roadmap flagged — that one is settled — it is the strictly narrower question of whether
`Ring` is a value or an arena owner, and that is what `MPoly`'s term type is inherited from.

There is one further factual defect worth naming separately: **`api-shape.md` L0-12 declares
ℤ/n for composite `n` out of scope because "it is not needed by any modular method (all of
which use prime moduli)". That is false.** Hensel lifting to `p^k` — M5, lane K2, and the row
in `verification.md` §2.4 — is arithmetic modulo a composite. ADR-003 and M1's exit gate both
require `Zn`. One document deletes a capability another gates a milestone on.

**Fix.** This is not a research task; it is thirty minutes of arbitration and it must happen
before lane H1 creates a single `Cargo.toml`.

1. Declare a precedence rule in one place. `architecture.md` :11 already says "Where the two
   disagree, the ADR wins" — but says it only about itself and the ADRs, and `api-shape.md`
   §Status says it is "binding on the founding architecture unless explicitly overturned".
   Two documents each claim supremacy. Pick one; the honest pick is **ADRs win, and any
   api-shape item that contradicts a ratified ADR is a proposed amendment to that ADR, not a
   binding statement.**
2. Rewrite `roadmap.md` §2.5 against the current file state: strike the two stale
   contradictions, insert the six above, and mark which lanes each one blocks (H1 for #1;
   A1/A2 for #2,#3,#4,#5; P2/P3 for #6).
3. Four of the six are settlable by fiat from evidence already in the repo. Only #3
   (`Interval<f64>` public or not) and #6 (`Ring` value vs arena) plausibly need an
   experiment, and #6's experiment is already specified in ADR-008 :165-171.

---

## 3. C3 — both gcd certificates are circular (serious)

**Layer 0** (`verification.md` §2.1):

> `gcd(a,b) = g` | `g|a`, `g|b`, and `gcd(a/g, b/g) == 1` | **Proves: Full correctness** |
> Does not prove: — | `~1×` (the coprimality check is another gcd) | CERT

The cost column concedes the problem and the "Proves" column does not act on it. Take the
implementation `fn gcd(_a, _b) -> Integer { Integer::ONE }`:

- `1 | a` ✓
- `1 | b` ✓
- `gcd(a/1, b/1) == 1` — evaluated by the same function, which returns `1` ✓

**Full correctness**, certified, on an implementation that is wrong for every coprime-free
input.

**Layer 2** (`verification.md` §2.3) is worse, because the document specifically warns about
this row:

> (a) `H|A` and `H|B` by exact division; (b) `deg H == deg gcd(A mod p, B mod p)` for one
> certified-good prime `p`. **Proves: Complete.** … *the half people forget:* Divisibility
> **alone** accepts any common divisor. The degree half is mandatory.

The degree half is mandatory and it is computed by the routine under test. `H = 1` gives
`deg H = 0`, and the GF(p) gcd — the same modular gcd whose ℤ lift is being certified —
returns `1`, giving `deg gcd(A mod p, B mod p) = 0`. Certificate passes. Marked **Complete**,
with an empty "does not prove" cell.

The mathematical argument in the row ("(a) gives `H | G`; (b) with `deg gcd mod p ≥ deg G`
gives `deg H ≥ deg G`") is correct, but its second premise — `deg gcd(A mod p, B mod p) ≥
deg G` for a good prime — is a fact about the *true* modular gcd, not about the computed one.
The certificate silently assumes correctness of the component it is grading.

**Fix, and it restores completeness non-circularly.** Use a Bézout witness, which the document
already has in the adjacent `gcd_ext` row:

- **Over ℤ:** `g | a`, `g | b`, **and** `(u, v)` with `u·a + v·b == g`. That is complete
  (any common divisor dividing a Bézout combination equal to `g` forces `g` to be *the* gcd),
  costs one multiply-add to check, and shares no code with the gcd routine's control flow.
  Delete the recursive coprimality clause.
- **Over `F[x]`:** identically — `H | A`, `H | B`, and `u·A + v·B == H`. Complete over a field.
- **For the ℤ[x] two-part certificate:** have the per-prime modular gcd return its Bézout
  cofactors in GF(p) (the extended Euclid that computes it already has them; retaining them is
  free). Then `deg gcd(A mod p, B mod p)` is itself certified, and the ℤ-level completeness
  argument holds against the *computed* modular gcd rather than the ideal one.

The same circularity, in smaller doses, appears in the `Rational` canonical-form row
(`gcd(num,den)==1`, checked by the gcd under test) and in the `UPoly` mul row
(`(a·b)/b == a`, checked by the division under test — mitigated there by the independent naive
reference and the evaluation homomorphism, but see C5 for what the evaluation homomorphism is
actually worth). **General rule to add to §2's preamble: a certificate may not invoke the
operation it certifies, nor any routine on that operation's call graph. Where it must, the row
is INV, not CERT.**

---

## 4. C4 — oracle independence has no gate (serious)

§3.14 is the second-best section in the document and it ends without a verdict function, which
is the one thing this plan is not allowed to do. The closing instruction is:

> This table must be maintained and audited whenever a shared helper is introduced.

Audited by whom, in a project whose founding constraint is that agents build it and oracles
grade it? Every other rule in the document is mechanical *on purpose* ("Make it mechanical,
not cultural", `roadmap.md` §7). This one is cultural.

**The predictable failure, in two named instances.**

1. **Sturm.** Day 7 builds Sturm "naive, over ℚ, low degree — it is an oracle, not a product".
   §3.14 then records that "Sturm's chain is a PRS, so a subresultant/PRS bug corrupts Sturm
   *and* the resultant lane simultaneously". That is true of a *subresultant* Sturm chain. The
   day-7 Sturm shares only `divrem`. Between M2 and M4 someone will notice that Sturm over ℚ
   has catastrophic coefficient growth and will "fix" it by routing it through the Ducos PRS
   that lane T1 just landed. At that moment the strongest certificate in Layer 2 (§2.3: "(a)
   is the strongest single check in Layer 2") silently becomes a check of a component against
   itself, and *no test changes colour*.
2. **Buchberger.** G1 exists to grade G2/G3. F4's symbolic preprocessing, its normal-form
   routine, and its monomial handling are exactly the code an agent will factor out and share.
   §3.14 already rates `groebner_certified` vs `groebner` as "weaker than it looks"; nothing
   stops it becoming weaker still.

**Fix.** Make the independence table executable. Two mechanisms, both cheap:

- **A module-import gate.** Each oracle module declares, in a committed manifest, the set of
  modules it is permitted to reach transitively. CI walks the module graph (or the crate graph
  where the split is crate-level) and fails on any edge into the lane the oracle grades. This
  is the same shape as gate L1 in `architecture.md` §1.3 ("checked-in expected dependency
  graph; CI diffs `cargo tree --edges normal` against it"), one level finer.
- **A frozen-oracle rule.** An oracle module is marked `#![doc = "ORACLE: graded lane = U5"]`
  and any PR touching it that also touches the graded lane fails CI without an explicit
  `oracle-independence-reviewed:` trailer naming what was checked. Weaker than the import
  gate, and worth having as well because it catches "I made both sides agree" edits.

Add to §7.5 as item 11: *if the lane is an oracle for another lane, its permitted-import set
is committed and enforced.*

---

## 5. C5 — randomized certificates run at one fixed seed (serious)

Several of §2's strongest rows are Schwartz–Zippel arguments:

- `UPoly` mul: "**evaluation homomorphism** `eval(a·b,x) == eval(a,x)·eval(b,x)` at random
  points in a large `GF(p)`" — "the evaluation check is a Schwartz–Zippel argument with failure
  probability `deg/p`";
- subresultant chain: the specialization property "for random good ring maps `Ψ` (random
  primes, random evaluation points) — a free, strong, per-instance self-check … essentially a
  randomized proof";
- L4 rewrite soundness: "by evaluating both sides at random points over a large `GF(p)`".

§1.3 and ADR-012 then require that the library be deterministic: "prime selection is a
deterministic sequence or takes an explicit `u64` seed", "The default seed is a **fixed
checked-in constant**, not entropy, so the default path is reproducible" (`architecture.md`
§4.1b), and "Primes are never 'random'. `prime(i)` is a pure function of `i`."

Both requirements are right. Together they mean **the "random points" are the same points on
every run of every CI job forever.** A failure probability of `deg/p` is a statement about a
draw; when the draw is fixed and committed, there is no probability left. What remains is a
golden test at one point. An implementation whose error happens to vanish at `prime(0)` and
`prime(1)` — for instance a coefficient error that is a multiple of a small fixed prime, which
is not a contrived class in modular arithmetic — is certified forever.

This is a genuine tension between §1.3 and §2, and neither section mentions the other.

**Fix — the resolution is that the two uses of randomness are different uses.**

1. **Inside the library**, at the default seed: deterministic, as specified. Unchanged.
2. **In the harness**, a certificate whose soundness argument is probabilistic is graded
   **across the fleet seed schedule**, never at the default seed alone. §5.3 already commits a
   seed schedule and already varies it across the falsification budget; this just states that
   randomized certificates are wired to *that* seed source, not to `Session::default()`.
3. **State the rule in §2's preamble:** *a row whose "Proves" column relies on a randomized
   argument is CERT only when evaluated over the fleet seed schedule; at a single fixed seed it
   is a golden test and is graded INV.* Then tag the three rows accordingly.
4. **Add to the sharpness/anti-gaming rules in §5.3:** the number of distinct seeds at which a
   randomized certificate was checked is reported alongside the score, for the same reason
   generator deletions are reported — a silent reduction from 64 seeds to 1 is otherwise
   invisible and improves every number.

---

## 6. C6 — the 4-bit-field overflow sweep is trivially satisfied (serious)

§3.1 item 2 and M6's "Overflow sweep" gate:

> A dedicated test mode that runs the *entire* Gröbner corpus at a deliberately narrow field
> width (4-bit fields) and asserts that every instance either completes with the same answer
> as the wide run or reports overflow. **Zero silent divergences permitted.**

ADR-008 §4 gives the capacity: 4-bit fields with one guard bit have a total-degree bound of
**7**. Every instance in the benchmark corpus — Katsura-`n`, Cyclic-`n`, Eco-`n` — has S-pairs
and intermediate bases well past total degree 7. So the sweep's outcome is: *every instance
reports overflow*, the assertion "either matches or reports overflow" is satisfied by the
second disjunct universally, and the test is green.

A green test whose expected outcome is "everything declines" is the exact shape §3.13 was
written to prohibit, applied to the gate that §3.1 calls "the only detector" for the library's
most dangerous failure. The sweep as specified exercises the *overflow-detection* path and
never once exercises the path where a monomial multiply succeeds near the boundary — which is
where a guard-bit off-by-one lives.

**Fix.** Make it a distribution assertion, not a disjunction. The wide run knows each
instance's true maximum total degree `D_max`, so the expected partition is computable:

> For each width `w ∈ {4, 8, 16}` and each corpus instance, let `D_max` be the maximum total
> degree observed in the *wide* run. The narrow run **must complete and match** iff
> `D_max ≤ 2^(w−1) − 1`, and **must report overflow** otherwise. Any instance that overflows
> when it should have completed is a false positive and fails; any that completes when it
> should have overflowed is a silent wrap and fails. CI prints the completed/overflowed counts
> per width, and **a width at which zero instances complete is a failed sweep, not a passed
> one.**

The last clause is the whole fix. Additionally, add a *boundary* sub-corpus — the generator
fleet already has "Capacity-boundary monomials: total degree exactly `D`, exactly `D+1`,
exponents exactly at the field max" — and require that at each width, both the `D` and `D+1`
instances are present and land on opposite sides.

---

## 7. C7 — "any decline is a failure" contradicts the decline-rate gate (serious)

§5.3 anti-gaming rule 4:

> A budget-exhausted (`Decline`) outcome inside a property test counts as a **failure**, not
> as a survived instance. Otherwise declining everything maximizes the score.

§3.13 row 3:

> `Result<_, BudgetExhausted>` | trivially-sound useless implementation: always decline |
> sharpness gate: Decline rate at the standard budget; **zero declines on the "must complete"
> sub-corpus**

These are different rules. Rule 4 says *every* decline anywhere is a failure. §3.13 says
declines are permitted, are counted, and are forbidden only on a designated sub-corpus. Rule 4
is also in tension with §1.2, which makes declining a *designed* behaviour on every entry
point, and with the Hexapod instance (1102 primes), which is in the corpus precisely because
it is expensive.

The reconciliation matters because of what an agent does under rule 4. Facing "any decline
fails the suite", the cheapest fix is to raise the default budget until nothing declines. That
converts declines into long runs, and a long run in a CI job is exactly the hang that §3.5
identifies as the deadliest failure mode — with the added property that it is now *sanctioned*.

**Fix.** Replace rule 4 with:

> 4. Declines are classified before they are scored. A decline is a **failure** if (a) the
>    instance is in the must-complete sub-corpus, or (b) the operation's budget was derived
>    from a *proven* bound (Landau–Mignotte, Mignotte–Davenport, Hadamard, Cauchy) — in which
>    case exhaustion is impossible for a correct implementation and the decline is a bug
>    (`architecture.md` §3.4 already says this: "exhaustion is **proven impossible** and the
>    budget is a bug detector"). Otherwise a decline is a **survived instance** and is counted
>    in the decline rate, which is a §3.13 sharpness number with a committed ceiling.
>    **Budget defaults are committed values; raising one is a diff, is counted in CI output,
>    and requires a recorded justification** — the same discipline as a generator parameter
>    reduction.

The last sentence is what closes the gaming route that rule 4 was reaching for.

---

## 8. C8 — no sharpness ceiling is a number (serious)

§3.13 is the best idea in the document. It is also, at present, a table of seven rows in which
every gate is described and none is set:

- "Unknown rate on the corpus **below a tracked ceiling**"
- "`Proved` rate on the corpus; **per-operation floors**"
- "Decline rate **at the standard budget**"
- "**False-positive rate**; and the divisor-query benchmark"
- "**Ratio** of the returned bound to the observed separation, tracked as a distribution"
- "Interval width **relative to** the separation bound"
- "Unknown rate on the intersection corpus"

Not one number. §7.2 then makes Gate 1 fail on "Sharpness rates computed and compared against
**committed ceilings**", and `roadmap.md` M3's exit gate requires "the Unknown-rate ceiling
met". An exit criterion evaluated against a ceiling that does not exist is not a gate.

This is not a criticism that the numbers should have been invented — §6.1 forbids inventing
numbers and is right to. It is a criticism that the *mechanism for establishing them* is
missing, so the gate has no way to become real.

**Fix — the ratchet.** Add to §3.13:

> Every sharpness rate is established by measurement in the first PR that lands the API it
> guards. That PR commits the measured value as the ceiling, rounded outward by a stated
> margin, to `sharpness-ceilings.toml`. Thereafter:
> - CI fails if a measured rate exceeds its committed ceiling.
> - **A PR may lower a ceiling freely.** Lowering is progress and needs no justification.
> - **A PR may not raise a ceiling** without a recorded justification in the file itself and a
>   line in CI output, counted the same way generator deletions are counted (§5.3 rule 2).
> - A rate with **no** committed ceiling fails Gate 1. `TBD` is not a ceiling.
>
> Per-operation floors that are stated as absolutes — GCD, resultant, factorization-product,
> and isolation must be 100% `Proved` — are committed as `1.0` on day one and are never
> ratcheted.

That makes M3's exit gate evaluable, makes the ceilings monotone in the right direction, and
costs one TOML file.

---

## 9. C9 — the fan-out plan overstates independence at three points, and omits a lane (serious)

The roadmap's central claim is: "the ADR freeze in §2 is the only true global barrier. Before
it, everything is harness work. After it, three trunks run concurrently and never touch each
other's one-way doors." Three specific defects.

**(a) Wave 0 is not five independent lanes; it contains three implementations of one artifact.**

- H2 delivers "canonical-bytes harness, golden-file machinery".
- H3 delivers "corpus format, generator interface, seed schedule, minimizer, score reporter".
- H4 delivers "Tier-0 sympy adapter, **S-expression protocol**, triage classifier".

All three serialize polynomials. `architecture.md` §4.5 puts the canonical serializer in
`resolvent-base` and says "the serializer is in `resolvent-base` so every crate **and every
oracle adapter** shares one implementation". And `verification.md` §9 item 4 lists precisely
this as unsettled and load-bearing: "**Fix a canonical serialization *before* any
cross-implementation oracle is written**; §3.12's golden files depend on the same decision."

So the roadmap's own inputs say H2, H3 and H4 share a blocking artifact, and the fan-out table
lists them as parallel with "no shared state". Three agents will write three serializers and
the merge will be a rewrite of two of them.

*Fix:* split H2 into **H2a — the canonical serializer and its schema version** (blocking, ~half
a lane) and **H2b — the determinism/golden harness over it**. Make H3 and H4 depend on H2a.
Wave 0's honest concurrency is then 1 → 6, not 7.

**(b) Wave 1's Z7 is a prerequisite, not a peer.** Z7 is "`Certificate` type, error taxonomy,
budget plumbing". §1.2 makes `budget` a parameter on every Layer-2/3 entry point and §0.1 makes
`Certified<T>` a return type; Z1, Z3, Z4, Z5 and Z6 all have signatures containing both. Six
agents writing signatures against an unwritten error taxonomy produce six taxonomies. Sequence
Z7 first; real concurrency is 1 → 5.

**(c) The expression trunk is not independent of the multivariate trunk.** `roadmap.md` M7:
"**Independent of M2–M6.** This is the cleanest parallel trunk in the plan." But:

- X4 is "`is_polynomial_in` bridge to Layer 1", and its signature is
  `is_polynomial_in(&syms) -> Option<MPoly>` (M7 Lands; `api-shape.md` L4-5). `MPoly` is lane
  P3, in the multivariate trunk.
- `architecture.md` §1.1 states the crate DAG directly: `resolvent-expr` "depends on: base,
  int, **poly**, **algebra**. NOT on real."
- M7's own exit gate requires `diff` to equal `UPoly::derivative` exactly — lane U2.

So the truly independent part of M7 is X1 + X3 (`Store`, node set, `walk_topological`,
canonical bytes). X2 needs U2 for its oracle; X4 needs P3. State it that way; the trunk is
still the best parallel work in the plan, just not free of the other two.

**(d) `resolvent-base` has no lane.** `architecture.md` §1.4: "`resolvent-base` | trait
vocabulary | certificate (trait-law property tests) | **Must land first; everything inherits
it.**" ADR-006 marks the trait signature a **one-way door**. The `Ring` / `CommutativeRing` /
`Field` / `EuclideanDomain` / `Ordered` / `Reducible` / `Liftable` / `BulkOps` tower is the
single most inherited artifact in the library.

It appears in no lane in `roadmap.md` §3, in no wave, and in no milestone's "Lands" list. M1
lists `resolvent-int` and `resolvent-modular` and the ADRs, and nothing else. The first week
(§6) never mentions it either. This is a straightforward omission of the most-depended-on
deliverable in the project.

*Fix:* add **Z0 — `resolvent-base`: trait tower, `Sign`, `Verdict`, `Certified`/`Certainty`,
`Error`/`Unsupported`/`Budget`, and the canonical serializer** as the sole Wave-1 blocking lane,
absorbing Z7 and H2a, with the verdict function "trait-law property tests green for every
instantiation in ADR-006's closed set, and `cargo public-api` snapshot committed". Everything
else in Wave 1 depends on it.

**Restated critical path.** The plan's claimed serialization point is the ADR freeze. The real
one is *the ADR freeze **and** `resolvent-base` **and** the canonical serializer*, in that
order, all three global, all three currently either unstaffed or mislabelled as parallel.

---

## 10. C10 — two Gröbner certificates are weaker than their "Proves" columns (serious)

**(a) The S-pair certificate is vacuous if it reuses the criteria.** §2.5:

> — **`G` is a Gröbner basis** | Every S-pair of `G` reduces to zero modulo `G` (Buchberger's
> criterion) | Proves: The basis property | cost: ≈ recomputing the basis

The cost column implies all pairs are checked, which is correct. But nothing *says* so, and
ADR-008's own driver ranking puts the Gebauer–Möller criteria at "four orders of magnitude"
— `yang1`: 1,998,099,720 pairs generated, 148,812 surviving. That is the single largest
speedup in the library and it is precisely the code an agent will reach for when the
verification pass is slow. If the certificate applies the chain and product criteria, then a
bug in those criteria — dropping a pair that was not actually redundant — is invisible to the
certificate, and the resulting object is not a Gröbner basis while passing "Buchberger's
criterion".

*Fix:* one sentence in the row — **"the certificate enumerates **all** `C(|G|,2)` S-pairs and
may not consult any pair-elimination criterion; a criteria-aware verifier is a separate,
explicitly-named `*_fast_recheck` used only as a pre-filter."** And add a mutant (C1): a
Gebauer–Möller implementation that drops one extra pair class, which must be rejected.

**(b) The FGLM certificate does not check the thing FGLM produces.** §2.5:

> FGLM change of order | The lex basis reduces every element of the drl basis to zero **and**
> vice versa; both ideals have the same dimension and degree | **Proves: Ideal equality, hence
> correctness of the conversion**

FGLM's output is not "a generating set of the same ideal"; it is **a Gröbner basis in the lex
order**. The two-way reduction test is evidence of ideal equality on finitely many elements —
and note that reduction modulo a non-Gröbner generating set is not even well-defined as a
normal form, so "reduces to zero" is a weaker statement than it reads. A generating set that
happens to reduce the drl basis to zero, is not a lex GB, and is returned as one, passes this
row and then silently breaks every downstream use (elimination, RUR, univariate projection,
`msolve`-shaped 0-dimensional solving — i.e. all of M8).

*Fix:* add the missing clause — **the lex output must satisfy Buchberger's criterion in the
lex order** (all S-pairs reduce to zero, per (a) above), **and** the lex staircase must have
exactly `dim_ℚ ℚ[x]/I` standard monomials, a number FGLM already computes as the dimension of
the multiplication-matrix space and can therefore assert for free. The second half is the
cheap one and it is nearly complete on its own for the 0-dimensional case FGLM is used in.

---

## 11. C11 — the separation bound's role is unstated, and one reading is silently wrong (serious)

Three statements, none of which pins the question.

- `verification.md` §2.3: "Separation bound | The bound is *valid*: for every pair in the
  corpus, `|α − β| ≥ bound`; and **every verdict reached under the bound** equals the verdict
  reached by unbounded refinement".
- `architecture.md` §5.3: "`Ord` is a real, total, infallible `Ord`, and the separation bound
  is what makes that honest: with a Mignotte–Davenport bound, comparison terminates in a
  computable number of steps, so there is no failure to report."
- ADR-013 §5: "Equality is decided **algebraically** — `g = gcd(a.poly, b.poly)`; if
  `deg g = 0` they cannot be equal and refinement is guaranteed to separate them; if
  `deg g > 0`, a **sign change of `g` across the overlap** certifies equality."

ADR-013's mechanism is sound and does not need the bound to decide `Equal`. But
`verification.md`'s phrase "every verdict reached under the bound" reads as though verdicts
*are* produced by exhausting it, and `architecture.md`'s "there is no failure to report" reads
the same way. The difference is total:

- **If `Equal` is ever concluded from "refined past the separation bound without separating"**,
  then an over-large bound (an off-by-one in the Mignotte–Davenport exponent, a
  `bit_length` vs `ceil(log2)` confusion, a missing factor of the leading coefficient) produces
  **`Equal` for distinct numbers**. That is F2/F3 by another route, it is intransitive, and it
  is exactly the class §2.6's transitivity property exists to catch — except it will not,
  because a systematically over-large bound is over-large for *all* pairs consistently, and
  consistent equality collapse is transitive.
- **If `Equal` comes only from the gcd + sign-change certificate**, an over-large bound causes a
  premature loop exit with no verdict, i.e. an `Internal`/budget failure — loud, debuggable,
  and caught by the step-budget rule.

**Fix, and it is one sentence in ADR-013 plus one row edit:**

> **No `Equal` verdict is ever produced by exhausting the separation bound.** `Equal` is
> produced only by the gcd-plus-sign-change certificate. The bound's sole role is to bound the
> number of refinement rounds before the *inequality* branch is guaranteed to have separated;
> reaching it without either a certificate or a separation is an internal-invariant failure,
> not an answer.

And downgrade the §2.3 row from **CERT** to **INV+PROP**: "for every pair in the corpus,
`|α − β| ≥ bound`" is a finite check of a universally quantified claim, and the corpus is least
likely to contain, by chance, exactly the near-degenerate inputs where an off-by-one bites.
Grade it additionally by *derivation*: the bound's implementation carries the citation and a
symbolic unit test against brute-force certified separations at degree ≤ 6, where the true
separation is computable.

---

## 12. C12 — a composite in the prime table is undetectable by every listed certificate (serious)

§2.1: "Prime generation | Miller–Rabin with the **known deterministic witness sets** for
`n < 2^64` (a proof, not a probable-prime test) | Proves: Primality, deterministically | Does
not prove: — ".

The claim is true of the *mathematics*. The deterministic witness sets (Jaeschke;
Sorenson–Webster) are published results and are Tier A. But the certificate for the
implementation is: the implementation of a table lookup plus a modular exponentiation ladder.
A transcription error in the witness table, or the use of a witness set valid for a smaller
bound than the range actually queried, yields a routine that declares some composites prime —
and it will do so on a *sparse* set, which random testing will not find.

Now trace what a composite `p` in the prime registry does downstream:

- `Fp` arithmetic silently stops being field arithmetic; `inv()` fails for non-units, which the
  code will treat as a bad-prime rejection or produce garbage for.
- The GF(p) gcd can return the wrong degree — which, per C3, is the certificate for the ℤ gcd.
- **CRT combine still certifies.** Its check is `result ≡ rᵢ (mod pᵢ)` and the symmetric-range
  bound. Both hold regardless of whether `pᵢ` is prime. (They also hold if two `pᵢ` are equal —
  see C20.)
- **Rational reconstruction still certifies.** `n ≡ d·a (mod M)`, `gcd(n,d)==1`,
  `|n|,|d| ≤ √(M/2)` are all statements about `M`, not about `M`'s factorization.

So the primality of the prime registry is a load-bearing assumption of the entire modular
architecture, and it is the one assumption with no downstream detector.

**Fix, cheap and complete:**
- **Cross-check the primality routine against an independent one** — a segmented sieve — over
  a committed window (all primes below `2^24`, plus the first `N` entries of the actual
  registry at every magnitude class used: near `2^27`, `2^31`, `2^63`). The registry is
  index-addressed and deterministic (ADR-012), so the exact entries used are known and finite.
  Commit the count and a hash of the accepted set as a golden file.
- **Add the mutant (C1):** a witness table with one entry corrupted, which the sieve
  cross-check must reject.
- Add a row to §3 ("Where certificates run out") naming this as the modular architecture's
  root of trust, alongside §3.4's unlucky primes — they are different failures and only one is
  currently covered.

---

## 13. C13 — the Landau–Mignotte bound has no certificate and feeds §3.2's failure (serious)

§2.4's Hensel row: "Does not prove: Nothing about whether `k` was large enough for
recombination — that is the Landau–Mignotte bound's job and is **a separate assertion**."

Search the document for where that separate assertion is graded. It is not in §2 (no LM row),
not in §3 (no entry), not in M5's exit gate, and not in §7.5's checklist. §3.4 mentions the
bound approvingly — "Where a bound exists — Landau–Mignotte for factors and GCDs, Hadamard for
resultants and determinants — use it and be **deterministic** … and the answer is provably
right" — which is the strongest possible statement of dependence on an uncertified component.

The failure direction matters. A **too-large** bound is a slowdown. A **too-small** bound:

- with Zassenhaus, usually produces no valid recombination, which multiply-back catches. Benign.
- with **van Hoeij**, produces a lattice that has not yet stabilized, in which spurious 0/1
  vectors appear and are accepted by the algorithm's own termination witness (§2.4's van Hoeij
  row item (c) is explicitly "at sufficient Hensel precision" — the precision being the thing
  that is wrong). The output is a **coarse factorization that multiplies back correctly**.
  That is §3.2's named failure, reached through an uncertified input, in the lane the roadmap
  itself calls "the worst agent target in the plan".

**Fix.** Add a row to §2.4 and a generator to §5.2:

> Landau–Mignotte / Hadamard / Cauchy bounds | For every instance from the **known-factorization**
> generator (§5.2), the computed bound is `≥` the true maximum coefficient of any true factor,
> and the ratio `bound / actual` is tracked as a distribution (a sharpness number: an
> astronomically loose bound is valid and makes Hensel lifting unaffordable) | Proves:
> validity on the generated class | Does not prove: validity outside it — the bound's
> derivation must be cited and unit-tested symbolically | `O(1)×` | CERT+SCORE

The known-factorization generator already exists in the fleet, so the true factor coefficients
are known by construction and this check is free.

---

## 14. C14 — the deferred integration: one real collision, and what to do now (serious)

The task asks specifically whether the deferral is as cheap as ADR-018 claims and whether a
hidden collision makes later unification expensive. I checked the consumer directly.

**What I checked and found *not* to be a problem** (stated because it is the obvious hypothesis
and it is wrong):

- `arrangements`' `RealRoot` derives only `Clone, Debug`
  (`crates/lazy-exact/src/roots.rs:316-317`) — **no `PartialEq`, `Eq`, `Ord`, or `Hash`**. So
  resolvent's value-equality `Eq` and its deliberate absence of `Hash` (ADR-014) collide with
  nothing. There is no `HashMap<RealRoot, _>` to break. `QPoly` does derive `PartialEq, Eq`
  (`:42-43`) but that is structural equality on a coefficient vector, which resolvent's
  `UPoly` will also have.
- ADR-018's enumerated cost list (one conversion layer, two `Sign` types, two interval
  implementations) is accurate, and its "what to avoid doing now" list is the right list.
- The ℚ-vs-ℤ coefficient difference is real but cheap at the boundary: `clear_denominators`
  exists in the API sketch and the conversion is one call per polynomial.

**The collision that is real and is not in ADR-018's list.**

`RealRoot::multiplicity(&self) -> u32` is a **method on a stored value**
(`roots.rs:438`), backed by a field. resolvent's ADR-014 removes it: multiplicity "is not a
field of the number", and `isolate_roots` returns `Vec<(AlgebraicReal, u32)>`.

ADR-014 is right about *why* — a root's multiplicity is a property of the polynomial it came
from, not of the number, and two roots with equal value and different source multiplicities
must compare `Equal` (§2.6's F9 property). But the API consequence is stronger than it needs to
be: it moves multiplicity from *the value the consumer stores* into *the tuple the call
returned*, and a consumer that stores roots and later asks for a multiplicity (which this one
does) must now thread a parallel structure. Under option C that is a mechanical edit at every
storage site; under option B the adapter must define its own pair type and the "rename plus
`&mut → &self`" claim in `api-shape.md` §5 becomes false.

**What to do now, and it costs nothing.** Return a named struct rather than a tuple:

```rust
pub struct IsolatedRoot { pub value: AlgebraicReal, pub multiplicity: u32 }
pub fn isolate_roots(p: &UPoly<Integer>) -> Result<Vec<IsolatedRoot>>;
```

This preserves ADR-014's *actual* safety property in full — multiplicity is not part of
`AlgebraicReal`, does not participate in `Eq`/`Ord`/`sign_of`, and cannot leak into identity —
while keeping the consumer's call-site shape (`root.multiplicity`) intact and keeping the
value storable as one thing. It is additive, it is free, and it is strictly better than a
tuple for documentation and for `serde` besides. ADR-014 should say so explicitly, because
"multiplicity is not a field of the number" is currently read by `api-shape.md` L3-1 as license
to put it back on the number, and by ADR-014 as license to make the caller carry two values;
the struct is the reading that is right on both counts.

**Two smaller items for ADR-018's "what to avoid" list, both currently absent:**

- **`SqrtExt<T>` is generic in `architecture.md` §5.4 (`impl<T: Ordered + Field> SqrtExt<T>`)
  and monomorphic in `api-shape.md`.** ADR-018 forbids a generic parameter on `AlgebraicReal`
  by name and is silent about `SqrtExt`, which is the type it *also* requires stay
  first-class. A public generic parameter on `SqrtExt` is the same one-way door for the same
  reason. Decide it in the same place.
- **Pin the `f64` enclosure rounding direction *now*, as a committed conformance vector file.**
  ADR-018 item 4 correctly identifies that two enclosure semantics disagreeing at a filter
  boundary produce a wrong *verdict*, not a wrong number, and that this is the specific failure
  ADR-015 exists to prevent. But it is listed as something a future measurement would settle.
  It is not a measurement; it is a specification, and writing it costs an afternoon: a few
  hundred `(exact value, expected (lo, hi))` pairs including subnormals, values at powers of
  two, exact halves, and the largest finite double, committed in a `publish = false` crate that
  *any* consumer can run against its own interval type. That artifact makes option C's
  hardest item checkable before anyone commits to option C, and it makes option B's adapter
  testable. Nothing else on the deferred list has that property.

---

## 15. C15 / C16 — the license posture: sound framing, one gate that fails its own standard (serious)

**The framing is sound and the mirroring is accurate.** I checked `arrangements/DESIGN.md` §1
verbatim against R1 §6's quotation of it and against ADR-001's adaptation. The quote is
accurate, the substitution of references (Faugère, van Hoeij, Zassenhaus, Collins/Brown,
Rouillier–Zimmermann, von zur Gathen & Gerhard, Cohen, Geddes) is appropriate, and ADR-001's
strongest move — putting **Symbolica in a stricter tier than GPL**, and stating the reason
*because an agent will infer the opposite* — is genuinely good and I have no criticism of it.
The two-category workspace rule and the three-case license-gate regression corpus are the right
mechanisms and are correctly scheduled for day 1.

**The defect.** ADR-001's fourth mechanical gate:

> **A `Derivation:` line in the module doc-comment of every non-obvious algorithm**, citing the
> *paper*, not a reference implementation … A module that cannot cite a paper is a signal it
> was written from a source tree, and review must catch it. CI checks that every file in the
> algorithm crates has one.

Apply `verification.md` §0's own rule to it: *a fail-closed verdict that is trivially
satisfiable is not a verdict.* A `Derivation:` line is satisfied by pasting a citation for a
paper the author never opened, which is exactly what an agent that worked from a source tree
would do. CI checks the line **exists**. It cannot check it is true. The gate detects only the
laziest possible violation.

More importantly, it is **weaker than the posture it claims to mirror**. `arrangements/DESIGN.md`
§1 says: "we did line-level reading, **and the reports in `docs/research/` document it**" — the
claim is tethered to committed artifacts. ADR-001's Tier-B *procedure* has the same tether
("read → write a note in `docs/research/` in your own words → **close the source** → implement
from the note. The note is the artifact that proves the discipline was followed"), but the
*gate* does not check for the note. The one enforceable thing points at a paper; the one
probative thing is unenforced.

**Fix, cheap now and impossible to reconstruct later:**

> `Derivation:` cites **both** the paper **and** a path into `docs/research/`, e.g.
> `//! Derivation: van Hoeij, J. Symbolic Comput. 33(5):425-445, 2002, §3; see
> docs/research/notes-van-hoeij-recombination.md §2.` CI resolves the path, fails if it does not
> exist, and fails if the note lacks a `Sources:` block with a tier tag per reference (R1's own
> §6 recommendation 2, currently unenforced). A note may serve many modules; a module may not
> exist without one.

**On the standing "defer licensing until release prep" policy.** The policy is correct and
most of the licensing work genuinely can wait. But the split is not licensing-vs-not, it is
**provenance-vs-paperwork**, and provenance cannot wait because it is not reconstructible:

| Cannot wait — unreconstructible later | Can wait to release prep |
|---|---|
| `cargo-deny` + the three planted cases (already day 1 — correct) | `cargo-about` attribution file generation |
| The two-category workspace rule (already day 1 — correct) | SPDX headers on every file |
| The `Derivation:` → note tether above | The README framing paragraph |
| A per-lane record of which Tier-B sources were consulted, written at the time | Any actual legal review |
| Resolving `flint-sys`'s missing repo LICENSE — only if it is ever used | DCO/CLA, contributor policy |
| A Tier-A source for every benchmark family (see below) | Trademark, crate-name reservation |

**C16 — the benchmark-corpus provenance hole.** §5.2's pinning rule:

> Katsura-`n` has a checkable invariant … Cyclic-`n` is pinned by its explicit formula.
> **Eco-`n`, Noon-`n`, and Reimer-`n` have no such published invariant, so pin them to a
> specific generator source and commit the SHA-256 of the generated system.**

"A specific generator source" for these three families, in practice, is a Singular `.lib`, an
msolve test directory, or a Groebner.jl benchmark file — GPL-2.0, GPL-2.0, GPL-2.0. §5.1 says
benchmark instances are "generated by committed generators", so an agent following this
instruction transcribes a generator out of a GPL test suite into an MIT repository. That is
precisely the Tier-B transcription ADR-001 forbids without exception, arrived at by following
the verification plan literally, in the one lane where nobody would think to look for a
licensing problem.

*Fix:* for each benchmark family, name a Tier-A source (the original paper — Katsura, Cyclic,
Noonburg, Reimer all have them) or mark the family **unusable** and drop it. Where only a
system's test file states the system, treat the *system itself* as the published mathematical
object (it is), transcribe it from the paper, and assert an invariant that pins it — for Eco-`n`
and Noon-`n` the defining recurrences are published and are short. Add "the family's Tier-A
citation" as a required field of the benchmark generator's metadata, checked by the same CI
rule as `Derivation:`.

---

## 16. C17 — is any of this already solved? (serious, as a scoping question)

**R1 §3.5's answer is sound and I am not disputing it.** I re-read the license verifications
and the reasoning. `symbolica` is proprietary and correctly blocklisted; `algebraics` is
LGPL-2.1 and dead; `polynomen` is GPL-3.0; `num-modular`/`num-prime`/`nalgebra` are Apache-only
and correctly excluded by the §0.9 MIT-arm argument, which is a subtle point most audits miss.
The conclusion — "permissive, stable-Rust, modular-methods-throughout, with algebraic numbers
as a first-class exported type" is unoccupied — holds.

**But the plan never evaluates the one option that would narrow the scope.** R1 §3.2 records
that `feanor-math` (MIT) already ships:

- Cantor–Zassenhaus → **lane K1**
- Hensel-lifting factorization over ℤ/ℚ → **lanes K2, K3**
- LLL → **lane K4**
- "Buchberger's algorithm (F4-style)" → **lane G1**
- `zn_64` Barrett reduction for sub-64-bit moduli, and an RNS `zn_rns` → **lane Z3**

Those five lanes are sized in `roadmap.md` §4 at roughly 25–30 agent-sessions plus two of the
three lanes §4.1 names as the worst targets in the project. `feanor-math`'s stated gaps are
exactly the things resolvent is *for* — no modular-methods coefficient control ("very slow …
coefficient blowup"), no algebraic numbers, no root isolation, ordered-vector monomials — so
this is not "depend on it and abandon the project". It is a narrower question the plan never
asks:

> Would forking `feanor-math`'s finite-field factorization and LLL — MIT, so relicensable into
> MIT OR Apache-2.0 with attribution — be cheaper than writing K1/K2/K4 from papers?

R1 rejects it as a *dependency* on two grounds, both correct for a dependency and neither
decisive for a fork: the `nightly-2026-03-01` pin (a fork can remove nightly features, or find
they are confined to a few call sites) and the missing `LICENSE` file (R1 §8 item 4 already
identifies the fix as "a one-line issue upstream", and it is only load-bearing *if* something
is lifted — i.e. exactly in the fork scenario).

**What would settle it, as a Wave-0 measurement lane alongside Z2 and Y1:**
1. Count and classify `#![feature(...)]` uses in `feanor-math`, and specifically whether any is
   in `factorization`/`lll`/`zn_64` or only in the ring-framework glue. If the algorithm
   modules are stable-compatible, the nightly objection does not apply to a fork of those
   modules.
2. File the upstream `LICENSE`-file issue immediately regardless — it costs one issue and
   removes a blocker that takes weeks of latency to clear later.
3. Decide **per lane**, and record the decision. My prior is that K4 (LLL) and K1
   (Cantor–Zassenhaus) are the plausible lifts and G1 (Buchberger) is not, because ADR-008's
   entire monomial design is different from `feanor-math`'s ordered vectors and the port would
   be a rewrite.

Even if the answer is "write it all from papers", the decision should be *recorded* rather than
*implied*, because "we never considered the only permissive prior art as a starting point" is
the shape of finding an outside reviewer will raise, and R1 §3.2's own verdict — "read it
closely, cite it, differentially test against it" — already licenses everything except the
fork, without saying why the fork is excluded.

**Two smaller "already solved" checks, both handled correctly and worth confirming:** `lll-rs`
(MIT) is named as a candidate with a stated precision risk and a named experiment
(`verification.md` §9 item 6) — correct. `egg`/`egglog` are deferred behind a
resolvent-owned trait with four stated reasons (ADR-017) — correct, and X1 independently
confirms it.

---

## 17. C18 — milestone exit criteria: five vibes, rewritten as gates (minor)

Most exit criteria in `roadmap.md` §1 are genuine gates and I am not going to list the twenty
that are fine. These five are not.

| Milestone | As written (vibe) | As a gate |
|---|---|---|
| **M0** | "The minimizer reduces a planted 20-term counterexample to **its minimal form** automatically." | Delta-debugging yields a *1-minimal* form, not a global minimum. → "The minimizer reduces each of the three planted cases to a form that is **1-minimal** (no single further reduction step in the §4.4 order preserves the disagreement) and to ≤ `k_i` terms, within `T` seconds, where `k_i` and `T` are committed in the test." |
| **M0** | "The score harness … reports a falsification **within budget**." | → "…within `B` CPU-seconds at fleet version 1, where `B` is committed; and reports `survived` at `B` once the stub is fixed; and the two runs at the same `(fleet_version, commit)` are byte-identical." |
| **M1** | "`dashu` measurement notes **committed to `docs/research/`**." | A gate on a file existing. → "A machine-readable table `docs/research/bignum-ladder.toml` containing, for each of the six named instances, `(dashu_ns, rug_ns, ratio)` medians of `k` runs with IQR on the pinned machine; **plus** an explicit decision line: if `ratio > R` at 4k-bit `gcd_ext`, ADR-002 gains an amendment specifying the optional non-default `backend-gmp` feature's shape. `R` is committed before the run." |
| **M2** | "…including Mignotte instances **up to the degree where Sturm remains affordable**." | This is `verification.md` §9 item 5, an open question, used as an exit criterion. → "Measure `d*` = the largest degree at which Sturm's median runtime on the pinned machine is ≤ `T`. Commit `d*`. Below `d*` the isolation lane's verdict is CERT (Sturm-graded); above it the verdict **degrades to DIFF** (Descartes ↔ ANewDsc ↔ external oracle) and that degradation is recorded in the lane's status, not discovered as a slow CI job." |
| **M3** | "Bernstein: soundness certificate green **and the Unknown-rate ceiling met**." | The ceiling does not exist (C8). → gate on the C8 ratchet: "the Unknown rate is measured, committed to `sharpness-ceilings.toml` in the same PR, and is `0` on the clear-sign sub-corpus." |

**C19, related.** `roadmap.md` §2 gates lanes on ADRs being "**ratified** and merged". All
eighteen ADRs say `**Status:** Proposed (2026-07-31)`. There is no definition anywhere of what
ratification is, who does it, or what changes in the file. The freeze — the plan's single
declared global barrier — is therefore currently unenforceable, and §7's own mitigation ("Wave
2 CI jobs do not exist until the ADRs they inherit are merged. Make it mechanical, not
cultural") has nothing mechanical to key on.

*Fix:* define it in one line and make it greppable. `**Status:** Ratified 2026-08-xx by <name>`
plus a CI rule: for each lane, a committed `lane.toml` naming its gating ADRs; the lane's test
target is `#[ignore]`d (or its crate is absent from the workspace members list) while any
gating ADR's status line does not match `^\*\*Status:\*\* Ratified`. That is ten lines of CI and
it converts the freeze from an intention into an edge.

---

## 18. C20 / C21 / C22 — three concrete missing clauses and two harness gaps (minor)

**C20a — CRT does not check its moduli.** §2.1: "CRT combine | Result `≡ rᵢ (mod pᵢ)` for every
`i`; result in the symmetric range | **Proves: Full correctness of the combination**". Uniqueness
requires the `pᵢ` be pairwise coprime. If the prime registry yields a duplicate — an off-by-one
in an index-advance loop, which is a real bug class in an index-addressed registry (ADR-012) —
the congruence check passes trivially for the duplicate and the effective modulus is smaller
than `Π pᵢ`, so the reconstruction bound is wrong while every certificate is green. *Add:* the
moduli multiset is asserted pairwise distinct (they are prime by C12, so distinctness implies
coprimality), and `M = Π pᵢ` is asserted `≥` the bound the caller sized against.

**C20b — factorization does not check its factors are pairwise non-associate.** §2.4 half 1
proves the product; half 2 proves each factor irreducible. Neither rejects `f = g · g` returned
as two multiplicity-1 factors instead of `g²`. Both certificates pass; the multiplicity data
is wrong, and multiplicity is what M5 exists to give the consumer (`roadmap.md` §5: "Intersection
multiplicity beyond the parity heuristic"). *Add:* factors are pairwise non-associate after
canonical normalization, and the exponent multiset is asserted against the input degree.

**C20c — the resultant's fourth check inverts the milestone order.** §2.3 lists "(d) Poisson
product `Res = lc(f)^{deg g} Π g(αᵢ)`, exact for small degree **over a splitting field**".
Constructing a splitting field is `GF(p^k)` at best and number-field arithmetic at worst — M1
and M8 respectively. Listing it as a Layer-2 (M4) certificate creates a dependency on a lane
three milestones later. It is fine to keep as an *aspirational* fourth route; it is not fine to
leave it unlabelled in a table that lane briefs are written from. *Fix:* mark it "M8; not
available at M4" in the row.

Also worth pinning while three routes are being written: **the degenerate-input convention.**
`Res(f, g)` when a leading coefficient vanishes, when a degree drops, or when an argument is
constant or zero has genuinely different conventions across sources (0, `lc^k`, 1). The fleet
contains "degree-drop specializations" as an *adversarial* family, so all three routes will meet
these instances, and if the convention is unpinned they will disagree there permanently and
every disagreement will be triaged as a bug. Pin it in an ADR **before** T1/T2/T3 start; it is
five lines and it prevents an open triage queue.

**C21 — the regression corpus has no provenance field.** §5.1: the regression corpus is
"append-only", "deletion requires a recorded justification", and gates at "100% pass, always. A
gate, not a score". That is right for counterexamples, whose expected outcome is "does not
crash / self-certifies". It is dangerous for the "hand-authored known-answer instances" stored
in the same corpus: an expected answer that entered the corpus from a mis-triaged Class-B
disagreement, or from an oracle that was itself wrong (§4.4 step 3 explicitly contemplates "the
oracle is wrong or out of range"), becomes a permanent gate that a *correct* future
implementation fails. Append-only plus 100%-gate means the corpus can only accumulate such
entries.

*Fix:* every corpus entry carries `provenance ∈ { constructive-generator, oracle-consensus(k
systems), hand-computed(author, method), minimized-counterexample }`. Entries with
`provenance = oracle-consensus` must name the systems and versions and are **re-derivable**:
a nightly job re-asks the oracles and flags drift. Entries with `hand-computed` carry the
derivation. This costs one field and it is the difference between institutional memory and
institutional debt.

**C22 — oracle adapters are graded by round-trip, which proves nothing about the oracle.**
Lane H4's verdict function: "Round-trip; correct Class A/B classification on planted
disagreements." The round-trip is resolvent → S-expression → resolvent, which exercises
resolvent's own encoder and decoder and never establishes that *sympy read the same polynomial*.
An adapter that emits variables in the wrong order, or that emits a polynomial in `x` where
resolvent meant `y`, round-trips perfectly and then produces confident agreement or confident
disagreement about the wrong object. §4.3 correctly identifies "Order convention is the
number-one false disagreement" and then puts the burden on the *comparison*, not on the adapter.

*Fix:* each adapter ships an **oracle calibration corpus** — a dozen instances per operation
whose answers are hand-computed and committed, with the oracle's answer asserted against them.
`Res(x²−2, x²−3)`, `gcd(x²−1, x³−1)`, `factor(x⁴+1)` over ℚ and over `GF(3)`, `isolate_roots` of
a Chebyshev polynomial. If the oracle's answer to a known-answer instance is wrong, the adapter
is wrong, and this is the only test that can tell you so. It is also the test that catches an
oracle version bump changing a convention, which §4.4 records but does not detect.

---

## 19. What I checked and found sound

Listed by name so this document is a coverage report and not only a complaint list. I looked
for a problem in each of these and did not find one.

- **§3.13 (sharpness gates) is the best idea in the plan.** The observation that every
  soundness certificate is satisfied by a maximally conservative implementation, and that
  "an agent optimizing for a green suite converges on an implementation that is sound and
  worthless", is correct, non-obvious, and I have not seen it stated this cleanly in a test
  plan. My only criticism is C8 — that no ceiling is a number yet — which is a gap in
  execution, not in the idea.
- **§3.1 (exponent overflow) diagnoses the right failure** and the guard-bit-in-release,
  `Result`-returning-multiply, widen-and-restart design is right. ADR-008 §3's observation that
  a same-width field set means one degree-field comparison suffices is a genuine simplification.
  My criticism (C6) is of the *sweep*, not of the mechanism.
- **§3.14's diagnosis is correct even though its enforcement is missing (C4).** Specifically,
  identifying `Integer`, `UPoly` and the monomial layer as *common mode* for nearly every
  internal cross-check, and concluding that they need external differential testing more than
  anything else, is exactly right and is not the conclusion most people draw.
- **§2.6's Layer-3 property suite is the strongest section in the document.** Trichotomy,
  transitivity with a generator that produces `2^-1000` separations, sort stability under
  shuffling, enclosure consistency, refinement idempotence, and — the one most plans omit —
  step budget with exhaustion as *failure* rather than skip. The reasoning that "a wrong
  implementation of `AlgebraicReal` hangs; it does not return the wrong answer" is correct and
  is the right thing to build the layer's grading around.
- **§4.3's normalization table.** Every row is right, and the two hardest — never comparing
  Gröbner generator lists, and refusing "up to sign" unconditionally on resultants in favour of
  the explicit `(−1)^(mn)` rule — are exactly the two that a naive harness gets wrong.
- **§4.4's triage pipeline**, with automatic minimization before a human sees anything and with
  "not a bug" outcomes recorded. The ordering of the minimization steps (drop terms → halve
  coefficients → reduce degree → reduce variables → reduce generators) is cheapest-structural-
  first and is right. (Conditional on C1: the classifier is only as good as the certificates.)
- **§5.3's score shape.** "Falsification budget in CPU-seconds against a versioned generator
  fleet" is a better metric than a pass rate for precisely the reason given — a pass rate is
  gamed by weakening tests, and a fleet-version bump is a visible diff. The re-baseline
  semantics ("a dropping score after a re-baseline is progress and is labelled as such") are
  right.
- **§6.1's honesty rules**, especially "compare like with like" — recording the certification
  mode of both sides and refusing to print a cross-mode comparison unlabelled. A certified
  resolvent losing to uncertified msolve *by construction* is the correct thing to say out loud
  before the benchmark exists.
- **"Build the oracle side first, every time" (§4.3)**, and its CI enforcement ("a SCORE lane's
  CI job does not exist until its oracle lane is green and frozen"). Sturm-before-Descartes,
  Buchberger-before-F4, Zassenhaus-before-van-Hoeij, T5-before-T6. This is the single most
  valuable structural rule in the roadmap.
- **The M4-before-M6 reordering** and the two-trunk observation. The evidence is checked and
  correct: `arrangements` touches no multivariate machinery, so the consumer-unblocking release
  is the elimination milestone, not the Gröbner one. The roadmap calls this "the single largest
  schedule win available and it is free" and I agree.
- **`roadmap.md` §4.1 — "which milestones are NOT good agent targets"** is honest in a way plans
  usually are not, and the four it names (the ADR freeze, T6 curve analysis, K5 van Hoeij, G3
  row reduction) are the right four. The inversion written into G5's brief — external
  differential testing as the *primary* verdict rather than the secondary one, against the
  document's own general rule — is a good and unusual piece of self-awareness.
- **ADR-001's Tier C reasoning about Symbolica**, and the explicit statement that the reason is
  written down *because an agent will infer the opposite*. Also ADR-001's rejection of
  "clean-room" as both unnecessary and dishonest, which is the correct call and matches
  `arrangements/DESIGN.md` §1 exactly.
- **R1's license verification method** — crates.io API over GitHub's sidebar badge, with the
  rule stated for future agents, and the recorded self-correction on `Groebner.jl` being GPL
  rather than the assumed MIT. That is the discipline working, and recording the mistake is
  worth more than not making it.
- **ADR-008's demolition of the source spec's own claim** that packed monomials are "most of
  your Gröbner performance", with the measured 15% against linear algebra's 73–91% and the
  divisor index's 10–20×. Correcting the founding brief with evidence is what these documents
  are for.
- **X1 and X2's findings, which I did not re-file.** X2's S1/S2 (the `resolvent-seam` collision
  and the evidence vocabulary living in a zero-dep crate) and S6 (`!Sync`) overlap the territory
  of my C2 from the API side; I have cited the ADR-level contradiction rather than restating
  their evidence, and their conclusions and mine agree.

---

## 20. The five things to do before lane H1 writes a `Cargo.toml`

Compressed, in order, because everything above is long and this is the actionable residue.

1. **Arbitrate C2.** Declare ADR supremacy in one sentence, rewrite `roadmap.md` §2.5 against
   the current file state, and settle the six live contradictions. Four need no experiment.
   Until this is done, fanning out means building two libraries.
2. **Add the mutant requirement (C1)** to `verification.md` §7.5 as item 0, with the mutant-class
   table. This is the difference between a self-certifying library and a library that says it is
   one.
3. **Fix the two gcd certificates (C3)** to use Bézout witnesses, and add the general rule that
   a certificate may not invoke the operation it certifies. This is a ten-line edit to §2 and it
   repairs the two rows the document most confidently marks "Complete".
4. **Add lane Z0 — `resolvent-base` — and sequence it first (C9d)**, absorbing Z7 and the
   canonical serializer. It is the most-inherited artifact in the project and it currently has
   no owner.
5. **Ratchet the sharpness ceilings (C8) and define "ratified" (C19).** Both are ten-line CI
   rules, and without them the plan's two headline enforcement mechanisms — the sharpness gates
   and the freeze — have nothing to key on.

---

## Sources

Plans and decisions read in full: `plans/verification.md` (1,009 lines),
`plans/roadmap.md` (888), `plans/architecture.md` (865), `plans/api-shape.md` (950),
`docs/decisions/ADR-001…018`, `docs/research/prior-art-and-licensing.md` (810).
Existing challenge documents read to avoid duplication:
`docs/research/challenge-generality.md` (X1), `docs/research/challenge-evidence.md` (X2).
Source specification: `/home/dev/projects/IDEAS-crates.md` §4 (lines 112-172).

Consumer code re-read directly for the C14 findings (context only; resolvent depends on
nothing local):
`/home/dev/projects/arrangements/crates/lazy-exact/src/roots.rs:24, 42-43, 316-317, 438,
443, 450, 480, 497, 527, 549, 622`,
`/home/dev/projects/arrangements/DESIGN.md:1-40` (the license-posture paragraph ADR-001
mirrors, checked verbatim).

Capacity figures in C6 are taken from `docs/decisions/ADR-008` §4, not recomputed.
No benchmark number in this document is new; every figure cited is carried from the plan
document that sourced it.
