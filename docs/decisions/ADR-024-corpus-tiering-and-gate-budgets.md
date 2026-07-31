# ADR-024 — Corpus tiering, gate budgets, and the sharpness ratchet

**Status:** Ratified 2026-07-31
**Reversibility:** cheap now, expensive later — tiering a corpus that already has 400
entries means re-triaging 400 entries
**Gates lanes:** H3, H5, and every lane's exit gate.
**Evidence:** `docs/research/critique-engineering.md` §12;
`docs/research/critique-plan.md` C8, C7, C18, C21;
`plans/verification.md` §3.13, §5.1, §5.3, §7.

---

## Context

Three mechanisms in the verification plan are specified in a way that guarantees they will
be abandoned or that they cannot fire at all.

**Gate 0 cannot hold its five-minute budget.** Count its executions: the determinism check
is "every regression instance run twice in-process, twice cross-process, at 1/2/8 threads,
across feature combinations" — at minimum 4 × 3 = **12 full-corpus runs per commit** — plus
the 100 % regression gate, plus self-certification assertions on every operation on every
call in tests. Several of those certificates are `~1×` or `>1×` by the plan's own cost
column: the gcd certificate is a second gcd, Sturm's count is `>1×` at high degree, the
Gröbner S-pair certificate is "≈ recomputing the basis". So Gate 0 costs roughly
`13 × corpus × (1 + certificate overhead)` against a corpus that is **contractually
append-only** and stocked with Mignotte, Swinnerton–Dyer and Hexapod instances that exist
*because* they are slow.

Month three: 400 minimized instances, Gate 0 takes 40 minutes. Someone reduces the thread
matrix to {1, 8}, then drops cross-process, then moves the corpus to Gate 1. The determinism
gate — the one the plan calls "the one that must exist from day 1 because every other
regression artifact depends on it" — is the first casualty, because it is the most expensive
and the least often red.

**No sharpness ceiling is a number.** The sharpness table is seven rows of "below a tracked
ceiling", "per-operation floors", "at the standard budget", "false-positive rate", "ratio …
tracked as a distribution". Not one number. Gate 1 then fails on "sharpness rates compared
against **committed ceilings**", and M3's exit gate requires "the Unknown-rate ceiling met".
**An exit criterion evaluated against a ceiling that does not exist is not a gate.** The
criticism is not that numbers should have been invented — inventing them is forbidden and
rightly — it is that the *mechanism for establishing them* is missing, so the gate has no
way to become real.

**"Any decline is a failure" pushes agents toward hangs.** The anti-gaming rule says a
budget-exhausted outcome inside a property test counts as a failure, "otherwise declining
everything maximizes the score". The sharpness table says the opposite: declines are
permitted, counted, and forbidden only on a designated sub-corpus. Facing the blanket rule,
the cheapest fix available is to **raise the default budget until nothing declines** — which
converts declines into long runs, i.e. into the sanctioned version of the failure the plan
calls deadliest.

**And the corpus has no provenance field.** Append-only plus a 100 % gate is right for
minimized counterexamples, whose expected outcome is "does not crash / self-certifies". It
is dangerous for hand-authored known-answer instances stored in the same place: an expected
answer that entered from a mis-triaged Class-B disagreement, or from an oracle that was
itself wrong — which the triage pipeline explicitly contemplates — becomes a permanent gate
that a **correct** future implementation fails, and can never be removed.

---

## Decision

### 1. The regression corpus is tiered on day 1, before it has entries

| Tier | When it runs | Contents | Budget |
|---|---|---|---|
| **`fast`** | Every commit (Gate 0) | Every instance by default, at 1 and 8 threads, in-process only, certificates on | **90 s wall, committed** |
| **`full`** | Every PR (Gate 1) | Everything, **complete determinism matrix**: twice in-process, twice cross-process, 1/2/8 threads, each feature combination | 25 min |
| **`slow`** | Nightly (Gate 2) | The Mignotte / Swinnerton–Dyer / Hexapod class, the narrow-field overflow sweep at three widths, extended fuzz | hours |

Three rules make the tiering hold rather than erode:

- **Instances enter `fast` by default and are *promoted out*** when they exceed a committed
  per-instance time cap. Promotion is a diff.
- **CI prints the tier census on every run** — instance counts and measured wall time per
  tier — and **fails if `fast` exceeds its budget.** So gate erosion becomes a deliberate,
  visible act instead of a silent one.
- **Self-certification is a profile flag** (`cfg(resolvent_self_check)`): on in `full` and
  `slow`, **sampled at 10 %** in `fast`. The sampling is seeded and index-addressed, so which
  10 % is deterministic and rotates with the fleet seed schedule rather than being the same
  10 % forever.

The determinism matrix moves to `full` **by design rather than by attrition**, which is the
point: it is the check that must never be quietly dropped, so it is placed where it can be
afforded and its cost is stated instead of discovered.

### 2. Every corpus entry carries provenance, and oracle-derived answers are re-derivable

```toml
provenance = "constructive-generator"   # answer known by construction
provenance = "oracle-consensus"         # systems = ["pari 2.17.4", "singular 4.4.1"]
provenance = "hand-computed"            # author, method, derivation
provenance = "minimized-counterexample" # class = "A" | "B", origin commit
```

Entries with `oracle-consensus` name the systems and versions and are **re-derived by a
nightly job that re-asks the oracles and flags drift**. Entries with `hand-computed` carry
the derivation in the file. This costs one field and it is the difference between
institutional memory and institutional debt: without it, a wrong expected answer is
indistinguishable from a right one and the append-only rule makes it permanent.

Deletion still requires a recorded justification and is still counted in CI output. What
changes is that a *provenance-driven* correction — the oracle was wrong, the hand computation
was wrong — is a legitimate justification with an audit trail, rather than an argument.

### 3. Sharpness ceilings are established by a ratchet

> Every sharpness rate is established **by measurement in the first PR that lands the API it
> guards**. That PR commits the measured value as the ceiling, rounded outward by a stated
> margin, to `sharpness-ceilings.toml`. Thereafter:
>
> - CI **fails** if a measured rate exceeds its committed ceiling.
> - **A PR may lower a ceiling freely.** Lowering is progress and needs no justification.
> - **A PR may not raise a ceiling** without a recorded justification *in the file* and a
>   line in CI output, counted the same way generator deletions are counted.
> - **A rate with no committed ceiling fails Gate 1. `TBD` is not a ceiling.**
>
> Per-operation floors stated as absolutes — gcd, resultant, factorization-product and
> isolation must be **100 % `Proved`** — are committed as `1.0` on day one and are never
> ratcheted.

That makes M3's Bernstein exit gate evaluable, makes every ceiling monotone in the right
direction, and costs one TOML file. It also composes with ADR-023's trivial-constant mutant:
the mutant proves the *certificate* rejects "always `Unknown`", and the ceiling proves the
*implementation* is not drifting toward it.

The same discipline governs budgets: **budget defaults are committed values in
`tuning-thresholds.toml`; raising one is a diff, is counted in CI output, and requires a
recorded justification.** That is what closes the gaming route the blanket decline rule was
reaching for.

### 4. Declines are classified before they are scored

Replacing the blanket rule:

> A decline is a **failure** iff (a) the instance is in the must-complete sub-corpus, or
> (b) the operation's budget was derived from a **proven** bound — Landau–Mignotte,
> Mignotte–Davenport, Hadamard, Cauchy — in which case exhaustion is impossible for a
> correct implementation and the decline is a bug (ADR-011 §4).
>
> Otherwise a decline is a **survived instance**, counted in the decline rate, which is a
> sharpness number with a committed ceiling under §3.

### 5. Five milestone exit criteria are rewritten as gates

These were vibes. Restated so a CI job can evaluate them; the numbers marked `⟨committed⟩`
are set by the first measurement and then ratcheted per §3.

| Was | Is |
|---|---|
| "The minimizer reduces a planted 20-term counterexample to **its minimal form**" | Delta-debugging yields a **1-minimal** form — no single further reduction step in the triage order preserves the disagreement — and ≤ `⟨k_i⟩` terms, within `⟨T⟩` seconds, for each of the three planted cases. (Delta-debugging does not find a global minimum and must not claim to) |
| "The score harness reports a falsification **within budget**" | Reports a falsification within `⟨B⟩` CPU-seconds at fleet version 1; reports `survived` at `⟨B⟩` once the stub is fixed; two runs at the same `(fleet_version, commit)` are byte-identical |
| "`dashu` measurement notes **committed to `docs/research/`**" | `docs/research/bignum-ladder.toml` exists and contains, for each named instance, `(dashu_ns, rug_ns, ratio)` medians of `k` runs with IQR on the pinned machine, **plus** the ADR-002 §Decision 7 verdict line evaluated against its pre-committed trigger |
| "Mignotte instances **up to the degree where Sturm remains affordable**" | Measure `d*` — the largest degree at which Sturm's median runtime on the pinned machine is ≤ `⟨T⟩` — and commit it. Below `d*` the isolation lane's verdict is CERT (Sturm-graded); **above `d*` it degrades to DIFF** and the degradation is recorded in the lane's status rather than discovered as a mysteriously slow CI job |
| "Bernstein: soundness green **and the Unknown-rate ceiling met**" | The Unknown rate is measured, committed to `sharpness-ceilings.toml` in the same PR, and is **exactly 0** on the clear-sign sub-corpus |

---

## Consequences

- **Gate 0 stays under two minutes for the life of the project**, because exceeding the
  budget is itself a CI failure and the only remedy is deliberate promotion.
- **The determinism matrix survives**, because it is placed where it is affordable rather
  than where it is aspirational.
- **Every "don't know" and "probably" outcome acquires a number on the day its API lands**,
  which is the only day the measurement is cheap and the only day nobody has to argue about
  what the number should be.
- **Cost: three TOML files and a census line in CI output.** `sharpness-ceilings.toml`,
  `tuning-thresholds.toml`, and the per-instance tier/provenance metadata.
- **A `fast` tier that samples self-certification at 10 % is weaker per commit.** Stated,
  not hidden: the per-commit signal is thinner and the per-PR signal is unchanged. The
  alternative — full certification per commit — is what produces a 40-minute Gate 0 and then
  no determinism matrix at all.
- **Ratchets create pressure to set the first ceiling loosely**, since lowering is free and
  raising is not. Mitigation: the first ceiling is the *measured* value plus a **stated**
  margin, and the margin is in the file where review can see it.

---

## Alternatives considered and why rejected

**Keep one corpus and one 5-minute Gate 0, and trust that instances stay fast.** Rejected.
The corpus is append-only by contract and its most valuable entries are slow by construction
— Mignotte and Swinnerton–Dyer are in it precisely because they separate implementations.
The arithmetic does not work at any growth rate.

**Move the whole corpus to Gate 1 and make Gate 0 build + lint only.** Tempting and cheaper,
and rejected: the per-commit signal is what makes bisection work, and a commit that breaks a
regression instance should not reach a PR queue.

**Set sharpness ceilings now, from the literature.** Rejected — it would be inventing
numbers, which the honesty rules forbid, and a ceiling derived from someone else's corpus is
wrong for ours in exactly the way a copied tuning threshold is (ADR-001 Tier B).

**Let a ceiling be raised silently when a corpus addition makes an instance harder.**
Rejected. That is the mechanism by which a sharpness gate becomes decorative. The correct
response to a harder corpus is a **re-baseline event**, labelled as such — the same
semantics the score already has for a fleet-version bump.

**Drop the provenance field and rely on the append-only rule plus review.** Rejected. The
triage pipeline explicitly contemplates "the oracle is wrong or out of range"; without
provenance there is no way to tell, later, which entries rest on an oracle and which on a
construction, and the 100 % gate makes a wrong entry permanent.

---

## What would reverse this

- **The `fast` tier's 90 s budget proving unachievable even with promotion** — i.e. the
  minimum viable per-commit corpus is itself slow. Response: raise the committed budget
  explicitly, as a diff with a justification, rather than sampling the corpus. A budget that
  is honestly 4 minutes is fine; a budget that is nominally 90 s and actually 40 minutes is
  not.
- **Ratchets producing chronic false failures** from run-to-run noise in a rate. Response:
  ceilings on *rates* are compared against the median of the last `k` runs, calibrated
  per-series exactly as the performance change-point detector is — not per-run thresholds,
  which either flap or detect nothing.
- **Provenance re-derivation flagging drift constantly** because an oracle changed a
  convention. Response: that is the mechanism working; pin oracle versions in the tier
  metadata and treat a version bump as a re-baseline event.
