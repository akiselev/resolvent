# ADR-021 — Document precedence, machine-readable ratification, and the contradiction register

**Status:** Ratified 2026-07-31
**Reversibility:** cheap (it governs process, not representation) — but the *cost of not
having it* is one-way, because lanes fan out against whichever document they read first
**Gates lanes:** all of them. This is the freeze mechanism.
**Evidence:** `docs/research/critique-engineering.md` §2 (fatal);
`docs/research/critique-plan.md` C2 (fatal), C19.

---

## Context

Four planning documents and twenty ADRs were written in parallel by separate tracks. Two of
them each claim supremacy over the same subject matter:

- `plans/architecture.md` line 11 — "Where the two disagree, the ADR wins" — said only about
  itself and the ADRs.
- `plans/api-shape.md` §Status — "Binding on the founding architecture unless explicitly
  overturned." Later superseded by `API.md`, whose header says "canonical".

The result is not a stylistic mismatch. **The two tracks specify different libraries**, and
the adversarial reviews found at least eleven divergences at *signature* level, four of them
on decisions explicitly marked one-way. `plans/roadmap.md` §2.5 exists specifically to catch
this, flagged two contradictions, and both of those had since been settled by ADR files that
now exist — while none of the eleven live ones was named.

Second, the freeze has nothing to key on. `plans/roadmap.md` gates lanes on ADRs being
"ratified and merged". Every ADR file said `Status: Proposed`, every ADR file is merged, and
nothing in the repository defines what ratification *is*, who performs it, or what changes.
The roadmap simultaneously reported ADR-010…018 as "declared, unwritten" while the files
existed. So the plan's single declared global barrier was, mechanically, a convention:

- An agent reading the roadmap blocks lanes A1 and P1/P2/P3 on experiments the ADRs
  pre-empted.
- An agent reading the ADRs starts them.
- Both are following the plan.

Third, the specific failure this produces is not "confusion". It is that lane Z0/Z7's
deliverable is *the `Certificate` type*, three documents describe three incompatible shapes
with disjoint `ProofKind` variant sets, the gate that would have caught it keys on "is the
ADR merged", and every ADR is merged. The gate fires green and every Layer-2 and Layer-3
lane downstream is written against the wrong type.

---

## Decision

### 1. One precedence rule, stated once, in one place

> **ADRs are normative. Where a plan document, `API.md`, or a research note contradicts a
> Ratified ADR, the ADR wins, and the contradicting text is a *proposed amendment* to that
> ADR — not a binding statement.**
>
> `API.md` is normative for *consumer-facing shape* wherever no ADR speaks: which
> capabilities are core, adapter, or out of scope; how the 200-line test is reported;
> the consumer dossiers. When `API.md` states a signature, the authority is `API.md`
> **plus the named ADR**, and where they differ the ADR governs the signature and `API.md`
> governs whether the capability exists at all.
>
> `plans/*.md` are **working notes**, superseded in every case by a document at the root or
> in `docs/decisions/`. `plans/api-shape.md` is superseded by `API.md`; `plans/roadmap.md`
> is superseded by `ROADMAP.md`; `plans/architecture.md` and `plans/verification.md` remain
> readable and are non-normative wherever an ADR speaks.

The tie-break direction is not arbitrary. `API.md`'s consumer-facing decisions are better
evidenced — they come from three real consumer evaluations with file-and-line citations —
and the ADRs' internal decisions are better argued. Neither wins wholesale; the split above
is by *subject*, so every item has exactly one home.

### 2. `Status:` is a machine-readable field, and ratification is a merge

**The first `**Status:**` line in the file** — line 3, immediately under the title — matches
one of exactly four forms. The CI check reads that line and only that line, so a form quoted
inside a fenced code block (as they are here) is not mistaken for a declaration:

```
**Status:** Draft
**Status:** Proposed
**Status:** Ratified YYYY-MM-DD
**Status:** Superseded by ADR-NNN
```

**Ratification is the repository owner merging a commit that sets the line to `Ratified`.**
Nothing else counts — not a discussion, not an agent's assessment, not the file existing.
An agent may *draft* an ADR and may *run the experiment* an ADR needs; it may not ratify
one. Trying to grade ratification automatically produces an ADR that argues for whatever is
easiest to test.

An ADR that is amended after ratification keeps its `Ratified` date and gains an
`**Amended:** YYYY-MM-DD — <what changed>` line. The amendment is ratified by the same act.

### 3. The freeze is a checked-in manifest, not a convention

`lanes.toml` at the workspace root maps every lane to the ADRs it inherits:

```toml
[lane.Z3]
crate  = "resolvent-modular"
gates  = ["ADR-003", "ADR-006", "ADR-011", "ADR-012", "ADR-021"]
grade  = "certificate"
oracle = []                 # score lanes name the lane they are graded against

[lane.G3]
crate  = "resolvent-algebra"
gates  = ["ADR-006", "ADR-008", "ADR-009", "ADR-010", "ADR-022"]
grade  = "score"
oracle = ["G1", "G2"]       # must be green and frozen before this lane's CI job exists
```

CI enforces three things off that file, all of them ten lines:

1. **A lane's test target is `#[ignore]`d — and its crate is absent from the workspace
   members list — while any gating ADR's `Status:` line does not match `^Ratified`.** That
   converts the freeze from an intention into a dependency edge.
2. **A score lane's CI job does not exist until every lane named in `oracle` is green and
   frozen.** Same mechanism, different column. This is the rule that keeps "build the oracle
   side first" from being cultural.
3. **Every lane in `ROADMAP.md` §3 appears in `lanes.toml` and vice versa.** A lane with no
   manifest entry has no gate.

*Amended 2026-08-08 (ADR-030).* The `conformance` grade adds three fields, and **`oracle` is
not one of them** — that field holds **lane ids** and CI rule 2 resolves each entry against
`[lane.*]`, so putting an external system name in it would make the rule fail to resolve.
External oracle systems get their own key:

```toml
[lane.X5q]
crate           = "resolvent-expr"
gates           = ["ADR-029", "ADR-030", "ADR-033"]
grade           = "conformance"
oracle          = ["X5"]        # lane ids, as everywhere else: X5 must be green and frozen
self_certifying = false         # REQUIRED, and REQUIRED to be false for this grade
oracle_systems  = ["sympy"]     # external systems; non-empty; a missing one FAILS, never skips
divergence_ceiling = "rewrite.quality.divergence"   # key into sharpness-ceilings.toml
```

CI gains three checks off those fields, each as small as the existing three: `grade =
"conformance"` ⟹ `self_certifying = false` and `oracle_systems` non-empty and
`divergence_ceiling` present and not `TBD`; **no lane may name a `conformance` lane in its own
`oracle` list** (ADR-030 §3 — a conformance lane gates nothing and is an oracle for nothing);
and a declared `oracle_systems` entry that is absent at run time fails the job rather than
skipping it (`CLAUDE.md` §7).

### 4. A grep gate on headline type names

CI extracts every fenced `rust` code block from `plans/`, `docs/decisions/`, `API.md` and
`ROADMAP.md`, and fails when a headline type or trait — `Ring`, `Reducible`, `Liftable`,
`Certified`, `Certificate`, `Certainty`, `ProofKind`, `AlgebraicReal`, `MPoly`, `UPoly`,
`Ring` (the multivariate context), `MonomialEntry`, `Store`, `Node`, `IsolatedRoot`,
`SqrtExt` — is *defined* differently in two places. Redeclaring a type to show a signature
is fine; two divergent definitions are not.

It is cheap and it is the only thing that would have caught eleven of the twelve
divergences below.

---

## The contradiction register

Every divergence found by the adversarial reviews, its resolution, and where the resolution
now lives. **A new divergence is appended here with the same three columns.** This table is
the reason §4's grep gate exists: it is what the gate is protecting.

| # | Subject | Resolution | Now lives in |
|---|---|---|---|
| 1 | Crate graph: `base/int/modular/poly/algebra/real/expr` vs `seam/int/modular/poly/linalg/engine/alg/expr/lazy` | **Architecture's graph.** No `resolvent-seam` (ADR-019), no `resolvent-lazy`, `linalg` contents live in `-algebra`. Gate L1 diffs `cargo tree` against it | ADR-005 |
| 2 | A public `Scalar`/`ScalarOrd`/`TryDiv`/`Hom` seam: forbidden vs "the single highest-leverage hook" | **No ops-surface scalar trait, no seam crate.** One open coefficient tower serves both questions | ADR-019 |
| 3 | `Interval<f64>`: forbidden vs core, in adapter signatures | **No float interval type in any published crate.** Rational bounds + an outward-correct `(f64, f64)` pair, plus committed conformance vectors | ADR-015 |
| 4 | `AlgebraicReal`'s polynomial: `UPoly<Integer>` vs `Arc<SqfrPoly<Rational>>` | **ℤ-primitive.** ℚ is a transport type and appears in no stored field | ADR-004 |
| 5 | Multiplicity: pair element vs `mult: u32` field | **Neither, exactly:** `IsolatedRoot { value, multiplicity }`. Not on the number; not a bare tuple | ADR-014 §3 |
| 6 | `AlgebraicReal`: `Send + Sync` vs `Send + !Sync` | **`Send + Sync`**, `Arc<Inner>`, monotone `&self` refinement. Decided by experiment E-MUT, which is respecified so it does not need the type it gates | ADR-013 |
| 7 | Terms: `(MonomialId, C)` interned vs `(PackedMon, C)` inline | **Ownership settled** (arena belongs to the `Ring`); **term type open**, decided by E-MONO, respecified against a recorded S-pair trace | ADR-020 §1, ADR-008 |
| 8 | `MPoly` holds `&'a Ring` vs no lifetime on any public owned type | **Owned handle** (`Arc<Ring>` or a ring-table index). No lifetime | ADR-007, ADR-020 §2 |
| 9 | Three `Certificate` shapes with disjoint `ProofKind` sets | **`API.md`'s shape** — claim tether, no public mint, public read — with `ProofKind` unified by union | ADR-010 §2 |
| 10 | Budgets: only where no bound exists vs on every entry point | **On every looping entry point.** The two regimes govern what *exhaustion means*, not whether the parameter exists | ADR-011 §4 |
| 11 | `Zn` in the instantiation set vs ℤ/n out of scope | **In.** Hensel lifting to `p^k` is arithmetic modulo a composite; it is lane K2 and M1's exit gate requires it | ADR-006 |
| 12 | L4: `Simplifier` + `RuleSet` + two backends + integrator vs "no `simplify()`, out of scope" | **In.** `simplify(expr, &RuleSet)` ships, never implicitly, every rule classified R/S/D by its soundness argument; e-graph adapters stay deferred for their own separate reasons. *Closed "out" on 2026-07-31 and reopened 2026-08-08: that resolution rested on L4 being "not the point", which ADR-029 retired.* | ADR-033 |
| 13 | Scope: "not a general-purpose CAS" (`README.md`) vs the standing-CAS admission test (`API.md` §4.2) vs "symbolic integration, limits, series — out of scope **permanently**" (`API.md`:622) | **General-purpose.** The standing-CAS test is promoted from tiebreaker to primary admission rule; the analytic surface is *in scope and unspecified*, which is a distinct state from out of scope and blocks a lane until its own ADR ratifies | ADR-029 §1 |
| 14 | Numerics: "Not numeric — the only `f64` is an outward enclosure returned to callers" vs two consumers each reimplementing an exactness lattice resolvent does not offer | **`f64` enters L4 as an inexact leaf** under a monotone `Exact`/`Enclosed`/`Approximate` lattice. L0–L3 stay exact-only; ADR-012 §6 and ADR-015 are unamended, and the lattice is what *enforces* them | ADR-031 |
| 15 | Zero-testing: "no transcendental zero-test, at any layer, **ever**" vs `sin(π/6)` denoting an algebraic number L3 can decide exactly | **No *unsound* zero-test ever.** Sound tests over named decidable subclasses, assumption in the return type. The old rule classified expressions by the symbols they are written with rather than the values they denote | ADR-032 |
| 16 | Verification: the prime directive requires a green certificate per operation, and part of ADR-029's surface has no self-certificate | **A fourth lane grade, `conformance`** — external differential agreement at a committed rate. Soundness is never conformance-graded and a conformance lane gates nothing | ADR-030 |

Two further items were reported as live contradictions and are **stale**, recorded so they
are not re-opened: `plans/roadmap.md` §2.5's contradiction 1 (`AlgebraicReal` mutability) was
settled by ADR-013, and its contradiction 2 (interning) was reconciled by ADR-008 §1 and
ADR-020 §1. The roadmap's parallel claim that ADR-010…018 were "declared, unwritten" was
false when written; `ROADMAP.md` supersedes it.

---

## Consequences

- **The freeze becomes a dependency edge.** A lane cannot start against an unratified
  decision because its crate is not in the workspace. That is the mechanism
  `plans/roadmap.md` §7 asked for and did not have.
- **Ratification stays human, and is visible in `git log`.** The cost is a serialization
  point: someone must read and merge. That is correct — the deliverable is judgment about
  irreversible tradeoffs and there is no verdict function for it.
- **Twelve divergences are closed and one register holds them.** A thirteenth appends a row
  rather than starting a fourth normative document.
- **The grep gate will produce false positives** on illustrative code blocks that abbreviate
  a signature. Mitigation: a `// abbreviated` marker line that the extractor skips, used
  sparingly and visible in review.
- **`plans/*.md` become historical.** They are not deleted — the arguments in them are the
  evidence the ADRs cite — but nothing may be gated on them, and `ROADMAP.md` and `API.md`
  say so in their headers.

---

## Alternatives considered and why rejected

**Declare `API.md` normative wholesale.** Rejected. It is the better document on consumer
shape and the weaker one on internals: it carried the receiverless `Ring::zero()`, the
`Reducible::Image: Field` bound, and `BulkOps`, all three of which are unimplementable or
counterproductive (ADR-006). Wholesale precedence in either direction picks up the other
document's mistakes.

**Declare the ADRs normative wholesale and stop maintaining `API.md`.** Rejected for the
mirror reason: the consumer-facing decisions — budgets everywhere, a `String`-free closed
error enum, runtime ring arity, no lifetime on owned types — are the ones with file-and-line
evidence behind them, and the ADR track had them wrong or absent.

**Merge everything into one document.** Rejected. A single 6,000-line specification is not
reviewable, is not diffable per decision, and destroys the property that makes an ADR
useful: one decision, one file, with its rejected alternatives attached to it.

**Leave ratification informal and rely on review.** Rejected. Founding constraint #3 puts
agents in the loop and agents key on what is greppable. "Ratified" that means nothing is
worse than no gate, because it produces confident wrong starts rather than blocked ones.

**Keep a separate `RECONCILIATION.md`** (referenced by `API.md` and ADR-019 and never
written). Rejected as a *separate* file: a reconciliation record that is not an ADR has no
status field, no ratification, and nothing keyed on it — which is how it came to be
referenced twice and written zero times. Its content is §3 and the register above.

---

## What would reverse this

- **The grep gate proving unmaintainable** — more than a handful of false positives per
  month. Response: narrow the type list to the ten that appear in one-way-door decisions,
  not the mechanism.
- **A second repository adopting these ADRs**, at which point precedence must name the
  boundary between shared and local decisions. Response: a scope field in the front matter,
  additive.
- **Ratification becoming the bottleneck** — lanes idling on unread ADRs. Response: batch
  ratification per wave, which is what `NEXT.md` day 0 already does; not delegating it.
