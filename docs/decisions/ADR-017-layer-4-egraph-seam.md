# ADR-017 — Layer 4 is a resolvent-owned seam; no `egg` or `egglog` dependency now

**Status:** Ratified 2026-07-31
**Reversibility:** cheap
**Amended:** 2026-07-31 — `Simplifier`, `RuleSet`, the built-in rewriter, simplex
integration and rational-function normalization are moved to an explicit **post-v1** section;
the `resolvent-algebra` dependency is dropped (critique-engineering §15).
**Gates lanes:** X1, X2, X3, X4.
**Evidence:** `docs/research/prior-art-and-licensing.md` §4;
`docs/research/consumer-requirements.md` §5.1, §7 D8, §6 Tier B R19/R20;
`docs/research/critique-engineering.md` §15.

---

## Context

The source spec's Layer 4 is "a hash-consed DAG, `egg`-compatible so you get simplification
by e-graph rather than by a rats' nest of rewrite passes." It also says, in the same
document, that symbolic calculus is "a thin layer on top, not the point."

Both `egg` (0.11.0, MIT, 1793★, roughly annual releases, 26 open issues) and `egglog`
(2.0.0 released 2026-02, MIT, 803★, accelerating cadence, 118 open issues) are permissively
licensed, so **there is no licensing pressure here at all**. The question is purely
engineering, and the relevant facts are:

- `egg`'s own README points readers at `egglog`; `egglog`'s crates.io description calls
  itself "the successor to the popular rust library egg". Adopting `egg` means adopting a
  library whose own maintainers point elsewhere. Adopting `egglog` means adopting an API
  that hit 2.0 in February 2026 with 118 open issues.
- `alkahest-cas` depends on `egglog ^0.4` — **two majors behind current**. That is empirical
  evidence about how fast the API churns under a real consumer.
- **`egg`'s `Language` trait requires the expression type to be an enum with `Id`
  children — it wants to *own* the term representation.** resolvent's L4 is specified as a
  hash-consed DAG that L0–L3 already produce. Handing representation ownership at the top of
  a five-layer stack to a third-party crate is the coupling constraint #1 warns about, one
  level up.
- **L4 is optional to the value proposition.** The geometry consumer (#28) calls L0–L3 and
  nothing else. The medial-axis consumer (#27) calls L0–L2. Only the FEM form compiler (#34)
  wants L4.

And R2 §5.1 pins down exactly what #34 wants, which is narrower than "a CAS":

- **Symbolic differentiation** of weak forms, and Newton with a symbolically derived
  Jacobian. A rewrite rule, not a decision procedure.
- **Method of manufactured solutions**: pick `u_exact` symbolically (conventionally
  `sin(πx)sin(πy)`, `exp(x)`), apply the differential operator symbolically, get the forcing
  term exactly. This is the *first genuinely transcendental requirement* on resolvent — and
  it needs **differentiation and code emission only, never zero-testing**.
- **CSE, sum factorization, contraction reordering**, and straight-line-code lowering. This
  is what forces L4 to be a real DAG with stable node identity rather than a tree.
- **Usable from a `build.rs`**, so compile-time cost matters and runtime cost mostly does
  not — a completely different performance regime from geometry.

Meanwhile geometry needs a zero-test and no transcendentals: all three "non-polynomial"
curve families rationalize before they ask anything (Weierstrass `t = tan(u/2)` for
`sine_waves` and `sine_radical`; the longitude key `W = tan(u/2)` plus a monotone `acos` for
`spherical_circle`). The transcendental coordinates are never compared.

---

## Decision

### 1. No e-graph dependency now. The L4 seam is resolvent-owned.

`resolvent-expr` defines its own hash-consed DAG and its own rewrite-application seam:

```rust
pub struct ExprId(u32);
pub struct Store { /* hash-consed nodes, rustc-hash, deterministic id assignment */ }
pub enum Node {
    Const(Rational), Symbol(SymbolId),
    Add(SmallVec<[ExprId; 4]>), Mul(SmallVec<[ExprId; 4]>), Pow(ExprId, i64),
    Apply(FuncId, SmallVec<[ExprId; 2]>),    // OPAQUE. Semantics live in the FuncTable.
}
```

**There is no `Simplifier` trait in v1, and no `RuleSet`.** *Amended 2026-07-31.* This ADR
previously shipped a `Simplifier` seam, a built-in bottom-up rewriter, `simplify(expr,
rules)` as "the public entry point", named default rule sets, and both `egg` and `egglog`
adapters "later". Individually each is defensible. Together they are a rewriting engine with
a rule language, two backends, an integrator and rational functions, **in the layer the
source spec calls "not the point"**, specified across documents that contradict each other
about whether it exists at all: `API.md` §4.1 puts a general `simplify()` and e-graph
integration out of scope, with one L4 consumer *actively hostile* to canonicalization
because a canonicalizing rewriter destroys its `Cos2` certificate tether.

The line held is exactly what M7's exit gate tests: **hash-consing, `diff`/`diff_with`,
constant folding, `walk_topological`, `is_polynomial_in`, canonical bytes, `FuncTable`,
`rebuild_from`.** That is v1 of Layer 4. See §6 for what moved and what would bring it back.

The **structural encoding** — `walk_topological` plus `NodeRef` plus canonical bytes — is
retained and is the whole of the "egg-compatible" promise: it is what makes an external
`egg`/`egglog` adapter, an external code emitter and an external pretty-printer writable by
a third party without resolvent depending on any of them.

**`FuncId` semantics are caller-owned, not resolvent-owned.** `resolvent-expr` ships **no
transcendental semantics in its core** — only a table the caller constructs:

```rust
pub struct FuncTable { /* FuncId -> { name, arity, derivative rule (optional) } */ }
impl FuncTable {
    pub fn empty() -> FuncTable;
    pub fn standard_elementary() -> FuncTable;      // sin, cos, exp, log, sinh, …
    pub fn register(&mut self, name: &str, arity: u8, deriv: Option<DerivRule>) -> FuncId;
}
```

A consumer that needs `sin`/`exp` with derivative rules calls `standard_elementary()`. A
consumer that needs an opaque symbol with **no** rule (so that differentiating it is a
structured refusal rather than a wrong answer) registers it with `deriv: None`. A consumer
that must never see a given function simply never registers it, so the symbol is
structurally unrepresentable in its world. One mechanism, three needs, no built-in
semantics to argue about.

**Differentiation takes an explicit leaf-rule table, not a closure:**

```rust
pub type LeafRules = BTreeMap<SymbolId, ExprId>;   // default for absent keys: Zero | Refuse
impl Store {
    pub fn diff(&mut self, e: ExprId, wrt: SymbolId, ft: &FuncTable) -> Result<ExprId>;
    pub fn diff_with(&mut self, e: ExprId, wrt: SymbolId,
                     leaves: &LeafRules, ft: &FuncTable) -> Result<ExprId>;
    pub fn symbols_in(&self, e: ExprId) -> BTreeSet<SymbolId>;
}
```

`diff_with` is what lets a consumer express `d/dt` of a symbol that is *implicitly*
time-dependent — without it, the consumer reimplements the chain rule. A `BTreeMap` rather
than a callback because a closure that mints nodes would need `&mut Store` while
`diff_with` holds it; the table form is borrow-clean, reentrancy-free, and deterministic by
construction (`BTreeMap` iteration order is the key order, per ADR-012 §4). `symbols_in`
returning a `BTreeSet` is what makes the two-phase "collect, then build rules, then
differentiate" loop possible.

The `Refuse` default is the fail-closed one and should be the documented default:
differentiating with respect to a symbol that has no declared rule returns
`Unsupported::…`, not a silent zero. `Zero` is available for callers who genuinely mean
"all other symbols are constants".

`egg` and `egglog` adapters, if they ever ship, are **post-v1 and outside this repository**
(§6). Nothing in v1 depends on either.

### 2. Transcendentals live in L4 only, permanently

- **L4 may carry transcendental function symbols** (`sin`, `cos`, `exp`, `log`, …) as
  **opaque `Apply(FuncId, args)` nodes** whose semantics live entirely in a caller-owned
  `FuncTable`, and may lower them to
  straight-line code.
- **L0–L3 never see them.** No `UPoly`, `MPoly`, `AlgebraicReal`, or `SqrtExt` can contain
  one; the type system enforces this because those types are parameterized over coefficient
  *rings*, and a `FuncId` is not one.
- **resolvent offers no transcendental zero-test at any layer.** It is
  Richardson/Schanuel territory and undecidable in general. Attempting it is a research
  project, and no consumer needs it: FEM needs differentiation and evaluation; geometry
  rationalizes before it asks.

An attempt to evaluate a transcendental symbol into an exact algebraic context returns
`Unsupported::TranscendentalSymbol { name }` (ADR-011 §3) — a structured, matchable,
fail-closed refusal.

### 3. `resolvent-expr` depends on `base`, `int` and `poly`. Not on `algebra`, not on `real`.

L4 must not be able to hold the L3 lane hostage, and nothing the FEM consumer wants requires
root isolation. *Amended 2026-07-31:* the `resolvent-algebra` edge is **dropped**. It was
justified by "it wants gcd for rational-function normalization" — and rational functions are
out of scope with no consumer (`API.md` §4.1, L1-12). A dependency justified by an
out-of-scope capability is not a dependency. `poly` is what `is_polynomial_in` needs, and
that is the whole L4→L1 bridge. Add the edge back when something in scope needs it.

### 4. There is exactly one `canonicalize`, it is explicit, and it is value-preserving

`canonicalize(expr) -> Expr` returns a **new** node and is never applied implicitly.
Construction hash-conses and constant-folds, and stops. That single rule is what lets one
mechanism serve a consumer that content-addresses a canonical form and a consumer whose
certificate tether requires its `Cos2` atom to survive un-rewritten: the first calls
`canonicalize` and pays one line per call site, the second never calls it.

### 5. No `simplify()`, in v1, at all

The source spec names refusing a clever `simplify()` as its own risk, and both L4 consumer
evaluations independently confirm it. v1 ships **no** `simplify`, no `RuleSet`, no rewriter
and no rule language — not "a `simplify` whose rules are an argument", which is the same
function with a parameter that nobody has a use for yet. `canonicalize` (§4) exists, is
explicit, is opt-in, and is defined as value-preserving normalization rather than
cleverness.

### 6. Post-v1, on consumer demand — named, so the scope line is checkable

Each of these was previously specified as shipping. Each is now out until a named consumer
asks, and the ask is the trigger:

| Deferred | Would return when |
|---|---|
| `Simplifier` trait + built-in bottom-up rewriter | A consumer demonstrates a needed simplification `canonicalize` cannot reach |
| `RuleSet` and named default rule sets | Same, and only alongside the trait |
| `egg` / `egglog` adapters | The above ships, **and** the backend's API has stabilized. The adapter lives outside this repository and depends on both sides (ADR-005). Writing the `egg` adapter *early and throwaway*, as a design test of any future trait, remains the recommended way to find out the trait is wrong while it has no users |
| Exact symbolic integration over reference simplices | The FEM consumer exists as code rather than as a prospect. It is cheap and its certificate is trivial (differentiate and compare) — which is why it is easy to add later and was never a reason to add it now |
| Rational-function normalization / `RatFunc` | A second consumer. Today it has none, and it was the sole justification for the `resolvent-algebra` edge |

`FuncTable` **stays in v1**. It is not scope creep: it is the synthesis that serves all
three surveyed consumers with one mechanism precisely *because* resolvent ships no
transcendental semantics of its own, and `API.md` prices it honestly as a deliberate
addition over the spec's polynomial-only L4.

---

## Consequences

- **L4 blocks nothing and is blocked by nothing.** It can be built by an independent lane at
  any time, or never, without affecting the geometry or SMT tracks.
- **The bet on `egg` vs `egglog` is deferred indefinitely, at the cost of one trait.** If
  both churn, resolvent's own DAG is unaffected; if one wins decisively, an adapter is a
  few hundred lines.
- **resolvent owns its own term representation**, which is the whole point — the DAG that
  L0–L3 produce is the DAG L4 simplifies, with no translation layer in the default path.
- **The built-in rewriter will be worse than a real e-graph** at finding non-obvious
  simplifications. Accepted, and it is the correct default: CSE, differentiation, and
  polynomial normal form (the three things #34 actually needs) do not require equality
  saturation.
- **Lane grade is honest about being weak.** Rewrite *soundness* is property-testable
  (evaluate both sides at random points over GF(p) and compare — a genuine automatic
  verdict). "Did simplification produce a *good* result" is a number with no certificate.
  Sequence L4 last and do not let it block anything.
- **The transcendental scope decision is permanent and must be stated in the crate docs**,
  because it will be repeatedly re-proposed. The separation is clean: differentiation and
  emission are decidable and useful; zero-testing is undecidable and unneeded.

---

## Alternatives considered and why rejected

**Adopt `egg` now** (the source spec's suggestion). Rejected. Its `Language` trait wants to
own the term representation; its own maintainers point at the successor; its release cadence
is roughly annual. The "egg-compatible" goal is satisfied by an adapter, not by adoption.

**Adopt `egglog` now.** Rejected. 2.0 landed in February 2026 with 118 open issues, and
`alkahest-cas` is pinned two majors behind — empirical evidence of churn under a consumer.
Adopting a moving API at the top of a five-layer stack, for a layer that is explicitly "not
the point", is a bad trade.

**Skip L4 entirely.** Rejected. #34 (FEM form compiler) is a real prospective consumer, the
requirement is narrow and well-specified (differentiation, CSE, lowering, exact simplex
integration), and it is the only consumer that gives resolvent a compile-time-cost user,
which is useful design pressure.

**Allow transcendentals into L3 with a "best effort" zero test.** Rejected absolutely.
"Best effort" zero-testing is the F2 failure in a new costume: it produces an intransitive
equality, which produces a topologically inconsistent arrangement. Fail closed with
`Unsupported::TranscendentalSymbol` instead.

**Ship a global `simplify()` with built-in heuristics.** Rejected — the source spec's own
named risk, and the reason most CAS simplification is unpredictable.

**Ship `simplify(expr, rules)` with the rules as an argument, plus a `Simplifier` seam and
default rule sets (this ADR's original §5).** Superseded 2026-07-31. Parameterizing the
rules answers the *unpredictability* objection and not the *scope* one: it is still a
rewriting engine with a rule language in the layer the spec calls "not the point", for zero
consumers, and shipping it alongside two backend adapters and an integrator is how a
"thin layer on top" becomes the largest surface in the library. Deferring costs nothing —
adding a trait later is additive — and the deferral is now named per item in §6 rather than
left as a mood.

---

## What would reverse this

- **`egglog` stabilizing (say, 2.x with a slowing issue rate) and a consumer demonstrating a
  simplification resolvent's built-in rewriter cannot reach.** Response: ship the `egglog`
  adapter as a non-default feature. That is the plan, not a reversal — the seam exists for
  it.
- **The `Simplifier` trait proving unbridgeable to an e-graph backend.** This is the real
  risk of designing a seam before a consumer exists (flagged in R1 §8.8). Mitigation: write
  the `egg` adapter *early and throwaway*, as a design test of the trait, before committing
  to the trait's shape. If the adapter is awkward, the trait is wrong, and fixing it costs
  nothing while L4 has no users.
- **A consumer needing transcendental zero-testing.** Response: refuse, and point at the
  undecidability. If the consumer's need is actually "decide zero-ness for a restricted
  class" (e.g. exp-log constants under Schanuel), that is a separate, scoped, opt-in module
  with its own honesty about what it assumes — not a change to this ADR's default.
