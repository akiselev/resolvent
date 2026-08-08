# ADR-033 — rewriting and simplification: explicit, classified by soundness argument, never implicit

**Status:** Proposed (2026-08-08)
**Reversibility:** cheap for the surface; **one-way** for §2's never-implicit rule, which is a
promise consumers build certificate tethers on top of.
**Supersedes:** ADR-017 §5 ("No `simplify()`, in v1, at all") and the first three rows of
ADR-017 §6's deferral table. Reopens and closes the other way
contradiction-register item 12 (ADR-021:155). ADR-017 §1, §3 and **§4 survive** — §4 verbatim.
**Gates lanes:** **X5** (rule soundness, certificate) and **X5q** (rewrite quality,
conformance). Not X2 or X4.
**Evidence:** ADR-029 §1 (the scope declaration that retires this deferral's premise);
`docs/research/critique-engineering.md` §15 (whose premise expired);
`docs/research/consumer-cadabra2.md` §11 and `API.md`:585 (the `Cos2` tether);
`docs/research/consumer-sinbad.md` §5.3; ADR-030 (the grade this needs).

---

## Context

ADR-017 §5 removed `simplify` entirely, on an argument that was correct at the time and
turned on one premise:

> …it is still a rewriting engine with a rule language **in the layer the spec calls "not the
> point"**, for zero consumers…

ADR-029 declares that layer to be the point. The premise is retired, so the deferral is
retired with it. Two things do **not** follow, and confusing either with the scope change is
how a CAS becomes unpredictable:

1. **Critique-engineering §15 was not wrong.** Its finding was a *coherence* defect — three
   documents giving three answers about whether `simplify` existed — and the fix it demanded
   (one answer, stated in one place) is what this ADR provides. Scope moved; the finding
   holds.
2. **The consumer constraint that drove the refusal is code-resident and survives any scope
   change.** cadabra2 keeps `Cos2` as a first-class atom *deliberately*, because a
   canonicalizing rewriter destroys the certificate tether that admits resolvent to its
   trusted computing base (`DESIGN.md`:151, `API.md`:585). A strong consumer is actively
   hostile to implicit rewriting. That is not a preference to be traded against convenience.

The remaining question — and the one this ADR exists to answer — is **what makes a rewrite
rule sound**, because "the rules are an argument" answers the unpredictability objection and
not the correctness one. A rule set is a set of claims, and most CAS unsoundness lives in
claims that are true on a domain and applied off it.

---

## Decision

### 1. Rewriting ships. Rules are always explicit and always passed by name.

```rust
impl Store {
    /// Value-preserving normalization. Explicit, opt-in, returns a NEW node.
    /// ADR-017 §4, unchanged.
    pub fn canonicalize(&mut self, e: ExprId) -> Result<ExprId>;

    /// Rewriting under an explicit rule set. There is no argument-free `simplify`,
    /// and no rule set is ever applied implicitly.
    pub fn simplify(&mut self, e: ExprId, rules: &RuleSet, budget: Budget)
        -> Result<Certified<ExprId>, Error>;
}
```

*The error type is `Error`, not `Decline`.* ADR-011's model is one error type with
`is_decline()` distinguishing a recoverable decline (budget exhausted — the caller may retry
or climb a rung) from a fault. A draft of this ADR wrote `Result<_, Decline>`, which would have
made budget exhaustion the *only* expressible failure and left no channel for a malformed rule
set — the distinction `API.md` X-1 requires.

There is no `simplify(expr)`. A caller names the rule set every time. `RuleSet::default()`
does not exist, because a default rule set is an implicit rule set with an extra step.

### 2. Never implicit. This is the one-way half.

**Construction hash-conses and constant-folds — and folds only when every operand is `Exact`,
per ADR-031 §6 — and stops.** No rewriting, no canonicalization, no normalization happens as a side effect of
building a node, of `diff`, of `walk_topological`, of serialization, or of any future entry
point.

The invariant consumers may rely on, stated so it is testable:

> **A consumer that never calls `canonicalize` or `simplify` never has its terms rewritten.**

That is what keeps cadabra2's `Cos2` tether intact, and it is why this half is one-way:
consumers build certificates on top of the promise, and breaking it invalidates their proofs
rather than merely surprising them. Certificate:
`rewriting::construction_is_not_rewriting` — build a corpus of terms whose canonical form
differs from their constructed form, round-trip each through construction, `diff`,
`walk_topological` and `canonical_bytes`, and assert structural identity is preserved.

### 3. Every rule carries its soundness argument, and the argument determines its grade

This is the substance of the ADR. A `RuleSet` is a set of claims; each claim is classified,
and the classification decides how it is verified and whether it may be default-reachable.

| Class | What it claims | Verdict | Grade |
|---|---|---|---|
| **R — ring identity** | True with every `Apply(f, …)` treated as a **free** symbol, e.g. `f(x) + f(x) → 2·f(x)`, `x·(y+z) → x·y + x·z` | Replace each distinct `Apply` node by a fresh variable; assert the two sides agree by evaluation at random points over GF(p), **across the fleet seed schedule** (ADR-023 §3). A genuine automatic verdict | `certificate` |
| **S — semantic** | Depends on what a `FuncId` *means*, e.g. `sin²(x) + cos²(x) → 1`, `exp(a)·exp(b) → exp(a+b)` | No automatic verdict exists. Requires a committed `justification` citing the identity, plus differential testing against an oracle | `conformance` (ADR-030) |
| **D — domain-restricted** | Semantic **and false somewhere**, e.g. `√(x²) → x`, `log(a·b) → log a + log b` | Requires a machine-checkable `side_condition`. Applying the rule without discharging the condition is a **bug**, not a heuristic | `conformance`, and see below |

Three consequences, and they are the ones that keep this from becoming an ordinary CAS
simplifier:

- **Class D rules may not be applied on an undischarged condition, ever.** No "assume the
  principal branch", no "assume positive". If the condition cannot be discharged from what
  is known, the rule does not fire. That is where `√(x²) = x` and every branch-cut bug comes
  from, in every system.
- **A rule set containing any Class S or D member says so in its type**, and
  `simplify` returns `Certified<ExprId>` whose `Certainty` is `Proved` only when every rule
  that actually fired was Class R. Otherwise it is `Probable` with the firing rules named.
  Soundness stays visible in the return type, per `CLAUDE.md` §4.
- **`RuleSet::ring_identities()` is Class R only**, is fully certificate-graded, and is the
  set a caller who wants no surprises names.

**Two classification cases that will be got wrong, decided here so they are not re-argued:**

- **`x · x⁻¹ → 1` is class D, not class R.** `Pow(e, -1)` is division, division is not a ring
  operation, and the identity is false at `x = 0`. The side condition is `x ≠ 0`, and
  discharging it is exactly what §5's assumptions exist for. Anything whose truth needs a
  non-vanishing denominator is D. The test: if the claim is false for *some* substitution into
  a *field*, it is not a ring identity, however familiar it looks.
- **A class-R check that hits a zero denominator mod `p` resamples; it does not pass.** GF(p)
  evaluation of a rule containing `Pow(e, k)` with `k < 0` can divide by zero. The harness
  draws a fresh point from the seed schedule and retries, bounded. **Exhausting the retry
  bound is a failure, not a skip** — a denominator that vanishes at every sampled point is
  evidence the denominator is identically zero, which means the rule is not class R and was
  misclassified. Reporting it as a pass is the silent-green failure `CLAUDE.md` §7 forbids.

### 4. What each half is graded by

Per ADR-030 §2, the rewriting work splits into two lanes, and the grades follow the class:

- **Rule soundness** — class R by GF(p) evaluation (`certificate`, lane X5); classes S and D
  by committed justification plus differential testing (`conformance`). ADR-030 §2(b) is what
  makes the second admissible: S and D are **not default-reachable** — there is no default rule
  set, and any set containing one makes `simplify` return `Probable`, never `Proved`.
- **Rewrite quality** — does `simplify` reach the form a human expects, measured against an
  oracle on a committed corpus, with a divergence ceiling (`conformance`, lane X5q). Never
  gates anything.

"Did simplification produce a *good* result is a number with no certificate" — ADR-017's own
sentence — is now a grade rather than a reason to refuse the capability.

### 5. Rational functions return to scope, and the `resolvent-algebra` edge with them

ADR-017 §3 dropped `resolvent-expr → resolvent-algebra` because its sole justification was
rational-function normalization, which was out of scope with no consumer. ADR-029 §1 puts
rational functions in scope, so the justification is live again and **the edge returns**.
`RatFunc` and its normal form (gcd cancellation, certified by the existing L2 gcd
certificate) are in scope and unspecified — a lane may open for them once their ADR is
ratified.

### 6. e-graphs: unchanged

`egg` / `egglog` adapters remain **post-v1 and outside this repository**, for exactly
ADR-017's reasons, none of which were about scope: `egg`'s `Language` trait wants to own the
term representation, its maintainers point at the successor, and `egglog` 2.0 churns.
The structural-encoding promise — `walk_topological` + `NodeRef` + canonical bytes — is what
makes an external adapter writable, and it is retained verbatim.

ADR-017's recommendation stands and is worth repeating: **write the `egg` adapter early and
throwaway, as a design test of `RuleSet`**, before `RuleSet` has users. If the adapter is
awkward, the type is wrong, and fixing it costs nothing today.

---

## Consequences

- **`RuleSet` is a public type with a classification in it**, which means adding a rule is a
  reviewable act with a declared soundness argument rather than a line in a table.
- **`API.md`:585 (L4-10, "a general `simplify()` — out of scope") is wrong** and must be
  rewritten. `API.md` is canonical for public signatures, so this is a defect until fixed.
- **ADR-021's contradiction-register item 12 closes the other way**, and the register row
  should say so rather than being deleted.
- **cadabra2 is unaffected by construction.** It never calls a rewriting entry point, so §2
  guarantees its terms are untouched. The `Cos2` tether survives without a special case, which
  is the test of whether §2 was specified correctly.
- **Class D is where this design will be pushed hardest.** Every consumer who wants
  `√(x²) → x` will have a reason why their case is fine. The answer is the side condition, and
  the cost of holding it is that resolvent will simplify *less* than established systems on
  some inputs. That is the trade being made deliberately.
- **Risk: Class S grows into a grab-bag.** A semantic rule is cheap to add and its verdict is
  an oracle comparison. Counter-pressure: each carries a committed `justification` and lands
  with corpus instances; a Class S rule with no justification is a review defect.

---

## Alternatives considered and why rejected

**Keep ADR-017 §5 — no `simplify` at all.** Rejected by ADR-029's scope declaration. Recorded
because its argument was sound on its premise, and because the premise, not the reasoning,
is what changed.

**Ship `simplify(expr)` with a built-in default rule set, like every other CAS.** Rejected.
An implicit default is what makes CAS simplification unpredictable, and it would break §2 for
every consumer at once. Naming the rule set costs one argument at the call site.

**Ship rules without the R/S/D classification, and verify all of them by differential testing.**
Rejected. It throws away a real automatic verdict for the class that has one, and — worse — it
puts domain-restricted rules and universally-valid ones on the same footing, which is exactly
how `log(a·b) → log a + log b` ships enabled by default and produces wrong answers on the
negative reals in some system every few years.

**Allow Class D rules to fire under a documented assumption.** Rejected. A documented
assumption is a string in a doc comment; the value returned carries no trace of it, and the
consumer's fail-closed path cannot match on it (ADR-011 §3). If the assumption is real it is a
side condition and it is checkable.

**Make rewrite quality certificate-graded by defining a normal form.** Considered, and it is
the right answer where a normal form exists — rational functions have one, and their lane
should be certificate-graded for exactly that reason. Rejected as a general approach because
no normal form exists for the mixed transcendental class.

---

## What would reverse this

- **§2 being violated once in released code.** That is a consumer-visible correctness
  incident, not a bug, because consumers build proofs on the promise. The response is a
  regression test in the tether's shape, not a softening of the rule.
- **Class R turning out to be nearly empty** on real rule sets — i.e. almost every useful rule
  is semantic. That would mean the classification buys little and the whole surface is
  conformance-graded, which is ADR-030's own reversal trigger and a signal the scope claim
  outran the verification model.
- **A normal form appearing for a class currently graded by conformance.** That lane moves to
  `certificate` and this ADR's §4 shrinks. A good outcome, additive to adopt.
- **`egglog` stabilizing with a consumer demonstrating a simplification `RuleSet` cannot
  reach.** Response is unchanged from ADR-017: ship the adapter as a non-default feature
  outside this repository. That is the plan, not a reversal.
