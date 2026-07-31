# RECONCILIATION — the consumer track audited against the founding document set

**Status:** dated audit record, 2026-07-31. **Normative for nothing.**
**Method:** line-by-line read of `DESIGN.md` (2,087), `VERIFICATION.md` (1,581),
`ROADMAP.md` (833), `NEXT.md` (514), `README.md`, `CLAUDE.md`, `ADR-001…025`, and `API.md`,
against `docs/research/critique-plan.md` (C2, 1,110 lines), `critique-engineering.md` (C1),
`challenge-generality.md` (X1), `challenge-evidence.md` (X2), and the three consumer
evaluations.
**Output:** two things only — a per-finding verification of C2, and a list of divergences
that survived both tracks, each with a proposed amendment naming the ADR that owns it.

---

## 0. Why this file exists, and what it is not

`ADR-019` (§Note, §Consequences) and `API.md` both cite a `RECONCILIATION.md` that had never
been written. `DESIGN.md` §0.4 records that as the repository's "one outstanding
document-integrity defect" and gives two acceptable closures: write it, or delete the
citations. This is the first.

**ADR-021's Alternatives section rejects a *normative* reconciliation file, and it is right
to.** Its argument — "a reconciliation record that is not an ADR has no status field, no
ratification, and nothing keyed on it, which is how it came to be referenced twice and
written zero times" — applies with full force to any file that tries to *decide* something
here. So this file decides nothing. Every actionable item below terminates in one of three
places that *do* have a status field:

- an **append to ADR-021's contradiction register**, which is where a divergence lives;
- an **amendment to a named ratified ADR**, in the form ADR-021 §2 defines;
- a **new ADR (026–028)**, drafted alongside this file with `Status: Proposed`, because an
  agent may draft an ADR and may not ratify one.

If the repository owner prefers ADR-021's shape strictly, the correct closure is to merge
§6's amendments and ADRs 026–028, append §2's rows to ADR-021's register, and then **delete
this file and the two citations to it**. That is a legitimate outcome and it is stated first
so that this document does not become the fourth normative specification the whole document
set exists to prevent.

**Bottom line.** The founding documents are in far better shape than the brief for this
audit assumed. C2's twelve fatal-and-serious findings are **all absorbed**; the residue is
one partially-absorbed finding (C17) and one stale open-item list. The declared L0 one-way
door is **settled**, not contradicted. What is genuinely wrong is smaller and more specific:
five public signatures on which three normative documents disagree, one core capability with
no lane, and one milestone ordering that puts the strongest consumer's largest lift behind a
trunk that consumer has zero demand for.

---

## 1. Unabsorbed C2 findings — the per-finding verification

The brief for this audit stated that C2 died returning its structured output, that its
findings may never have reached the authors, and that any finding not carried into the
design documents is a first-class entry here. **The premise is false and the evidence is
unambiguous.** `DESIGN.md`'s Inputs block names `critique-plan.md` explicitly, declares "the
two critiques are authoritative", and Appendix A maps sixteen C2 findings to the sections
that carry their fixes. `VERIFICATION.md` §14 is a 40-row register of corrections with a
`Source` column, seventeen rows of which cite `C2 §n`. `ADR-023`'s Evidence line reads
"critique-plan.md C1 (fatal), C3, C5, C6, C10, C12, C13, C20". `CLAUDE.md` §3 restates six
of them as build-time rules.

So this section is a coverage report, not a complaint list. Every row was checked against a
specific location, not against a summary table, because an entry in Appendix A is a *claim*
that a fix landed and the claim is what is under audit.

| # | Sev | C2 finding | Absorbed? | Evidence I used to decide |
|---|---|---|---|---|
| C1 | fatal | Certificates are unverified code; no mutant sets | **Yes, fully** | `VERIFICATION.md` §2.1 states rule M verbatim and reproduces the mutant-class table with **two classes C2 did not name** (criterion over-elimination; schedule). `ADR-023` §1 carries it. `CLAUDE.md` §1 bullet 2 restates it. `ROADMAP.md` M1/M2/M5/M6 exit gates name specific required mutants (`gcd → 1`; the `p` vs `p²` loop bound; a Gebauer–Möller variant dropping one pair class). `NEXT.md` day 6 starts mutant sets on day 6, not "later" |
| C2 | fatal | Two plans specify two libraries; six live contradictions, four on one-way doors | **Yes, and over-delivered** | `ADR-021` is a whole ADR for the precedence rule, machine-readable `Status:`, `lanes.toml`, the code-block grep gate, and a **twelve-row** contradiction register — C2 listed six. `DESIGN.md` §0.3 settles eleven divergences in a table. The two stale contradictions C2 identified are recorded as stale in ADR-021 so they are not re-opened. C2's separate factual defect (api-shape L0-12 declaring ℤ/n out of scope while Hensel lifting needs it) is register item 11, `ADR-006` §Tier G, and `ROADMAP.md` M1's Lands list |
| C3 | serious | Both gcd certificates are circular; `gcd ≡ 1` passes both | **Yes** | `VERIFICATION.md` §2.2 is rule C ("a certificate may not invoke what it certifies") and §14 row 2 records the Bézout rewrite. `DESIGN.md` §8.3 carries the argument in the API sketch itself. `ROADMAP.md` M2 exit gate: "gcd certificate 100 % `Proved`, in its **non-circular** form … the identity mutant (`gcd → 1`) is rejected". `NEXT.md` day 6 test row spells out "**Not** `gcd(a/g, b/g) == 1`, which is circular". Note that C2's third sub-case — the ℤ[x] two-part certificate's degree half — is also fixed: the per-prime modular gcd returns its GF(p) cofactors |
| C4 | serious | Oracle independence is asserted, audited by hand, and has no mechanical gate | **Yes** | Not in `DESIGN.md` Appendix A — the one fatal/serious finding that is not — but it landed anyway and in the right document: `VERIFICATION.md` §4.17 is titled "…and its gate", §11.2 walks an "Oracle-independence import manifest", §11.5 item 8 is the lane-checklist entry, and §14 row 5 cites C2 §4. `ADR-021` §3's `lanes.toml` carries the `oracle` column that mechanizes it. `ROADMAP.md` U4, A3 and M4 each commit a permitted-import set; U4's brief names the exact failure C2 predicted ("so it cannot later absorb the PRS it grades") |
| C5 | serious | Randomized certificates run at one fixed seed, which makes them golden tests | **Yes** | `VERIFICATION.md` §2.3 is rule S. `DESIGN.md` §7.2 carries the two-uses-of-randomness resolution in the determinism section, which is the right home. `ADR-023` §3. Seed count reported alongside the score, per C2's fourth sub-point |
| C6 | serious | The 4-bit-field overflow sweep is satisfied by universal overflow | **Yes, in C2's exact corrected form** | `DESIGN.md` §5.2 ("the narrow-field sweep is a distribution assertion, not a disjunction") and `ROADMAP.md` M6's Overflow-sweep gate both carry the `D_max ≤ 2^(w−1) − 1` partition, the false-positive and silent-wrap failure directions, the printed per-width counts, and the clause that does the work: "a width at which zero instances complete is a failed sweep" |
| C7 | serious | "Any decline is a failure" contradicts the decline-rate gate | **Yes** | `DESIGN.md` §6.6 final paragraph; `ADR-011` Amended line cites `critique-plan C7` by name; `ADR-024` §4; `CLAUDE.md` §7. All four carry C2's second-order argument (raising budgets converts declines into sanctioned hangs), which is the part that is easy to drop |
| C8 | serious | No sharpness ceiling is a number | **Yes** | `DESIGN.md` §6.6 states the ratchet with all four of C2's clauses including "`TBD` is not a ceiling" and the never-ratcheted `1.0` floors. `ADR-024` §3. `ROADMAP.md` M3's Bernstein gate is rewritten to gate on the ratchet rather than on a ceiling that does not exist — which was C2 §17's specific complaint about that gate |
| C9 | serious | Fan-out overstates independence at three points; `resolvent-base` has no lane | **Yes, all four sub-points** | (a) `ROADMAP.md` Wave 0 splits **H2a** (serializer, "Blocking for H2b, H3, H4") from H2b. (b) Z7 is gone; its contents are inside Z0. (c) Wave 2's expression trunk marks X1/X3 "M1 only", X2 "**U2**", X4 "**P3**" — exactly C2's decomposition — and `NEXT.md` Week 2 repeats it. (d) **Z0 exists**, is the sole Wave-1 blocking lane, and `NEXT.md` gives it a whole day with the sentence "it appeared in no lane, no wave and no milestone in the previous plan" |
| C10 | serious | S-pair certificate vacuous under Gebauer–Möller; FGLM certificate proves generation, not the GB property | **Yes, both halves** | `ROADMAP.md` M6: "**The S-pair certificate enumerates all `C(\|G\|,2)` pairs and consults no elimination criterion**"; FGLM gate gains both the lex Buchberger criterion **and** the `dim_ℚ ℚ[x]/I` staircase count. `VERIFICATION.md` §14 rows 8–9. `CLAUDE.md` §1 table |
| C11 | serious | Unstated whether `Equal` can be reached by exhausting the separation bound | **Yes** | `ADR-013` Amended line leads with it. `DESIGN.md` §5.3 detail 4 reproduces C2's argument that a systematically over-large bound collapses equality *transitively* and is therefore invisible to the transitivity property test. `ROADMAP.md` M3 turns it into a fault-injection gate (inflate the bound 2×; require internal-invariant failures, never `Equal`). The §2.3 row is downgraded to INV+PROP with the degree-≤6 brute-force test, as C2 asked |
| C12 | serious | A composite in the prime table is undetectable by every listed certificate | **Yes** | `DESIGN.md` §7.2(c) states it as the modular architecture's root of trust with C2's exact reasoning (CRT and rational reconstruction certify statements about `M`, not about `M`'s factorization). `ROADMAP.md` M1 exit gate specifies the sieve window, the magnitude classes, the golden hash, and the corrupted-witness mutant. `CLAUDE.md` §1 table and `NEXT.md` Week 2 Z6 |
| C13 | serious | The Landau–Mignotte bound has no certificate and feeds van Hoeij's coarse factorization | **Yes** | `DESIGN.md` §10.7.6. `ROADMAP.md` M2 (bound validity on every known-factorization instance, `bound/actual` as a tracked distribution) and M5 ("**The Landau–Mignotte bound has its own certificate row** and is not assumed"). Lane U6 and lane K2 both name it. `CLAUDE.md` §1 factorization row |
| C14 | serious | The deferred merge has one unflagged collision: `RealRoot::multiplicity()` is a method on a stored value | **Yes, in C2's proposed form** | `ADR-014` Amended line and §3: `IsolatedRoot { value, multiplicity }`, a named struct, with C2's full argument (the bare tuple forces a consumer that stores a root to thread a parallel structure, which falsifies the "a merge is a rename" claim). `ADR-018` Amended line: "one collision that was not on it is named and fixed". C2's two smaller items also landed: `SqrtExt<T>`'s generic parameter is decided in `ADR-014` §4 and `DESIGN.md` §9.5, and the f64 enclosure contract ships **now** as a committed conformance-vector file (`ADR-015` Amended, `DESIGN.md` §9.4, `ROADMAP.md` M3 exit gate) rather than as a future measurement |
| C15 | serious | The `Derivation:` gate is trivially satisfiable and weaker than the posture it mirrors | **Yes** | `ADR-001` Amended line is exactly this. `DESIGN.md` §2.4 gate 4 requires the research-note path, CI resolution of the path, and a `Sources:` block with a tier tag per reference. `CLAUDE.md` §6 and `NEXT.md`'s "what NOT to do" both carry it. C2's provenance-vs-paperwork split is carried in `CLAUDE.md` §6's closing paragraph |
| C16 | serious | Benchmark-family pinning instructs transcription from a GPL test suite | **Yes** | `DESIGN.md` §2.4 gate 5 with the Katsura ideal-degree `2^(n−1)` invariant as the pinning mechanism. `ADR-016` Amended. `VERIFICATION.md` §8.6 is a whole subsection titled "a licensing hazard inside the verification plan". `ROADMAP.md` §6 risk row |
| C17 | serious | Forking `feanor-math` (MIT) is never evaluated, and it already ships four lanes | **Partially — recorded, not scheduled** | Recorded in two places (`DESIGN.md` §10.7.3 with C2's per-lane prior that LLL and Cantor–Zassenhaus are the plausible lifts and Buchberger is not; `VERIFICATION.md` §13.8). **But C2's fix was a Wave-0 measurement lane "alongside Z2 and Y1", and `ROADMAP.md` Wave 0 contains H1, H2a, H2b, H3, H4, H5, Z2, Y1 and no such lane.** The two concrete actions — classify `#![feature(...)]` uses per module, and file the upstream missing-`LICENSE` issue "immediately regardless" — have no owner, no wave, and no exit gate. See §4 addition A5 |
| C18 | minor | Five milestone exit criteria are vibes | **Yes, all five** | M0's minimizer gate is now "1-minimal … of at most `⟨k_i⟩` terms within `⟨T⟩` seconds"; M0's score gate carries `⟨B⟩` plus byte-identity across two runs; M1's `dashu` gate is `bignum-ladder.toml` with medians, IQR and a pre-committed 8× verdict line; M2 measures and commits `d*` with the CERT→DIFF degradation recorded in the lane's status; M3 gates on the C8 ratchet |
| C19 | minor | "Ratified" has no mechanical definition | **Yes** | `ADR-021` §2 defines it as the owner merging a commit that sets line 3, defines the four legal forms, and states that an agent may draft but not ratify. All 25 ADRs now read `Ratified 2026-07-31`. `ROADMAP.md` M0 requires the gate be **observed blocking** — the scratch-commit test in `NEXT.md` day 1 |
| C20 | minor | CRT moduli distinctness; factors pairwise non-associate; the Poisson-product dependency inversion | **Yes, all three, plus the bonus** | `ROADMAP.md` M1 ("moduli asserted **pairwise distinct**, `M = Π pᵢ ≥` the caller's sizing bound"), M5 ("factors asserted pairwise non-associate … so `f = g·g` returned as two multiplicity-1 factors fails"), M4 ("The Poisson-product check is marked **M8, not available here**"). C2's closing note — pin the degenerate-input resultant conventions before three implementations disagree permanently — became **ADR-025**, a whole ADR |
| C21 | minor | The regression corpus is append-only, gates at 100 %, and has no provenance field | **Yes** | `DESIGN.md` §7.7, `VERIFICATION.md` §8.4, `ADR-024` §2, `NEXT.md` day 3 — all four carry the four-valued `provenance` field, the re-derivable `oracle-consensus` entries and the nightly drift check |
| C22 | minor | Oracle adapters are graded by round-trip, which proves nothing about the oracle | **Yes** | `ADR-016` Amended. `VERIFICATION.md` §6.3 is titled "the check that grades the oracle, not the round-trip". `ROADMAP.md` M0 exit gate lists C2's exact calibration instances (`Res(x²−2, x²−3)`, `gcd(x²−1, x³−1)`, `factor(x⁴+1)` over ℚ and `GF(3)`, `isolate_roots` of a Chebyshev polynomial). `NEXT.md` day 4 |

**Verdict: 21 of 22 fully absorbed, 1 partially.** The C2 digest that never reached the
authors would have been redundant. The finding worth acting on is C17, and it is worth acting
on because its cost is asymmetric: filing an upstream `LICENSE` issue costs one issue and
takes weeks of latency to clear later, and it is load-bearing only in the fork scenario —
which is precisely the scenario nobody has ruled out.

### 1.1 Two open-item lists have gone stale against decisions that landed

Not C2 findings, but the same class of defect and cheap to fix:

- **`VERIFICATION.md` §13 item 7** asks "whether `BulkOps` survives in the trait tower" and
  says "`API.md` INV-14 retains it". `ADR-006` and `ADR-019` **deleted `BulkOps`** on
  2026-07-31. The item is answered; the observation about `API.md` was correct and is now
  fixed (§7 item 3).
- **`VERIFICATION.md` §13 item 9** asks "whether the f64 enclosure semantics can be pinned as
  a committed conformance vector file now" and attributes the "future measurement" framing to
  ADR-018. `ADR-015`'s Amended line and `ROADMAP.md` M3's exit gate both ship it now.
- **`VERIFICATION.md` §13 item 10** asks whether `SqrtExt` carries a public generic
  parameter. `ADR-014`'s Amended line says "`SqrtExt<T>`'s public generic parameter is
  decided" and `DESIGN.md` §9.5 gives the argument (it ranges over resolvent's *own* closed
  set and is bounded by resolvent's own tower, which is what makes it a different door from a
  generic parameter on `AlgebraicReal`).
- **`NEXT.md` §Links** points at `plans/verification.md` for "the verdict functions in
  detail". `VERIFICATION.md` supersedes it and is 1,581 lines to the draft's 1,008.

An open-item list that still asks a settled question is how a future agent re-opens a
one-way door in good faith.

---

## 2. Contradictions that survived both tracks

These are live in the **committed** document set. None was flagged by either track. Three of
the five involve a type name that is not in `ADR-021` §4's grep-gate list, which is why the
gate would not have caught them — so the first proposed action is to extend that list.

### 2.1 `isolate_roots` has three incompatible signatures in three normative documents

| Document | Signature |
|---|---|
| `ADR-014` §3 (ratified, **one-way**, gates A1/A2/A3/K3) | `pub fn isolate_roots(p: &UPoly<Integer>, b: Budget) -> Result<Vec<IsolatedRoot>>;` |
| `DESIGN.md` §5.3 (repeated verbatim at §8.4) | `pub fn isolate_roots(p: &SqfrPoly, window: Option<(&Rational, &Rational)>, b: Budget) -> Result<Certified<Vec<IsolatedRoot>>>;` |
| `API.md` §7.1(c), before this audit | a separate `isolate_roots_in(&p, &lo, &hi, budget)` returning `Certified<Vec<(AlgebraicReal, u32)>>` |

Four axes disagree: the input type, the presence of a window, the `Certified` wrapper, and
the function's name. `ADR-021` §1 gives the ADR the last word on a signature, so as the
register stands **ADR-014's form is binding** — and it is the one form that is provably
wrong on two of the four axes.

**Resolution, on the merits: `(p: &UPoly<Integer>, window: Option<(&Rational, &Rational)>,
b: Budget) -> Result<Certified<Vec<IsolatedRoot>>>`.**

- **`&UPoly<Integer>`, not `&SqfrPoly`.** This is decisive rather than a preference. `DESIGN.md`
  §5.3 defines `SqfrPoly` as "squarefree, primitive, lc > 0" and `square_free` as returning
  `Vec<(SqfrPoly, u32)>`. **A square-free polynomial has every root at multiplicity 1.** So
  `isolate_roots(p: &SqfrPoly) -> …Vec<IsolatedRoot>` returns a struct whose `multiplicity`
  field is the constant `1` — and `IsolatedRoot` exists precisely because `ADR-014` was
  amended, on C2 §14's argument, to keep multiplicity available to a consumer that stores a
  root. `DESIGN.md` §8.4 therefore contains a type whose only interesting field its own
  argument type makes vacuous. Neither track noticed because each wrote one half.
- **Window as a parameter, not a second entry point.** `API.md` L2-1 requires the window
  ("sinbad isolates only within `[t_n, t_n+h]`") and `ADR-014`'s form omits it entirely.
  A second `isolate_roots_in` doubles the surface for an `Option`.
- **`Certified<…>`, not a bare `Vec`.** `API.md` §5.3 and `DESIGN.md` §5.4 both adopt X1
  §1.1's correction that an isolating interval certifies nothing and that the sign-variation
  witness must be returned under `ProofKind::RootCount`. A bare `Vec<IsolatedRoot>` has
  nowhere to put it. `ADR-014`'s signature predates that correction.

`SqfrPoly` stays public and stays on `AlgebraicReal::new`, `defining_poly` and `square_free`
— `DESIGN.md` §5.3's fail-closed-by-type argument is right there, and `API.md`'s previous
claim that `SqfrPoly` is not caller-visible was wrong. **→ ADR-026.**

### 2.2 `rational_between` is total in one document and fallible in another

`ADR-014` §2: `pub fn rational_between(a: &AlgebraicReal, uppers: &[AlgebraicReal]) -> Rational;`
`DESIGN.md` §8.4: `pub fn rational_between(a: &AlgebraicReal, uppers: &[AlgebraicReal], b: Budget) -> Result<Rational, Decline>;`

Both are half right and `INV-6` already contains the whole answer. Two distinct algebraic
reals are separated by a computable bound, so the refinement loop is bound-derived: regime 1,
exhaustion proven impossible, **the query is total**. INV-6's own last clause then says "a
budgeted sibling ships alongside every total query that can allocate unboundedly", and this
one can — it is the same shape as `cmp`/`try_cmp` (`API.md` §7.4 conflict 5), and cadabra2's
two hand-rolled implementations both carry a hard 256-step budget (`API.md` L3-4). So both
signatures ship, named `rational_between` and `try_rational_between`. **→ ADR-026.**

### 2.3 `linalg`'s signatures cannot compile under ADR-006's amended tower

`DESIGN.md` §8.3:

```rust
pub fn row_echelon<C: Field>(rows: Vec<Vec<C>>) -> Result<Echelon<C>>;
pub fn bareiss_det<C: CommutativeRing>(m: &Matrix<C>) -> Result<C>;
```

Under the amended tower there is no `C::zero()` — `Ring::zero` takes `&Self::Ctx`, because
`Fp` carries its modulus by value and cannot answer "zero of which prime field" from
nothing. Row reduction must produce a zero for an all-zero pivot column and Bareiss must
produce a zero determinant, so both need `&C::Ctx`. This is a **knock-on the architecture
track applied to `UPoly` and `MPoly` and missed in `linalg`**: `DESIGN.md` §5.1 detail 1
states the consequence for polynomial types explicitly ("`UPoly<C>` and `MPoly<C>` store one
`C::Ctx`") and §8.3 was not revisited. `API.md` §7.3's sketch already passes `&fp` and is
right. **→ ADR-027.**

The same knock-on applies to `map_coefficients`, where `DESIGN.md` §8.2 has the `ctx`
parameter and `ADR-019` §6's block does not. `DESIGN.md` is right; `ADR-019` §6 needs the
one-word amendment (§6 below).

### 2.4 `ProofKind::DivisibilityAndDegree` names a certificate that was retired as circular

Both `DESIGN.md` §5.4 and `API.md` §5.1 carry the variant `DivisibilityAndDegree` with the
comment "the gcd certificate". That certificate is `H|A`, `H|B` plus
`deg H == deg gcd(A mod p, B mod p)` — and it is exactly the one C2 §3 showed is passed by
`fn gcd(_,_) -> 1`, because the degree half is computed by the routine under test. It was
replaced by the Bézout witness in `VERIFICATION.md` §14 row 2, `ADR-023` §2, `DESIGN.md`
§8.3, `ROADMAP.md` M2's exit gate, `CLAUDE.md` §1 and `NEXT.md` day 6.

The certificate changed everywhere; **the name of the public enum variant that reports it did
not**. That is not cosmetic: `ProofKind` is a public, consumer-read type, and a variant named
for a retired and demonstrably circular argument advertises that argument to every consumer
and invites a future implementor back into it. `API.md` is patched to
`DivisibilityAndBezout`. **→ ADR-028**, which also states the general rule: a `ProofKind`
variant names the certificate that is actually run, renaming one is a breaking change, and
the rename accompanies the certificate change in the same commit.

### 2.5 `Ring::new` takes variable names in one document and an arity in the other

`DESIGN.md` §8.2: `Ring::new(vars: &[&str], order: Order)`, with `var(&self, name: &str) -> Option<VarId>`.
`API.md` §7.3: `PolyRing::new(vars.len() as u32, Order::GrevLex)`.

The adapter that forces the question is solverang's, and its evidence is already in the
document set: it holds `ParamId`s, not names, and builds a ring **per constraint** at runtime
arity 2..14 (`ADR-020` §2, citing `solverang/src/sketch3d/constraints.rs` `Parallel3D` at 12
params and `assembly/constraints.rs` `Insert` at 14). Under the names-only constructor that
adapter must synthesize a `Vec<String>` per constraint per rank query, inside the per-edit
MUS loop that `API.md` §6.3 identifies as the latency class.

Names are a `Display` affordance: `ADR-012` §9's canonical form serializes "exponent vectors
as full-length comma-separated non-negative integers", not names. So **arity is primary**
(`Ring::new(arity: u32, order: Order)`) and `Ring::with_names(&[&str], order)` is an additive
convenience that keeps `var(name)` for consumers that want it. Recorded as an amendment to
`ADR-020` rather than a new ADR, because it does not change the ownership rule that ADR owns.

### 2.6 Smaller divergences, recorded so the grep gate can be pointed at them

| Item | `DESIGN.md` | `API.md` | Which wins |
|---|---|---|---|
| L4 symbol type | `SymbolId` (§8.5) | `Sym` (§7.1 sketches) | `SymbolId`. A rename neither track noticed; `API.md` §10 item 11 records it |
| `sign_over`'s coefficient domain | `sign_over(p: &UPoly<Rational>, …)` (§8.4) | — | Internally inconsistent with `ADR-004` (ℤ-primitive, ℚ is a transport type). Either `UPoly<Integer>`, or `DESIGN.md` must say why this one entry point is the exception |
| `ROADMAP.md` §5 M6 row | "exact medial axis \| Trivariate elimination, **ideal saturation, 0-dimensional solving**" | — | Saturation and 0-dimensional solving are M8's Lands list, not M6's. A ladder row promising an M8 capability at M6 |

---

## 3. The declared L0 one-way door: settled, not contradicted

The brief for this audit reported "a fatal process defect: a declared L0 one-way door with
two incompatible decisions recorded across the ADR set, unflagged". **Both decisions exist,
both are locatable, and the contradiction was found, arbitrated and closed before this audit
began.** Reporting it as live would be wrong.

**The two decisions.**

| | Location | The decision |
|---|---|---|
| **A** | `plans/api-shape.md` §3.2(b), §3.3 and INV-14, quoted verbatim in `ADR-019` §Context | A consumer-implementable coefficient trait is **rejected**. Coefficients are a **sealed** set `{Rational, Integer, FpElem, NfElem}`; "a consumer cannot add a coefficient ring". Separately an **open** six-method `Scalar` trait is introduced for *evaluation* scalars, in a zero-dependency `resolvent-seam` crate |
| **B** | `plans/architecture.md` §2.3, now `DESIGN.md` §5.1, ratified as `ADR-006` | A public **open** tower `Ring → CommutativeRing → {Field, EuclideanDomain, UniqueFactorizationDomain}` with orthogonal markers, and "the modular pipeline is bounded by `C: Reducible + Liftable`, not by `C: Ring`". No scalar seam; `ADR-018` §6.4 forbids adding one by name |

**B wins, and the record is `ADR-019`**, whose title is "one open trait tower, no ops-surface
scalar trait, no seam crate", whose §Context reproduces both positions in a two-row table,
and whose Alternatives section rejects the sealed set on three named grounds. It is also
register item 2 in `ADR-021` and divergence 2 in `DESIGN.md` §0.3.

**Three things make the arbitration sound rather than merely decided**, and they are worth
restating because this is the door everything above L0 inherits:

1. **The argument for A was self-refuting and the refutation is on the record.** api-shape
   rejected an open coefficient trait because it "pushes bignum-shaped obligations … into a
   type whose entire purpose was to be word-sized", then six paragraphs later justified the
   open `Scalar` with "nothing in `Scalar` obliges an implementor to be a bignum". X1 §2
   caught it; `ADR-019` §Context quotes both sentences adjacently. The second sentence is
   what a well-factored tower achieves, and B is that tower.
2. **A failed the founding acceptance criterion outright.** Rule 4 requires an adapter in
   under 200 lines *with zero changes to resolvent*; api-shape §8.4's answer to a
   cryptography consumer needing GF(p^k) with its own basis was "add it to the sealed set",
   i.e. resolvent changes and the consumer waits on an upstream release. `API.md` §8.3
   records that consumer as previously failing and now served.
3. **A's one genuine virtue is preserved.** `ADR-006` Tier G still bounds what *resolvent*
   instantiates over — `Fp`, `Fp4`, `Integer`, `Rational`, `Zn`, `GFpk`, `NumberFieldElem`
   behind a feature — which is the compile-time budget the sealed set was reaching for. What
   was abandoned is only the claim that the set is closed *to consumers*.

**What was genuinely still wrong, and is now fixed here.** The arbitration was recorded in
the ADRs and in `DESIGN.md`, and **`API.md` §3.2 was never updated to match** — it carried
the pre-amendment tower with receiverless `zero()`/`one()`, `Reducible: Ring` with
`type Image: Field` and an `Option` return, `Liftable: Ring` naming an associated type it
does not have, and `BulkOps`; and `API.md`'s solverang sketch wrote `vec![fp.zero(); n]`,
which is the ring-object arithmetic `ADR-006` forbids by name. `CLAUDE.md` §3.1 flags this
at the file-and-line level ("`API.md` §3.2 … `API.md:1188`"). Patched (§7 items 2–4).

So the honest finding is not "an unflagged contradiction on a one-way door" but "a correctly
arbitrated one-way door whose resolution had not propagated to the document that is
normative for the public surface" — which is a smaller defect and a live one, because
`resolvent-base` is `NEXT.md` day 5 and `API.md` is what a lane agent reads for a signature.

---

## 4. Additions: what the architecture documents are missing entirely

| # | Addition | Evidence, and why it is missing |
|---|---|---|
| **A1** | **A lane and a milestone for dense linear algebra.** `row_echelon` with rank, pivot rows, dependent rows **and the transform**; `bareiss_det` | `DESIGN.md` §1.2 lists it in L2's in-scope set, §3.3 puts it in `resolvent-algebra::linalg`, §3.4 argues about which crate it belongs in, and §8.3 gives its signatures. `API.md` L2-6 and L2-7 place it core on two consumers. **`ROADMAP.md` schedules nothing.** T3 is "Bareiss / Bézout determinant route", scoped inside M4 as the third independent *resultant* route — not a public module. Worse, `ROADMAP.md` §5's consumer ladder attributes solverang's entire M1 value to "`resolvent-modular` plus a row-echelon returning rank, pivot rows, dependent rows **and the transform**", and M1's Lands list contains no linear algebra at all. This is the only capability solverang needs above L0, and it is the one cadabra2 needs to replace a measured 2.448 ms recursive Laplace determinant. **→ ADR-027 and §5 item 1** |
| **A2** | **A public signature and an owner for seeded uniform random points over GF(p)** | `API.md` L0-7 places it core on two independent consumers with hard requirements (E3 R2; E1 D2 as a prohibition on the alternative). `DESIGN.md` §7.2(b) specifies the counter-based RNG and `Session`/`Seed`, but §8 states no signature for drawing a point, and no `ROADMAP.md` lane names it — Z6 is deterministic *prime* selection, H3 is the harness seed schedule. Schwartz–Zippel, sparse interpolation and modular gcd all need it internally, so it is zero marginal implementation |
| **A3** | **`SqfrPoly`, `Echelon`, `IsolatedRoot`, `Certified` in ADR-021 §4's grep-gate type list** | The gate names `Ring, Reducible, Liftable, Certified, Certificate, Certainty, ProofKind, AlgebraicReal, MPoly, UPoly, Ring, MonomialEntry, Store, Node, IsolatedRoot, SqrtExt`. `Certified` is there; `SqfrPoly` and `Echelon` are not, and they are two of the three types on which §2's divergences turn. A gate that would not have caught the divergences found the week it was written should be widened while widening is free |
| **A4** | **A `no_std` verdict for `resolvent-base` on day 5, not "once the crate exists"** | `DESIGN.md` §1.3 excludes `no_std` as a non-goal while §10.7.7 keeps the question live for `resolvent-base` alone; `API.md` §8.4 and §10 item 2 make it the one thing an embedded/robotics consumer needs to know. `NEXT.md` day 5 writes the crate. Adding `#![cfg_attr(not(feature = "std"), no_std)]` and a `cargo build --no-default-features --target thumbv7em-none-eabi` CI job on the day the crate is written costs an hour; retrofitting after `Error` grows a `String` costs a breaking change. INV-5 already forbids the `String` |
| **A5** | **A Wave-0 owner for the `feanor-math` question (C17)** | Recorded in `DESIGN.md` §10.7.3 and `VERIFICATION.md` §13.8, scheduled nowhere. Two concrete actions with asymmetric cost: file the upstream missing-`LICENSE` issue *now* (one issue; weeks of latency if deferred; load-bearing only in the fork scenario, which is the one not ruled out), and count and classify `#![feature(...)]` uses per module to decide whether the nightly objection applies to a fork of `factorization`/`lll`/`zn_64` or only to the ring-framework glue. Both fit in one S-sized measurement lane next to Z2 and Y1 |
| **A6** | **A stated position on what happens to `API.md` §7.5 and this file once §6's amendments land** | Absent by construction: this file is the first of its kind. §0 states the two acceptable closures |

---

## 5. Roadmap impact

The consumer analysis **confirms** `ROADMAP.md`'s central sequencing bet. M4-before-M6 is
right, the two-trunk split is right, and the claim that the geometry consumer never does
algebraic-number arithmetic is verified against its source. Two orderings are wrong.

### 5.1 The linear-algebra gap, and it is on the shortest path to consumer value

`ROADMAP.md` §5 says CAD constraint solving becomes possible at **M1**. It does not: M1 lands
`resolvent-base`, `resolvent-int` and `resolvent-modular`, and generic rank of a constraint
Jacobian additionally needs `row_echelon` over a field with the transform, which no lane
builds. The gap is small and its position is unusually good:

- `row_echelon<C: Field>` over `FpElem` depends on **nothing above `ADR-006`'s `Field` and
  `resolvent-modular`**. Not `UPoly`, not the monomial layer, not `MPoly` arithmetic.
- It is the whole of solverang's demand above L0 (`API.md` §6.3).
- `bareiss_det` over an integral domain is what cadabra2 asks for to replace its 2.448 ms
  determinant, and `API.md` L2-7 records the shape constraint that matters: **no prime
  appears in the signature** — modular is *how* you make it fast, not *what* was asked.
- The transform is the same object as a Gröbner cofactor representation one layer down, so
  building it early gives G4 a worked precedent rather than an inheritance.

**Proposed:** add lane **`LA`** — *dense linear algebra over a field and over a domain:
`row_echelon` with rank/pivots/dependent rows/transform, and `bareiss_det`* — to Wave 2,
gated on Z0 + Z1 + Z3, sized S–M, certificate-graded (`A = T·A₀` checked by multiplication;
rank cross-checked against an independent minor computation at small size; Bareiss against
the naive Laplace expansion at `n ≤ 6`). It lands in **M2**, not M4, and `ROADMAP.md` §5's
M1 row is corrected to M2. **→ ADR-027.**

### 5.2 M8's dependency on M6 mis-sequences the strongest consumer's largest lift

`ROADMAP.md` M8 declares "**Depends on.** M4, M5, M6" and bundles two unrelated things:

- **number fields** — `UPoly<NumberFieldElem>` plus lane M8-N (the multi-modular
  split-factor driver), subresultant chains and principal subresultant coefficients as
  returned data, cofactor plumbing;
- **0-dimensional solving** — RUR or triangular decomposition, ideal saturation.

Only the second needs M6. RUR reaches lex through FGLM (lane G7) and saturation is a Gröbner
computation; `UPoly<NumberFieldElem>` needs M5's Cantor–Zassenhaus (to factor `f mod p` for
the split-factor driver) and M4's subresultant chain, and nothing from the Gröbner trunk.

Why it matters, in consumer terms rather than schedule terms. cadabra2 is the
`strong-consumer`, and `API.md` §6.2 identifies **number-field linear algebra** as its
largest genuine lift — `classification.rs:441-445`, its largest fail-closed site. `API.md`
§4.4 shows that site closes with **zero new resolvent API**: cadabra2's existing 49-line
`inertia` (`classification.rs:292-338`) becomes generic over `Ordered + Field` and
instantiates at `NumberField`. So the gating capability is `NumberField` existing — which
`ROADMAP.md` schedules dead last, behind M6, a trunk for which cadabra2 has **zero demand**
(zero occurrences of "Gröbner", "Buchberger" or "F4" anywhere in its crates). The same
argument covers cadabra2's quadratic-form factorization over ℚ(√d) and its degenerate-tower
detection.

**Proposed:** split the milestone.

- **M8-A — number fields.** `UPoly<NumberFieldElem>` behind `number-fields`, minimal
  polynomials, degenerate-tower detection, lane M8-N, subresultant chains and principal
  subresultant coefficients as returned data, cofactor/certificate return. **Depends on M4,
  M5.** Its exit gate keeps the ℚ(√2, √3) corpus instance and the `Err(BadPrime)`
  requirement, which are the parts `ADR-006` §Context defect 3 makes load-bearing.
- **M8-B — 0-dimensional solving.** RUR or triangular decomposition, ideal saturation.
  **Depends on M6.** Its consumer is SMT NRA (`API.md` §8.2), not geometry.

This is a re-labelling of an existing dependency graph, not new work, and it moves the
strongest consumer's largest lift off the far side of a trunk it does not use.

### 5.3 Nothing else moves, and cadabra2 is not blocked-now on anything

The brief asked whether cadabra2's "long blocked-now list" collides with M0/M1 scope. **It
does not, and the list itself was wrong.** `API.md` §6.2 records the correction (X2's S5): E2
§4.6 tagged eleven rows `blocked-now`, a value defined nowhere in that document, of which at
least seven are capabilities cadabra2 **runs in production today** via `lazy-exact` — exact
order (`roots.rs:549`), radical sign, root isolation (`roots.rs:327`), gcd/`is_root_of`
(`roots.rs:480`), Bernstein enclosure, interval arithmetic (431 lines), the lazy filtered
real (724 lines), and the scalar seam. Those are **substitution** work with no user-visible
outcome, not unblocking work. Only six rows are genuine lifts, and they land:

| cadabra2 lift-now item | Milestone under the current roadmap | Under §5.2's split |
|---|---|---|
| Resultant / subresultant elimination | M4 | M4 |
| Plane×torus spiric quartic (`plane_torus.rs:24-27`, refusal at `:373-378`) | M4 | M4 |
| Bivariate curve topology | M4 (T6), and `API.md` §10 item 6 flags it as possibly unnecessary | unchanged |
| Degree-4 plane-curve factorization / degenerate-tower detection | M5 | M5 |
| Quadratic-form factorization over ℚ / ℚ(√d) | **M8** (behind M6) | **M8-A** (behind M4+M5) |
| Number-field linear algebra | **M8** (behind M6) | **M8-A** (behind M4+M5) |

So M0 and M1 need no change on cadabra2's account, M4 remains correctly identified as "the
unlock", and the only ordering defect is the last two rows — which §5.2 fixes.

The mirror finding is worth stating because it is rarer and argues against resolvent's own
interest: **sinbad does not need resolvent to ship.** `API.md` §6.1 records that its
strongest use case, MMS forcing generation, is blocked on a *numerical assembly* seam —
`residua`'s volumetric source is `SourceField { per_region: BTreeMap<RegionTag, f64> }`
(`sinbad/crates/residua/src/lib.rs:358-361`), piecewise constant, so a spatially varying
`f(x,y)` cannot be assembled regardless of how `f` is derived — and that the "plexus needs a
small CAS" plan never hardened. `ROADMAP.md` §5's M7 row says "Both currently blocked in
code", which is true of the *symbolic* half and overstates the unblocking. M7's schedule
position (X1+X3 free from the start) is nonetheless correct and should not change.

### 5.4 Do not let a refuted number reach a priority argument

Checked and clean: `ROADMAP.md`, `DESIGN.md`, `VERIFICATION.md` and `NEXT.md` contain no
occurrence of the "18×/40× against LAPACK" figure that X2 §S3 refuted against the adjacent
column of E3's own table (the modular echelon is 4.4× faster at n=200, 1.5× at n=400, and
**2.9× slower at n=800**, so the headline is false at the largest size measured). `API.md`
§6.3 carries the corrected statement and names column-pivoted float QR as the alternative E3
never considered. The durable wins are exactness at near-degenerate configurations and the
`implied_by` certificate, not wall-clock — and lane `LA` above should be sized on that basis,
not on a speedup.

---

## 6. ADR impact

### 6.1 Confirmed by the consumer analysis — no change

`ADR-001` (license posture), `ADR-002` (bignum wall), `ADR-003` (modular in-house),
`ADR-004` (ℤ-primitive — confirmed independently: `API.md` §7.1(c)'s adapter clears
denominators on ingress and every L3 query in §7.2 takes `&UPoly<Integer>`),
`ADR-005` (crate split), `ADR-007` (three representations), `ADR-008`, `ADR-009`,
`ADR-010`, `ADR-011` (INV-6's two regimes are the consumer-facing form of it),
`ADR-012` (two consumers with independent hard determinism requirements),
`ADR-015` (no float interval — confirmed by conflict 10),
`ADR-016`, `ADR-017`, `ADR-018` (the deferral is cheaper under ADR-019, not more expensive),
`ADR-021`, `ADR-022`, `ADR-023`, `ADR-024`, `ADR-025`.

`ADR-013` and `ADR-019` deserve a note because they were the two the consumer track had
argued the other way, and **both are confirmed against this track's own earlier position**.
`ADR-013`'s `Send + Sync` beats the `Send + !Sync` inline-`RefCell` design on every outside
consumer X1 evaluated — a `pyo3` binding is forced to `#[pyclass(unsendable)]`, which
*panics* on foreign-thread access, reintroducing outside resolvent the panic INV-4 forbids
inside it, on the headline type. `ADR-019` is the correct resolution of §3.

### 6.2 Amendments required

| ADR | Amendment | Source |
|---|---|---|
| **014** | §3's `isolate_roots` signature becomes `(p: &UPoly<Integer>, window: Option<(&Rational, &Rational)>, b: Budget) -> Result<Certified<Vec<IsolatedRoot>>>`; §2's `rational_between` gains the `try_rational_between(.., Budget)` sibling and states that the bare form is INV-6 regime 1 | §2.1, §2.2, ADR-026 |
| **019** | §6's `map_coefficients` block gains the `ctx: D::Ctx` parameter, matching `DESIGN.md` §8.2. The block predates ADR-006's `Ctx` amendment, which §1 of the same ADR already adopts — so the ADR currently contradicts itself between §1 and §6 | §2.3 |
| **020** | §1's `Ring::new(vars, order)` becomes `Ring::new(arity: u32, order: Order)` with `Ring::with_names(&[&str], order)` additive. The ownership rule the ADR owns is unaffected | §2.5 |
| **021** | §4's grep-gate type list gains `SqfrPoly`, `Echelon`, `Certainty`, `FuncTable`. The contradiction register gains rows 13–17 for §2.1–§2.5 | §2, §4 A3 |
| **005** | §Decision's sentence placing `row_echelon` and `bareiss_det` in `resolvent-algebra` gains their corrected `&C::Ctx`-taking signatures by reference to ADR-027 | §2.3 |

### 6.3 New ADRs, drafted with this file, `Status: Proposed`

- **ADR-026 — Layer-3 entry-point signatures: `isolate_roots`, `SqfrPoly`, `rational_between`.**
  Resolves §2.1 and §2.2. Amends ADR-014.
- **ADR-027 — Dense linear algebra is public, context-taking, and scheduled.**
  Resolves §2.3 and §5.1. Amends ADR-005; adds lane `LA` to Wave 2 and M2.
- **ADR-028 — A `ProofKind` variant names the certificate that is actually run.**
  Resolves §2.4. Amends ADR-010 §2.

None is a one-way door in the sense ADR-006 is, but all three are public-surface decisions,
so all three are cheaper before `resolvent-base` is written than after.

---

## 7. What changes right now — an ordered checklist

Items 1–6 are done in this commit. Items 7–15 require an editor with write access to the
documents this track may not edit, or the repository owner's ratification.

**Already applied (this commit):**

1. **`docs/decisions/RECONCILIATION.md` exists**, closing `DESIGN.md` §0.4's dangling-pointer
   defect, and states in §0 that it is normative for nothing and how to retire it.
2. **`API.md` §3.2's trait block replaced** with ADR-006's amended tower — `type Ctx` with
   `zero(ctx)`/`one(ctx)`/`ctx()`, `Reducible::Image: CommutativeRing` with
   `reduce -> Result<_, BadPrime>`, `Liftable: Reducible`, `BatchField::inv_batch`, and
   `BulkOps` deleted — with the reason each was wrong stated inline.
3. **`API.md` INV-14 rewritten** to drop `BulkOps` and name `BatchField`, closing
   `VERIFICATION.md` §13 item 7.
4. **`API.md`'s solverang sketch de-ring-objected**: `vec![fp.zero(); n]` →
   `vec![FpElem::zero(&fp); n]`, `Fp` split into `FpParams`/`FpElem`,
   `map_coefficients` given its `ctx`, `row_echelon` given the context argument. This is the
   line `CLAUDE.md` §3.1 cites as `API.md:1188`.
5. **`API.md` §5.1**: `ProofKind` extended to the eight-variant union and
   `DivisibilityAndDegree` renamed `DivisibilityAndBezout`, with the reason;
   `#[non_exhaustive]` added.
6. **`API.md` §7.5 added** — the five divergent signatures, each with the position taken and
   which document ADR-021 §1 gives the last word to. Plus: multiplicity restated as
   `IsolatedRoot` at L2-1 and §5.3; `isolate_roots_in` folded into `isolate_roots` with a
   window; `plans/*` citations retargeted at `DESIGN.md`/`ROADMAP.md`/ADRs.

**Requires the repository owner (ratification) or an editor:**

7. **Ratify or reject ADR-026, ADR-027, ADR-028.** Nothing else in this list that touches a
   signature should land first.
8. **Apply §6.2's five amendments**, each keeping its ADR's `Ratified` date and gaining an
   `**Amended:**` line per ADR-021 §2. ADR-019's is the most urgent: it currently contradicts
   itself between §1 and §6.
9. **Append rows 13–17 to ADR-021's contradiction register** for §2.1–§2.5, and **extend §4's
   grep-gate type list** with `SqfrPoly`, `Echelon`, `Certainty`, `FuncTable`. Do this before
   lane H1 writes the gate, so the gate ships with the list that would have caught these.
10. **Add lane `LA` to `ROADMAP.md` Wave 2 and to M2's Lands list**, and correct §5's M1 row
    from M1 to M2. This is the only capability solverang needs above L0 and it is currently
    unowned.
11. **Split M8 into M8-A (number fields; depends on M4, M5) and M8-B (0-dimensional solving;
    depends on M6)** and update the milestone dependency graph and the §5 ladder.
12. **Add the `feanor-math` classification lane to Wave 0** next to Z2 and Y1, and file the
    upstream missing-`LICENSE` issue today regardless of the outcome.
13. **Fix `DESIGN.md` §8.3's `linalg` signatures and §8.4's `isolate_roots`** to match
    ADR-026 and ADR-027; rename `Sym` → `SymbolId` consistently; decide `sign_over`'s
    coefficient domain against ADR-004.
14. **Strike `VERIFICATION.md` §13 items 7, 9 and 10**, all three answered by ADRs that
    landed the same day, and repoint `NEXT.md`'s Links table from `plans/verification.md` to
    `VERIFICATION.md`.
15. **Decide `resolvent-base`'s `no_std` status on day 5**, when the crate is written, not
    "once the crate exists" — one CI job, and INV-5 already forbids the `String` that would
    break it.

---

## Sources

Documents read in full: `API.md`, `DESIGN.md`, `VERIFICATION.md`, `ROADMAP.md`, `NEXT.md`,
`CLAUDE.md`, `README.md`, `docs/decisions/ADR-001…025`, `docs/research/critique-plan.md`,
and the header, decision and alternatives sections of `docs/decisions/ADR-004, -006, -013,
-014, -019, -020, -021`.

Every claim about an existing repository here is carried from a document that cited it to a
file and a line; no external file was re-read for this audit and **no number in this file is
new**. Where this file states that a document says something, it was checked against that
document's current text on 2026-07-31, not against a summary of it.
