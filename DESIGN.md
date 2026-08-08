# resolvent — DESIGN

**Status:** canonical. **Supersedes `plans/architecture.md`**, which remains readable as the
working notes it was.

**Companion:** `API.md` is canonical for the *consumer-facing surface*. This document is
canonical for *layering, internal structure, and the decisions that shape both*. §0 states
the precedence rule and settles every item on which the two founding tracks disagreed.

**Inputs.** `docs/research/prior-art-and-licensing.md` (R1),
`docs/research/consumer-requirements.md` (R2),
`docs/research/algorithms-and-representation.md` (R3), the consumer evaluations
`consumer-sinbad.md` / `consumer-cadabra2.md` / `consumer-solverang.md`, the four adversarial
reviews `challenge-generality.md`, `challenge-evidence.md`, **`critique-engineering.md`**,
**`critique-plan.md`**, `docs/decisions/ADR-001…020`, `plans/{verification,roadmap}.md`, and
`/home/dev/projects/IDEAS-crates.md` §4.

**The two critiques are authoritative.** Where a critique found a defect in an earlier
document, this document states the corrected position, not the original one. Appendix A maps
every finding to the section that carries its fix, and §4.3 lists every ADR that must be
amended to match before its gated lanes start.

Every claim about existing code is cited to a file and a line and was re-verified on
2026-07-31. No benchmark number in this document is new; every figure is carried from the
research document that sourced it.

---

## 0. Normativity, and the contradiction census

`critique-engineering.md` §2 is fatal and it is procedural: **two documents each claimed to be
normative, and they specified different libraries** — different crate graphs, different
polynomial coefficient domains, a public scalar seam that a one-way-door ADR forbids by name,
three incompatible `Certificate` shapes, and eleven signature-level divergences in total, of
which the roadmap's own contradiction census flagged two. A freeze keyed on "is the ADR
merged" cannot catch that, because every ADR is already merged.

### 0.1 The precedence rule

> **An ADR is normative for the decision it records. `API.md` is normative for the shape of
> the public surface. `DESIGN.md` is normative for layering, crate structure, internal
> boundaries, and anything not visible to a consumer. `plans/*` are working notes and are
> normative for nothing.**
>
> Where a document contradicts a ratified ADR, the document is a *proposed amendment to that
> ADR*, not a binding statement. Where this document contradicts an ADR file's current text,
> **the amendment listed in §4.3 is required and the lane gated on that ADR does not start
> until it lands.**

### 0.2 Ratification is machine-readable, or it is a convention

Three mechanisms, all cheap, none of which exists yet:

1. **`Status: Ratified YYYY-MM-DD by <name>`** as a greppable line in every ADR's front
   matter. All twenty ADRs currently say `Proposed`, so the freeze — the plan's single
   declared global barrier — has nothing to key on (`critique-plan.md` C19).
2. **A checked-in `lanes.toml`** mapping each lane to its gating ADRs. A lane's test target is
   `#[ignore]`d, or its crate is absent from `[workspace.members]`, while any gating ADR's
   status line does not match `^Status: Ratified`.
3. **A code-block divergence gate.** CI greps every fenced Rust block in `plans/`,
   `docs/decisions/`, `API.md` and `DESIGN.md` for the headline type names — `Ring`,
   `Reducible`, `Liftable`, `Certified`, `Certificate`, `ProofKind`, `AlgebraicReal`, `MPoly`,
   `UPoly`, `Ring` (context) — and fails when two blocks define the same name differently.
   That single check would have caught eleven of the divergences below.

### 0.3 The eleven divergences, settled

| # | The divergence | Settled as | Where |
|---|---|---|---|
| 1 | Crate graph: `base/int/modular/poly/algebra/real/expr` + facade vs `seam/int/modular/poly/linalg/engine/alg/expr/lazy` | **`base/int/modular/poly/algebra/real/expr` + facade.** No `resolvent-seam`, no `resolvent-linalg`, no `resolvent-lazy`. Dense linear algebra lives in `resolvent-algebra::linalg` | §3.4, ADR-005 |
| 2 | A public `Scalar`/`ScalarOrd`/`TryDiv`/`Hom` seam crate vs no consumer-shaped scalar seam | **No scalar seam and no seam crate.** One open trait tower in `resolvent-base` serves both coefficients and evaluation | §5.1, ADR-019 |
| 3 | `Interval<f64>` as a core type vs no float interval anywhere | **No float interval type in any published crate.** Rational endpoints in, outward-correct `(f64, f64)` out | §5.5, ADR-015 |
| 4 | `AlgebraicReal { poly: Arc<SqfrPoly<Rational>> }` vs ℤ-primitive | **ℤ-primitive.** `SqfrPoly` survives as a fail-closed newtype, but over `UPoly<Integer>` | §5.3, ADR-004 |
| 5 | `mult: u32` as a field of the number vs multiplicity as a pair element | **Neither: a named struct.** `IsolatedRoot { value: AlgebraicReal, multiplicity: u32 }`. Multiplicity is not part of identity and does not participate in `Eq`/`Ord`/`sign_of`; the consumer still stores one value and writes `root.multiplicity` | §5.3, ADR-014 |
| 6 | `Send + !Sync` with an inline `RefCell` vs `Send + Sync` with `Arc<Inner>` | **`Send + Sync`, `Arc<Inner>`, `&self` monotone refinement, shared refinement progress** | §5.3, ADR-013 |
| 7 | Inline `(PackedMon, C)` terms vs `(MonomialId, C)` into an interned arena | **Ownership settled** (the arena is owned by the `Ring` context value; there is no global or implicit interner). **Term type still open**, decided by the replay microbenchmark in §10.2 before the multivariate trunk starts | §5.2, ADR-020 |
| 8 | "No public owned type carries a lifetime" vs `MPoly` borrowing `&Ring` | **Owned handle.** `MPoly` carries `Arc<Ring>` (or an index into a caller-held ring table), never `&'a Ring` | §5.2, ADR-020 |
| 9 | Three `Certificate`/`ProofKind` shapes | **`Certified<T> { value, certainty }` plus `Certificate<C: Claim>` with private fields, no public mint, public read accessors and a structural tether.** `ProofKind` is the union of the three variant sets | §6.4 |
| 10 | A `Budget` on every entry point vs budgets only where no a-priori bound exists | **Two regimes, declared per entry point.** Bound exists ⇒ the query is total and the budget is a bug detector. No bound ⇒ the budget is the exit. A budgeted sibling ships alongside every total query that can allocate unboundedly | §6.3 |
| 11 | `Zn` in the instantiation set vs "ℤ/n is out of scope" | **In scope, and required.** Hensel lifting to `p^k` is arithmetic modulo a composite; M5 cannot exist without it. The "no modular method needs a composite modulus" claim is false | §5.1 |

Items 2, 3, 4, 5 and 10 are real decisions, not editorial drift, and each is argued where it
is settled rather than asserted here.

### 0.4 One outstanding document-integrity defect

`ADR-019` and `API.md` both cite `docs/decisions/RECONCILIATION.md` as the record of how the
two founding tracks were reconciled. **That file does not exist.** Either write it or delete
the citations; a dangling pointer in a ratified ADR is the shape of the problem this section
exists to prevent.

---

## 1. Scope and non-goals

### 1.1 What resolvent is, in one paragraph

resolvent is the algebra engine — polynomials, ideals, algebraic numbers, resultants — that
exact computational geometry, FEM form compilers, and SMT NRA theories call. Its first useful
release is not a CAS. It is roughly twenty-five functions, generalized from degree ≤ 4 to
arbitrary degree, with the coefficient-growth control that makes arbitrary degree computable.
The measurement behind that number: the entire algebraic surface a shipping 17k-LOC exact
geometry engine consumes is re-exported under six names
(`arrangements/crates/lazy-exact/src/lib.rs:82` — `QPoly, RealRoot, RootError, isolate_roots,
sign_radical1, sign_radical2`, plus `SqrtExt` on the next line). ~~Symbolic calculus is a thin
optional layer at the top and is not the point.~~

*Amended 2026-08-08 (ADR-029).* **The scope is a general-purpose CAS**, and symbolic calculus
is a stratum of it rather than an optional layer. What survives unchanged is the sentence
above it: the *first useful release* is still roughly twenty-five functions, and the
twenty-five are still the algebraic ones. The distinction ADR-029 §1 draws is between what is
**in scope** and what is **specified** — most of the analytic surface is in scope and
unspecified, which blocks its lane rather than licensing it. Nothing in this document's
sequencing changes because of the scope declaration; what changes is that the ceiling was
removed, not that the floor moved.

Two framings correct the source specification and this architecture is built around both:

- **The Gröbner one-way doors do not gate the first consumer.** `arrangements` touches no
  multivariate machinery: its polynomial type is a dense `Vec<Rational>`
  (`lazy-exact/src/roots.rs:41-45`), its resultants are hand-rolled 2×2 conic determinants
  (`arrangements/src/geoms/conics.rs:276-287`), and its `RealRoot { poly, lo, hi,
  multiplicity }` (`roots.rs:316-322`) is exactly `AlgebraicReal`. The multivariate/F4 program
  is a **parallel track**, and it stays parallel only because `UPoly<C>` is defined first and
  standalone (ADR-007). This is the single largest sequencing win available and it is free.
- **Packed monomials are not "most of your Gröbner performance."** Measured, packing is worth
  ~15%; sparse GF(p) linear algebra is 73–91% of an F4 run and the divisor-query index is
  worth 10–20× (R3 §1.6). The genuine one-way door in Layer 1 is the *interning/id/key
  structure*, not the field width (ADR-008). A lane brief that says "optimize monomial
  comparison" buys 15% and misses a 20×.

### 1.2 In scope

Layer by layer, the capability set, stated as a boundary rather than a wish list:

- **L0** — ℤ, ℚ, GF(p) for word primes, ℤ/n for composite n, GF(p^k), and — behind a feature
  — ℚ(α). CRT, rational reconstruction, a deterministic prime registry, bulk GF(p) vector
  kernels.
- **L1** — dense univariate `UPoly<C>`; sparse distributed multivariate `MPoly<C>` with
  packed exponents and runtime arity; a borrowed `RecursiveView` for PRS; Kronecker
  substitution as a utility.
- **L2** — gcd and square-free decomposition, resultants and the full subresultant chain,
  univariate factorization over ℤ (Zassenhaus then van Hoeij), real root isolation
  (Descartes/VCA then ANewDsc), Buchberger then F4, FGLM, ideal operations, dense linear
  algebra over a field and over a domain (row echelon with the transform; Bareiss).
- **L3** — `AlgebraicReal` with exact comparison, `SqrtExt` as a first-class degree-2 case,
  radical-tower signs at arbitrary depth, separation bounds, certified Bernstein enclosure,
  `rational_between`, curve analysis, RUR.
- **L4** — a hash-consed expression DAG with an open caller-owned function table,
  differentiation with a caller-supplied leaf rule, constant folding, topological walk,
  canonical bytes, the `is_polynomial_in` bridge down to L1, the **exactness lattice**
  (ADR-031), and **explicit, never-implicit rewriting** — `canonicalize` plus
  `simplify(expr, &RuleSet)` (ADR-033).
- **L5** — series, limits, symbolic integration, ODE, integral transforms, special functions,
  and ADR-032's zero-test tiers. *In scope, and almost entirely unspecified: every capability
  here but the zero-test tiers is blocked on an ADR that does not exist* (ADR-029 §1).

### 1.3 Non-goals, and why each is excluded

These are stated hard enough to hold a line under pressure. Each has a named reason and, where
one exists, the consumer that would be damaged by including it.

| Excluded | Why |
|---|---|
| **Implicit rewriting of any kind** *(replaces "a general `simplify()`", 2026-08-08)* | The consumer evidence is unchanged and is what this row is really about: cadabra2 keeps `Cos2` as a first-class atom *deliberately*, and a canonicalizing rewriter destroys the certificate tether that admits resolvent to its trusted computing base. So the line held is **never implicit**, not **never at all** — a consumer that calls neither `canonicalize` nor `simplify` never has its terms rewritten, and that promise is what the tether rests on. `simplify(expr, &RuleSet, budget)` ships under ADR-033 with the rule set a required argument, no default rule set, and every rule classified by its soundness argument. What stays excluded: an argument-free `simplify`, a `RuleSet::default()`, rewriting as a side effect of construction/`diff`/serialization, and firing a domain-restricted rule on an undischarged side condition. |
| **Unsound zero-testing, at any layer, ever** *(narrowed from "transcendental zero-testing", 2026-08-08)* | Numeric zero-testing — evaluate to `n` digits, compare against a threshold — is banned permanently at every layer; it is the F2 failure in a new costume and produces an intransitive equality. What the old formulation got wrong is that it classified expressions by the *symbols they are written with* rather than the *values they denote*, so it refused `sin(π/6) == 1/2` — algebraic, decidable, and exactly what L3 exists for. ADR-032 permits a **sound** test over a named decidable subclass with its assumption visible in the return type: algebraic constants are `Proved`, a Schanuel-conditional exp-log test is opt-in and never returns `Proved`, everything else is `Unknown`. Richardson/Schanuel still rules out any *general* zero-test, which is why Tier 3 is the default. |
| **Any API taking a tolerance, epsilon, or "close enough" parameter** | Equality-by-tolerance is *intransitive*: with `α < β < γ` and `|α−γ| > ε > |α−β|,|β−γ|` you get `α = β`, `β = γ`, `α ≠ γ`; a sort then produces garbage and a geometry consumer produces a topologically inconsistent arrangement. The first consumer's exact families declare `type Error = Infallible` and its design permanently excludes snap rounding, so a tolerance argument would make resolvent unusable by the consumer it exists for. Grep-gated (L4 in §3.5). |
| **A float interval type in the public API** | Two interval implementations with two enclosure semantics at an adapter boundary produce a wrong *verdict*, not a wrong *number* — much harder to detect. The consumer already owns a careful one (`lazy-exact/src/interval.rs`, 431 lines, no global FPU mode). ADR-015. |
| **A filtered / lazy-exact real number type on the default path** | Filtering *arithmetic* is an orthogonal axis to filtering *algebra*. One consumer wants it and already has a working implementation. Root isolation's internal dyadic filter is a private module, never a published tier. |
| **Numeric root polishing, Newton correctors, homotopy continuation, interval-Newton solvers** | cadabra2 calls this an "attractive nuisance" and has a module that exists specifically so that no numeric polishing enters a decision path. An f64 root-finder in resolvent's API actively damages one consumer and helps none. |
| **BKK / mixed-volume root counting** | Convex geometry over Newton polytopes, not algebra. It belongs in a polytope crate. |
| **A code emitter (Rust/C/WASM printer)** | The consumer that wants one says resolvent must not ship it: it needs Rust closures, the next consumer needs its own opcode tape. resolvent exposes `walk_topological` and stops. |
| **An `egg`/`egglog` dependency** | *Narrowed 2026-08-08.* This row previously excluded five things — an e-graph dependency, a rule language, a built-in rewriter, symbolic integration, and a rational-function type — on the ground that L4 was "not the point". ADR-029 retired that ground and **four of the five are now in scope** (ADR-033 for rewriting and rational functions; `API.md` §4.2 for integration). What survives is the e-graph *dependency* alone, and for its own unchanged reasons: `egg`'s `Language` trait wants to own the term representation, its maintainers point at the successor, and `egglog` churns. Adapters stay external and post-v1 (ADR-033 §6). |
| **Adapter crates, and any optional dependency on an ecosystem crate** | Features are capability-named (`parallel`, `serde`, `simd`, `number-fields`). No feature flag is named after a consumer; a `lazy-exact` feature would be the deferred integration decision smuggled into the one place it must not live. |
| **`no_std`** | `AlgebraicReal` needs `Arc` and an atomic; `dashu` allocates. ~~None of the five prospective consumers is embedded.~~ *That reason is void as of 2026-08-08 — ADR-029 §2 declares embedding a first-class constraint.* The **conclusion** survives on the other reasons, and ADR-029 §2 says so explicitly: `no_std` is "neither promised nor foreclosed". `resolvent-base` holds no arena and no allocation, so the question stays live *for that crate only* (§10.7). |
| **Signature-based Gröbner (F5 and successors)** | The two fastest open implementations both chose non-signature F4. A serious signature implementation could not be shown beating F4-based systems, and F5's *termination* took years and multiple papers to settle — the wrong shape for an agent-built codebase graded by oracles. Recorded as a future lane if a consumer demands syzygies. |

---

## 2. Provenance and license posture

This is placed second, before any architecture, because it is a standing constraint on *how*
every subsequent section gets implemented, not a compliance appendix.

### 2.1 The posture, stated once

> MIT OR Apache-2.0. **Independent reimplementation informed by architectural study of the
> GPL/LGPL sources** — not "clean-room"; that term means the authors never saw the original,
> and we do read Singular, FLINT, PARI, and msolve at the level needed to understand *what*
> they do. Algorithms and ideas are not copyrightable, and here the published literature
> covers the substance more completely than it does for a library whose design lives only in
> its source: Faugère (F4, *J. Pure Appl. Algebra* 139, 1999), van Hoeij (*J. Symbolic
> Comput.* 33, 2002), Zassenhaus, Collins/Brown (subresultant PRS), Ducos (*J. Pure Appl.
> Algebra* 145, 2000), Rouillier–Zimmermann and Sagraloff–Mehlhorn (real root isolation),
> von zur Gathen & Gerhard *Modern Computer Algebra*, Cohen, Geddes/Czapor/Labahn. Process
> discipline: write Rust with the literature notes open, **not** the reference source tree;
> no copied constants, comments, or identifier structure; review diffs against the notes, not
> the sources.

This deliberately mirrors `/home/dev/projects/arrangements/DESIGN.md` §1 — which says of CGAL
"we did line-level reading, and the reports in `docs/research/` document it" — rather than
inventing a second, differently-worded posture for the same problem. **resolvent's position is
materially safer than that project's was**, and the plan should say so rather than performing
anxiety: CGAL's arrangement traits are a large body of design decisions that exist only in the
source, whereas F4, van Hoeij, subresultant PRS and VCA are fully specified in refereed
papers and two standard textbooks.

### 2.2 Three tiers of reading

**Tier A — freely readable, freely cited.** The refereed literature and textbooks above. The
*user-facing documentation and manuals* of any system: documentation describes behaviour, and
matching another system's documented behaviour is a compatibility goal, not a derivation.
Permissively licensed Rust — `feanor-math` (MIT), `dashu`, `ark-ff`. Read freely; still do not
copy verbatim, because MIT carries an attribution obligation and a copied block would need its
notice carried, which defeats the purpose.

**Tier B — readable for *understanding*, never for *transcription*.** Singular, FLINT, PARI,
msolve, CoCoALib, Macaulay2, Sage, `Groebner.jl` (GPL-2.0 — the research phase initially
assumed the usual Julia-ecosystem MIT and recorded the correction), `GroebnerWalk.jl`. SymPy
is BSD and carries no hazard, but takes the same non-copying discipline for consistency.

*Permitted:* reading to understand which algorithm variant is used, why a step exists, what
edge case a guard protects, what the pipeline is.

*Forbidden without exception:* copying code, comments, identifier names, file or module
structure, or **magic constants and tuning thresholds**.

*Procedure:* read → write a note in `docs/research/` in your own words → **close the source**
→ implement from the note.

**Tier C — do not read at all.** Symbolica (its licence grants no copying right and conditions
source-availability — a *stricter* hazard than GPL, and the reason is stated explicitly here
because an agent will otherwise infer that "source-available" is safer to read than GPL, and
the inference runs backwards); any commercial CAS source; and **any repository with no
declared licence**, which means all rights reserved, not fewer.

### 2.3 The threshold rule, which is two rules at once

**Every tuning threshold in resolvent is re-derived by measurement on resolvent's own corpus,
and the measurement is checked in.** A threshold lifted from a GPL source tree is the likeliest
accidental transcription *and* it is someone else's measurement on someone else's machine, so
it is wrong for ours. This rule is simultaneously a licensing rule and a correctness rule,
which is why it holds under pressure where an appeal to hygiene would not. §7.5 makes it
mechanical: every threshold lives in one `Tuning` struct, and a different `Tuning` may change
timing and may not change a value.

### 2.4 The mechanical gates

Habit does not survive agent fan-out. Five gates, all with automatic verdicts:

1. **`cargo-deny`** with an explicit `[licenses] allow` list and every copyleft SPDX id denied,
   running over the **published** graph (`--all-features` minus dev-only features), not just
   direct dependencies.
2. **A regression corpus for the gate**, containing at minimum `malachite` (LGPL-3.0-only,
   hiding behind a permissive-looking pure-Rust crate, because it is *derived from GMP, FLINT
   and MPFR source*), `polynomen` (GPL-3.0-only with an innocuous name), and a synthetic
   Apache-only crate depending on `rug` — the shape `alkahest-cas` 3.7.0 ships today, an
   Apache-2.0 crate with **non-optional** `rug` and `gmp-mpfr-sys` dependencies. **If the gate
   does not fail on all three, the gate is broken.** A gate that has never been observed to
   fail is not known to work.
3. **`cargo-about`** generates the attribution file; a stale attribution file fails CI.
4. **A `Derivation:` line in the module doc-comment of every non-obvious algorithm, citing
   both the paper and a note.** The bare form —"cite the paper" — is satisfied by pasting a
   citation for a paper the author never opened, which is exactly what an agent working from a
   source tree would do; CI can only check that the line exists. The strengthened form is:

   ```rust
   //! Derivation: van Hoeij, J. Symbolic Comput. 33(5):425-445, 2002, §3;
   //!             see docs/research/notes-van-hoeij-recombination.md §2.
   ```

   CI resolves the path, fails if the note does not exist, and fails if the note lacks a
   `Sources:` block with a tier tag per reference. A note may serve many modules; a module may
   not exist without one. This is what makes the discipline tethered to a committed artifact
   rather than to a claim, which is how `arrangements/DESIGN.md` §1 states it.
5. **Benchmark-family provenance.** `plans/verification.md` §5.2 currently says to pin Eco-`n`,
   Noon-`n` and Reimer-`n` "to a specific generator source" — which in practice is a Singular
   `.lib`, an msolve test directory, or a Groebner.jl benchmark file, all GPL-2.0. An agent
   following that instruction transcribes a generator out of a GPL test suite into an MIT
   repository, in the one lane nobody thinks to audit. **Every benchmark family carries a
   Tier-A citation as a required metadata field**, checked by the same CI rule as
   `Derivation:`; a family with no Tier-A source is marked unusable and dropped. The systems
   themselves are published mathematical objects — Katsura, Cyclic, Eco, Noon and Reimer all
   have defining recurrences in the literature — so they are transcribed from the paper and
   pinned by an asserted invariant (Katsura-`n` has ideal degree `2^(n−1)`; a generator that
   does not reproduce it is generating a different system).

### 2.5 The one rule that makes it enforceable

The workspace has **exactly two kinds of crate** and no third category and no per-crate
exception process:

- **`publish = true`** — gated by `cargo-deny` against the permissive allowlist, **and with an
  empty `[dev-dependencies]` table.** The zero-dev-dependency rule is not decorative: `rug` is
  LGPL-3.0+, `cargo publish` records dev-dependencies in the manifest, downstream `cargo test`
  on a published crate would build GMP, and `cargo-deny` is scoped to the published graph
  minus dev-only features so it would not catch it. One line of `cargo metadata` in CI asserts
  the table is empty, which is what makes gate L6 real rather than stated.
- **`publish = false`** — `resolvent-oracles`, `resolvent-bench`, `resolvent-fuzz`. These may
  carry LGPL dev-dependencies and shell out to GPL binaries.

**Every external oracle is a subprocess.** Nothing links. FLINT is LGPL and *could* be linked
via `flint-sys`, and would be materially faster for high-volume property testing — but LGPL §4's
relinking condition is unsettled for statically linked Rust, `flint-sys`'s repository has no
`LICENSE` file despite its crates.io metadata, and a uniform rule is enforceable where a
conditional one is not. The single exception that links is `rug`, as a bignum differential
oracle inside `resolvent-oracles`, testing only `resolvent-int`'s **public** surface — which
the newtype wall makes sufficient by design.

---

## 3. The layer architecture

### 3.1 What each layer owns, and what the layering forbids

| Layer | Owns | May depend on | Forbidden |
|---|---|---|---|
| **L0 — coefficient rings** | The trait tower; ℤ/ℚ over the bignum wall; GF(p), ℤ/n, GF(p^k), the batched tuple ring; CRT and rational reconstruction; the deterministic prime registry; bulk GF(p) kernels | Nothing above L0. `resolvent-base` depends on `thiserror` and nothing else | Any polynomial type. Any monomial concept. Any knowledge of an algorithm above it |
| **L1 — polynomials** | `UPoly<C>`; the `Ring` context value with the monomial arena, arity, order and field width; `MPoly<C>`; `RecursiveView<'a>`; Kronecker substitution | L0 | Any algorithm with a termination argument (gcd, isolation, Gröbner). L1 is representation and arithmetic only |
| **L2 — the engine** | gcd, square-free, resultants and subresultant chains, factorization, Gröbner (Buchberger, F4, FGLM), ideal operations, dense linear algebra | L0, L1 | Any algebraic-number type. Any geometric vocabulary. Any float |
| **L3 — algebraic numbers** | `AlgebraicReal`, `SqrtExt`, radical towers, separation bounds, root isolation, Bernstein enclosure, `rational_between`, curve analysis, RUR | L0, L1, L2 | Any geometric type in any signature. Any tolerance parameter |
| **L4 — expressions** | The hash-consed `Store`, the node set, the caller-owned `FuncTable`, `diff`/`diff_with`, constant folding, `walk_topological`, canonical + provenance bytes, `is_polynomial_in`, the exactness lattice, `canonicalize`, `simplify` + `RuleSet`, assumptions | L0, L1, **L2** *(the `-algebra` edge returns — ADR-033 §5)* | **L3.** No algebraic-number zero test at this layer: the zero-test tiers are L5, precisely so L4 never acquires an L3 edge (ADR-005, amended) |

Two of these deserve their reason spelled out.

**L1 owns no algorithm with a termination argument.** The boundary is exactly "does this
function need a budget or a bound?" If yes it is L2. This keeps `resolvent-poly` a lane whose
verdict is entirely structural — order axioms, round-trips, naive references — and it is why
`UPoly<C>` can ship while the monomial one-way doors are still being decided.

**L4 does not depend on L2 or L3.** `ADR-017` had `resolvent-expr` depending on
`resolvent-algebra` "because it wants gcd for rational-function normalization" — while the
rational-function type is out of scope with no consumer. The dependency is dropped: L4 must
not be able to hold the L3 lane hostage, and nothing in M7's exit gate needs it. If something
in scope later needs gcd at L4, the edge is added then, with the capability that justifies it
named.

### 3.2 Where the layering is a lie, stated honestly

The layer numbering is a dependency order, not a difficulty order, and two things cross it:

- **L2 needs L3's root isolation to *check* itself.** `Res(f,g) = 0 ⇔ deg gcd(f,g) > 0` is
  cross-checked over the reals by isolating the roots of the gcd. That check lives in the test
  harness, not in `resolvent-algebra`, so no dependency edge is created — but a lane brief that
  assumes L2 can be graded without L3 is wrong.
- **L3's curve analysis is where geometry pressure is strongest, and it must not leak.** Its
  inputs are `MPoly`, its outputs are algebraic numbers, counts and index maps. No `Point`,
  no `Curve`, no `Vertex`. That is what keeps "the consumer writes a thin adapter" true while
  still not making every consumer rebuild the hardest component.

### 3.3 The crate split

**Published crates in a strict linear order and four unpublished ones, versioned in lockstep,
with `resolvent` the only crate a consumer is expected to name.**

*Amended 2026-08-08 (ADR-005, ADR-029 §4):* two further published crates —
**`resolvent-calculus`** (L5: series, limits, integration, ODE, transforms, special functions,
and ADR-032's zero-test tiers; depends on `expr`, `algebra`, `real`) and
**`resolvent-display`** (pretty-printing and LaTeX; a leaf, conformance-graded, and still no
code emitter). `resolvent-expr` regains its `resolvent-algebra` edge and deliberately keeps
**no** `resolvent-real` edge — which is why the zero-test tiers sit in `-calculus` rather than
in `-expr`, so L4 stays buildable without L3.

```
resolvent-base      No algebra, no bignum, no allocation of consequence.
                    Trait tower + Sign/Verdict + Certified/Certainty/ProofKind/Certificate
                    + Error/Unsupported/Budget + the canonical serializer.
                    deps: thiserror.
   ↑
resolvent-int       Integer / Natural / Rational newtypes over dashu. The bignum wall.
   ↑
resolvent-modular   Fp (word primes), Zn, GF(p^k), the batched tuple ring, CRT,
                    rational reconstruction, the deterministic prime registry,
                    bulk GF(p) vector kernels, and the single `simd` leaf (§7.6).
   ↑
resolvent-poly      UPoly<C> (dense univariate) · Ring context + monomial arena +
                    MPoly<C> (sparse distributed) · RecursiveView<'a> · Kronecker.
   ↑
resolvent-algebra   gcd, squarefree, resultants + subresultant PRS, factorization
                    (Zassenhaus → LLL → van Hoeij), Buchberger, F4, FGLM, ideal ops,
                    and `linalg` (row echelon with the transform; Bareiss).
   ↑                                    ↖
resolvent-real      root isolation (Sturm oracle, Descartes/VCA, ANewDsc), separation
                    bounds, Bernstein enclosure, SqrtExt, AlgebraicReal, radical
                    towers, rational_between, curve analysis, RUR.
   ↑
resolvent           Facade. Re-exports, feature plumbing, docs, prelude. No algorithms.

resolvent-expr      L4. depends on: base, int, poly.  NOT algebra.  NOT real.

publish = false:
resolvent-oracles   Subprocess drivers for Singular/PARI/sympy/Sage/msolve; the `rug`
                    bignum differential oracle; the committed f64-enclosure
                    conformance vectors (§9.4); the oracle calibration corpora.
resolvent-bench     Benchmark corpus + harness + change-point tracking.
resolvent-fuzz      Structured fuzz targets.
```

### 3.4 Why this split and not a smaller one

The obvious shape is `resolvent-core` (rings + polys) + algebra + real + expr + facade. Two
objections, both load-bearing:

**`resolvent-core` would be the crate every lane touches.** Rings, the bignum wall, the GF(p)
kernels, and both polynomial representations in one compilation unit makes that crate the
serial bottleneck for compilation, for merge conflicts, and for lane independence — which is
exactly what an agent-built project cannot afford. Splitting it four ways gives four lanes with
disjoint files, disjoint test suites, and `cargo test -p` verdicts that do not interfere.

**A consumer implementing a coefficient ring must not have to pull `dashu`.**
`resolvent-base` has no third-party dependency except `thiserror`. A consumer that wants
`UPoly<TheirScalar>` depends on `resolvent-base` alone and never sees a version-pinned bignum
in its tree. That is the "thin adapter the consumer writes" made mechanical rather than
aspirational.

The counter-argument — more crates means more release coordination — is dissolved by
**lockstep versioning**: one `version` in `[workspace.package]`, all crates released together,
inter-crate dependencies pinned `=x.y.z`, and documentation that says the supported surface is
`resolvent`; the inner crates are published so they *can* be depended on directly, not so they
can be mixed across versions.

Two costs are recorded rather than argued away:

- **Ten crate names must be claimed on crates.io before any content**, because names are
  first-come. This is the concrete sense in which the split is costly to reverse.
- **`linalg` living in `resolvent-algebra` means a consumer wanting only "generic rank of a
  Jacobian mod p" compiles the polynomial layer it does not use.** That consumer exists
  (solverang's M1 value is exactly this), and the cost is a compile, not a correctness or
  API cost. If a second such consumer appears, splitting `resolvent-linalg` out is a
  mechanical refactor; merging two published crates strands a name, so the asymmetry says
  start merged.

### 3.5 What the layering forbids, mechanically

Each rule has a named CI gate. There is no habit-based rule in this table.

| # | Rule | Gate |
|---|---|---|
| L1 | No crate depends on a crate above it in §3.3. | Checked-in expected dependency graph; CI diffs `cargo tree --edges normal` against it. |
| L2 | `dashu` appears in exactly one `Cargo.toml`: `resolvent-int`. | `cargo tree -i dashu` lists exactly one direct dependent. |
| L3 | No published crate re-exports a third-party type, and no third-party type appears in any public signature outside `resolvent-int`'s private modules. | `cargo public-api` snapshot, reviewed on diff. |
| L4 | No geometric vocabulary in a published crate: no `Point`, `Curve`, `Segment`, `Arc` (the shape, not `std::sync::Arc`), `Vertex`, `Face`, `tolerance`, `epsilon`, `eps`, `snap`. | grep gate. |
| L5 | No published crate names `arrangements`, `lazy-exact`, `cadabra2`, `sinbad`, or `solverang` in source, `Cargo.toml`, feature name, or doc example. | grep gate. |
| L6 | `publish = false` crates may depend on `publish = true` crates; never the reverse. **Every `publish = true` crate has an empty `[dev-dependencies]` table.** | `cargo metadata` assertion. |
| L7 | `rayon` is behind a default-off `parallel` feature, appears only in `-algebra` and `-real`, and appears in no public signature. | grep + feature-matrix build. |
| L8 | The facade contains no `fn` with a body longer than a re-export or a feature `cfg`. | line-count lint on `resolvent/src`. |
| L9 | Every published crate denies `clippy::unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `arithmetic_side_effects` outside `#[cfg(test)]`. | `cargo clippy -- -D warnings`. |
| L10 | `cargo-deny` over the published graph against the permissive allowlist, with the §2.4 three-case regression corpus asserted to fail. | `cargo deny check`. |
| L11 | `#![forbid(unsafe_code)]` on every crate except the single named leaf `resolvent-modular::simd`, which carries a scoped `#![allow]`, a `SAFETY:` comment on every block, runtime feature detection, and a CI-asserted bit-identical scalar fallback (§7.6). | `unsafe` inventory diff. |
| L12 | Per-crate **absolute** front-end compile-time ceilings, plus `cargo llvm-lines` top-20 monomorphization counts. | §5.6. |

L1, L6 and L10 must exist on day zero, before any algebra. They are cheap now and expensive to
retrofit.

---

## 4. The load-bearing decisions

### 4.1 Index

Each row states the decision and points at its argument. Reversibility is the cost of changing
it *after* fan-out, not before.

| ADR | Decision | Reversibility |
|---|---|---|
| [001](docs/decisions/ADR-001-license-posture.md) | MIT OR Apache-2.0; Tier A/B/C reading discipline; mechanical `cargo-deny` gate | one-way |
| [002](docs/decisions/ADR-002-bignum-backend.md) | `dashu` behind the `resolvent-int` newtype wall; no re-export; conversions over primitives and slices | costly |
| [003](docs/decisions/ADR-003-modular-arithmetic-in-house.md) | Hand-roll `resolvent-modular`; reject `ark-ff` (compile-time modulus), `crypto-bigint` (constant-time tax, wrong sizing), `num-modular`/`num-prime` (Apache-only, which voids the MIT arm) | cheap |
| [004](docs/decisions/ADR-004-z-primitive-coefficients.md) | ℤ-primitive coefficients; ℚ is a boundary façade; root isolation on dyadic intervals | one-way |
| [005](docs/decisions/ADR-005-workspace-crate-split.md) | Published crates in a strict linear order + four unpublished, lockstep versioned. Amended 2026-08-08: `-calculus` and `-display` added | costly |
| [006](docs/decisions/ADR-006-generics-boundary.md) | Generics cross crate boundaries, never inner loops; three tiers; closed *instantiation* set; `LANES` kept open | one-way |
| [007](docs/decisions/ADR-007-polynomial-representations.md) | Three representations; `UPoly<C>` defined first and standalone | one-way |
| [008](docs/decisions/ADR-008-monomial-representation-and-overflow.md) | Packed key + raw exponents + divmask; guard-bit overflow detection; widen-and-restart | one-way (id structure), cheap (field width) |
| [009](docs/decisions/ADR-009-monomial-order-runtime.md) | The monomial order is runtime ring data, normalized into the comparison key at intern time | one-way |
| [010](docs/decisions/ADR-010-modular-methods-and-certificates.md) | Modular methods everywhere; certainty in the return type; two Gröbner modes | one-way |
| [011](docs/decisions/ADR-011-error-model.md) | Fail at construction, not at query; no panics; structured `Unsupported`; two budget regimes | one-way |
| [012](docs/decisions/ADR-012-determinism.md) | Counter-based seeded RNG; index-addressed primes; ordered combination; replayable traces; canonical bytes | one-way |
| [013](docs/decisions/ADR-013-algebraic-real-mutability.md) | `Arc<Inner>`, `&self` monotone refinement, `Send + Sync`, total `Ord` via a separation bound | one-way |
| [014](docs/decisions/ADR-014-algebraic-real-no-hash-no-arithmetic.md) | No `Hash`, no general arithmetic; `canonicalize()` opt-in; multiplicity is not part of identity; **`SqrtExt` stays first-class** | one-way |
| [015](docs/decisions/ADR-015-no-float-interval-type.md) | No float interval in the public API; rational bounds + outward `(f64, f64)` | cheap |
| [016](docs/decisions/ADR-016-oracles-are-subprocesses.md) | Subprocess-only oracles; two crate categories; no exception process | cheap |
| [017](docs/decisions/ADR-017-layer-4-egraph-seam.md) | Resolvent-owned L4 seam; no `egg`/`egglog` dependency. ~~**scope held at M7's exit gate**~~ **§2 bullet 3 and §5–§6 superseded 2026-08-08 by ADR-032/033** | cheap |
| [018](docs/decisions/ADR-018-deferred-consumer-integration.md) | Defer the `arrangements` question; adapter-by-consumer is the default; keep A and C open | cheap by design |
| [019](docs/decisions/ADR-019-numeric-type-seam.md) | One open trait tower; no ops-surface scalar trait; no seam crate; defaulted in-place forms | one-way |
| [020](docs/decisions/ADR-020-arena-and-handle-ownership.md) | Arenas are caller-owned values; handles are arena-relative; no global or implicit interner at any layer | one-way |

**This index stops at 020 and the ADR set does not.** 021–028 and 029–033 exist and are not
listed here; `docs/decisions/README.md` is the complete index and the one to read. Two from the
later set change what is written above rather than adding to it, so they are named here:
**ADR-029** (scope is a general-purpose CAS; embedding is a declared constraint) overrides
§3's framing wherever they disagree, and **ADR-031** (the L4 exactness lattice) is a new
one-way door that did not exist when §3 was written.

### 4.2 Three decisions that are usually got wrong, restated because they will be re-proposed

**ℤ-primitive, not ℚ-primitive (ADR-004).** The nearest reference implementation in this
workspace does the opposite: `QPoly { coeffs: Vec<Rational> }` (`roots.rs:41-45`) with a
monic-normalizing Euclid gcd. At degree ≤ 4 that is entirely fine and it ships. It does not
survive the workload resolvent exists for, for two independent reasons: rational arithmetic
renormalizes with a bignum GCD on every operation — and GCD is precisely where pure Rust is
structurally behind GMP — and coefficient growth is fatal, since two degree-10 bivariate curves
with 32-bit coefficients give a resultant of degree ~200 with ~500-bit coefficients. An agent
reading the nearest available Rust implementation will copy its shape, which is why this is a
written decision rather than a convention.

**Modular methods are structural, not an optimization (ADR-010).** Retrofitting modular
arithmetic under a working ℚ implementation is a rewrite. Sequencing consequence: modular gcd
and square-free decomposition land **before** `Res_y` and curve analysis.

**Order as runtime data normalized into the key (ADR-009).** All three practical orders are
non-negative matrix orders, so `M·a` packed big-endian makes comparison an order-free unsigned
word compare. The grevlex complement construction is proved in ADR-009 and the SWAR arithmetic
checks out: since `2c ≤ 2^w − 2` for any `c` within the guarded payload, the field-wise sum
cannot carry across a field boundary, so testing guard bits *after* the constant subtract is a
sound overflow **and** underflow detector. This dissolves the type-parameter-versus-runtime-data
question instead of trading it off, and it is what makes widen-and-restart possible without a
recompile.

### 4.3 Required amendments — an ADR file that still carries pre-critique text blocks its lanes

This is the concrete residue of §0. Each row is one sentence of ADR text and it must land
before the lanes in the last column start.

| ADR | Amendment | Blocks |
|---|---|---|
| **002** | Reconstruction *is* the large-integer regime; extend the measurement ladder to 64k and 256k bits and add a `rational_reconstruct` microbenchmark at Hexapod's modulus size; record incremental (Garner) CRT, early-termination reconstruction with a doubling modulus, and an in-wall half-GCD as a **planned M1 contingency with a numeric trigger** (§5.7) | Z1, Z2, Z6 |
| **006 / 019** | `Ring` gains `type Ctx` with `zero(ctx)`, `one(ctx)`, `ctx(&self)`; `Liftable: Reducible`; `Reducible::Image: CommutativeRing` with `reduce -> Result<_, BadPrime>`; `BulkOps` deleted; `BatchField::inv_batch -> Result<Self, LaneMask>` added; compile-time budget becomes absolute per-crate ceilings (§5.1, §5.6) | **Z0, Z3, U1 — everything** |
| **008** | `Unsupported::MonomialArenaFull { capacity }` and `Ring::arena_stats()`; `MonomialEntry` splits `W_KEY` from `W_RAW`; the dual-key entry for conversion-pair rings (§5.2) | P1, P2, G7 |
| **009** | Order-specific work is **two** places, not three — divisibility, lcm, gcd and degree are computed from `raw` and are order-free; the claim that "changing a ring's order means re-interning… that is what FGLM does anyway" is wrong and is replaced by the dual-key design (§5.2) | P1, P2, G7 |
| **010** | §5's shared-reducer claim corrected to *shared matrix construction, symbolic preprocessing, monomial layer and row format — not the reducer*; §7 gains the batch-split driver; the cofactor prototype's criterion becomes reconstruction cost, measured on Buchberger at Katsura-6/7 (§10.3); the modular path over algebraic extensions is multi-modular over split factors and is a **lane, not an instantiation** | G3, G4, G5, G6, M8 |
| **011** | `AlgebraicReal` budgets are **always** bound-derived; a decline may never depend on refinement history; `try_cmp(&self, &Self, Budget)` ships alongside `Ord`; no `Equal` verdict is ever produced by exhausting the separation bound (§6.3, §5.3) | A1 |
| **012** | Certificates, `Evidence`, `Telemetry` and `TraceEvent::BudgetTick` are **excluded from canonical bytes**; interning is serialized or two-phase; **no tie-break anywhere may consult `MonomialId` ordering** (§7.3, §7.4) | H2a, P2, A1 |
| **013** | The refinement cache may change how much work a call does and may never change what it returns, including whether it declines — stated as an invariant and property-tested; `Equal` comes only from the gcd-plus-sign-change certificate | A1 |
| **014** | `isolate_roots` returns `Vec<IsolatedRoot>` with a named struct, not a tuple; `SqrtExt<T>`'s generic parameter is decided explicitly (§9.5) | A1, A2 |
| **016** | Published crates have zero dev-dependencies; every oracle adapter ships a hand-computed calibration corpus; every benchmark family carries a Tier-A citation (§2.4) | H4, Z1 |
| **017** | `Simplifier`, `RuleSet`, the built-in rewriter, simplex integration and rational-function normalization move to an explicit "post-v1, on consumer demand" section; the `resolvent-algebra` dependency is dropped | X1–X4 |
| **new 021** | The `unsafe`/SIMD policy: one named leaf, and the published performance gate stated in both forms (§7.6) | G3, Z4 |
| **new 022** | Degenerate-input conventions for `Res(f,g)` — vanishing leading coefficient, degree drop, constant or zero argument — pinned before three independent implementations start disagreeing about them permanently | T1, T2, T3 |
| **new 023** | The ratification protocol: `Status: Ratified`, `lanes.toml`, and the code-block divergence gate (§0.2) | the freeze itself |

---

## 5. Type and trait design

### 5.1 Coefficient rings, and where generics stop

#### The boundary rule

> **Generics may cross a crate boundary. They may not cross into an inner loop.**
> Every generic algorithm has a `where C: CoeffRing` entry point and delegates to a
> monomorphic kernel selected by at most one runtime `match` **per call**, never per element.

**Tier G — generic, monomorphized, source-level.** `UPoly<C>` and `MPoly<C>` arithmetic and the
*reference* implementation of every algorithm. The trait tower is **open** — a consumer may
implement it for its own type — but resolvent itself instantiates over a closed set it
controls: `Fp`, `Fp4`, `Integer`, `Rational`, `Zn`, `GFpk`, and behind a `number-fields`
feature `NumberFieldElem`. A foreign `C` gets correctness, not speed, **and the trait's own
doc comment says so in those words**.

**Tier M — monomorphic, concrete, no trait bounds.** The kernels. The rule of thumb: *any loop
whose body is a single coefficient operation and whose trip count is data-dependent and
unbounded is written over a concrete type.* Named exhaustively:

| Kernel | Concrete over |
|---|---|
| F4 Macaulay row reduction | `u32` payloads + `FpParams` by value; sparse row format |
| GF(p) bulk vector ops (axpy, scale, normalize, dot) | `u32`/`u64`, `FpParams` by value |
| Descartes/VCA Taylor shift `x → x+1` and dyadic scaling `x → 2^k x` | `UPoly<Integer>` plus a dyadic `i64` filter path |
| Sign-variation counting | `&[Integer]`, with a dyadic-approximation fast path |
| CRT accumulation and rational reconstruction | `Integer` |
| Monomial SWAR add/sub/compare/divisibility | `[u64; W]`, `W` a **const generic** over `{1,2,4,8}` — const generics, not trait generics |
| Horner evaluation in the `AlgebraicReal` refinement loop | `UPoly<Integer>` at a dyadic rational |

**Tier D — dynamic, runtime data.** The `Ring` context value carries variable count, monomial
order, exponent field width, characteristic, and the coefficient-ring tag. It is consulted
**once per phase**, never per element.

**Bulk kernels are free functions in `resolvent-modular` over concrete types, selected by one
`match` on the `RingTag` at the top of each phase.** There is no `BulkOps` trait: it would be
either implemented once per `C` — duplicating the kernel across the instantiation set, which
is what Tier M exists to prevent — or a thin forwarder that buys nothing but a bound every
generic call site must carry. Worse, it reads as licence to add `fn row_reduce(...)` next to
it, which blurs the single boundary rule the whole generics design rests on. A generic caller
over an arbitrary `C` gets the naive loop, and the doc comment says so.

#### The trait vocabulary (`resolvent-base`) — corrected

The pre-critique tower is **unimplementable**, and the reason is worth stating precisely
because it is the highest-value hour in the project. `Ring::zero()` and `Ring::one()` were
receiverless associated functions, so they can only be implemented by a type that knows its
own ring **statically**. Of the closed instantiation set, exactly two — `Integer` and
`Rational` — qualify. `Fp` is `Copy` and carries `p` plus its Barrett reciprocal by value, so
`Fp::zero()` has to answer "zero of *which* prime field?" from no information at all. `Zn`
carries `n`; `GFpk` carries `p`, `k` and a modulus polynomial; `Fp4` carries four moduli;
`NumberFieldElem` carries a minimal polynomial. The plan's own worked example already knew and
worked around it without noticing, writing `vec![fp.zero(); n]` — ring-object arithmetic, the
exact shape ADR-006 forbids. Separately, `Liftable: Ring` declared `&[Self::Image]` in its
signature while `Image` is an associated type of `Reducible`; that block does not compile.

The minimal fix preserves everything the design actually wants: **the ring context is consulted
only at construction, which is per-call, which is exactly the boundary rule.** Element-to-element
arithmetic is untouched, so nothing enters the inner loop.

```rust
pub trait Ring: Clone + PartialEq + Send + Sync + 'static {
    const LANES: usize;              // 1 for scalar rings; 4 for the batched tuple ring.
    type Scalar: Ring;               // Self when LANES == 1.

    /// Ring-identifying data an element cannot carry statically.
    /// `()` for Integer and Rational (zero-sized).
    /// `FpParams` for Fp — already carried by value, so `ctx()` is free.
    /// `[FpParams; 4]` for Fp4; `Arc<GfpkParams>` for GFpk;
    /// `Arc<NumberFieldParams>` for NumberFieldElem.
    type Ctx: Clone + PartialEq + Send + Sync + 'static;

    fn zero(ctx: &Self::Ctx) -> Self;
    fn one(ctx: &Self::Ctx) -> Self;
    fn ctx(&self) -> &Self::Ctx;

    // Element-to-element, by reference, inlineable. Unchanged, and never sees a Ctx.
    fn add(&self, r: &Self) -> Self;
    fn sub(&self, r: &Self) -> Self;
    fn mul(&self, r: &Self) -> Self;
    fn neg(&self) -> Self;
    fn is_zero(&self) -> bool;

    // Defaulted in-place forms. A word-sized type ignores them; a bignum overrides them.
    fn add_assign(&mut self, r: &Self) { *self = self.add(r); }
    fn sub_assign(&mut self, r: &Self) { *self = self.sub(r); }
    fn mul_assign(&mut self, r: &Self) { *self = self.mul(r); }
}

pub trait CommutativeRing: Ring {}
pub trait Field: CommutativeRing { fn inv(&self) -> Option<Self>; }
pub trait EuclideanDomain: CommutativeRing {
    fn div_rem(&self, d: &Self) -> Option<(Self, Self)>;
}
pub trait UniqueFactorizationDomain: CommutativeRing { /* content, primitive part */ }

/// Orthogonal capability markers. Absence is a *capability statement*, not a defect.
pub trait Ordered: Ring { fn sign(&self) -> Sign; }   // Integer, Rational. NOT Fp.

pub trait Reducible: CommutativeRing {
    /// NOT `Field`. See below.
    type Image: CommutativeRing;
    fn reduce(&self, m: &Modulus) -> Result<Self::Image, BadPrime>;
}

pub trait Liftable: Reducible {
    fn crt_lift(images: &[Self::Image], moduli: &[Modulus], ctx: &Self::Ctx) -> Result<Self>;
}

/// Batched lanes must be able to report *which* lane failed.
pub trait BatchField: Field {
    /// `Ok` ⇒ every lane inverted. `Err(mask)` ⇒ bit `i` set means lane `i` is
    /// non-invertible, so the driver can split the batch and re-run the bad prime alone.
    fn inv_batch(&self) -> Result<Self, LaneMask>;
}
```

Five details do the work.

1. **`Ctx` is per-type and usually free.** `Integer::ctx()` returns `&()`, a promoted constant.
   `Fp::ctx()` returns a reference to the `FpParams` the element already carries. The only
   allocation is in `GFpk` and `NumberFieldElem`, which carry an `Arc` anyway.
   **Consequence:** `UPoly<C>` and `MPoly<C>` store one `C::Ctx` alongside their coefficients —
   which they need regardless, because a `UPoly<Fp>` that does not know its own `p` cannot be
   printed, serialized, or compared. For `C = Integer` that field is zero-sized.
   **Obligation:** typecheck the whole trait block with `impl`s for `Fp` and `Integer` **before
   the freeze.** A one-way-door trait signature that has never been typechecked is not a
   settled decision.
2. **`Reducible::Image` is a `CommutativeRing`, not a `Field`, and `reduce` is fallible.**
   Asserting that reduction mod `p` lands in a field is false over algebraic extensions: for
   ℚ(α) with minimal polynomial `f`, reduction lands in `GF(p)[x]/(f mod p)`, a field only when
   `p` is inert. And the set of inert primes can be **empty**: `f` has one only if the Galois
   group of its splitting field contains an `n`-cycle, and for the multiquadratic towers
   geometry actually produces — ℚ(√2, √3), Galois group `(ℤ/2)²`, no 4-cycle — *no prime is
   inert and the old trait has no valid implementation at all.* This is the same Chebotarev
   obstruction the plan already documents for Swinnerton–Dyer factorization certificates; it
   was never connected to the trait bound. The consequence for M8 is stated in §10.5: the
   modular path over ℚ(α) is **multi-modular over split factors** — factor `f mod p`, work in
   each `GF(p^{d_i})`, CRT back — which is a different algorithm with its own bad-prime
   predicate, and it is a **lane, not an instantiation**.
3. **The modular pipeline is bounded by `C: Reducible + Liftable`, not by `C: Ring`.** That is
   what makes "modular methods everywhere" a type-level statement rather than a slogan: a ring
   that cannot be reduced mod `p` **cannot compile** into the fast path and gets the generic
   reference implementation instead. Honest, and mechanically enforced.
4. **`Ord` is not required on the coefficient ring.** `Fp4` (four residues at once) has no
   meaningful order, and batching four primes is worth up to ~2.7× amortized. Requiring `Ord`
   on `Ring` would close that door permanently. Nothing sorts coefficients — terms are sorted
   by *monomial* — so the requirement would buy nothing. Sign-dependent algorithms carry
   `C: Ordered` explicitly.
5. **`inv_batch` exists from day one because it is one line now and a breaking change later.**
   Batched multi-modular works only while all `N` primes behave identically, and two things
   certainly break that: a pivot that is zero in one lane, and lead-monomial divergence across
   primes. `Field::inv -> Option<Self>` cannot say *which* lane failed, so the batch cannot be
   split. §7.4 states the batch-split driver that consumes the mask.

#### Explicitly forbidden shapes

- **`Box<dyn Ring>` / `&dyn Ring` in a hot path.** An indirect call per coefficient operation.
  Permitted only in `resolvent-oracles` and in diagnostic/printing code.
- **Ring-object arithmetic — `ring.add(&a, &b)`.** `feanor-math`'s `RingBase`/`RingStore`
  two-trait split exists precisely to work around Rust's borrow and blanket-impl limitations
  under this style, and it is a warning about the design space, not a template. The dismissal
  remains correct about the *cure* — a ring object in every arithmetic call is unacceptable —
  and the `Ctx` fix above is the correct concession to the *problem* it was responding to.
- **Specialization.** Nightly-only. resolvent is stable-Rust-only. Where the source spec says
  "specialization for the hot cases", the mechanism is Tier M.
- **Associated-type projections in public bounds beyond one level.** `where C: Reducible,
  C::Image: CommutativeRing` is the ceiling; deeper produces error messages an agent cannot act
  on.

#### One consequence of the open tower, recorded

`Ring: Send + Sync + 'static` is a real bound and it forecloses a blanket impl from an
ops-surface trait in a glue crate for non-thread-safe types. This is deliberate: `MPoly<C>` and
`UPoly<C>` must be `Send + Sync`, and that is downstream of the determinism contract, not
negotiable for the convenience of a blanket impl.

### 5.2 Polynomials, monomials, and the arena

#### Three representations, explicit conversions, no unification (ADR-007)

```
UPoly<C>            Vec<C> plus one C::Ctx. Low-to-high, trailing zeros trimmed.
                    No monomial type. No order. No Ring context.
                    Layer 2-univariate and all of Layer 3.

MPoly<C>            Terms sorted descending in the ring's order, plus an OWNED handle
                    to the Ring context (arena, arity, order, width).
                    Layer 1-multivariate, F4, elimination.

RecursiveView<'a>   A BORROWED view of an MPoly as D[x_main], built on demand for
                    subresultant PRS. Never owned.
```

There is no representation good at all three access patterns. Univariate root isolation touches
*every coefficient of every intermediate* at every subdivision node and wants O(1) indexed
access into a contiguous array — a Taylor shift is a binomial transform and is inherently
dense. Gröbner touches the *lead term* of many polynomials and does random-access divisibility
queries against a large index. Subresultant PRS is neither: it is univariate pseudo-division
parameterized over a coefficient domain that is itself polynomial, and it wants a recursive
view whose coefficients stay in the distributed arena — building an owned recursive tree is
what makes classical PRS implementations allocate themselves to death.

Conversions are explicit and one-directional in the dependency graph: `MPoly` knows how to
produce a `UPoly` and how to embed one; `UPoly` knows nothing about `MPoly`, `MonomialId`, or
`Ring`. **Defining `UPoly<C>` as a type alias for a one-variable `MPoly` is the specific
mistake this decision exists to prevent**, because it inverts the dependency and makes `UPoly`
require a ring context, an order and an arena to exist. The bridge that matters is
`Res_y: MPoly × MPoly → UPoly<Integer>`.

#### Arena ownership, and the handle rule (ADR-020)

> **Every arena is a value the caller constructs and owns. Handles are arena-relative, never
> serialized, and never escape into a computed result. There is no global, thread-local, or
> implicit interner at any layer.**

`MPoly` carries its ring by an **owned handle** — `Arc<Ring>`, or an index into a caller-held
ring table — never `&'a Ring`. Two requirements force it: `MPoly` must be `Send + Sync` and
storable in a consumer's own struct without infecting that struct with a lifetime; and an
adapter must be able to build rings **from data**, because a real consumer's per-constraint
arity runs 2..14 and a const-generic arity would make that adapter impossible.

The residual hazard is stated rather than hidden: every entry point bounds-checks a handle and
returns `Error::Domain { fault: ForeignNode }` when it is out of range, but **an in-range
handle from a different arena yields a wrong answer, not an error.** The earlier justification
— "a bug none of the three surveyed consumers would make" — was falsified by two of five
outside consumer classes. An optional default-off `store-tags` feature with a **caller-supplied**
tag closes it for consumers that need it; caller-supplied is what keeps it compatible with the
no-ambient-state rule.

#### The monomial entry — corrected

```rust
struct MonomialEntry {
    key_a:   [u64; W_KEY],   // primary order's comparison key. Compare = word compare.
    key_b:   [u64; W_KEY],   // second order's key. Present only in a conversion-pair ring.
    raw:     [u64; W_RAW],   // raw packed exponents. ORDER-FREE.
    divmask: u64,            // Bloom-style filter for fast NEGATIVE divisibility answers.
}
pub struct MonomialId(u32);
```

Three corrections to the pre-critique version, all local and all cheap now:

**`W_KEY` and `W_RAW` are different.** The earlier entry declared `key` and `raw` as `[u64; W]`
with a single `W`. The field counts differ by order: lex needs `n` key fields, grlex needs
`n+1`, grevlex needs `n`, while `raw` always needs `n`. For grlex at 8 variables with 8-bit
fields, `raw` fits in one word and `key` needs two. A single `W` either wastes a word or
overflows.

**Divisibility is computed from `raw` and is order-free.** ADR-009 listed "an order-specific
divisibility direction" as one of three O(1) order-specific sites, "all outside sort inner
loops". That is wrong twice: divisibility is *the* inner loop of symbolic preprocessing and
reducer selection — which is why the divisor-query index is worth 10–20× — and an
order-dependent branch there violates the boundary rule verbatim ("at most one runtime `match`
per call, never per element"). The plan already contained the fix without noticing: `raw` exists
for exactly "divisibility, lcm, gcd, degree queries", and raw exponents are order-free.
**Order-specific work happens in two places: encode at intern time, and the constant subtract
on multiply.** Stated explicitly, because as written a lane brief would produce an
`Order`-matching divisibility routine in the hottest loop in the library.

**Conversion-pair rings carry two keys.** ADR-009 said "changing a ring's order means
re-interning… that is what FGLM does anyway." FGLM does not do that. It walks monomials in
**lex** order while computing normal forms **modulo the drl basis** — which needs drl lead-term
comparison and drl divisibility queries against the drl divisor index — in the same loop. Both
orders are live on the same monomials for the whole computation. Under a single-key design FGLM
would need two `Ring`s, two arenas, two encodings of every monomial, a maintained id bijection
and two divisor indices, none of which is designed and none of which is priced into the lane.
The dual-key entry costs one extra word per *distinct* monomial for rings created as a
conversion pair, and it deletes the bijection entirely. `groebner(_, Order::Lex)` — which
internally runs drl + FGLM — is a conversion-pair ring by construction. **Lane G7 is XL, not
L.**

#### Overflow is fail-closed, recoverable, and never silent

Exponent wraparound is the single most dangerous failure in the library: a wrapped field
silently yields a correct Gröbner basis *of a different ideal*, every certificate passes, and
there is no downstream detector. Three facts make fail-closed cheap: if every field has the
same width, an overflow in any exponent field implies an overflow in the total-degree field,
since `a_i ≤ Σ a_j = deg` always; SWAR guard bits catch it with one AND and one compare per
word; and exponents only grow, so overflow is recoverable by widening and restarting with
bounded lost work.

```rust
impl Ring {
    pub fn mul_monomial(&self, a: MonomialId, b: MonomialId) -> Result<MonomialId>;
    //   Err(Unsupported::TotalDegree { got, max }) on a guard-bit trip. Never wraps.
    //   Err(Unsupported::MonomialArenaFull { capacity }) on id exhaustion.
    pub fn arena_stats(&self) -> ArenaStats;   // distinct monomials, bytes, load factor
}
```

`MonomialId(u32)` caps the arena at 2³² distinct monomials — probably enough, but exhaustion
had no error path and would have been an index panic, violating the absolute no-panic rule.
`arena_stats()` exists because the arena is **monotonic**: monomials interned for S-pairs later
eliminated by Gebauer–Möller are never reclaimed, and a long ℚ run over 2000 primes accumulates
the union across primes. That is probably fine — the monomial *set* is largely
prime-independent — but it is currently a claim made by omission, and one corpus assertion on
the largest instance turns it into a measured memory model.

The top-level driver owns the **widen-and-restart** loop: on overflow, abort, re-encode at
`w' = 2w` (or fall back to unpacked `Vec<u32>` above ~32 variables, where packing stops paying),
restart, and record a `TraceEvent::WidenRestart { from, to }` so the run stays replayable.

**The narrow-field sweep is a distribution assertion, not a disjunction.** The sweep — rerun the
entire Gröbner corpus at a deliberately narrow field width and assert every instance "either
matches the wide run or reports overflow" — is the *only* detector for the library's most
dangerous failure, and as specified it detects nothing. With 4-bit fields and one guard bit the
total-degree bound is **7**, and every instance in the corpus has intermediate bases well past
total degree 7, so the outcome is that *every* instance reports overflow, the second disjunct is
satisfied universally, and the test is green while never once exercising a multiply that
*succeeds* near the boundary — which is where a guard-bit off-by-one lives. Corrected: for each
width `w ∈ {4, 8, 16}` and each instance, let `D_max` be the maximum total degree observed in the
**wide** run. The narrow run **must complete and match** iff `D_max ≤ 2^(w−1) − 1` and **must
report overflow** otherwise; an instance that overflows when it should have completed is a false
positive and fails, one that completes when it should have overflowed is a silent wrap and fails.
CI prints the completed/overflowed counts per width, and **a width at which zero instances
complete is a failed sweep, not a passed one.** The generator fleet already contains
capacity-boundary monomials (total degree exactly `D`, exactly `D+1`, exponents exactly at the
field max); at each width both must be present and must land on opposite sides.

#### What is still open, and what decides it

Whether terms are `(MonomialId, C)` into the ring-owned arena or `(PackedMon, C)` inline is
**not settled**, and it is a term-type question, not an architecture question — ADR-020 fixed
the ownership rule precisely so that the experiment decides a representation. The trade:
interning buys one copy per distinct monomial rather than per occurrence, O(1) equality by id,
the multiplicative hash `h(u) + h(v) = h(uv)` that makes symbolic preprocessing's "have I seen
this monomial?" lookup cheap, and a divisor index over identities rather than copies. Against
that, comparing by id requires a random arena load whose cache miss may dominate the `u64`
compare it enables. §10.2 specifies the experiment, and specifies it against a *synthetic*
harness so that it does not gate on the artifact it gates.

### 5.3 `AlgebraicReal`

This is resolvent's headline type and the bridge to computational geometry. Its shape is
decided by one tension: **comparison must refine, refinement mutates, and `Ord::cmp` takes
`&self`.**

The prior art pays for the wrong resolution in the open: `RealRoot::refine` takes `&mut self`
(`roots.rs:450`) while every `Geometry` predicate takes `&self`, so four curve families each
independently define `type SharedRoot = Rc<RefCell<RealRoot>>` together with a
`Rc::ptr_eq` self-deadlock guard (`conics.rs:32-46` and three siblings). Four copies of the
same workaround, including four copies of a guard that is load-bearing and easy to forget.

#### The decision (ADR-013)

```rust
#[derive(Clone)]
pub struct AlgebraicReal(Arc<Inner>);

struct Inner {
    poly:  SqfrPoly,        // UPoly<Integer>, squarefree, primitive, lc > 0. IMMUTABLE.
    state: Mutex<Bounds>,   // (lo, hi) rationals. MONOTONE: only ever shrinks.
    hint:  AtomicU64,       // f64-pair enclosure cache; a torn read is still a valid enclosure.
    ceiling: u32,           // §6.3: the bound-derived refinement ceiling, fixed at construction.
}
```

1. **`Arc<Inner>`, `&self` methods, `Send + Sync`.** Cloning shares refinement progress — which
   is what makes sorting `n` algebraic numbers affordable, and what the consumer currently
   hand-builds four times.
2. **Refinement is monotone.** The interval only shrinks and always contains the root, so *any*
   observation is a valid enclosure, including one interleaved with a concurrent refinement.
   There is no unsound intermediate state. This is the property that makes `&self` safe, and it
   is lifted from a protocol the same codebase already built for a different problem
   (`lazy-exact/src/real.rs`).
3. **No two locks are ever held simultaneously.** `cmp` fast-paths `Arc::ptr_eq`, then reads a
   snapshot of each operand's bounds, releases, refines each independently, and re-reads.
   Monotonicity makes a stale snapshot merely less precise, so the loop runs one more round.
   **Pointer-ordered locking is forbidden** — a lock order derived from an address is
   nondeterministic. This is subtle enough that it must be written where the code is: a future
   edit that "simplifies" it into two simultaneous locks reintroduces a deadlock class.
4. **`Eq`, `Ord` are implemented and total.** Equality is decided **algebraically**:
   `g = gcd(a.poly, b.poly)`; if `deg g = 0` they cannot be equal and refinement is guaranteed
   to separate them; if `deg g > 0`, a **sign change of `g` across the overlap** certifies
   equality. **No `Equal` verdict is ever produced by exhausting the separation bound.** That
   distinction is not pedantic: if `Equal` could be concluded from "refined past the bound
   without separating", then a systematically over-large bound — an off-by-one in the
   Mignotte–Davenport exponent, a `bit_length`-versus-`ceil(log2)` confusion, a missing leading
   coefficient — returns `Equal` for distinct numbers, *consistently*, which is transitive and
   therefore invisible to the transitivity property test that exists to catch exactly this class.
   Under the correct reading an over-large bound causes a loud internal-invariant failure
   instead.
5. **A failed equality certificate is never evidence of inequality.** If an overlap endpoint
   happens to be a root of `g`, the sign-change test sees a zero and **cannot conclude**; the
   response is refine-and-retry, not "return Less". Returning an ordering there is intransitive
   in exactly the same way as equality-by-tolerance. Property test, not review item. (The prior
   art gets this right at `roots.rs:578-592`.)
6. **No `Hash`.** Two `AlgebraicReal`s can be *equal* with different defining polynomials —
   `x²−2` and `x⁴−4` both have root √2 — so a `Hash` over the polynomial breaks the `Eq`/`Hash`
   contract and silently puts two entries in a `HashMap` for one number. No unit test catches
   it; it surfaces as nondeterministic consumer behaviour. `Hash` exists only on
   `CanonicalAlgebraicReal`, minted by an explicit `canonicalize()` that costs a factorization
   over ℚ and says so. A "cheap" `Hash` is not offered at any price.
7. **No general arithmetic.** `α + β` has degree ≤ `deg α · deg β`; without reducing to the
   minimal polynomial at each step, degree 4 + 4 → 16 → 256 → 65536 after three operations.
   Reducing at each step costs a factorization per operation. The decisive empirical
   observation: **the consumer never does algebraic-number arithmetic** — `RealRoot` has no
   `add`, `mul` or `div`, and four curve families ship without them, carrying points as
   *(isolated root ξ, a representation over ξ)* and signing them with a radical ladder. That is
   why its predicates stay in degree 4. The documented fast path is the sign ladder;
   `tower::materialize` is the general fallback, opt-in, in its own module, with the cost in the
   doc comment.
8. **Multiplicity is not part of identity.**

   ```rust
   pub struct IsolatedRoot { pub value: AlgebraicReal, pub multiplicity: u32 }
   pub fn isolate_roots(p: &SqfrPoly, window: Option<(&Rational, &Rational)>)
       -> Result<Certified<Vec<IsolatedRoot>>>;
   ```

   √2 has no multiplicity; multiplicity is a property of the polynomial the root was isolated
   from. Two values that are equal but were isolated from different polynomials with different
   multiplicities compare **`Equal`**, and with multiplicity off the number that is impossible
   to get wrong. The **named struct** rather than a bare tuple is the correction: it preserves
   the safety property in full while keeping the consumer's call-site shape
   (`root.multiplicity` — the prior art has exactly this method at `roots.rs:438`) and keeping
   the value storable as one thing.

#### `SqfrPoly` — fail-closed construction, made structural

`AlgebraicReal::new` must reject a non-squarefree defining polynomial: with a double root the
sign never changes across the interval, bisection cannot decide which half to keep, and every
downstream guarantee collapses. Rather than a `Result` the caller always pre-checks — which is
a signature smell — squarefree-ness is a **type**:

```rust
pub struct SqfrPoly(/* UPoly<Integer>, squarefree, primitive, lc > 0 */);
impl SqfrPoly {
    pub fn new(p: &UPoly<Integer>) -> Result<SqfrPoly>;   // Err(Unsupported::NotSquarefree)
}
pub fn square_free(p: &UPoly<Integer>) -> Certified<Vec<(SqfrPoly, u32)>>;  // Yun
```

Note the correction of divergence #4: `SqfrPoly` is over **ℤ**, not ℚ. A `UPoly<Rational>`
handed in at the boundary is converted by `clear_denominators()` on ingress; ℚ is a transport
type, not a working type.

#### `SqrtExt` is first-class and is never subsumed

Stated as a decision rather than left implicit, because nothing else forbids subsuming it and
the regression it would cause is silent. `circle_segments.rs` is 931 lines that use `SqrtExt`
exclusively and never import `RealRoot` or `QPoly`, and `SqrtExt::cmp_cross` has 31 call sites.
Routing degree-2 radicals through defining-polynomial + isolating-interval machinery would
replace an exact sign-by-squaring with a gcd, an isolation and a refinement loop, on the
cheapest and most common case in the entire consumer. It is also the return type of the
`cmp_y_right_of` witness fast path: evaluating a branch at a *rational* abscissa yields a
degree-2 radical, not an algebraic number, and that path must not allocate a defining
polynomial. Conversion `SqrtExt<Rational> → AlgebraicReal` exists and is explicit; the reverse
does not. §9.5 decides its generic parameter.

### 5.4 Certificates and certainty

Resolvent's differentiator is emitting checkable evidence. The design question is not *whether*
but *what it costs when unwanted* and *how a consumer verifies one*.

```rust
pub struct Certified<T> { pub value: T, pub certainty: Certainty }

pub enum Certainty { Proved(ProofKind), Probable(ProbableReason) }

/// The union of the three variant sets the earlier documents each declared.
pub enum ProofKind {
    Identity,                                           // a·b/b == a and friends
    DivisibilityAndDegree,                              // the gcd certificate
    BoundDriven { bound_bits: u64, primes_used: u32 },  // Landau–Mignotte / Hadamard
    CofactorRepresentation,                             // Gröbner: f = Σ hᵢgᵢ
    ProductAndModularIrreducibility { primes: SmallVec<[u32; 4]> },
    RootCount,                                          // the sign-variation witness
    Enclosure,                                          // Bernstein / de Casteljau
    ExhaustiveSmallCase,
}

pub struct Certificate<C: Claim> {
    claim:     C,            // private
    evidence:  C::Evidence,  // private
    certainty: Certainty,    // private
}
impl<C: Claim> Certificate<C> {
    pub fn claim(&self)     -> &C;
    pub fn evidence(&self)  -> &C::Evidence;
    pub fn certainty(&self) -> Certainty;
    pub fn certifies(&self, claim: &C) -> bool;                 // structural tether
    pub fn verify(&self, budget: Budget) -> Result<(), Error>;  // re-checks via public ops
}
```

**No public constructor exists on any certificate type.** Mints are `pub(crate)`, so a
certificate exists iff resolvent proved the claim. But **the accessors expose the mathematical
content**, which is what lets a consumer with its own trusted computing base re-verify with its
own arithmetic instead of trusting `verify()`. Unforgeable means no public mint; checkable means
public read. Both, not either.

**The tether.** Every certificate carries the claim it attests and `certifies` is structural
equality against it, so a transplanted certificate fails the comparison instead of riding along.

**Cost tiering, not a boolean flag**, because the tiers differ by orders of magnitude: *free*
evidence is part of the return type; *cheap* verification runs by default with a separately
named `*_unchecked` escape returning `Probable`; *expensive* evidence gets a separate entry
point, which is why `groebner` and `groebner_certified` are two functions and not one with a
`certified: bool` parameter. Written as a rule: **no certificate may add more than a documented
constant factor to the answer path; where it would, it lives behind a separate entry point.**

Two corrections carried from the critiques:

- **An isolating interval certifies nothing.** The claim "`f` has exactly one root in `[a,b]`"
  is established by a Descartes/VCA sign-variation count or a Sturm chain; the interval is the
  *conclusion*. A consumer handed intervals and nothing else must redo the isolation.
  `isolate_roots` therefore retains the sign-variation witness per interval, `ProofKind::RootCount`
  names it, and the item moves from free to cheap with the constant factor documented.
- **`Probable` is legal but must be visible and must not be the default.** Every competing
  system defaults to uncertified Gröbner over ℚ and says so; a certified resolvent loses those
  benchmarks *by construction*, and the harness must compare like with like rather than hide
  the difference. Defaulting to `Probable` would violate fail-closed; hiding the comparison
  would be dishonest.

### 5.5 Bounds, enclosures, and the one float

`AlgebraicReal::bounds()` returns exact `(Rational, Rational)`. `enclosure_f64()` returns a
plain outward-correct `(f64, f64)` pair — `lo` rounded down, `hi` rounded up, the true value
guaranteed to lie in the closed interval, never `NaN`, infinities permitted and meaning "no
finite bound on that side". **There is no `Interval` type, no `interval` module, and no
`IntervalArithmetic` trait in any published crate** (grep-gated), and **no resolvent API consumes
an `(f64, f64)` to decide anything** — the floats are an output for consumers and diagnostics
only, which is what keeps "no floating point in a decision path" true. The one place a rational
interval is a public concept is `sign_over(p, lo, hi)`, and there `lo`/`hi` are two `Rational`s,
not an interval type.

The internal dyadic-approximation filter inside sign-variation counting is permitted and is not
an interval type: it is a private filter whose declining case falls through to exact arithmetic,
and no verdict depends on which branch ran.

### 5.6 The compile-time budget

Monomorphization count is `|generic algorithms| × |instantiations|`. Controls: resolvent's own
instantiation set is closed and `number-fields` is feature-gated; kernels are Tier M, so the
expensive code compiles once, not once per `C`; large cold generic functions use the
inner-function trick, a thin generic wrapper that converts to a concrete representation and
calls a non-generic body; and the list of generic *algorithm texts* is closed at design time —
Horner, Bernstein/de Casteljau, Bareiss, dense row echelon, matrix multiplication, sign
ladders — and is not grown casually.

**The gate is absolute, not relative.** A ">20% regression in total front-end time" gate is
unusable early: in Wave 0 the workspace has no algebra, so adding `resolvent-int` is a >20%
regression and so is adding `resolvent-modular`. Every early lane trips it against a near-empty
baseline, so it gets disabled within a fortnight — and a compile-time budget disabled once never
returns, which is exactly how monomorphization explosions arrive unannounced. Instead:
per-crate absolute ceilings set after M1 and revised only at milestone boundaries (for example
`resolvent-poly` front-end ≤ 20 s, workspace clean debug ≤ 90 s on the pinned machine),
ratcheting **down only**, recorded alongside the tuning thresholds; plus `cargo llvm-lines`
top-20 monomorphization counts as the leading indicator, because that moves before wall-clock
does.

### 5.7 The bignum wall, and the operation that actually hurts

`resolvent-int` wraps `dashu` (MIT OR Apache-2.0) in `Integer`, `Natural` and `Rational`
newtypes. `dashu` types appear in no public signature and are not re-exported, so `dashu`'s
semver is not part of resolvent's semver and swapping the backend is a change inside one crate.
The **conversion surface is over primitives and slices** — `From<{i8..i128, u8..u128, isize,
usize}>`, `TryFrom<&Integer>`, `FromStr`, `from_le_limbs64`/`to_le_limbs64`,
`from_signed_bytes_be`/`to_signed_bytes_be` — never over a third-party bignum type, so an
adapter can be written without naming resolvent's dependency.

`malachite` is the fastest pure-Rust bignum and it is **LGPL-3.0-only**, because it is *derived
from GMP, FLINT and MPFR source*. It has no dual arm and no permissive subset. `num-bigint` is
permissively licensed and is rejected on **capability**, not licence: its multiplication ladder
is schoolbook → half-Karatsuba → Karatsuba → Toom-3 and stops, with no FFT/NTT path at all.

**The correction that matters.** ADR-002 rests its case on "megabit integers appear exactly when
someone computes over ℤ or ℚ directly instead of mod several primes and reconstructing."
**That is false, and the plan's own numbers refute it.** Modular methods do not eliminate large
integers; they **concentrate** them in exactly two places:

- **The CRT modulus `M = Π pᵢ`.** Cyclic-10 needs >2000 primes of 29 bits (≈58 000 bits).
  Hexapod needs 1102 primes for a computation whose single modular run takes 0.00 s — ≈70 kbit
  at 63-bit primes — and Hexapod is deliberately in the corpus from the first modular milestone.
- **Rational reconstruction**, which is `gcd_ext` on integers of size `M` — precisely the one
  identified structural pure-Rust deficit, since `dashu` has Lehmer (quadratic worst case) and
  GMP has a subquadratic half-GCD. At ~70 kbit (≈1100 limbs) that is ~10⁶ word operations per
  reconstruction against ~10⁴–10⁵, once per reconstructed coefficient, on the **default
  certified path**.

The measurement ladder as planned stops at 16k bits — an order of magnitude below the regime
that matters — so it could not detect the problem it exists to detect. **The ladder is extended
to 64k and 256k bits and gains a `rational_reconstruct` microbenchmark at Hexapod's modulus
size, and all of it runs before `resolvent-int` is written.** Three mitigations go on the record
now rather than being discovered later: incremental (Garner) CRT keeps the accumulation
small-step; early-termination rational reconstruction with a doubling modulus avoids the
full-size `gcd_ext` in the common case; and a **half-GCD implemented inside `resolvent-int` is a
planned M1 contingency with a numeric trigger** (§10.1), not a hypothetical. It is a
self-contained, `rug`-certifiable lane, which is a good lane shape, and the wall is what makes
it possible without touching anything above L0.

---

## 6. The error model and fail-closed semantics

### 6.1 The centerpiece

> **Fail at construction, not at query.**
> Every invariant is checked when a value is built — squarefree-ness, isolation, nonzero at
> interval endpoints, ring compatibility, degree and variable bounds, exponent range.
> Construction returns `Result`. Every method on a well-formed value that is *mathematically*
> total is total *in the type system* too.

Three facts about this domain make a conventional Rust error model wrong. **The characteristic
failure is a silent hang, not a wrong answer**: `sign_of(h)` where `h(α) = 0` never terminates
unless zero-ness is settled algebraically first, refinement stalls forever on a non-squarefree
defining polynomial, and comparison of two equal algebraic numbers loops forever if equality is
not decided by gcd. A hang is worse than a wrong answer in a library, because it is undebuggable
in production and invisible to a test suite that grades on assertions. **The consumer's exact
paths are infallible by construction** — its exact families declare `type Error = Infallible`
and fail closed with a structured value where a case genuinely is not handled — so a `cmp` that
returned `Result` would force it to invent an error path it does not have. And **tolerance
parameters are permanently out** (§1.3).

### 6.2 What is a `Result`, and what panics

`Result` covers: all constructors and parsers; every operation whose *input domain* is narrower
than its input type (`div_rem` by zero, `inv` of a non-unit, a zero-dimensional routine given a
positive-dimensional ideal); every operation that can hit a documented capability limit
(exponent overflow, arena exhaustion, variable count over the arena width, degree over a packed
bound); and every operation whose termination argument is a *budget* rather than a theorem.

**Nothing panics, in any published crate, outside `#[cfg(test)]`.** `debug_assert!` is
encouraged and compiles out. A violated **internal** invariant returns
`Error::Internal { invariant: &'static str }`; it does not panic. Two reasons: an embedding
kernel may sit behind an `extern "C"` boundary where unwinding is UB and callers under
`panic = "abort"` cannot recover; and more fundamentally, to a user of an exact kernel a panic
and a hang are the same event — an operation that produced no answer. Allocation failure keeps
Rust's default (abort); resolvent does not pretend to handle OOM and says so in the crate docs.

The cost of returning a bug rather than panicking is that callers may silently discard it.
Mitigated by `#[must_use]` through `Result`, by a diagnostics hook that counts internal errors,
and by CI failing any test run with a nonzero internal-error count.

### 6.3 Budgets, and the two regimes

Every loop without an a-priori termination proof takes a `Budget`, counted in **steps** — never
wall-clock time, because wall-clock is nondeterministic. Which regime applies is **declared per
entry point**:

- **A proven bound exists** — Mignotte–Davenport root separation for comparison,
  Landau–Mignotte for factor coefficient size, Hadamard for resultant and determinant
  coefficient size. The default budget is *derived from the bound*, so exhaustion is proven
  impossible and **the budget is a bug detector, not a control-flow exit**. Exceeding it is a
  `debug_assert!` in debug and a diagnostics counter in release; the loop continues, because it
  is still mathematically correct. **This is what makes the query surface total.**
- **No proven bound exists** — van Hoeij lattice iteration, stabilization-driven modular
  reconstruction. The budget *is* the exit; exhaustion returns
  `Err(Error::BudgetExhausted { consumed, partial })` carrying enough state to resume.

**`AlgebraicReal` is always in the first regime, and that is a correction.** ADR-013's value
proposition is that clones share refinement progress, so the step count of a given `cmp` depends
on what has already been compared — and, under `parallel`, on what other threads did. If a
budget were charged against *work actually done*, then budget exhaustion would be history- and
schedule-dependent: a call that declines when run first succeeds after a warm-up, and at eight
threads a comparison might take 3 steps where at one it takes 40. Property outcomes would then
depend on execution order (which shrinking reorders), and `Ok` versus `Err(BudgetExhausted)`
could differ by thread count — which is precisely what the determinism gate asserts cannot
happen. So:

> **Invariant.** The refinement cache may change *how much work* a call does. It may never
> change *what the call returns*, including whether it declines. Property-tested as idempotence
> under refinement and as sort stability under shuffling.

Enforced by deriving every `AlgebraicReal` budget from the separation bound at construction
time (the `ceiling` field in §5.3), never from elapsed steps.

**`Ord` is total, and that needs one more piece of honesty.** The separation-bound argument
makes comparison terminate in a *computable* number of steps, which is what buys the `Ord` impl.
It does not make that number *attainable*: for the degree-~200, ~500-bit resultants the
elimination milestone predicts, the Davenport–Mahler bound is tens of thousands of bits of
refinement in the worst case, and `Ord::cmp` has no `Result`, no budget, and no way out. That
is the hang this whole section calls the deadliest failure mode, sitting inside the library's
most-called function, on the path `sort()`, `BTreeMap`, `binary_search` and `max()` all take.
The resolution has three parts and it must land **before M3**, not be filed under "unsettled":

1. **Measure the step distribution** on the elimination corpus (the consumer-workload lane can
   do it) and **publish it**. If the 99.9th percentile is small, `Ord` is fine and the claim is
   made with evidence instead of a bound.
2. **Keep `Ord`, with a diagnostic ceiling far below the theoretical bound.** Reaching it
   increments a counter and is a CI failure in the test profile; it does not change a verdict.
3. **Ship `try_cmp(&self, &Self, Budget) -> Result<Ordering, Decline>` alongside**, documented
   and benchmarked, and direct latency-path consumers to it in those words.

### 6.4 `Unsupported` is a structured value, never a string

```rust
#[non_exhaustive]
pub enum Unsupported {
    CoefficientRing      { got: RingTag, required: &'static [RingTag] },
    Characteristic       { got: u64, required: CharacteristicClass },
    VariableCount        { got: u32, max: u32 },
    TotalDegree          { got: u64, max: u64 },
    MonomialArenaFull    { capacity: u32 },
    MonomialOrder        { got: OrderTag, required: &'static [OrderTag] },
    NotSquarefree,
    NotZeroDimensional   { dimension: u32 },
    PositiveDimensionalRealSolve,
    BadPrimeExhausted    { tried: u32 },
    TranscendentalSymbol { name: SymbolId },
    NoDerivativeRule     { func: FuncId },
    NoLeafRule           { symbol: SymbolId },
}
```

A consumer's fail-closed path matches on variants; a string forces string-matching and breaks
silently on rewording. The error enum as a whole is small, closed, `Clone + PartialEq`,
`String`-free, with the offending data on the variant, and with **declines**
(`BudgetExhausted`, `Unsupported`) distinguishable from **faults** (`Domain`, `Overflow`,
`Internal`) via `is_decline()`. resolvent's error type never tries to be a consumer's error
type; the consumer maps it upward in about twenty lines.

### 6.5 One verdict vocabulary, and the rule that keeps it one

> A function returns a **bare `Sign`** if and only if it is total and exact.
> A function that can be indeterminate returns **`Verdict<Sign>`** and never `Sign`.

```rust
pub enum Sign { Negative, Zero, Positive }
pub enum Verdict<T> { Certain(T), Unknown }
```

`Verdict` is produced **only** by enclosure and filter APIs — Bernstein `sign_over`, the f64
enclosure comparison — and **never** by an algebraic-decision API. `Unknown` means "this cheap
rung declined to decide", and the caller's response is to climb to the exact rung, never to
guess. The alternative — one verdict type everywhere — forces every exact call site to handle an
`Unknown` that cannot occur, which trains callers to write `unwrap`-shaped code at exactly the
boundary where correctness matters most.

### 6.6 Fail-closed is trivially satisfiable, so soundness alone is not a gate

This is an error-model consequence, not only a testing one, and it belongs here because it
shapes signatures. **Every soundness certificate in the plan is satisfied by a maximally
conservative implementation**: `sign_over` that always returns `Unknown`; a `Certainty` that is
always `Probable`; a routine that always declines; a `divmask` that always says "maybe"; a
separation bound of zero; isolating intervals that are the whole Cauchy box; a `CrossingKind`
that is always `Unknown`. An agent optimizing for a green suite converges on an implementation
that is sound and worthless.

**Rule: any API with a "don't know" or "probably" outcome ships with a tracked rate, and the
rate is a CI-visible number with a committed ceiling.** The ceilings are established by
measurement in the first PR that lands the API they guard, committed to
`sharpness-ceilings.toml` rounded outward by a stated margin; a PR may lower a ceiling freely
and may not raise one without a recorded justification counted in CI output; and a rate with no
committed ceiling fails the PR gate — `TBD` is not a ceiling. Per-operation floors stated as
absolutes (gcd, resultant, factorization-product and isolation must be 100% `Proved`) are
committed as `1.0` on day one and are never ratcheted.

Correspondingly, **a decline is classified before it is scored.** A decline is a *failure* if
the instance is in the must-complete sub-corpus, or if the operation's budget was derived from a
proven bound — in which case exhaustion is impossible for a correct implementation and the
decline is a bug. Otherwise it is a survived instance counted in the decline rate. The blanket
rule "any decline anywhere is a failure" is worse than useless: the cheapest way to satisfy it
is to raise the default budget until nothing declines, which converts declines into long runs,
which is the sanctioned form of the hang this section exists to prevent.

---

## 7. Determinism and reproducibility

### 7.1 The requirement

> **Same input ⇒ same output, bit-for-bit, on any machine, at any thread count, in any build
> profile. And the *path* taken is recorded and replayable.**

This is a verification constraint before it is a consumer constraint: **a non-deterministic
library cannot have a regression corpus.** Every golden file, every minimized counterexample,
every change-point baseline assumes the same input produces the same bytes. It is also a
licensing constraint, because the "re-derive every threshold by measurement" rule is only
meaningful if changing a threshold provably cannot change an *answer*.

### 7.2 The four sources of nondeterminism, each closed

**(a) Ambient randomness — closed by banning it.** `rand` is not a dependency of any published
crate. `SystemTime`, `Instant` in any decision path, `std::process::id`, address-derived values,
and `HashMap`'s default `RandomState` are denied by lint. There is exactly one RNG type in the
workspace.

**(b) Randomized algorithms — closed by counter-based substreams.** The RNG is **counter-based**
(`output = F(key, counter)`, Philox/ChaCha-shaped), not sequential. A `Session` carries a
`Seed`; a worker at logical index `k` uses `rng.substream(k)`, so the value drawn at a given
logical position is a function of that position and not of scheduling, thread count, or chunk
size. A sequential RNG cannot give this without a lock that serializes the computation. The
default seed is a **fixed checked-in constant**, not entropy, so the default path is
reproducible without the caller doing anything.

**(c) Prime and evaluation-point selection — closed by index-addressing.** Primes are never
random. `prime(i)` is a pure function of `i` over a checked-in generator. A modular run consumes
primes in index order; a prime rejected as bad is recorded **by index with its rejection
reason**. Evaluation points come from the counter RNG at index-derived positions and are
recorded identically.

One consequence must be recorded because it is load-bearing and easy to miss: **the primality of
the prime registry is the modular architecture's root of trust and it is the one assumption with
no downstream detector.** A composite in the table makes `Fp` silently stop being field
arithmetic and can make the GF(p) gcd return a wrong degree — while CRT combination still
certifies (`result ≡ rᵢ (mod pᵢ)` holds regardless of primality) and rational reconstruction
still certifies (its conditions are statements about `M`, not about `M`'s factorization). The
registry is therefore cross-checked against an **independent** implementation — a segmented
sieve — over a committed window, with the accepted set's hash committed as a golden file.

**(d) Hash iteration order — closed by never letting it reach an output.** Interning uses a
fixed-seed hasher. Any table iterated to produce output is sorted by a declared total order
first. No `HashMap` iteration order is observable in any return value or in any decision.

**One consequence for the harness, because the two uses of randomness are different uses.**
Several of the strongest certificates are Schwartz–Zippel arguments — the `UPoly` multiplication
check "evaluate both sides at random points in a large GF(p)", the subresultant specialization
property at random good ring maps, L4 rewrite soundness. Their failure probability is a statement
about a *draw*; when the draw is the fixed default seed, there is no probability left and what
remains is a golden test at one point, which certifies forever an implementation whose error
happens to vanish at `prime(0)` and `prime(1)` — not a contrived class in modular arithmetic.
**Inside the library, at the default seed: deterministic, as specified above, unchanged. In the
harness, a certificate whose soundness argument is probabilistic is wired to the fleet seed
schedule and is graded across it, never at the default seed alone**, and the number of distinct
seeds it was checked at is reported alongside the score — otherwise a silent reduction from 64
seeds to 1 is invisible and improves every number.

### 7.3 The interning tension, stated rather than papered over

Three requirements are pairwise fine and jointly contradictory: ids are assigned in
first-encounter order under a deterministic traversal so that they are reproducible; shared
mutable accumulators updated from `for_each` are banned; and the monomial arena is a shared
mutable accumulator reached by every operation that creates a monomial. **An interner *is* a
shared mutable accumulator**, and symbolic preprocessing — the natural second parallel target
after row reduction — is nothing but interning. The failure is subtle: an agent writes
`terms.par_iter().map(|t| ring.intern(t)).collect()`, which *looks* like the permitted
ordered-combination shape because the collection is ordered. The ids are not, and the thread
matrix catches it only on instances whose tie-breaks actually consult id order, which is
data-dependent — so it passes for months, then fails once, and the minimizer cannot shrink a
schedule bug.

Two resolutions, and the design takes both:

1. **Interning is serialized, or two-phase.** Candidates are collected in parallel into
   per-chunk ordered vectors; a single-threaded merge assigns ids in a canonical order (chunk
   index, then position within chunk). The cost is stated rather than hidden: **it caps the
   parallel speedup of matrix construction, and the plan stops implying that phase is
   parallelizable as written.** Content-derived ids were the attractive alternative — a hash of
   the packed key with a fixed collision-resolution order — but a compact `u32` arena index
   cannot be content-derived under open addressing without a sorted merge, and the sorted merge
   *is* the two-phase scheme. If the term-type experiment (§10.2) picks inline `(PackedMon, C)`,
   the question evaporates entirely, because there are no ids.
2. **Invariant: no tie-break anywhere may consult `MonomialId` ordering.** Tie-breaks are taken
   on `key`, which is content-derived and totally ordered by construction. This is what makes
   the id-assignment question a performance question rather than a correctness one, and it is
   the more important of the two.

### 7.4 Parallelism

Determinism under parallelism is a **combining-order** property, not a locking property.

- Results are combined in **index order**, never completion order. The permitted shape is
  `par_iter().map(..).collect::<Vec<_>>()` and reductions over the resulting ordered `Vec`.
  Shared mutable accumulators updated from `for_each` are banned — see §7.3 for the one place
  that bites.
- Work-splitting granularity (chunk size, batch size, thread count) may change timing and must
  not change values. CI asserts it: the corpus runs at `RAYON_NUM_THREADS ∈ {1, 2, 8}` and the
  serialized outputs must be byte-identical.
- `rayon` is behind a default-off `parallel` feature, appears only in `-algebra` and `-real`,
  and appears in no public signature.

**Batched multi-modular needs a split driver, not just a batch.** Batching `N` primes as a tuple
ring shares all non-arithmetic work and is worth up to ~2.7× amortized — but it works only while
all `N` primes behave identically, and two things certainly break that. A pivot that is zero in
one lane: `inv_batch` returns the `LaneMask` (§5.1) and the driver splits the batch, finishes
the good lanes, and re-runs the offending prime alone. Lead-monomial divergence: the Gröbner
bad-prime rule is a majority vote over lead-monomial sets, but under batching all `N` primes
share one matrix construction and one pair-selection path — *that sharing is the 2.7×* — so a
diverging prime corrupts shared control flow instead of producing a minority to discard. The
driver therefore splits on lead-monomial divergence too, and records the offending prime index
in the `Trace`. The "componentwise equality with `N` scalar runs" oracle is complete for
*arithmetic* and silent on both of these, which are control-flow failures. **Lane G6 is
"batching and splitting", not "batching".**

### 7.5 Traces, tuning, and what is *not* in canonical bytes

```rust
pub struct Trace { seed: Seed, tuning: Tuning, events: Vec<TraceEvent> }
pub enum TraceEvent {
    PrimeAccepted { index: u32 },
    PrimeRejected { index: u32, reason: BadPrime },
    BatchSplit    { at: u32, lanes: LaneMask },
    EvalPoint     { index: u32, value: i64 },
    Stabilized    { rounds: u32 },
    TracerDecision{ matrix: u32, kept: u32, dropped: u32 },
    WidenRestart  { from: u8, to: u8 },
    BudgetTick    { site: &'static str, consumed: u64 },
}
```

`op_with_trace(input) -> (Certified<T>, Trace)` pairs with
`op_replay(input, &Trace) -> Certified<T>`, and CI asserts the replay is byte-identical. A bug
report is `(input, trace)` and nothing else.

Every crossover threshold lives in one `Tuning` struct with documented defaults — the
Karatsuba/Toom/NTT handoffs consumed from the bignum layer, the fast-Taylor-shift crossover
(around degree 512), the Zassenhaus→van Hoeij `r` threshold (~10), the F4 batch size, the
modular batch width `N`, the delayed-reduction cutoff, the packed/unpacked variable-count
switch, the Barrett/Montgomery selection. **Same input + same `Tuning` ⇒ same output; different
`Tuning` ⇒ same values, different timing.** CI asserts value-equality across a small `Tuning`
matrix, which doubles as a free implementation-agreement oracle forcing the naive path and the
fast path to agree on every corpus instance.

**Canonical serialization, fixed before any oracle is written**, because a SHA-256 certificate
only works if normalization is byte-identical:

- **Polynomials** — content removed; leading coefficient positive; terms **descending** in the
  ring's declared order; coefficients as decimal integers with an explicit `-` and no `+`;
  exponent vectors as full-length comma-separated non-negative integers.
- **Gröbner bases** — each element canonicalized as above, then the *list* sorted by leading
  monomial descending.
- **Algebraic numbers** — the **minimal** polynomial plus a 0-based ascending root index, which
  requires factorization, which is exactly why `Hash` is not implemented on the un-canonicalized
  type.
- The certificate is SHA-256 of that byte string, and the serializer lives in `resolvent-base`
  so every crate **and every oracle adapter** shares one implementation.

> **Explicitly excluded from canonical bytes: certificates, `Evidence`/`ProbableReason`,
> `Telemetry`, and `TraceEvent::BudgetTick`. Only the mathematical value is serialized.**

That exclusion is not tidiness. `Evidence` carries `primes_used`; the batch width `N` is a
tuning knob that changes `primes_used`; and the tuning-matrix byte-identity gate would therefore
fail on its first run if evidence were in the bytes. `Telemetry { tier_reached, bisections,
precision_bits, primes_used }` is plain `Copy` data with no proof type attached, which is what
lets a consumer cache warm-start hints without laundering them into evidence.

### 7.6 The one `unsafe` leaf, and the honest performance gate

`#![forbid(unsafe_code)]` and a "Competitive ≈ 2× SOTA" F4 gate cannot both hold. Linear algebra
is 73–91% of an F4 run and msolve reports AVX2 halving it, so forgoing AVX2 forgoes roughly a
1.6–1.8× overall factor — the policy and the published target are within noise of each other.
Stable-Rust-only also forecloses `portable_simd`, so the only route to AVX2 is `core::arch`
intrinsics, which are `unsafe`.

**Decision.** One `unsafe`-permitted leaf: `resolvent-modular::simd`, behind a default-off
`simd` feature, with `#![allow(unsafe_code)]` scoped to that module, a `SAFETY:` comment on
every block, runtime feature detection, and a **CI-asserted bit-identical scalar fallback**.
These are exact integer operations, so the SIMD path is a pure speed change and cannot alter a
value — which is what makes the exception auditable and keeps the determinism story intact. And
the published gate is stated in **both forms**, because publishing a target the policy forbids
reaching is not honest: **Competitive ≈ 2× SOTA with `simd` enabled; ≈ 3–4× SOTA without, with
AVX2 named as the reason.**

One quieter consequence goes on the record: auto-vectorization of a sparse GF(p) `axpy` with
Barrett reduction — a widening multiply plus a conditional subtract — is inconsistent across
LLVM versions, so a performance series will level-shift on a compiler upgrade with no code
change and trip the change-point detector. **A compiler bump is a re-baseline event**, recorded
the same way a fleet-version bump is, and the compiler version is part of every benchmark
record.

### 7.7 The determinism gate must be affordable, or it is the first thing cut

Determinism has no algebraic certificate, so it is graded by running every corpus instance twice
in-process, twice cross-process, at 1/2/8 threads, across feature combinations, and comparing
canonical bytes. That is at minimum twelve full-corpus runs per commit — against a corpus that
is contractually append-only and is deliberately stocked with instances that exist *because*
they are slow — plus self-certification on every call, where the gcd certificate costs about
another gcd, Sturm's count is `>1×` at high degree, and the S-pair certificate is roughly
recomputing the basis. By month three the per-commit gate takes forty minutes, and the
determinism matrix — the gate everything else depends on — is the most expensive and least often
red, so it is the first thing sacrificed.

**Tier the corpus on day one, before it has entries.**

| Tier | Runs | Budget | Contents |
|---|---|---|---|
| `fast` | per commit, 1 and 8 threads, in-process | **90 s, enforced** | every instance by default; promoted out on a committed per-instance time cap |
| `full` | per PR | — | the complete determinism matrix |
| `slow` | nightly | — | the Mignotte / Swinnerton–Dyer / Hexapod class |

CI prints the tier census and **fails if `fast` exceeds its budget**, so promotion is a
deliberate, visible act rather than silent gate erosion. Self-certification becomes a profile
flag: on in `full` and `slow`, sampled at 10% in `fast`.

**Every corpus entry carries a provenance field**, for a related reason. The regression corpus is
append-only and gates at 100%, which is right for minimized counterexamples whose expected
outcome is "does not crash / self-certifies" and dangerous for hand-authored known-answer
instances: an expected answer that entered from a mis-triaged disagreement, or from an oracle
that was itself wrong, becomes a permanent gate that a *correct* future implementation fails, and
append-only means such entries can only accumulate. So
`provenance ∈ { constructive-generator, oracle-consensus(k systems), hand-computed(author,
method), minimized-counterexample }`; oracle-consensus entries name the systems and versions and
are **re-derivable**, with a nightly job re-asking the oracles and flagging drift; hand-computed
entries carry the derivation. One field, and it is the difference between institutional memory
and institutional debt.

---

## 8. Public API sketch

**Illustrative.** These signatures show the *shape* a reader should expect; `API.md` is normative
for the surface and is where a divergence is resolved. Types elided for readability are marked.

### 8.1 Numbers, and how a consumer gets values in and out

```rust
pub struct Integer(/* private */);
pub struct Rational(/* private */);

impl From<i8|i16|i32|i64|i128|u8|..|u128|isize|usize> for Integer { .. }
impl TryFrom<&Integer> for i64 / i128 / u64 / u128 { .. }
impl FromStr for Integer / Rational { .. }               // decimal; 0x/0b prefixes

impl Integer {
    pub fn from_le_limbs64(sign: Sign, limbs: &[u64]) -> Integer;
    pub fn to_le_limbs64(&self) -> (Sign, Vec<u64>);      // and a borrowing variant
    pub fn from_signed_bytes_be(b: &[u8]) -> Integer;
    pub fn to_signed_bytes_be(&self) -> Vec<u8>;
    pub fn bits(&self) -> u64;
}
impl Rational {
    pub fn new(num: Integer, den: Integer) -> Result<Rational>;   // den != 0
    pub fn try_from_f64(x: f64) -> Result<Rational>;              // EXACT dyadic; Err on NaN/±∞
    pub fn numer(&self) -> &Integer;
    pub fn denom(&self) -> &Integer;
    pub fn num_bits(&self) -> u64;
    pub fn den_bits(&self) -> u64;
    pub fn round_to_f64_grid(&self) -> Rational;                  // explicit, caller-driven
    pub fn demote_exact(&self) -> Result<i64>;
    pub fn enclosure(&self) -> (f64, f64);                        // outward-correct
    pub fn approx_lossy(&self) -> f64;                            // diagnostic only
}
```

There is **no** `Rational::from_f64` heuristic sibling, ever: silently turning `sin(30°)`'s f64
into `1/2` analyses a different system than the one the caller authored. Lift-then-operate is
the only expressible order.

### 8.2 Polynomials

```rust
pub struct UPoly<C: Ring> { /* ctx: C::Ctx, coeffs: Vec<C> */ }

impl<C: CommutativeRing> UPoly<C> {
    pub fn from_coeffs_low_to_high(ctx: C::Ctx, c: Vec<C>) -> UPoly<C>;
    pub fn degree(&self) -> Option<usize>;            // None for the zero polynomial
    pub fn lc(&self) -> Option<&C>;
    pub fn eval_horner(&self, at: &C) -> C;           // SAME RING. No hom parameter.
    pub fn map_coefficients<D: Ring, E>(&self, ctx: D::Ctx, f: impl Fn(&C) -> Result<D, E>)
        -> Result<UPoly<D>, E>;                       // the ONLY cross-ring path
    pub fn derivative(&self) -> UPoly<C>;
    pub fn add(&self, o: &Self) -> Self;
    pub fn sub(&self, o: &Self) -> Self;
    pub fn mul(&self, o: &Self) -> Self;
    pub fn pseudo_div_rem(&self, d: &Self) -> Result<(Self, Self, u32)>;
    pub fn compose_affine(&self, a: &C, b: &C) -> Self;
    pub fn reverse(&self) -> Self;                    // xⁿ·p(1/x) — PUBLIC. See note.
    pub fn taylor_shift_1(&self) -> Self;
}
impl<C: Field> UPoly<C> { pub fn div_rem(&self, d: &Self) -> Result<(Self, Self)>; }
impl UPoly<Integer> {
    pub fn content(&self) -> Integer;
    pub fn primitive_part(&self) -> UPoly<Integer>;
    pub fn canonical_associate(&self) -> UPoly<Integer>;   // content-free, lc > 0
    pub fn scale_pow2(&self, k: i32) -> Self;              // p(2^k·x), dyadic
}
impl UPoly<Rational> { pub fn clear_denominators(&self) -> (UPoly<Integer>, Integer); }
```

`reverse` is public deliberately: it is private in the prior art (`roots.rs:242`) and consumers
need it, because a Weierstrass-rationalized family moves the point at infinity to zero with
exactly this transform. `canonical_associate` is what makes the associate test an `==`; the
consumer currently hand-rolls it as all 2×2 minors over six coefficients.

```rust
pub struct Ring { /* arity, order, field width, monomial arena, coefficient tag */ }
pub struct MonomialId(u32);
pub struct MPoly<C: Ring> { /* terms sorted descending; owned Arc<Ring> */ }

impl Ring {
    pub fn new(vars: &[&str], order: Order) -> Result<Ring>;
    pub fn conversion_pair(vars: &[&str], a: Order, b: Order) -> Result<Ring>;  // FGLM
    pub fn var(&self, name: &str) -> Option<VarId>;
    pub fn order(&self) -> Order;                     // runtime data, not a type parameter
    pub fn arena_stats(&self) -> ArenaStats;
}
impl<C: Ring> MPoly<C> {
    pub fn total_degree(&self) -> u64;
    pub fn derivative(&self, v: VarId) -> Self;
    pub fn map_coefficients<D: Ring, E>(&self, ctx: D::Ctx, f: impl Fn(&C) -> Result<D, E>)
        -> Result<MPoly<D>, E>;
}
```

### 8.3 Layer 2

```rust
pub fn gcd(a: &UPoly<Integer>, b: &UPoly<Integer>) -> Certified<UPoly<Integer>>;
pub fn gcd_ext(a: &UPoly<Integer>, b: &UPoly<Integer>)
    -> Certified<(UPoly<Integer>, UPoly<Integer>, UPoly<Integer>)>;   // (g, u, v)
pub fn square_free(p: &UPoly<Integer>) -> Certified<Vec<(SqfrPoly, u32)>>;
pub fn resultant(f: &MPoly<Integer>, g: &MPoly<Integer>, elim: VarId)
    -> Result<Certified<UPoly<Integer>>>;
pub fn subresultant_chain(f: &..., g: &..., elim: VarId) -> Result<Certified<Chain>>;
pub fn factor(p: &UPoly<Integer>, budget: Budget) -> Result<Certified<Vec<(UPoly<Integer>, u32)>>>;

pub fn groebner(ideal: &[MPoly<Fp>], budget: Budget) -> Result<Certified<Vec<MPoly<Fp>>>>;
pub fn groebner_certified(ideal: &[MPoly<Rational>], budget: Budget)
    -> Result<(Certified<Vec<MPoly<Rational>>>, Cofactors)>;

pub mod linalg {
    pub fn row_echelon<C: Field>(rows: Vec<Vec<C>>) -> Result<Echelon<C>>;
    // Echelon exposes rank(), pivot_rows(), dependent_rows(), transform_rows() — the
    // transform is not a bonus: it is the same object as a Gröbner cofactor
    // representation, one layer down, and one consumer ships an empty stub for it today.
    pub fn bareiss_det<C: CommutativeRing>(m: &Matrix<C>) -> Result<C>;
    // No prime appears in the signature: modular is HOW you make it fast, not WHAT was asked.
}
```

`gcd_ext` is not a convenience. The gcd certificate as originally specified is **circular**:
`H|A`, `H|B` plus `deg H == deg gcd(A mod p, B mod p)` is passed by `fn gcd(_,_) -> 1`, because
the degree half is computed by the routine under test. The non-circular replacement is the
**Bézout witness** — `H|A`, `H|B`, and `(u,v)` with `u·A + v·B == H` — which is complete, costs
one multiply-add to check, and shares no control flow with the gcd routine. Retaining the
cofactors is free in the extended Euclid that already computes them, and it is the same data an
SMT consumer needs for external proof production. **General rule: a certificate may not invoke
the operation it certifies, nor any routine on that operation's call graph.**

### 8.4 Layer 3

```rust
#[derive(Clone)]
pub struct AlgebraicReal(/* Arc<Inner> */);

impl AlgebraicReal {
    // ---- construction is where fallibility lives ----
    pub fn from_rational(q: Rational) -> AlgebraicReal;
    pub fn new(poly: SqfrPoly, lo: Rational, hi: Rational) -> Result<AlgebraicReal>;
    //   Err if (lo,hi) does not isolate exactly one root, or poly(lo)==0, or poly(hi)==0.

    // ---- the query surface is total ----
    pub fn defining_poly(&self) -> &SqfrPoly;
    pub fn bounds(&self) -> (Rational, Rational);      // exact, monotone
    pub fn enclosure_f64(&self) -> (f64, f64);         // outward-correct. NOT an Interval.
    pub fn refine_to(&self, width: &Rational);         // &self, idempotent, monotone
    pub fn as_rational(&self) -> Option<Rational>;     // Some iff collapsed to a point
    pub fn sign_of(&self, h: &UPoly<Integer>) -> Sign; // zero-ness settled by gcd FIRST
    pub fn is_root_of(&self, h: &UPoly<Integer>) -> bool;
    pub fn cmp_rational(&self, q: &Rational) -> Ordering;
    pub fn canonicalize(&self) -> Result<CanonicalAlgebraicReal>;  // costs a factorization

    // ---- the budgeted sibling, for latency-bounded callers (§6.3) ----
    pub fn try_cmp(&self, o: &Self, b: Budget) -> Result<Ordering, Decline>;
}
impl PartialEq/Eq/PartialOrd/Ord for AlgebraicReal {}   // total; see §5.3, §6.3
// NO impl Hash. CanonicalAlgebraicReal has one.

pub struct IsolatedRoot { pub value: AlgebraicReal, pub multiplicity: u32 }
pub fn isolate_roots(p: &SqfrPoly, window: Option<(&Rational, &Rational)>, b: Budget)
    -> Result<Certified<Vec<IsolatedRoot>>>;

pub fn rational_between(a: &AlgebraicReal, uppers: &[AlgebraicReal], b: Budget)
    -> Result<Rational, Decline>;

pub struct SqrtExt<T> { /* a + b·√r */ }
impl<T: Ordered + Field> SqrtExt<T> {
    pub fn new(a: T, b: T, r: T) -> Result<Self>;      // Err on a negative radicand. NOT a panic.
    pub fn sign(&self) -> Sign;                        // by squaring, exact
    pub fn cmp_cross(&self, o: &SqrtExt<T>) -> Ordering;   // total across different r
}

/// Exact sign of Σ cᵢ(α)·√hᵢ(α) at arbitrary depth. Generalizes the prior art's depth-1
/// and depth-2 ladders; depth ≥ 3 exists nowhere today, and every consumer that needs it
/// currently writes ~150 lines of by-hand ladder per curve family.
pub fn sign_radical_tower(
    coeffs: &[UPoly<Integer>], radicands: &[UPoly<Integer>], at: &AlgebraicReal,
) -> Sign;

pub fn sign_over(p: &UPoly<Rational>, lo: &Rational, hi: &Rational) -> Verdict<Sign>;

pub struct CurveAnalysis { /* … */ }
impl CurveAnalysis {
    pub fn of(f: &MPoly<Integer>, x: VarId, y: VarId, b: Budget) -> Result<CurveAnalysis>;
    pub fn critical_abscissas(&self) -> &[AlgebraicReal];
    pub fn branch_count_over(&self, interval_index: usize) -> u32;
    pub fn branch_map_across(&self, critical_index: usize) -> &BranchMap;
}
```

`CurveAnalysis` mentions **no geometric type**: `MPoly` in, algebraic numbers and counts out.
Its construction being an explicit **handle** is the point — the prior art recomputes the
resultant *and* re-isolates its roots on every `cmp_y_right_of` and every `intersect` call, with
no cache (`conics.rs:459-460`, `:565-567`).

### 8.5 Layer 4

```rust
pub struct Store { /* hash-cons table + symbol interner + FuncTable. Owned. Send. */ }
pub struct Expr(u32);   // Copy, store-relative, never serialized

pub enum Node {
    Const(Rational), Symbol(SymbolId),
    Add(SmallVec<[Expr; 4]>), Mul(SmallVec<[Expr; 4]>), Pow(Expr, i64),
    Apply(FuncId, SmallVec<[Expr; 2]>),   // OPAQUE. Semantics live in the FuncTable.
}

impl FuncTable {
    pub fn empty() -> FuncTable;
    pub fn standard_elementary() -> FuncTable;   // a constructor, never a default
    pub fn register(&mut self, name: &str, arity: u8, deriv: Option<DerivRule>) -> FuncId;
}
impl Store {
    pub fn diff(&mut self, e: Expr, wrt: SymbolId) -> Result<Expr>;
    pub fn diff_with(&mut self, e: Expr, wrt: SymbolId, leaves: &LeafRules) -> Result<Expr>;
    pub fn symbols_in(&self, e: Expr) -> BTreeSet<SymbolId>;
    pub fn walk_topological(&self, e: Expr) -> impl Iterator<Item = (Expr, NodeRef<'_>)>;
    pub fn is_polynomial_in(&self, e: Expr, syms: &[SymbolId]) -> Option<MPoly<Rational>>;
    pub fn canonical_bytes(&self, e: Expr) -> Vec<u8>;     // + SCHEMA_VERSION
    pub fn rebuild_from(&mut self, src: &Store, e: Expr) -> Result<Expr>;
}
```

One mechanism serves three incompatible needs because resolvent ships **no transcendental
semantics in core** — only a table the caller constructs. A consumer needing `sin`/`exp` with
derivative rules calls `standard_elementary()`; one needing an opaque domain atom with **no**
rule registers `deriv: None`, so differentiating it is a structured refusal rather than a wrong
answer; one that must never see a given function simply never registers it, making the symbol
structurally unrepresentable in its world. `diff_with` takes a `BTreeMap` rather than a closure
because a closure that mints nodes would need `&mut Store` while `diff_with` holds it; the table
form is borrow-clean, reentrancy-free, and deterministic by construction.

---

## 9. Deferred: consumer integration

**Status: explicitly deferred. Not deferred by omission — deferred by a decision (ADR-018) with
a named list of things not to do, so that all three options stay open at approximately zero
cost.**

### 9.1 The question

`/home/dev/projects/arrangements` ships 3,602 lines of `lazy-exact` whose `roots.rs` (927 lines)
is resolvent's L1+L2+L3 in miniature, built over dense ℚ with a monic-normalized Euclid gcd and
explicitly no separation bounds — its own module header says comparisons "terminate because
distinct algebraic numbers are eventually separated by bisection", which is true, correct, and
unbounded. resolvent will do the same work over ℤ with modular methods. The two will coexist.
Whether they should eventually be one thing is the deferred decision.

Note that a merge is smaller than it sounds: `lazy-exact` filters *arithmetic* (its lazy `Real`,
`Interval`, expansions, error-free transforms) while resolvent does *algebra*. Those are
orthogonal axes. Even under a full merge, `lazy-exact` survives as the filtered-arithmetic layer
and resolvent supplies the algebra; what merges is `roots.rs` and `sqrt_ext.rs`, not two kernels.

### 9.2 The three options, none currently foreclosed

| | Option | Shape | Cost if chosen | Cost if wrong |
|---|---|---|---|---|
| **A** | resolvent adopts a consumer-shaped scalar seam | resolvent's polynomial and algebraic-number types become generic over a consumer-supplied scalar | A public generic parameter on the headline types; monomorphization over an *open* instantiation set; the modular fast path becomes conditional on a consumer trait impl | **Very high.** A public generic parameter cannot be removed without a major version and a rewrite of every consumer |
| **B** | the consumer writes an adapter | a small consumer-side crate maps `resolvent::{Integer, Rational, UPoly, AlgebraicReal, SqrtExt}` onto `lazy-exact`'s vocabulary | conversion at the boundary; two enclosure implementations; two `Sign` types, trivially mapped | **Low.** Delete the adapter |
| **C** | eventual merge | `roots.rs` and `sqrt_ext.rs` are deleted; the consumer depends on `resolvent` directly | one number vocabulary — but resolvent inherits geometry's latency requirements and the filtering layer stays behind, so the seam moves rather than disappearing | **Medium.** Reverting means reinstating deleted code |

**B is the default and the only one being built toward. A and C are kept reachable.**

Note that option A is *not* the same as resolvent's `Ring` tower being open. The tower is open
over resolvent's own algebraic vocabulary, which a consumer implements for its own coefficient
type; option A is resolvent becoming generic over a *consumer's* ops surface, threading that
parameter through the headline types. The first is additive and already decided (ADR-019); the
second is the expensive door and stays shut.

### 9.3 What would settle it — each a measurement or an event, not an opinion

1. **The degree/coefficient profile of the real workload.** Generate degree 3–8 curve pairs with
   realistic coefficient bit-size; record `Res_y` degree and coefficient length, and the wall
   time of `isolate_roots` plus a `sign_of` sweep, against the existing `QPoly`. If resolvent's
   ℤ + modular pipeline wins by a large factor on that corpus, C becomes attractive; if the
   crossover sits above the workload, B is correct indefinitely and A never pays for itself.
   **This is the single most informative measurement available, it is cheap, it runs against the
   existing crate, and every performance requirement on the geometry path currently rests on a
   guess it would remove.** It also carries the `Ord` step distribution (§6.3).
2. **Whether resolvent's `SqrtExt` matches `sqrt_ext.rs`'s cross-root comparison** on the
   `circle_segments.rs` path. If it is slower, C is off the table — 931 lines of the shipping
   consumer never touch an algebraic number and must not start.
3. **Whether a second consumer with a different number type materializes.** Two consumers with
   different scalars argue for A or B and against C, because C is a merge with *one* consumer.
4. **Whether the f64 enclosure semantics agree exactly** with the consumer's outward-widening
   `Interval` (431 lines, no global rounding-mode state). See §9.4 — this one is **not** a
   measurement and should stop being described as one.
5. **Whether `AlgebraicReal`'s `Arc` + `&self`-refinement model actually removes the
   `Rc<RefCell<_>>` tax.** If the adapter still needs a wrapper, ADR-013's model is wrong and
   must be revisited *before* C, not after.
6. **Whether `sign_radical_tower` at arbitrary depth actually beats materialization.** If it
   does not, the consumer's ~150-lines-per-family ladders can be deleted in favour of the
   general path, which makes C substantially smaller.

### 9.4 What is being done NOW to keep all options open at zero cost

- **`AlgebraicReal` is shape-identical to the incumbent.** `RealRoot { poly, lo, hi,
  multiplicity }` (`roots.rs:316-322`) against `AlgebraicReal` plus `IsolatedRoot`. The two
  deliberate deviations are the fixes: multiplicity is not part of identity, and refinement is
  `&self` on an `Arc<Inner>` rather than `&mut self`. A merge is then "take resolvent's version
  of these two decisions", not "reconcile two designs". Note that the incumbent derives only
  `Clone, Debug` — **no `PartialEq`, `Eq`, `Ord` or `Hash`** — so resolvent's value-equality
  `Eq` and its deliberate absence of `Hash` collide with nothing: there is no `HashMap<RealRoot,
  _>` to break.
- **The f64 enclosure contract is pinned NOW, as a committed conformance-vector file, not as a
  future measurement.** Two enclosure semantics that disagree at a filter boundary produce a
  wrong *verdict*, not a wrong number, which is the specific failure ADR-015 exists to prevent —
  and that is a **specification**, not something an experiment discovers. A few hundred
  `(exact value, expected (lo, hi))` pairs, including subnormals, values at powers of two, exact
  halves, and the largest finite double, committed in `resolvent-oracles` so that *any* consumer
  can run it against its own interval type. That artifact makes option C's hardest item
  checkable before anyone commits to C, and it makes option B's adapter testable. Nothing else
  on this list has that property, and it costs an afternoon.
- **The oracle harness is written so the incumbent can be *added* as a differential oracle** —
  subprocess, or a `publish = false` dev-only path. That is how items 1, 2, 5 and 6 get measured
  without any coupling, and `publish = false` means it cannot leak into a consumer's graph.
- **`gcd_ext` cofactors, resultant cofactors and Gröbner cofactors are designed in from the
  start**, because an SMT consumer's own documentation calls proof production
  non-retrofittable, and because the same data grades resolvent internally.

### 9.5 What would foreclose an option

- **Do not put a consumer-shaped scalar seam in the public API.** No trait mirroring
  `lazy-exact`'s `RingOps` / `ExactRing` / `ExactField`, which are explicitly an *ops surface*
  and "not an algebraic claim" — `Interval` implements them too. resolvent's traits are
  algebraic claims: `Field::inv` means a multiplicative inverse, not a best-effort division.
  Two similarly-named traits with different contracts across an adapter boundary is a bug
  generator, and a competing vocabulary makes the merge *more* expensive: under the orphan rule
  neither repository could write the impl, so a glue crate becomes mandatory and nobody owns it.
- **Do not add a generic parameter to `AlgebraicReal`.** It is `AlgebraicReal`, not
  `AlgebraicReal<S>`. If A is ever chosen, the generic type is a **new** type and the
  monomorphic one stays. This is the single most expensive thing to get wrong.
- **`SqrtExt<T>` keeps its generic parameter, and here is the reason it is not the same door.**
  `T` ranges over resolvent's *own* closed set (`Rational` today, `NumberFieldElem` later) and is
  bounded by resolvent's own tower (`T: Ordered + Field`). `SqrtExt` is a *construction over a
  coefficient ring*, which is exactly what the open tower is for; `AlgebraicReal` is a
  *representation over ℤ*, and a generic parameter on it would be a consumer-shaped seam by
  another name. The distinction is recorded because ADR-018 forbids the generic on
  `AlgebraicReal` **by name** and is silent about `SqrtExt`, and silence in a one-way-door list
  is how doors get closed by accident.
- **Do not expose a float interval type.** One of the two enclosure semantics must be the
  adapter's, and it must be the consumer's.
- **Do not name any consumer anywhere in a published crate** — not in a feature flag, not in a
  doc example, not in a comment. A consumer-named feature would be option B smuggled into
  resolvent, which is the one place it must not live.
- **Do not copy the incumbent's trait or type names** (`RingOps`, `ExactRing`, `ExactField`,
  `Scalar`, `Uncertain`, `USign`, `UOrd`). Deliberately different names force the adapter to be
  explicit.
- **Do not subsume `SqrtExt` into `AlgebraicReal`.** Keeping degree-2 radicals first-class is
  what keeps C from being a regression on 931 lines.
- **Do not accept a tolerance parameter anywhere**, at any layer, under any name.
- **Do not let resolvent's error surface force a fallible query path.** "Fail at construction,
  not at query" exists partly for this: the adapter converts once, where it already has an error
  path, and the consumer's predicates stay infallible.

### 9.6 Triggers that convert the deferral into a decision

- Measurement 1 shows a large win at the consumer's real sizes **and** measurements 2 and 5 come
  back clean → choose **C** and write an ADR that supersedes ADR-018.
- Measurement 1 shows the crossover sits above the workload → choose **B** permanently, write it
  down, and stop paying for C's constraints (specifically, the obligation to keep enclosure
  semantics reconcilable — though note that §9.4's conformance file is worth having regardless).
- A second consumer with an incompatible scalar type adopts resolvent → B is confirmed, C is
  foreclosed, and A becomes a live question again, answered by whether *both* consumers would
  use the generic path.
- The adapter, once written, exceeds ~500 lines or needs a wrapper around `AlgebraicReal` →
  that is evidence a decision in ADR-013 or ADR-015 is wrong, and it is fixed **there**, not
  here.

---

## 10. Known risks, and what would falsify this design

Each risk carries a numeric or event trigger. A risk with no trigger is a worry, not a risk.

### 10.1 The bignum floor is lower than the architecture assumes

**Claim at risk:** modular methods keep bignum work in the regime where the permissive bignum is
competitive.
**Why it might be false:** §5.7 — reconstruction concentrates the work at 58–70 kbit `gcd_ext`,
where `dashu`'s Lehmer is quadratic and GMP's half-GCD is subquadratic, on the default certified
path.
**Trigger:** the extended ladder. If `gcd_ext` at 64k bits is **>10×** `rug`, the in-wall
half-GCD becomes a scheduled M1 lane. If it is **>50×**, or if `rational_reconstruct` at
Hexapod's modulus size dominates the whole Hexapod run, the optional non-default `backend-gmp`
feature *seam* is designed immediately — cheap now, expensive later — while the default build
stays permissive.
**Falsifies:** ADR-002's cost model, not its licence posture.

### 10.2 The monomial term type is decided by an experiment that cannot run

**Claim at risk:** the interned `(MonomialId, C)` term type is the right one-way door.
**Why it might be false:** comparing by id requires a random arena load whose cache miss may
dominate the `u64` compare it enables — which would deflate packing's already-modest 15%
further.
**The scheduling defect, and its fix:** the experiment as specified needs "a realistic S-pair
queue workload", i.e. a working Buchberger/F4 — which is three waves later and is itself gated on
the experiment. As scheduled the freeze **deadlocks**. The fix is a synthetic harness specified
in the ADR: record an operation trace (`lcm-query`, `divisibility-query`, `insert`) from a
200-line throwaway Buchberger over GF(p) with `Vec<u32>` exponents on Katsura-6 / Cyclic-6 — one
day of work, discarded afterwards — and replay it against both term representations, measuring
the divisor-query index's speedup under each.
**Trigger:** if inline packed keys win the replayed trace, terms carry inline keys and the arena
shrinks to a divisor-query index. The ownership rule (§5.2) does not change; only what the
`Ring` context holds does.

Two sibling experiments have the same defect and the same fix. The cofactor multiplier was to be
measured on Katsura-8/Cyclic-7, needing an engine that reaches Katsura-8 while gating whether
that engine's certified mode exists — measure it on **Buchberger with cofactors at Katsura-6/7
over ℚ** and report the multiplier as a function of instance size. The `AlgebraicReal` mutability
prototypes were to sort 10³ degree-8 algebraic numbers, needing a working `AlgebraicReal` while
gating the lane that builds it — the four prototypes need only `cmp`, `refine` and a polynomial
sign evaluation, roughly 300 lines over `UPoly<Integer>` with roots built as `Π(x−rᵢ)`, and
**they do not need the production isolator.** Say so in the ADR, or the lane blocks on itself.

### 10.3 Certified Gröbner may not be affordable, and it is the fast mode's only internal oracle

**Claim at risk:** `groebner_certified` is the regression workhorse that grades `groebner`.
**Two problems, both corrected here.**

First, the two modes **cannot share a reducer**, and ADR-010 §5 says they do. F4 row reduction is
Tier M, concrete over `u32` payloads with `FpParams` by value; the certified mode's cofactors
must be checked over ℚ or ℤ, because a cofactor identity that holds mod `p` proves nothing about
ℚ. They are different code. What they genuinely share is the *matrix construction*, the *symbolic
preprocessing*, the *monomial layer*, and the *row format* with its optional cofactor block. The
consequence is that a bug in the fast reducer's pivot selection, delayed-reduction cutoff, or
Barrett reduction is **invisible** to the certified mode. Two corrections follow: the fast
reducer's primary verdict is **external** differential testing (Singular, msolve) — an inversion
of the normal rule that must be written into the lane brief — and the fast reducer gets an
internal oracle it can actually have, **a naive dense `u32` Gaussian elimination over the same
`FpParams`, in the same crate**. That is a genuine same-arithmetic cross-check and it costs one
agent-session.

Second, the prototype measures the wrong number. Cofactors must be **CRT-reconstructed** to be a
ℚ certificate; cofactor coefficients are systematically larger than basis coefficients (that is
what "cofactor swell" means), so **the prime count is set by the cofactors, not the basis** — and
there are `|F| × |G|` of them. Measuring the GF(p) time and memory multiplier does not answer
that.
**Trigger:** measure "primes needed and wall time to reconstruct the cofactor system over ℚ on
Katsura-6/7" (§10.2's harness). If the multiplier is above ~20× in memory or the prime count
exceeds ~5× the basis's, `groebner_certified` becomes small-instance-only and is documented as
an oracle rather than an API; the `Certificate` type does not change, only which variants are
reachable at which sizes.

### 10.4 `Ord` hangs on a pathological pair

Covered in §6.3. **Trigger:** the measured step distribution on the elimination corpus. If the
99.9th percentile exceeds the committed diagnostic ceiling on realistic input, `Ord` goes and the
explicit-context form returns — which is why the ceiling is a counter rather than a comment: it
surfaces the condition *before* it ships. **Do not leave this unsettled past M3.**

### 10.5 M8 compiles and is useless

**Claim at risk:** `UPoly<NumberField>` arrives as "an added instantiation, not a rewrite,
because `UPoly<C>` was generic from day zero."
**Why it is only half true:** the instantiation compiles; `C: Reducible + Liftable` cannot be
satisfied over a multiquadratic tower (§5.1), so it silently gets the Tier-G reference path —
correctness without speed — for the consumer M8 exists for, whose inner loop is root isolation
over ℚ(α₁…α_k).
**What must land instead:** the multi-modular-over-split-factors lane, with its own bad-prime
predicate, sized as a lane rather than as an instantiation.
**Trigger and corpus:** **ℚ(√2, √3) is in the M8 corpus specifically**, because it is the
instance where the naive implementation divides by a zero divisor and where `reduce` must return
`Err(BadPrime)` for *every* prime.

### 10.6 The gates erode before the algebra arrives

Three gates are structurally fragile and each has a fix in this document: the determinism matrix
against an append-only corpus (§7.7 — tier it on day one); the compile-time budget measured
relatively against a near-empty baseline (§5.6 — absolute ceilings, ratchet down only); and the
sharpness ceilings that are not numbers (§6.6 — the ratchet, with `TBD` failing the gate). A
fourth is not a gate at all yet: **"ratified" has no mechanical definition** (§0.2), so the
freeze — the plan's single declared global barrier — is currently an intention.

A fifth belongs here because it is the difference between a self-certifying library and a library
that says it is one: **every certificate ships with a mutant set.** A certificate is code, and
the failure mode of certificate code is not "it rejects a correct answer" (loud) but "it accepts
everything" (silent). At least one deliberately wrong implementation per operation, committed
under `#[cfg(test)]` in the same module, with a test asserting the certificate **rejects** it;
mutants that the type system rejects do not count. The classes are prescribed so they are not
chosen to be easy: coarsening, refining, off-by-one in a bound, identity, trivial constant, sign
flip, silent wrap. The second-order reason this matters: the triage pipeline classifies a
disagreement by whether the self-certificate also fails, so **a vacuous certificate routes every
real bug into "normalization or convention"**, where the prescribed response is to write an ADR
— a plan that metabolizes its own bugs into documentation.

### 10.7 Smaller open items, each with what would settle it

1. **Barrett/Shoup vs Montgomery for word-size GF(p).** The argument for Barrett — the same `p`
   is reused against many operands, so Montgomery conversion is not amortized — is an
   architecture argument, not a measurement, and the answer may differ between the scalar path
   and the F4 bulk-row path. *Settle:* the first benchmark of the modular lane. Both are
   implemented; the default is chosen by measurement and recorded.
2. **Whether any permissively licensed F4 exists in any language.** The search found only GPL,
   and `feanor-math`'s is "F4-style Buchberger" rather than true F4. If none exists, the F4 lane
   has no Tier-A reference and must be built from Faugère's paper plus the Macaulay-matrix
   literature — feasible but slower, and worth knowing before the lane is sized.
3. **Whether forking `feanor-math`'s (MIT) Cantor–Zassenhaus and LLL is cheaper than writing
   them from papers.** The prior-art survey rejected it as a *dependency* on two grounds — a
   pinned nightly and a missing repository `LICENSE` file — neither of which is decisive for a
   *fork*: nightly features may be confined to the ring-framework glue rather than the algorithm
   modules, and the licence file is a one-line upstream issue that is load-bearing only in the
   fork scenario. *Settle:* count and classify the `#![feature(...)]` uses per module, file the
   upstream issue regardless, and decide **per lane**, recording the decision. The prior is that
   LLL and Cantor–Zassenhaus are plausible lifts and Buchberger is not, because the monomial
   design differs. Even if the answer is "write it all from papers", the decision should be
   recorded rather than implied.
4. **Whether `lll-rs` (MIT) is usable at van Hoeij precision.** The lattice has dimension ~`r`
   with entries of size `p^k` exceeding twice the Landau–Mignotte bound — potentially thousands
   of bits. *Settle:* run it on a Swinnerton–Dyer degree-64 lattice. If it fails, LLL becomes its
   own lane and van Hoeij's schedule doubles. LLL is fully self-certifying (its output conditions
   are directly checkable), which makes it a good lane in either case.
5. **The exact hypotheses of the Idrees–Pfister–Steidel theorem after Noro–Yokoyama's
   correction.** This decides whether the fast Gröbner path can ever return `Proved` without
   cofactors. Both papers were paywalled at research time. Until obtained, the plan assumes it
   cannot.
6. **Whether the Landau–Mignotte bound is itself certified.** It is not, in any current
   document, and it feeds van Hoeij directly: a **too-small** bound produces a lattice that has
   not stabilized, in which spurious 0/1 vectors are accepted by the algorithm's own termination
   witness, and the output is a **coarse factorization that multiplies back correctly** — the
   named failure of the hardest lane, reached through an uncertified input. *Settle:* a row in
   the certificate catalogue and a generator — for every instance from the known-factorization
   generator, the computed bound is `≥` the true maximum factor coefficient, with `bound/actual`
   tracked as a sharpness distribution.
7. **Whether `resolvent-base` compiles as `#![no_std]`.** Nothing in its contents needs `alloc`;
   the claim fails the moment `Error` grows a `String` or a `Box`, which §6.4 forbids anyway.
   Cheap to check once the crate exists.
8. **Whether ANewDsc's Newton acceleration interacts safely with the refinement API.** Newton
   steps *jump*, so the isolating interval does not shrink by halving. Monotone containment
   still holds, but the endpoint invariants (`poly(lo) ≠ 0 ≠ poly(hi)`, collapse-to-a-point on an
   exact hit) were derived for bisection. *Settle:* re-derive them for the Newton path **before**
   implementing, not after.

---

## Appendix A — where each critique finding is carried

| Finding | Severity | Carried in |
|---|---|---|
| `Ring::zero()` unimplementable; `Liftable` does not compile | fatal | §5.1 (corrected tower), §4.3 (ADR-006/019 amendment) |
| Two normative specs, eleven divergences | fatal | §0 (precedence, ratification, the census) |
| `groebner_certified` cannot share a reducer; cofactor prototype measures the wrong number | fatal | §10.3, §4.3 (ADR-010 amendment) |
| `Reducible::Image: Field` false over number fields | serious | §5.1 detail 2, §10.5 |
| The interner is a shared mutable accumulator | serious | §7.3, §4.3 (ADR-012 amendment) |
| Shared refinement makes declines schedule-dependent | serious | §6.3 (the invariant), §7.5 (bytes exclusion) |
| `Ord` is unbounded and undeclinable on the default path | serious | §6.3, §10.4 |
| Reconstruction *is* the large-integer regime | serious | §5.7, §10.1 |
| FGLM needs two orders live simultaneously | serious | §5.2 (dual key), §4.3 (ADR-009 amendment) |
| Divisibility is an order-free inner loop; `W_KEY` ≠ `W_RAW` | serious | §5.2 |
| Three blocking experiments require the artifact they gate | serious | §10.2 |
| Gate 0's budget cannot survive an append-only corpus | serious | §7.7 |
| `forbid(unsafe_code)` vs the Competitive gate | serious | §7.6, §4.3 (new ADR-021) |
| Batched lanes have no story for a bad lane | serious | §5.1 (`inv_batch`), §7.4 (split driver) |
| L4 scope has grown a rewriter, two backends and an integrator | minor | §1.3, §3.1, §4.3 (ADR-017 amendment) |
| Three `Certificate` shapes; evidence in canonical bytes | minor | §5.4, §7.5 |
| `MonomialId` exhaustion; the arena never forgets | minor | §5.2 |
| The `rug` dev-oracle has nowhere to live | minor | §2.5, §3.5 (L6) |
| The compile-time gate is relative | minor | §5.6 |
| `BulkOps` puts a generic back in the kernel | minor | §5.1 |
| Certificates are unverified code (no mutant sets) | fatal (plan) | §10.6 |
| Both gcd certificates are circular | serious (plan) | §8.3 |
| Randomized certificates run at one fixed seed | serious (plan) | noted in §7.2 — a randomized certificate is graded across the fleet seed schedule, never at the default seed alone |
| The 4-bit overflow sweep is trivially satisfied | serious (plan) | §5.2 — the sweep is a distribution assertion against each instance's true `D_max`, and a width at which zero instances complete is a **failed** sweep |
| "Any decline is a failure" contradicts the decline-rate gate | serious (plan) | §6.6 |
| No sharpness ceiling is a number | serious (plan) | §6.6 (the ratchet) |
| `resolvent-base` has no lane; Z7 and the serializer are prerequisites | serious (plan) | §3.3, §4.3 (lane Z0 absorbs Z7 and the canonical serializer, and is the sole Wave-1 blocking lane) |
| The S-pair and FGLM certificates are weaker than stated | serious (plan) | §10.6 (mutant classes), §4.3 (ADR-010, ADR-009) |
| No `Equal` from exhausting the separation bound | serious (plan) | §5.3 detail 4 |
| A composite in the prime table is undetectable | serious (plan) | §7.2(c) |
| The Landau–Mignotte bound has no certificate | serious (plan) | §10.7.6 |
| `RealRoot::multiplicity()` is a stored method — the merge collision | serious (plan) | §5.3 (`IsolatedRoot`), §9.4 |
| The `Derivation:` gate is trivially satisfiable | serious (plan) | §2.4 gate 4 |
| Benchmark-family provenance instructs GPL transcription | serious (plan) | §2.4 gate 5 |
| Forking `feanor-math` was never evaluated | serious (plan) | §10.7.3 |
| "Ratified" has no mechanical definition | minor (plan) | §0.2 |
| CRT moduli distinctness; factors pairwise non-associate; resultant degenerate conventions | minor (plan) | §7.2(c), §10.6, §4.3 (new ADR-022) |
| The regression corpus has no provenance field | minor (plan) | §7.7 — every entry carries `provenance ∈ {constructive-generator, oracle-consensus(k), hand-computed(author, method), minimized-counterexample}`, and oracle-consensus entries are re-derivable |
| Oracle adapters are graded by round-trip only | minor (plan) | §4.3 (ADR-016: every adapter ships a hand-computed calibration corpus) |
