# ADR-030 — the `conformance` lane grade, for capabilities with no self-certificate

**Status:** Proposed (2026-08-08)
**Reversibility:** cheap — adding a grade is additive; the risk is in what it is allowed to
gate, which §3 bounds.
**Gates lanes:** **X5q** (the first conformance lane, Wave 2), plus every lane in ADR-029 §1's
analytic and presentation strata that cannot be certificate-graded.
**Evidence:** ADR-029 §1; `CLAUDE.md` §1 (prime directive) and §7 (verifying honestly);
`ROADMAP.md` §0 lane grades; ADR-023 (mutant sets); ADR-016 §5 (calibration corpora);
ADR-017's own admission that L4 rewrite *quality* "is a number with no certificate".

---

## Context

`CLAUDE.md`'s prime directive is that nothing merges without its certificate green, and
`ROADMAP.md` §0 offers four lane grades: `certificate`, `score`, `measurement`, `decision`.
That set was complete for an exact algebraic engine, where every operation either emits a
proof of its own answer or is a number to optimize.

ADR-029 declares a general-purpose scope, and most of the new surface still fits: symbolic
integration is graded by differentiating the result and comparing to the integrand; ODE
solutions by substitution; transforms by inversion; series by a truncation bound plus
agreement with direct evaluation; special functions by their functional equations and
recurrences. These are genuine self-certificates in the ADR-023 sense — they do not invoke
the operation they certify, and each has an obvious mutant family.

A residue does not fit, and it is the residue where every established CAS is weakest:

- **Simplification quality.** "Is this expression *better*" has no verdict function. ADR-017
  already says so: rewrite *soundness* is property-testable, but "did simplification produce
  a good result is a number with no certificate."
- **Presentation.** Whether a printed form is correct is checkable by round-trip; whether it
  is *readable* is not.
- **Assumption inference.** Whether a derived assumption set is sound is checkable; whether
  it is as strong as it could be is not.

Without a grade for these, an agent facing one of them does one of two things, and both are
worse than the problem: it invents a certificate that certifies nothing — the exact failure
ADR-023 exists to catch, and the one `CLAUDE.md` §1 calls "a certificate that accepts
everything" — or it stalls, because the prime directive forbids merging and nothing it can
write will satisfy it.

---

## Decision

### 1. `conformance-graded` is a fourth lane grade

| Grade | Definition | Convergence | Fan-out |
|---|---|---|---|
| **`conformance-graded`** | The primary verdict is **external differential agreement** against a declared oracle tier, at a committed rate, on a committed corpus. There is no self-certificate, and the lane brief says so in those words. | Weeks. Monotone in the rate, not in the capability. | One or two agents. The corpus is shared state. |

Its entry in `lanes.toml` carries three fields the other grades do not:

```toml
[lane.X5q]
grade = "conformance"
self_certifying = false          # REQUIRED and REQUIRED to be false. A conformance lane
                                 # that claims a self-certificate is miscategorized.
oracle = ["X5"]                  # LANE IDS, as for every other grade — see the note below.
oracle_systems = ["sympy"]       # EXTERNAL systems. Non-empty: a conformance lane with no
                                 # external oracle has no verdict at all.
divergence_ceiling = "rewrite.quality.divergence"   # key into sharpness-ceilings.toml.
                                 # `TBD` fails Gate 1 (ADR-024 §3).
```

**`oracle` and `oracle_systems` are different fields on purpose.** ADR-021's CI rule 2
resolves every entry in `oracle` against `[lane.*]` and requires it green and frozen; an
external system name in that field does not resolve. ADR-021 §3 is amended in the same commit
to carry both keys and the three checks over them.

### 2. Nothing on the default path has conformance-graded soundness

Class-S rules like `sin²+cos²→1` and class-D rules like `log(ab)→log a+log b` have **no**
automatic soundness verdict — no inverse operation checks them, so their grade is conformance.
A blanket "soundness is never conformance-graded" would therefore be false of this package on
its first lane. The rule that keeps the grade from becoming an escape hatch is narrower, and it
binds harder:

**(a) Where a self-certificate exists, soundness is certificate-graded. Always.** Choosing
oracle comparison over an available inverse-operation check is a review defect, and the
reviewer's question is "what is the inverse operation?"

**(b) Where no self-certificate exists, the capability may not be reachable on the default
path, and its use must be visible in the return type.** Conformance-graded soundness is
admissible only for something a caller opts into by name and receives a non-`Proved` answer
from.

ADR-033 is the worked case and it satisfies (b) exactly:

| | Verdict | Grade | Default-reachable? |
|---|---|---|---|
| **Class-R rule soundness** | GF(p) evaluation with `Apply` nodes as free variables, across the fleet seed schedule | `certificate` | Yes — `RuleSet::ring_identities()` is R-only |
| **Class-S / class-D rule soundness** | Committed justification + differential testing; class D additionally needs a discharged side condition | `conformance` | **No.** There is no default rule set; a caller names any set containing S or D members, and `simplify` then returns `Probable` with the firing rules named, never `Proved` |
| **Rewrite quality** (any class) | Node count / expected form against an oracle, under a divergence ceiling | `conformance` | n/a — gates nothing |

So a conformance-graded soundness argument never silently backs a `Proved` result.

A lane whose soundness is conformance-graded **and** whose capability is default-reachable is
not ready to be briefed. That combination is the escape hatch, and it is what §3 forbids.

### 3. What a conformance lane may not do

- **It may not gate a certificate lane.** Dependency edges run one way. A certificate lane
  blocked on a conformance lane inherits an unverdicted dependency, which is founding
  constraint #3 in reverse.
- **It may not be an oracle for anything.** `CLAUDE.md` §1's "build the oracle side first"
  requires the oracle to be the *stronger* implementation. A conformance-graded artifact is
  by definition the one being graded.
- **It may not report a pass when its oracle is absent.** `CLAUDE.md` §7's counted-`SKIP`
  rule applies with full force: a conformance lane with no oracle installed has produced *no
  evidence at all*, not weak evidence. Its CI job fails.
- **Its divergence rate is a tracked number with a committed ceiling**, on the ADR-024
  ratchet: lowering free, raising counted and requiring a recorded justification, `TBD` is
  not a ceiling.

### 4. The mutant-set analogue: a planted-divergence corpus

ADR-023 requires every certificate to be observed rejecting a wrong implementation. The same
argument applies here and the mechanism transfers: **every conformance lane ships a
planted-divergence corpus** — instances where resolvent's answer is deliberately wrong in a
plausible way, with a test asserting the differential harness *reports the divergence*.

Without it, a conformance lane whose adapter silently normalizes both sides into agreement is
green forever, which is the same failure as a certificate that accepts everything and is
harder to see. Draw plants from the family the capability actually has: a dropped branch
cut, an off-by-one series truncation, a sign error under an odd-order transform, a variable
ordering swap.

This composes with ADR-016 §5's calibration corpus and does not replace it. Calibration
asserts *the oracle* answers known instances correctly; planted divergence asserts *the
harness* notices when resolvent does not.

### 5. Triage inherits ADR-016's classifier unchanged

A conformance disagreement runs through the existing Class A / Class B classifier: re-run
resolvent's own soundness certificate on the disagreeing instance. Fails too → resolvent bug,
certain. Passes → normalization, convention, or oracle limitation. The classifier is what
keeps a conformance lane from generating triage work that looks like bugs, and it already
exists for exactly this reason (`NEXT.md` Day 4).

---

## Consequences

- **`ROADMAP.md` §0's grade table gains a row**, and `lanes.toml`'s schema gains three fields.
  Both are mechanical.
- **The prime directive survives ADR-029 intact.** "Nothing merges without its certificate
  green" becomes "nothing merges without its verdict function green, and for a conformance
  lane the verdict function is external, declared, rate-ceilinged, and observed catching a
  planted divergence." That is weaker than a self-certificate and it is stated as weaker,
  which is the point — `CLAUDE.md` §1's own ranking of self-certificates over external
  oracles is preserved rather than blurred.
- **Oracle dependency becomes structural rather than convenient.** Today sympy is Tier 0 and
  the only oracle installed on this machine. A conformance-graded stratum makes the oracle
  set a first-class dependency of the roadmap, which strengthens the case for installing more
  of them (PARI, Maxima) before those lanes open, and makes the skip census load-bearing.
- **The grade is a pressure valve and will be abused.** The counter-pressure is §2 and §3:
  soundness is never conformance-graded, and a conformance lane gates nothing. If a lane
  brief proposes a conformance grade for something with an available self-certificate, that
  is a review defect, and the reviewer's question is "what is the inverse operation?"

---

## Alternatives considered and why rejected

**Extend `score-graded` to cover these.** Rejected. A score lane optimizes a number against a
frozen baseline and has no completion condition; a conformance lane converges on agreement
with an external reference and does terminate. Conflating them would put "how good is our
simplifier" on the same footing as "how fast is `Fp::mul`", and would inherit score's
"do not fan out" rule for no reason.

**Grade quality capabilities as `measurement`.** Rejected. A measurement lane commits a
number once and terminates. Conformance is a standing gate that runs every round —
`corpus-atlas-findings.md` §70's lesson in another workspace was precisely that per-round
controls catch what one-time controls do not.

**Refuse the capabilities that cannot be self-certified.** This is the founding set's
position and it is coherent — it is what "not a general-purpose CAS" bought. Rejected by
ADR-029. Worth recording that the cost of ADR-029 is exactly this: a stratum whose primary
verdict is an external oracle rather than a proof.

**Allow a conformance lane to gate a certificate lane when the dependency is "obviously
fine".** Rejected. There is no mechanical form of "obviously fine", and the gate exists
because founding constraint #3 requires every lane to have a verdict function that is at
least as strong as what depends on it.

---

## What would reverse this

- **The divergence ceilings ratcheting upward across two consecutive releases.** That is
  evidence the conformance stratum is not converging, and the response is to narrow scope
  (ADR-029's reversal trigger), not to raise the ceiling again.
- **A conformance lane being found to gate a certificate lane in practice**, through a
  transitive edge `lanes.toml` did not model. Response: model the edge, and treat the
  discovery as evidence the lane graph needs the same mechanical check the ratification gate
  already has.
- **Self-certificates turning out to exist for the residue.** If simplification quality gains
  a real verdict function — a normal-form theorem for a restricted class, say — that lane
  moves to `certificate` and this grade shrinks toward presentation only. That would be a
  good outcome and costs nothing to adopt.
