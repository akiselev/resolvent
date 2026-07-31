# ADR-001 — License posture, and the read-but-don't-copy discipline

**Status:** Ratified 2026-07-31
**Reversibility:** one-way
**Amended:** 2026-07-31 — gate 4 (`Derivation:`) now requires a research-note tether
(critique-plan C15).
**Supersedes:** nothing.
**Gates lanes:** H1, and every Layer-2 lane.
**Evidence:** `docs/research/prior-art-and-licensing.md` §0, §1.1, §3.1, §6, §7;
`docs/research/algorithms-and-representation.md` §0;
`docs/research/critique-plan.md` C15, C16.

---

## Context

resolvent's product thesis is "a permissively licensed computer algebra kernel." That is
not a preference; it is the entire reason the project exists, because the niche is
occupied by exactly two kinds of thing: copyleft implementations (Singular, FLINT, PARI,
msolve, Macaulay2, CoCoALib, Sage, Groebner.jl) and one proprietary source-available one
(Symbolica). R1 §3.5 checked and found no permissive Rust implementation that combines
modular methods, packed monomials, algebraic numbers, and stable-Rust compatibility.

Three findings from R1 make this harder than "pick MIT crates":

1. **The fastest pure-Rust bignum is not permissive.** `malachite` is LGPL-3.0-only —
   not by preference but because it is *derived from GMP, FLINT and MPFR source*. It has
   no dual arm and no permissive subset. This eliminates the obvious performance answer
   (ADR-002).
2. **Apache-2.0-only dependencies silently void the MIT arm.** Apache-2.0 is
   FSF-incompatible with GPLv2. An Apache-only crate anywhere in the published graph
   removes GPLv2-compatibility from downstream consumers who chose resolvent's MIT arm
   precisely to get it. This is the non-obvious half of the constraint and it bars
   `num-modular`, `num-prime`, and `nalgebra`.
3. **The failure mode is live and shipping.** `alkahest-cas` 3.7.0 declares
   `license = "Apache-2.0"` while carrying **non-optional** dependencies on `rug` and
   `gmp-mpfr-sys`, both LGPL-3.0+. That is exactly the audit failure a habit-based
   posture produces.

Separately: resolvent's best algorithmic references are copyleft, and one is worse than
copyleft. GPL grants an explicit right to read and study. Symbolica's `License.md` grants
no copying right at all and conditions source-availability — it is a *stricter* hazard
than GPL, and an agent will reliably infer the opposite, because "source-available" sounds
safer than "GPL".

---

## Decision

**resolvent is MIT OR Apache-2.0. Every non-dev dependency must offer an MIT-or-equivalent
arm. Copyleft is used only as a subprocess oracle or as a `publish = false` dev-dependency,
never linked into a published crate.**

### The reading discipline, in three tiers

**Tier A — freely readable, freely cited.**
Refereed literature and textbooks (Faugère, van Hoeij, Zassenhaus, Collins/Brown,
Rouillier–Zimmermann, Ducos, Sagraloff–Mehlhorn, von zur Gathen & Gerhard, Cohen,
Geddes/Czapor/Labahn). The *user-facing documentation and manuals* of any system — those
describe behaviour, and matching documented behaviour is a compatibility goal, not a
derivation. Permissively licensed Rust: `feanor-math` (MIT), `dashu`, `ark-ff`. Read
freely; still do not copy verbatim, because MIT carries an attribution obligation and a
copied block would need its notice carried, which defeats the purpose.

**Tier B — readable for *understanding*, never for *transcription*.**
Singular, FLINT, PARI, msolve, CoCoALib, Macaulay2, Sage, Groebner.jl (GPL-2.0),
GroebnerWalk.jl (GPL-3.0), SymPy (BSD, so no license hazard — but the same non-copying
discipline for consistency).

*Permitted:* reading to understand which algorithm variant is used, why a step exists,
what edge case a guard protects, what the overall pipeline is.

*Forbidden without exception:* copying code, comments, identifier names, file/module
structure, or **magic constants and tuning thresholds**. Thresholds are the likeliest
accidental transcription and the least defensible — "switch to Karatsuba at 32 limbs" is
someone's measurement on someone's machine, and it is *wrong for ours*. **Every threshold
in resolvent is re-derived by measurement against resolvent's own corpus, and the
measurement is checked in** (see ADR-012 §Tuning). This rule is simultaneously a licensing
rule and a correctness rule, which is why it holds under pressure.

*Procedure:* read → write a note in `docs/research/` in your own words → **close the
source** → implement from the note. The note is the artifact that proves the discipline
was followed.

**Tier C — do not read at all.**
- **Symbolica.** Its license grants no copying right and conditions source-availability.
  There is no algorithmic content in it that is not in the published literature, so there
  is no upside and a large downside. It is blocklisted at a **stricter** tier than the GPL
  sources, and the reason is stated here explicitly to pre-empt the backwards inference.
- Any commercial CAS source (Magma, Maple, Mathematica internals).
- **Any repository with no declared license.** No license means all rights reserved, which
  is stricter than GPL, not looser. `ederc/GroebnerBasis.jl` is a live example. Agents
  reliably get this backwards.

### The framing, stated once, for the README and for a downstream lawyer

> MIT OR Apache-2.0. Independent reimplementation informed by architectural study of the
> GPL/LGPL sources — **not "clean-room"**; that term means the authors never saw the
> original, and we do read Singular, FLINT, PARI, and msolve at the level needed to
> understand *what* they do. Algorithms and ideas are not copyrightable, and the published
> literature covers the substance. Process discipline: write Rust with the literature
> notes open, **not** the reference source tree; no copied constants, comments, or
> identifier structure; review diffs against the notes, not the sources.

This deliberately mirrors `/home/dev/projects/arrangements/DESIGN.md` §1 rather than
inventing a second, differently-worded posture for the same problem.

### The mechanical gates

Habit does not survive agent fan-out. Four gates, all with automatic verdicts:

1. **`cargo-deny`** with an explicit `[licenses] allow` list, `deny` for every copyleft
   SPDX id, running over the **published** graph (`--all-features` minus dev-only
   features), not just direct dependencies. This is what catches the `alkahest-cas` shape.
2. **A regression corpus for the gate**, containing at minimum `malachite` (LGPL hiding
   behind a permissive-looking pure-Rust crate), `polynomen` (GPL-3.0-only with an
   innocuous name), and a synthetic Apache-only crate depending on `rug`. **If the gate
   does not fail on all three, the gate is broken.**
3. **`cargo-about`** generates the attribution file; a stale attribution file fails CI.
4. **A `Derivation:` line in the module doc-comment of every non-obvious algorithm**,
   citing **both** the *paper* **and** a path into `docs/research/` — e.g.
   `//! Derivation: van Hoeij, J. Symbolic Comput. 33(5):425-445, 2002, §3;`
   `//! see docs/research/notes-van-hoeij-recombination.md §2.`

   *Amended 2026-07-31.* The original gate cited only the paper, and a paper citation is
   satisfied by pasting a reference the author never opened — which is exactly what an
   agent working from a source tree would do. That gate detects only the laziest possible
   violation, and it is *weaker than the posture it claims to mirror*:
   `/home/dev/projects/arrangements/DESIGN.md` §1 tethers its claim to committed reports.
   The Tier-B procedure below already produces the note; the gate now checks for it.
   **CI resolves the path, fails if the file does not exist, and fails if the note lacks a
   `Sources:` block carrying a tier tag per reference.** A note may serve many modules; a
   module may not exist without one.

5. **Every benchmark family carries a Tier-A citation** in its generator's metadata,
   checked by the same CI rule. Katsura, Cyclic, Eco, Noon and Reimer all have original
   papers. "Pin them to a specific generator source" — which in practice means a Singular
   `.lib`, an msolve test directory or a Groebner.jl benchmark file, all GPL-2.0 — is an
   instruction to transcribe from a copyleft test suite into an MIT repository, in the one
   lane nobody would think to look for a licensing problem. A family with no Tier-A source
   is **dropped**, not transcribed (critique-plan C16; ADR-016 §8).

Gates 1 and 2 land **before any algebra exists**. They are cheap now and expensive to
retrofit.

**Provenance cannot be deferred; paperwork can.** The standing "defer licensing work until
release prep" policy is correct, but the split is not licensing-vs-not — it is
*provenance-vs-paperwork*, and provenance is not reconstructible after the fact.

| Cannot wait (unreconstructible later) | Can wait to release prep |
|---|---|
| `cargo-deny` + the three planted cases | `cargo-about` attribution generation |
| The two-category workspace rule | SPDX headers on every file |
| The `Derivation:` → note tether (gate 4) | The README framing paragraph |
| A per-lane record of Tier-B sources consulted, written at the time | Legal review |
| A Tier-A citation per benchmark family (gate 5) | DCO/CLA, trademark, name reservation |

**One rule makes the whole thing enforceable:** the workspace has exactly two kinds of
crate — `publish = true`, gated by `cargo-deny` against the permissive allowlist, and
`publish = false` (`resolvent-oracles`, `resolvent-bench`, `resolvent-fuzz`), which may
carry LGPL dev-dependencies and shell out to GPL binaries. There is no third category and
no per-crate exception process (ADR-016).

---

## Consequences

- **Positive.** The dependency table stays tiny and the reason each entry is there is
  written down. Downstream consumers get real GPLv2 compatibility, which is the thing the
  MIT arm was for. The Tier-B procedure produces `docs/research/` notes as a side effect,
  which is exactly the artifact an agent-built codebase needs anyway.
- **Negative — performance.** GMP/FLINT-class bignum speed is unavailable (ADR-002). This
  is the single largest cost of this ADR and it is accepted, because the modular-methods
  architecture (ADR-010) keeps almost all bignum work in the sub-kbit regime where the
  permissive option is competitive or better.
- **Negative — the F4 lane may have no Tier-A reference.** R1 §8.3 could not find a
  permissively licensed F4 in any language. If that holds, F4 must be built from Faugère's
  paper plus the Macaulay-matrix literature. Feasible, slower, and it must be known before
  the lane is sized.
- **Negative — van Hoeij must be built from papers.** SymPy (BSD) explicitly lacks LLL;
  FLINT/PARI/Magma are LGPL/GPL/proprietary. Budget it as the hardest correctness lane in
  Layer 2 (R3 §0).
- **Operational.** Reading a Tier-B source *is permitted*, so an agent that has not read
  this ADR may over-restrict itself and reinvent badly. The ADR must be linked from
  `CONTRIBUTING.md` and from every lane brief that touches Layer 2.

---

## Alternatives considered and why rejected

**LGPL for the bignum layer only (i.e. depend on `malachite` or `rug`).** Rejected. LGPL
§4 conditions the permission on the recipient being able to relink the combined work
against a modified library; Rust has no stable ABI and statically links by default, so
discharging §4 for a Rust crate that `use`s LGPL types in its public API is at best
unsettled. More decisively, a library whose ℤ type is LGPL cannot honestly offer "a
permissively licensed CAS" to a downstream that wants to ship a closed or GPLv2 binary.
That is the whole product.

**MIT-only, dropping the Apache arm.** Rejected. The Apache arm carries an explicit patent
grant, which matters for a numerical/algebraic library that consumers embed. Dual-licensing
is the Rust ecosystem norm and costs nothing.

**Apache-2.0-only.** Rejected. Loses GPLv2 compatibility for downstream, which is a
material fraction of the audience for an exact-geometry / theorem-proving substrate.

**"Clean-room": forbid reading copyleft sources entirely.** Rejected as both unnecessary
and dishonest. Algorithms are not copyrightable, the substance is in refereed papers, and
pretending we never looked would be a false claim in a document a lawyer might read. The
Tier-B discipline is the honest version and it is stricter where it matters (constants and
structure) and looser where it does not (understanding).

**A per-crate license exception process.** Rejected. Exception processes are how
`alkahest-cas` happens. Two categories, no exceptions.

---

## What would reverse this

Nothing short of abandoning the product thesis. Concretely, only one scenario is coherent:
resolvent measures a performance gap against GMP that is *fatal to its first consumer*
(not merely embarrassing on a benchmark), and no permissive path closes it. Even then the
response is an **optional, non-default, never-in-CI-release `backend-gmp` feature** that
documents loudly that enabling it subjects the build to LGPL-3.0+, and the default build
stays permissive. That is a feature-flag decision, not a reversal of this ADR.

The Tier C blocklist reverses only if Symbolica relicenses permissively, at which point it
moves to Tier A. It does not move to Tier B under any circumstance, because the tiering is
about the *grant*, not about the code quality.
