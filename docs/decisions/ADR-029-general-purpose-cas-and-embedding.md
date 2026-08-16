# ADR-029 — resolvent is a general-purpose CAS, and embedding is a first-class constraint

**Status:** Proposed (2026-08-08)
**Reversibility:** costly — the scope declaration drives the crate graph, the milestone
order, and the lane-grade set. Narrowing later means deleting published surface.
**Supersedes:** the scope claims in `README.md`:34, :38-39; `DESIGN.md`:106, :151-152;
`API.md`:107, :622; `ROADMAP.md`:803; and ADR-017 §5-§6 (via ADR-033). It supersedes no
soundness rule anywhere — see §3.
**Gates lanes:** every L4 lane (X1–X4) and every lane in the analytic surface not yet
numbered.
**Evidence:** repository owner's scope decision, 2026-08-08;
`docs/research/consumer-sinbad.md` §5.1, §7 Q4; `docs/research/consumer-cadabra2.md` §11;
`docs/research/critique-engineering.md` §15 (whose premise this ADR retires).

---

## Context

The founding document set inherited two framings from the source specification and treated
both as settled:

> Symbolic calculus is a thin optional layer at the top and is not the point.
> (`README.md`:34, `DESIGN.md`:106)

> Symbolic integration, limits, series — **reject**. Out of scope permanently, not merely
> deferred. (`API.md`:622)

Three consumer evaluations were then read as confirming that bound. They do not. Each
evaluation measured **demand from one surveyed consumer**, and each says so in its own
verdict line: sinbad is `would-benefit` and touches only L4 plus one unbuilt L2 path
(`consumer-sinbad.md` §0); cadabra2 is a `strong-consumer` of L0–L3 and explicitly *hostile*
to a canonicalizing rewriter (`consumer-cadabra2.md` §11). Neither says the analytic surface
should not exist. They bound what those two consumers will call, which is a statement about
sequencing, not about scope.

Meanwhile the embedding requirement was already load-bearing and nowhere declared. Four
ratified decisions exist substantially because resolvent must sit inside a host process:

- The absolute no-panic rule, justified in `CLAUDE.md` §4 by "an embedding kernel may sit
  behind an `extern "C"` boundary where unwinding is UB".
- Caller-owned arenas with arena-relative handles that never escape into a result (ADR-020).
- Budgets counted in steps, never wall-clock (ADR-011) — the only budget model that survives
  a host's scheduler.
- Interning as an owned `Store` value, never ambient (ADR-017 §1, `consumer-sinbad.md` §5.6).

None of them cites embedding as the goal. A constraint maintained by four independent
accidents is a constraint an agent will violate on the first day it becomes inconvenient —
a thread-local interner and a process-global prime cache are both natural, both faster, and
both silently fatal to an embedded consumer.

---

## Decision

### 1. The scope is general-purpose, and the analytic surface is specified, not deferred

resolvent's scope is a complete computer algebra system. Concretely, and enumerated so the
line is checkable rather than a mood:

| Stratum | In scope | Governing decision |
|---|---|---|
| Exact algebraic core (L0–L3) | Unchanged from the founding set | ADRs 004–016, 019–028 |
| Term algebra and calculus (L4) | Hash-consed DAG, `diff`/`diff_with`, CSE, `FuncTable`, `walk_topological`, `is_polynomial_in`, canonical bytes, `rebuild_from` | ADR-017 §1 and §4 (§3's crate-edge half is superseded by ADR-033 §5) |
| Rewriting | `canonicalize`, `simplify(expr, &RuleSet)`, explicit rule sets, rational-function normal form | **ADR-033** |
| Numeric provenance | Inexact leaves carrying an exactness lattice, monotone under composition | **ADR-031** |
| Decision procedures | Zero-testing over named decidable subclasses, assumption visible in the return type | **ADR-032** |
| Analytic capabilities | Series and limits; symbolic integration; ODE solving; integral transforms; special functions; an assumptions system | scope declared here; each needs its own ADR before its lane opens |
| Presentation | Pretty-printing, LaTeX, and other emitters | scope declared here; **not** in the core crate graph (§4) |

`API.md`:622's "out of scope permanently" is retired. The capabilities on the last three rows
are **in scope and unspecified**, which is a different state from out of scope: a lane may be
opened for them, and may not start until the ADR that specifies it is ratified (ADR-021 §3).
Declaring scope is not declaring a schedule.

### 2. Embedding is a declared goal with enumerated invariants

resolvent is designed to run inside a host it does not control: another Rust workspace, a
numerical desktop application across an `extern "C"` boundary, or a WASM sandbox. The
invariants below are promoted from accident to requirement, and each gets a gate.

| Invariant | Gate |
|---|---|
| No ambient state. No `static mut`, no `thread_local!`, no lazily-initialized global cache, in any published crate | grep gate **L13** (new) |
| No panic and no unwind across any published entry point | already `CLAUDE.md` §4; unchanged |
| Every arena, store, and table is a caller-owned value | ADR-020; unchanged |
| Budgets in steps, never wall-clock; every looping entry point takes one | ADR-011; unchanged |
| No process-global allocator assumption, no environment-variable configuration in a decision path | grep gate **L14** (new) |
| No dependency on process identity, working directory, or filesystem in any published crate | grep gate **L15** (new) |

Gates **L13–L15** are cheap now and archaeology later, which is the same argument ADR-015 §5
makes about conformance vectors. They land with H1 in Wave 0. *(Numbered from 13: `DESIGN.md` §3.5 already defines L11 —
`forbid(unsafe_code)` outside the named SIMD leaf — and L12, per-crate compile-time ceilings.)*

**This is not a `no_std` commitment.** `no_std` is a separate, larger question with its own
allocator and bignum consequences; it is neither promised nor foreclosed here.

### 3. Layering discipline survives the scope growth, and it is the load-bearing rule

The scope declaration changes **what may be built**. It changes **nothing** about what may
depend on what, and nothing about soundness. Specifically, and non-negotiably:

- **The analytic surface may not leak downward.** No capability in the calculus or
  presentation strata may appear in a signature at L0–L3, and no L0–L3 algorithm may call
  one. The dependency direction is the whole architecture (ADR-005).
- **Every soundness rule in `CLAUDE.md` §4 stands unamended**: no tolerance parameter at any
  layer under any name; no silent approximation; no floating point in a decision path
  (ADR-012 §6); no "probably correct" mode invisible in the type; fail at construction, not
  at query.
- **A general-purpose CAS is where those rules pay off**, not where they are relaxed. The
  systems this scope now competes with are precisely the ones that lose soundness in the
  analytic surface — heuristic simplification, numeric zero-testing, silent branch-cut
  choices. Matching their capability list while refusing their failure modes is the thesis.

### 4. The crate graph

The analytic surface does not belong in `resolvent-expr`. **ADR-005 was amended in the same
commit as this ADR** and now carries the extended graph: `resolvent-calculus` (L5 — series,
integration, ODE, transforms, special functions, and ADR-032's zero-test tiers; depends on
`expr`, `algebra`, `real`) and `resolvent-display` (a leaf).

One placement in that amendment is not obvious and is the reason it could not be deferred:
**ADR-032's zero-test tiers go in `-calculus`, not `-expr`.** A Tier-1(b) reduction produces an
`AlgebraicReal`, so siting it in `-expr` would give L4 an edge to L3 and undo ADR-017 §3's
separation — which exists so the expression trunk can never hold the algebraic-number lane
hostage. A code emitter for a consumer's target language stays out of the graph entirely, for
ADR-017 §1's unchanged reason.

### 5. Sequencing: L4 is no longer last

ADR-017's consequence "L4 blocks nothing and is blocked by nothing — sequence it last and do
not let it block anything" was correct while L4 was a thin optional layer. It is now the
foundation of a stratum, and the deferral no longer holds. Two facts survive the change and
should govern the resequencing:

- The exact core is still what everything above inherits, and its one-way doors are still
  one-way. **No analytic lane may start before M1's representation freeze.**
- M4 (elimination) is still the release that unlocks the strongest evidenced consumer
  (`consumer-cadabra2.md` §3). Scope growth is not a reason to deprioritize it.

`ROADMAP.md` §1's milestone graph and §2's wave table require a rewrite against this ADR.
That rewrite is the largest downstream item and is not attempted here.

---

## Consequences

- **The certificate table grows substantially.** `CLAUDE.md` §1 currently lists ~50 rows. The
  analytic surface adds more, and the honest news is that most of them have *strong*
  inverse-operation certificates: integration is graded by differentiating the result and
  comparing; ODE by substituting back; transforms by inverting; series by a truncation bound
  plus agreement with direct evaluation; special functions by their functional equations and
  recurrences. The prime directive extends into analysis better than the "thin layer" framing
  assumed. The genuine residue — simplification *quality*, printing, assumption inference —
  is what ADR-030 exists for.
- **A third lane grade is now required, not optional.** Without it an agent facing a
  quality-graded capability either invents a certificate that certifies nothing or stalls.
  ADR-030.
- **`API.md` §4 scope tables are wrong and must be rewritten**, specifically :107, :585
  (L4-10) and :622. `API.md` is canonical for public signatures (`CLAUDE.md` §0), so this is
  a defect until fixed.
- **Contradiction-register item 12 reopens** (ADR-021:155). It was closed as "Out. v1 is what
  M7's exit gate tests." Its premise is retired; ADR-033 closes it the other way.
- **The three consumer evaluations remain valid as demand measurements** and invalid as scope
  bounds. Their verdict lines should be read as what they say.
- **Risk: scope declared, capability not delivered.** A README claiming a general-purpose CAS
  over an empty workspace is worse than the narrow claim it replaced. Mitigation: `README.md`
  §Status already states plainly that no implementation exists; that line becomes more
  important, not less.

---

## Alternatives considered and why rejected

**Keep engine-first scope; treat general-purpose as a destination without specifying it.**
Rejected by the owner, 2026-08-08. It is the cheapest option and it leaves the two live
questions — rewriting and zero-testing — unsettled, which is what produced the
three-documents-three-answers defect that critique-engineering §15 found.

**Declare general-purpose scope but keep L4 sequenced last.** Rejected as incoherent: a
stratum cannot be the foundation of the scope and also the thing that blocks nothing and is
built if there is time.

**Fold embedding into ADR-020 as an amendment rather than declaring it here.** Rejected.
ADR-020 covers arena ownership, which is one of four invariants. An amendment to one of them
would leave the other three uncited, which is the current failure mode.

**Commit to `no_std` as part of the embedding declaration.** Rejected as premature. It is a
larger decision with allocator and bignum consequences (`dashu` allocates), it has no
surveyed consumer asking for it, and declaring it now would foreclose choices in
`resolvent-int` for no present benefit.

---

## What would reverse this

- **The certificate discipline proving unsustainable on the analytic surface**, measured
  rather than felt: if the fraction of analytic-surface operations that can only be
  conformance-graded exceeds a committed ceiling, the scope claim outruns the verification
  model and one of the two must give. Set the ceiling in `sharpness-ceilings.toml` when the
  first analytic lane lands (ADR-030 §4).
- **A soundness rule in `CLAUDE.md` §4 being proposed for relaxation to reach a capability.**
  That is the signal this ADR was read as permission rather than as scope. The correct
  response is to drop the capability, and to say so in the ADR that proposed it.
- **The extended crate graph failing to keep the analytic surface out of L0–L3 signatures.**
  §3's leak rule is the one thing here that is genuinely one-way; if it cannot be held
  mechanically, the scope should narrow rather than the rule.
