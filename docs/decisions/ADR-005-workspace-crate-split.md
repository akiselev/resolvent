# ADR-005 — Workspace shape: seven published crates, three unpublished, lockstep versioned

**Status:** Ratified 2026-07-31
**Reversibility:** costly (crate names on crates.io are sticky)
**Amended:** 2026-07-31 — gate L6a (published crates have zero dev-dependencies) added;
the crate graph is declared normative against the alternative sketched in
`plans/api-shape.md` §1.4 (ADR-021 §3, item 1).
**Amended:** 2026-08-08 — **nine** published crates, not seven: `resolvent-calculus` (L5) and
`resolvent-display` are added, and `resolvent-expr` regains its `resolvent-algebra` edge.
ADR-032's zero-test tiers are placed in `-calculus` specifically so `resolvent-expr` does not
acquire a `resolvent-real` edge (ADR-029 §4, ADR-033 §5).
**Gates lanes:** H1, and every lane thereafter.
**Evidence:** `plans/architecture.md` §1; `docs/research/consumer-requirements.md` §0.1;
`docs/research/algorithms-and-representation.md` §2.5;
`docs/research/critique-engineering.md` §2 item 1, §18.

---

## Context

Two forces pull in opposite directions.

**Toward one crate:** simplest for consumers, no version skew, whole-program inlining
without LTO gymnastics, no artificial API boundaries where an internal function would do,
one `Cargo.toml` to audit.

**Toward many crates:** resolvent is to be built primarily by AI agents working in
independently fannable lanes, each needing a self-contained automatic verdict
(constraint #3). A crate boundary is the strongest lane boundary Rust offers: disjoint
files, disjoint test suites, `cargo test -p` verdicts that do not interfere, independent
compilation units that parallelize, and a dependency edge that CI can assert.

There is also a hard requirement that pushes toward *more* crates than the obvious split: a
consumer implementing a coefficient ring for its own type must be able to do so **without
pulling a bignum dependency into its tree** (constraint #1). That requires the trait
vocabulary to live below `resolvent-int`.

And one sequencing gift that must not be thrown away: the first consumer touches none of
the multivariate machinery. Its polynomial type is a dense `Vec<Rational>`
(`crates/lazy-exact/src/roots.rs:43-45`), its resultants are hand-rolled degree-2 conic
determinants (`conics.rs:276-287`), and its `RealRoot` (`roots.rs:317-322`) is exactly
`AlgebraicReal`. The Gröbner one-way doors do not gate it — **provided the univariate type
is not defined in terms of the multivariate one** (ADR-007).

---

## Decision

**A workspace of seven `publish = true` crates and three `publish = false` crates, in a
strict linear dependency order, versioned in lockstep.**

```
resolvent-base    → resolvent-int → resolvent-modular → resolvent-poly
                  → resolvent-algebra → resolvent-real → resolvent (facade)
resolvent-expr    depends on base, int, poly, algebra.  NOT on real.
resolvent-calculus  depends on expr, algebra, real.     L5.
resolvent-display   depends on expr.  Nothing depends on it but the facade.
publish = false:  resolvent-oracles, resolvent-bench, resolvent-fuzz, xtask
```

*Amended 2026-08-08 (ADR-029 §4):* **two further published crates** — `resolvent-calculus`
and `resolvent-display`. ADR-029 declares a general-purpose scope, and the analytic surface
does not belong in `resolvent-expr`: putting series, integration and transforms in the DAG
crate would make the one crate every L4 consumer depends on the largest in the workspace.
(This ADR's title says "seven" and the graph above lists eight including the facade; the
discrepancy predates this amendment and the graph is what gate L1 diffs against.)

| Crate | Holds | Depends on |
|---|---|---|
| `resolvent-expr` (L4) | Hash-consed DAG, `diff`/`diff_with`, `FuncTable`, CSE, `walk_topological`, `is_polynomial_in`, canonical + provenance bytes, the exactness lattice (ADR-031), assumptions, `canonicalize`, `simplify` + `RuleSet` (ADR-033) | base, int, poly, **algebra** |
| `resolvent-calculus` (L5) | Series and limits, symbolic integration, ODE, integral transforms, special functions, **and ADR-032's zero-test tier machinery** | expr, algebra, real |
| `resolvent-display` | Pretty-printing, LaTeX. Conformance-graded (ADR-030); nothing in the core graph depends on it | expr |

Three consequences of the split, each load-bearing:

- **The `resolvent-algebra` edge on `resolvent-expr` returns**, dropped by ADR-017 §3 and
  reinstated by ADR-033 §5 — rational-function normalization is back in scope, which was the
  edge's original justification.
- **`resolvent-expr` still does not depend on `resolvent-real`, and ADR-017 §3's reason
  survives intact.** This is the non-obvious part. ADR-032's Tier-1(b) reduction — `sin(π/6)`
  into an `AlgebraicReal` — is an L4→L3 movement and would force the edge. It is therefore
  placed in `resolvent-calculus`, **not** in `resolvent-expr`. L4 stays buildable without L3,
  the two-trunk fan-out is preserved, and the zero-test tiers become an L5 capability. A
  zero-test entry point appearing in `resolvent-expr` is a layering defect, and gate L1 catches
  it as a new dependency edge.
- **`resolvent-display` is a leaf, deliberately.** A code emitter for a consumer's target
  language remains out of scope (ADR-017 §1, `API.md` L4-8) for its unchanged reason: sinbad
  wants Rust closures, the next consumer wants its own opcode tape, and `walk_topological` is
  where resolvent stops. Presentation of the mathematical object *itself* is a different thing,
  every CAS has one, and no consumer wants to write it — but it is graded by conformance, so it
  must not sit under anything that is certificate-graded.

Two gate interactions, checked rather than assumed:

- **Gate L7 is unchanged, and that is a decision.** It reads "`rayon` appears only in
  `-algebra` and `-rea`l". Neither new crate is added to it, so **`resolvent-calculus` is
  single-threaded** until an ADR says otherwise. No surveyed consumer asks for parallel
  calculus, and widening a determinism-adjacent gate for a crate that does not exist yet is
  how the `parallel` feature stops meaning anything.
- **Gates L13–L15 (ADR-029 §2) apply to both new crates from their first commit**, like every
  other published crate. `resolvent-display` is the likeliest place someone reaches for a
  `thread_local!` formatting buffer.

*Amended 2026-07-31:* `xtask` (CI helper commands — the layering, grep, ratification and
census checks) is a fourth `publish = false` crate. ADR-016 §2's rule is "exactly two crate
**categories**, no third and no per-crate exception", not a fixed crate count, so this is
within it. `resolvent-oracles` additionally hosts **all property tests and differential
oracles**, because gate L6a leaves published crates with no dev-dependencies and therefore no
`proptest`.

Contents per crate are in `plans/architecture.md` §1.1.

### The rules the layering enforces

Ten mechanical gates, listed in `plans/architecture.md` §1.3. The four that matter most:

- **L1** — no crate depends on a crate above it. CI diffs `cargo tree --edges normal`
  against a checked-in expected graph.
- **L2** — `dashu` appears in exactly one `Cargo.toml`. This is ADR-002's wall, made
  mechanical.
- **L4/L5** — no geometric vocabulary and no mention of `arrangements`/`lazy-exact` in any
  published crate. This is constraint #1, made mechanical.
- **L6** — `publish = false` crates may depend on published ones; never the reverse,
  including dev-dependencies. This is ADR-001's two-category rule, made mechanical.
- **L6a** *(added 2026-07-31)* — **every `publish = true` crate has an empty
  `[dev-dependencies]` table.** One `cargo metadata` assertion. L6 as originally written
  was a statement; `cargo deny` is scoped to the published graph minus dev-only features
  and so cannot enforce it, while `cargo publish` records dev-dependencies in the manifest
  and a downstream `cargo test` then builds them. Without L6a, the `rug` oracle lands in
  `resolvent-int/tests/` and a published MIT crate carries an LGPL-3.0+ dev-dependency
  (ADR-002 §Decision 5). Consequence, stated: **all differential and property tests that
  need a third-party oracle live in `publish = false` crates and test only the public
  surface.** In-crate `#[cfg(test)]` unit tests remain, and may use only workspace crates.

### The crate graph is normative here

`plans/api-shape.md` §1.4 sketched a different graph
(`seam/int/modular/poly/linalg/engine/alg/expr/lazy`). It is superseded: `API.md` §2.4
adopts this one explicitly, there is no `resolvent-seam` (ADR-019), `resolvent-lazy` is not
built (ADR-015, ADR-019 §4), and `resolvent-linalg`'s contents (`row_echelon`,
`bareiss_det`) live in `resolvent-algebra`. Gate L1 diffs `cargo tree --edges normal`
against the checked-in graph, so this is enforced from day one rather than argued.

### Versioning

One `version` in `[workspace.package]`; all crates released together; inter-crate
dependencies pinned `=x.y.z`. The published documentation says: **the supported surface is
the `resolvent` facade**; the inner crates are published so they *can* be depended on
directly (a consumer that only wants `resolvent-base` to implement a ring should not have
to pull the world), not so they can be mixed across versions.

---

## Consequences

- **Lane boundaries are crate boundaries.** `plans/architecture.md` §1.4 maps them, with a
  verdict type per lane. Two lanes in the same crate (e.g. `resolvent-modular`'s
  correctness lane and its bulk-kernel performance lane) are still separate lanes with
  separate definitions of done — the crate boundary is a floor on lane independence, not a
  ceiling.
- **A consumer can depend on `resolvent-base` alone**, with no bignum, to implement a ring.
  That is what makes ADR-018's option A cheap to *keep open* without building it.
- **Compilation parallelizes.** `resolvent-algebra` and `resolvent-real` are the two big
  crates and they compile concurrently with `resolvent-expr`.
- **Lockstep versioning removes the multi-crate release tax** — the standard objection to
  workspaces — at the cost of forcing a version bump on every crate for a change in any
  one. That is the right trade for a pre-1.0 library built by parallel agents.
- **Ten crate names must be claimed on crates.io early**, before any content, because names
  are first-come. This is the concrete sense in which the decision is costly to reverse.
- **Cross-crate inlining needs care.** Hot code that must inline across a crate boundary is
  either generic (monomorphized at the call site) or `#[inline]`. The kernels (ADR-006
  Tier M) are deliberately placed in the *same* crate as their callers where possible:
  GF(p) bulk ops live with `Fp` in `resolvent-modular`, and F4's row reduction lives in
  `resolvent-algebra` alongside the matrix construction that feeds it. Where that is not
  possible, `lto = "thin"` in the release profile is the fallback, not the plan.
- **`resolvent-expr` deliberately depends on neither `resolvent-real` nor
  `resolvent-algebra`.** L4 must not be able to hold the L3 lane hostage, and the FEM
  consumer that wants L4 needs differentiation and lowering, not root isolation. *Amended
  2026-07-31:* the `-algebra` edge is dropped too — it was justified by "gcd for
  rational-function normalization", and rational functions are out of scope with no consumer
  (ADR-017 §3). `-poly` is what `is_polynomial_in` needs, and that is the whole L4→L1 bridge.

---

## Alternatives considered and why rejected

**One crate with features.** Rejected. It makes every lane touch the same compilation unit,
which is the serial bottleneck constraint #3 cannot afford; it forces a consumer wanting
only the ring traits to compile (and license-audit) the bignum, the field kernels, and the
Gröbner engine; and feature-gating an internal module is a much weaker boundary than a
crate — nothing stops a `#[cfg(feature = "groebner")]` module from reaching into the root
isolator's internals.

**The brief's five-crate sketch: `resolvent-core` (rings + polys) + algebra + real + expr +
facade.** Rejected for two reasons given in `plans/architecture.md` §1.2: `resolvent-core`
would be the crate every lane touches, and it would force a bignum dependency on a consumer
that only wants the trait vocabulary. Splitting `core` into `base`/`int`/`modular`/`poly`
costs three extra `Cargo.toml` files and buys four independent lanes.

**Three crates: `resolvent-core` (base+int+modular+poly), `resolvent-algebra` (algebra+real),
`resolvent` (facade+expr).** The strongest alternative — fewer names, less ceremony.
Rejected because merging `algebra` and `real` merges the *certificate-heavy* lanes
(gcd, resultants, `AlgebraicReal`) with the *number-heavy* ones (F4, ANewDsc) into one
compilation unit and one test-suite verdict, which is precisely the distinction constraint
#3 says must drive sequencing.

**Splitting further — e.g. `resolvent-groebner`, `resolvent-factor`, `resolvent-roots`
as separate crates.** Rejected for now. The cost is real (more names, more version
surface, more cross-crate inlining friction) and the benefit is speculative until those
lanes exist. `resolvent-algebra` may be split later if it becomes a bottleneck; splitting a
crate is a mechanical refactor, whereas merging two published crates strands a name.

**Not publishing the inner crates at all (workspace-private, only `resolvent` on
crates.io).** Tempting for semver hygiene. Rejected because it defeats the
`resolvent-base`-only dependency path, which is the concrete mechanism by which
constraint #1 stays cheap.

---

## What would reverse this

- **Compile times or cross-crate inlining prove to be the dominant cost.** Response: merge
  adjacent crates (`modular` into `poly`, or `algebra` into `real`). Merging is mechanical;
  the stranded crate name is the only real cost, and a stranded name can be left published
  as a deprecated re-export shell.
- **A lane turns out to straddle a crate boundary repeatedly** — e.g. curve analysis needing
  intimate access to both the resultant implementation and the isolator. Response: move the
  code, not the boundary. If it happens three times, merge those two crates.
- **The `resolvent-expr` / `resolvent-real` independence turns out to be false** — i.e. L4
  simplification genuinely needs algebraic-number zero-testing. That would contradict
  ADR-017's scope decision and should be re-litigated there first, not here.
