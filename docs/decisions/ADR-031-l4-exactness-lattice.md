# ADR-031 — L4 carries inexact leaves under a monotone exactness lattice

**Status:** Proposed (2026-08-08)
**Reversibility:** **one-way.** The lattice is a field on every L4 node, an input to the
digest split in §5, and a precondition on the L4→L1 bridge. Retrofitting it under a working
`Store` is a rewrite of `resolvent-expr`, not a refactor.
**Supersedes:** `README.md`:48-50's "Not numeric" bullet, in the half that claims the only
`f64` in the library is an outward-correct enclosure returned to callers. It supersedes
**none** of ADR-012 §6 — see §2.
**Gates lanes:** X1, X3, X4, and every consumer adapter that content-addresses an L4 artifact.
**Evidence:** repository owner's decision, 2026-08-08;
`docs/research/consumer-sinbad.md` §3 D1/D5/D6, §5.5;
`docs/research/consumer-cadabra2.md` §4.5, :349-350; ADR-015; ADR-012 §6, §9;
`CLAUDE.md` §3.2 (canonical-bytes exclusions).

---

## Context

`README.md`:48 states the anti-goal as one bullet doing two jobs:

> **Not numeric.** No floating-point in any decision path. The only `f64` in the library is
> an outward-correct enclosure returned *to* callers, and it is never a decision input.

The first sentence is a soundness rule. The second is a scope claim, and it is strictly
stronger — it forbids `f64` from *existing* in the library except as an output. The two are
routinely read as one rule, so relaxing the scope claim reads as relaxing soundness.

The surveyed consumers want the scope claim relaxed and the soundness rule kept, and both
already implement the relaxation themselves because resolvent does not:

- **sinbad** grades every result on a lattice (`tiered-core`: `Proven` / `Measured` /
  `Estimated`) with the rule that "unaccounted error-budget sources cap the effective grade —
  a partial bound cannot masquerade as a total one" (`consumer-sinbad.md` §3 D5). That rule
  *is* a monotonicity law on a lattice, written in prose in another workspace.
- **cadabra2** derives residual sup-norm bounds whose "residual evidence in this file is
  `f64`-computed and self-measured from the same `f64` that produced" the residual
  (`consumer-cadabra2.md`:349-350) — a self-referential bound it flags as a weakness, and
  exactly what a tracked lattice fixes.

The demand, stated precisely: an expression DAG in which some leaves are exact and some are
not, where **any node can be asked what it inherited**, and where inexactness cannot be
laundered into exactness by any operation. That is not a tolerance and not an approximation
mode. It is the strongest available enforcement of "no floating point in a decision path",
because it replaces a rule agents must remember with a property the type system carries.

---

## Decision

### 1. `Exactness` is a three-element lattice on every L4 node

```rust
pub enum Exactness {
    /// Every leaf below this node is an exact L0 element or a symbol.
    /// This is the only state L0–L3 will accept.
    Exact,
    /// Not exact, but the leaf that introduced the inexactness carries a
    /// rigorous outward enclosure supplied by the caller.
    /// Endpoints are Rationals — NOT an interval type (ADR-015 §1 stands).
    Enclosed,
    /// Inexact with no rigorous bound: a measurement, a fitted coefficient,
    /// a truncation with no error term. No decision may rest on it, ever.
    Approximate,
}
```

Order: `Exact > Enclosed > Approximate`. Meet is the greatest lower bound.

Inexact leaves are constructed explicitly and never by coercion — there is no
`impl From<f64> for Expr`, because a silent lift is how an approximate value enters a
computation nobody meant to make approximate:

```rust
impl Store {
    pub fn approx(&mut self, v: f64) -> Result<ExprId, Error>;              // Err on NaN/±inf
    pub fn enclosed(&mut self, lo: Rational, hi: Rational) -> Result<ExprId, Error>; // Err if lo > hi
}
```

Both record the leaf as its own witness (§4). `Store::constant(r: Rational)` stays the only
way to build an `Exact` leaf, and there is no method anywhere that raises a node's exactness.

`Enclosed` carries its bound as **two `Rational`s**, in the same vocabulary as
`sign_over(p, lo, hi)`. There is no new interval type, no new enclosure semantics, and no
second implementation to reconcile at an adapter boundary — **ADR-015 is unamended**, and its
committed conformance-vector file (§5) becomes the shared spec for the outward `f64`
conversion rather than a nicety.

**Bounds live on leaves only, and resolvent does not propagate them.** This is the
load-bearing restriction. An interior node's `Exactness` is the meet of its children's — a
three-element lattice operation, trivial and exact. Its **bound is not computed**, because
computing one is interval arithmetic, and ADR-015 §Context is explicit that resolvent "has no
filtered-arithmetic layer of its own and does not want one, because filtering *arithmetic* is
an orthogonal axis to filtering *algebra*." An interior `Enclosed` node answering a sign query
when its enclosure excludes zero is a filter API — the one ADR-015 §Alternatives rejects by
name (`try_cmp_f64`) — and it would make resolvent own an enclosure-propagation semantics at
exactly the boundary ADR-015 exists to keep clean.

So, precisely:

- **No resolvent API decides anything from an `Enclosed` or `Approximate` node.** Sign queries
  over either return `Unknown`; there is no width at which they return `Certain`. `Exactness`
  gates the L4→L1 bridge and describes provenance. It never enables a decision.
- **A consumer that wants propagated bounds computes them itself**, from the leaf bounds this
  lattice exposes plus its own interval type — which is ADR-015's "filtering arithmetic is the
  consumer's axis" applied unchanged, and which both surveyed consumers already have.
- **`Enclosed` versus `Approximate` therefore distinguishes *data the consumer can use*, not
  *decisions resolvent will make*.** That is still exactly the distinction sinbad's grade
  lattice needs (`Measured` versus `Estimated`), which is the demand this ADR serves.

### 2. ADR-012 §6 is unamended. This is what enforces it.

"No floating point in any decision path" stands verbatim. What changes is the enforcement
mechanism:

- **The L4→L1 bridge refuses anything not `Exact`.** `is_polynomial_in` keeps its ratified
  `-> Option<MPoly>` signature (`API.md` L4-5 — "a coercion would lie; a predicate cannot")
  and returns `None` for any subtree carrying `Enclosed` or `Approximate`. The *witness*
  accompanying a `None` — which M7's exit gate already requires — names the offending node and
  its exactness, so the refusal is diagnosable without changing the return type. **No `Result` and no new
  error variant on this function** — `API.md` L4-5, `ROADMAP.md` M7 and `DESIGN.md`:1750 all
  state the signature as `Option`, and it stays that way.
- **No inexact value can reach `UPoly`, `MPoly`, `AlgebraicReal`, or `SqrtExt`,** because the
  only door is closed by construction rather than by review.
- **Neither `Enclosed` nor `Approximate` may produce a `Sign`.** Both return `Unknown`. See
  §1's bounds-live-on-leaves rule for why there is no width at which `Enclosed` answers
  `Certain`.
- **There is still no tolerance parameter, anywhere, under any name.** A leaf's enclosure width
  is not a tolerance: it never converts an `Unknown` into a `Certain` — under this ADR nothing
  does — and that is property-tested as idempotence under refinement (`CLAUDE.md` §4).

### 3. Exactness is monotone under composition, and that is the certificate

The one law, and the reason the design is worth having:

> **`exactness(n) ≤ meet over children of exactness(child)`.**
> Exactness is never gained by combining. An operation may lose more than the meet — a
> truncated series over exact inputs is `Enclosed` at best — but no operation may return a
> node more exact than its least exact input.

This is sinbad's D5 ("a partial bound cannot masquerade as a total one") stated as an
algebraic property, which makes the adapter a three-arm match:
`Exact → Proven`, `Enclosed → Measured`, `Approximate → Estimated`.

**The law is absolute, including where it is over-conservative.** The case that tests it:
`diff(Approximate(0.1), x)` is mathematically *exactly* zero — the derivative of any constant
is zero regardless of how well that constant is known. Marking the result `Exact` would be
mathematically defensible and is **forbidden**. The result is `Approximate(0)`: zero, and
tainted.

The reason is that the alternative has no mechanical form. "This operation provably discards
its inexact input" is an analytic claim about each operation, it must be re-litigated per
operation, and *it is exactly the claim a promotion mutant makes about itself*. A law with an
exception for operations that assert they do not need it is not a law. The cost is real and
accepted: an expression can carry `Approximate` further than strict necessity requires, and a
consumer reading the witness set can see precisely why. Over-conservative is the correct
direction of error for a fail-closed library — it produces a refusal, never a wrong `Exact`.

Per `CLAUDE.md` §1, the certificate ships with the operation and with a mutant set:

| Certificate | Asserts |
|---|---|
| `exactness::monotone_under_composition` | Over generated DAGs with inexact leaves planted at every depth, every node satisfies the law — **including `diff` of an inexact constant, which must be `Approximate(0)` and not `Exact(0)`** (§3) |
| `exactness::folding_requires_all_exact` | No node with an inexact operand is ever folded; a planted folder that computes `Approximate(0.1)·Exact(3)` into a single leaf is rejected (§6) |
| `exactness::no_exact_node_has_inexact_descendant` | The corollary consumers actually rely on, tested directly rather than inferred |
| `exactness::bridge_refuses_inexact` | `is_polynomial_in` returns `None` with a witness naming the offending node for every planted non-`Exact` subtree, exhaustively over the corpus |
| `exactness::no_decision_from_inexact` | Sign queries over every `Enclosed` and `Approximate` node in the corpus return `Unknown`, **including where the leaf enclosure excludes zero** — the case a filter would have decided |
| `provenance::bytes_are_id_independent` | `provenance_bytes` is byte-identical across insertion orders, thread counts, processes and feature combinations, on the same matrix as `canonical_bytes` |

**Mutants, drawn from the failure family this actually has:**

- **The promotion mutant** — constant folding that evaluates an `Approximate` child into a
  `Rational` and labels the result `Exact`. This is the natural bug, not a contrived one: it
  is what any straightforward constant-folder does, it compiles, and it produces a plausible
  value. `monotone_under_composition` must reject it.
- **The join mutant** — a fold that computes the *join* rather than the meet, so one exact
  operand rescues an inexact one.
- **The bridge mutant** — `is_polynomial_in` checking exactness only at the root.

### 4. Provenance: which leaf introduced the inexactness

`Exactness` alone says a node is not exact; it does not say why, and "why" is what makes the
consumer's verification loop diagnostic rather than merely defensive. Each node additionally
carries `inexact_witnesses: BTreeSet<ExprId>` — the leaves that introduced inexactness,
unioned from its children. `BTreeSet` for deterministic iteration order (ADR-012 §4).

**`ExprId` is an in-memory handle and must never reach a serialized form.** It is
arena-relative (ADR-020), and `API.md` L4-6 requires canonical bytes to be "independent of
interning order, handles, arena addresses, insertion history and build configuration". So the
witness set is addressed **structurally in serialization and by handle only in memory**: a
witness appears in `provenance_bytes` as its own canonical bytes (or a fixed-width digest of
them), never as its id.

Sets are bounded by the number of inexact leaves in the DAG and are shared through
hash-consing. **If measurement shows the union cost matters on a real workload, the fallback
is a bounded witness — first `k` by id, plus a count — and the measurement is what decides
it**, committed per `CLAUDE.md` §7. Not a guess in either direction.

### 5. Two digests, and this is the fork that had to be settled

`CLAUDE.md` §3.2 excludes certificates, `Evidence`, `Telemetry` and `BudgetTick` from
canonical bytes, because a tuning knob that changes them would break ADR-012 §8's
tuning-matrix value-equality gate on its first run. Exactness has exactly this hazard: a
refinement budget is a tuning knob and it changes an `Enclosed` bound's width.

But an `Approximate` leaf and an `Exact` leaf of the same numeric value are **not the same
mathematical object**, and a consumer that content-addresses generated artifacts must not
conflate a derivation that was exact with one that was not. So:

```rust
impl Store {
    /// Mathematical value only. An Enclosed leaf's bounds ARE its value and are
    /// included; the Exactness label and the witness set are EXCLUDED.
    /// A finite f64 leaf serializes as its exact dyadic rational value, because that
    /// is what it is. ADR-012 §9 and the tuning-matrix gate are unaffected.
    pub fn canonical_bytes(&self, e: ExprId) -> Vec<u8>;

    /// Canonical bytes, plus the exactness annotation and the witness set.
    /// For consumers that must not conflate an exact derivation with an approximate one.
    /// Not stable under a refinement-budget change, and documented as such.
    pub fn provenance_bytes(&self, e: ExprId) -> Vec<u8>;
}
```

**The case that justifies two digests, worked.** `Const(Exact(1/10))` and
`Const(Approximate(0.1_f64))` where the f64's exact dyadic value happens to equal the rational:
these are **different nodes** (different variants, so hash-consing separates them), they have
**identical `canonical_bytes`** (same mathematical value — which is correct, and is what makes
the tuning matrix green), and **different `provenance_bytes`** (one derivation was exact, one
was not). A consumer content-addressing generated artifacts keys on the second; a consumer
comparing mathematical values keys on the first. Neither digest alone would serve both, which
is the whole argument for having two.

**A leaf's enclosure bounds are content and go in `canonical_bytes`.** An `Enclosed(lo, hi)`
leaf denotes an interval, that interval **is** its mathematical content, and two leaves with
different bounds are different objects — excluding the bounds would collide them. Bounds are
supplied by the caller and never move (§1), so no tuning knob can shift these bytes and
ADR-012 §8's value-equality gate is unaffected.

What `canonical_bytes` excludes is the **label and the provenance**, not the value:
`Approximate(0.1_f64)` and `Exact(1/10)` serialize identically — same value, and the f64 is
serialized as the exact dyadic rational it is. `provenance_bytes` adds the `Exactness` state and
each witness **addressed by its own canonical bytes** (§4). Both are functions of the DAG's
structure; neither is a function of any handle, arena, budget or thread count.

### 6. Inexact leaves are inert: resolvent carries them and never computes with them

**Constant folding fires only when every operand is `Exact`.** A node with any inexact operand
is built and left unfolded — resolvent performs no arithmetic on an `Approximate` or `Enclosed`
value, ever.

This is the rule that makes the whole design cheap and keeps it inside ADR-015. The
alternative readings both fail: folding `Approximate(0.1) · Exact(3)` into `Approximate(0.3)`
means resolvent is doing floating-point arithmetic internally, and folding
`Enclosed(lo,hi) · Exact(3)` into a wider enclosure means resolvent is doing interval
arithmetic — the filtered-arithmetic layer §1 already refused.

So the division of labour is: **resolvent tracks where inexactness entered and refuses to
launder it; the consumer evaluates.** A consumer that wants a number walks the DAG with
`walk_topological` and evaluates in whatever arithmetic it owns — which is what both surveyed
consumers already do, and what `resolvent-display` and a code emitter would do. What resolvent
adds is the part neither consumer can compute for itself: a per-node, non-forgeable statement
of which leaves the value depends on and how well each is known.

### 7. L0–L3 are untouched

No signature below L4 changes. No `Exactness` field appears on `Integer`, `Rational`,
`UPoly`, `MPoly`, `AlgebraicReal`, or `SqrtExt`. The algebra core remains exact-only by
construction, which is what keeps ADR-015's enclosure-semantics collision contained to one
layer and keeps the option-C merge question (ADR-018) exactly as cheap as it was.

---

## Consequences

- **`Node::Const` gains a variant** and the `Store` gains a per-node lattice element plus a
  witness set. That is the one-way part: it is in the hash-consed node identity.
- **Constant folding becomes exactness-aware.** ADR-017 §4's "construction hash-conses and
  constant-folds, and stops" now reads "…**and folds only when every operand is `Exact`**"
  (§6). A node with any inexact operand is built and left unfolded; resolvent performs no
  arithmetic on an inexact value.
- **The consumer adapters get shorter, not longer.** sinbad's grade mapping becomes a
  three-arm match instead of a hand-maintained discipline; cadabra2's self-referential
  residual bound (`:349-350`) gains a resolvent-side statement of *which* leaves it depends
  on — which is the half cadabra2 cannot compute for itself. **Note what it does not get:**
  resolvent does not carry a propagated bound for a derived quantity, because §1 forbids
  propagation. cadabra2 still computes the bound; what changes is that it can no longer
  accidentally treat an approximate input as exact.
- **`README.md`'s "Not numeric" bullet is wrong as written** and is replaced by the soundness
  half alone.
- **Risk: `Approximate` becomes a laundering channel through a consumer.** Nothing stops a
  caller reading an `Approximate` value out, computing with it, and feeding the result back as
  an `Exact` leaf. Resolvent cannot prevent that and should not pretend to; the crate docs
  state it plainly, and the witness set is what makes it auditable after the fact.
- **Risk: three states is too coarse.** `Enclosed` covers both a tight certified enclosure and
  a nearly-vacuous one. Deliberate — the width is carried and inspectable, and the *lattice*
  is about what may be decided, not about how tightly. Adding states later is additive as long
  as the order is extended, not reinterpreted.

---

## Alternatives considered and why rejected

**Keep the founding position: no `f64` in the library except as an output enclosure.**
Rejected by the owner, 2026-08-08. Recorded because it is coherent and cheap, and because its
real cost is now visible: both surveyed consumers reimplement the lattice themselves, one of
them with a self-referential bound it documents as a weakness.

**Push the lattice all the way down to L0.** Considered and rejected as the owner's explicit
choice of containment. It would put an enclosure semantics inside the algebra core — the
precise collision ADR-015 exists to prevent — and would change every L0–L3 signature to carry
a field that is `Exact` in every one of them by construction.

**Ship `Graded<T>` as a separate wrapper crate and let consumers compose it.** Rejected. It
gives up the property that makes the design worth having: if the lattice is outside the DAG,
nothing prevents an operation from returning a node more exact than its inputs, and the
anti-laundering law becomes a convention rather than a certificate.

**Put exactness in `canonical_bytes`.** Rejected: it breaks the tuning-matrix value-equality
gate the first time a refinement budget moves, which is the failure `CLAUDE.md` §3.2 already
predicts for `primes_used`.

**Leave exactness out of both digests.** Rejected: a consumer content-addressing generated
artifacts would collide an exact derivation with an approximate one, which is a wrong-artifact
bug that surfaces arbitrarily far downstream.

**Reuse `Certainty` instead of a new lattice.** Rejected. `Certainty` describes an
*algorithm's* confidence in an answer; `Exactness` describes a *value's* numeric provenance.
They are orthogonal — a `Proved` result over `Approximate` inputs is meaningful and must
remain expressible — and collapsing them would make one of the two unstatable.

---

## What would reverse this

- **The witness-set union measuring expensive** on a real DAG. Response: the bounded-witness
  fallback in §4. That is a tuning change, not a reversal.
- **A consumer needing a fourth state** — most likely splitting `Enclosed` into
  "certified-tight" and "certified-loose". Additive if the order extends; a new ADR if it does
  not.
- **The promotion mutant surviving the certificate.** That would mean the monotonicity law is
  tested somewhere the folder does not run, and it is the single most important thing to get
  right here: the whole design reduces to the claim that exactness cannot be forged.
- **`is_polynomial_in` proving to be a bridge consumers route around**, e.g. by a second L4→L1
  path added later. Response: there must be exactly one bridge, and any new one inherits the
  refusal. If that cannot be held, the containment in §6 fails and the lattice has to go all
  the way down after all.
