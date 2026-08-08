# resolvent

The algebra engine — polynomials, ideals, algebraic numbers, resultants — that exact
computational geometry, FEM form compilers, and SMT NRA theories call. Permissively
licensed, exact, fail-closed.

**Status: planning. No implementation exists yet.** This repository contains research,
decisions, and plans. There is no `Cargo.toml`, no crate, and no line of Rust. See
[Status](#status) for what that means for reading the rest.

---

## What resolvent is

An *engine*, not a playground. The first useful release is roughly twenty-five functions —
the entire algebraic surface a shipping 17k-LOC exact geometry engine consumes is
re-exported under six names at `arrangements/crates/lazy-exact/src/lib.rs:82` —
generalized from degree ≤ 4 to arbitrary degree, with the coefficient-growth control that
makes arbitrary degree computable.

Three commitments define it:

- **Modular methods everywhere.** Reduce mod several primes → compute in GF(p) → CRT and
  rational-reconstruct → *verify*. Naive rational arithmetic gives coefficient explosion
  and a dead project. This is structural, not an optimization: retrofitting it under a
  working ℚ implementation is a rewrite (ADR-010).
- **Exact or nothing.** No tolerance parameter, at any layer, under any name. No
  "probably correct" mode that is not visible in the return type. A routine that cannot
  decide returns a structured refusal, never a guess (ADR-011).
- **Self-certifying results.** Every operation that can emit a proof of its own answer
  does, and the proof is checked in the same test that exercises the operation. This is
  the development model, not a testing appendix (see [Verification](#verification)).

Two further commitments, decided 2026-08-08 and governed by
[ADR-029](docs/decisions/ADR-029-general-purpose-cas-and-embedding.md):

- **A general-purpose CAS.** The exact algebraic core is the foundation, not the ceiling.
  Rewriting with explicit rule sets, rational functions, series, symbolic integration, ODEs,
  integral transforms, special functions and an assumptions system are all in scope. Most of
  them are in scope and *unspecified*, which is a different state from out of scope: a lane
  may open for one once the ADR specifying it is ratified. The bet is that a system can have
  the capability list of an established CAS while refusing the failure modes — heuristic
  simplification, numeric zero-testing, silent branch-cut choices — that the capability list
  usually comes with.
- **Embeddable, as a declared constraint rather than an accident.** resolvent is built to run
  inside a host it does not control: another Rust workspace, a numerical application across
  an `extern "C"` boundary, a WASM sandbox. No ambient state, no `thread_local!`, no global
  cache, no panic across any published entry point, every arena and store a caller-owned
  value, every budget counted in steps rather than wall-clock. Grep gates L13–L15 enforce the
  first three. This is not a `no_std` commitment; that question is open (ADR-029 §2).

## What resolvent is not

Every entry here is a **soundness rule**, not a scope boundary. Scope is ADR-029; these hold
regardless of how far it grows, and they are what the scope is worth having.

- **Never silently approximate.** A routine that cannot decide returns a structured refusal
  or `Verdict::Unknown` and the caller climbs to an exact rung. A heuristic result is
  `Certified<T>` carrying `Certainty::Probable(reason)` with its evidence; the default path
  returns `Proved`. A caller who wants speed or a conjecture asks for it by name (ADR-010,
  ADR-011).
- **No tolerance, epsilon, `atol`, `rtol`, or "close enough" parameter, at any layer, under
  any name.** A grep gate enforces it. Equality by tolerance is intransitive, a sort over it
  produces garbage, and a geometry consumer produces a topologically inconsistent
  arrangement. `refine_to(width)` is not a tolerance: it never affects a verdict, and that is
  property-tested as idempotence under refinement.
- **No floating point in a decision path.** `f64` may enter Layer 4 as an inexact *leaf*
  carrying a monotone exactness lattice — `Exact` / `Enclosed` / `Approximate`, never gained
  by combining — because tracking inexactness is a stronger enforcement of this rule than
  banning it. Layers 0–3 are exact-only and the L4→L1 bridge refuses anything not `Exact`
  (ADR-031, ADR-012 §6, ADR-015).
- **No implicit rewriting.** A consumer that never calls `canonicalize` or `simplify` never
  has its terms rewritten — a promise consumers build certificate tethers on. `simplify`
  takes an explicit rule set every time; there is no argument-free form and no default rule
  set. Every rule declares its soundness argument, and a rule that is false somewhere carries
  a machine-checkable side condition that must be discharged before it fires (ADR-033).
- **No unsound zero-test, at any layer, ever.** Numeric zero-testing — evaluate to `n` digits
  and compare — is banned permanently. A *sound* test over a named decidable subclass is
  permitted with its assumption visible in the return type: algebraic constants are `Proved`,
  a Schanuel-conditional exp-log test is opt-in and never returns `Proved`, and everything else
  is a structured refusal — `is_zero` returns `Result<Certified<bool>, Error>`, never a
  `Verdict` (ADR-032, INV-18).
- **Not an ecosystem numeric vocabulary.** resolvent ships algebraic traits and algorithms.
  It does not try to be the scalar seam every Rust numerics crate depends on (ADR-019).
- **Not an application.** No REPL, no notebook, no session state. resolvent is the engine an
  application embeds, which is what ADR-029 §2's invariants exist to make possible.
- **Not coupled to any consumer.** resolvent depends on no local project, imports no
  consumer trait, exposes no geometric type, and names no consumer anywhere in a published
  crate. Grep gates enforce this (`plans/architecture.md` §1.3, gates L4/L5).

## Why it exists

The permissive niche is empty and the reason is licensing. The mature implementations —
Singular, FLINT, PARI, msolve, Macaulay2, CoCoALib, Sage, Groebner.jl — are all copyleft;
the one fast Rust CAS (Symbolica) is proprietary source-available; the fastest pure-Rust
bignum (`malachite`) is LGPL-3.0-only because it is derived from GMP/FLINT/MPFR source.
The closest permissive prior art, `feanor-math` (MIT), names the gap in its own README:
polynomial operations over infinite rings are slow "since efficient implementation require
a lot of care to prevent coefficient blowup." That care *is* modular methods, and it is
resolvent's Layer-2 thesis. Full audit: `docs/research/prior-art-and-licensing.md`.

The concrete pull is a **degree ceiling**. Exact arrangement engines, medial-axis
computations, and NRA decision procedures all cap out where hand-rolled elimination stops
generalizing. The nearest example is legible: an exact 2-D arrangement engine in this
workspace hand-rolls the Sylvester resultant of two conics inline
(`arrangements/crates/arrangements/src/geoms/conics.rs:276-287`, eleven lines that only
exist because degree 2 in the eliminated variable has a closed form), copies those same
eleven lines for a second family, and reaches a third by double-squaring — which introduces
spurious roots that must then be filtered. Its polynomial type is a dense `Vec<Rational>`
(`lazy-exact/src/roots.rs:41-45`) and its module header states plainly that it has no
separation-bound machinery, so comparisons "terminate because distinct algebraic numbers
are eventually separated by bisection" (`roots.rs:8-11`) — true, correct, and unbounded.

One general `Res_y(f, g)`, one certified `AlgebraicReal` with a separation bound, and one
curve-analysis routine replace all of that. Nothing in the list is geometry-specific: the
same operations are what an SMT NRA theory needs for CAD projection and what a medial-axis
computation needs for trivariate elimination.

## Architecture

Published crates in a strict linear order, plus four unpublished ones. `resolvent` is the
only crate a consumer is expected to name (ADR-005, amended 2026-08-08).

```
L5  resolvent-calculus  series · limits · integration · ODE · transforms
                        special functions · ADR-032's zero-test tiers
                        depends on expr, algebra, real
                        EVERY LANE BLOCKED but the zero-test tiers: no ADR specifies them

    resolvent-display   pretty-printing · LaTeX.  Conformance-graded; a leaf.
                        Still NO code emitter — walk_topological is where resolvent stops

L4  resolvent-expr      hash-consed expression DAG · diff / diff_with · CSE
                        caller-owned Store · no implicit rewriting
                        exactness lattice (Exact/Enclosed/Approximate) in the node
                        canonicalize · simplify(expr, &RuleSet) · assumptions
                        depends on base, int, poly, algebra — NOT on real

L3  resolvent-real      AlgebraicReal { squarefree UPoly<Integer>, isolating (lo,hi) }
        ▲               exact comparison · SqrtExt (first-class, never subsumed)
        │               radical-tower signs · separation bounds · Bernstein enclosure
        │               root isolation (Sturm oracle, Descartes/VCA, ANewDsc) · curve analysis
        │
L2  resolvent-algebra   gcd · squarefree · resultants + subresultant PRS
        ▲               factorization (Zassenhaus → LLL → van Hoeij)
        │               Buchberger (oracle) → F4 → modular Gröbner · FGLM · ideal ops
        │
L1  resolvent-poly      UPoly<C>       dense univariate — standalone, defined FIRST
        ▲               MPoly          sparse distributed, ring-owned monomial arena
        │               RecursiveView  borrowed view for subresultant PRS
        │               Kronecker substitution bridge
        │
L0  resolvent-modular   Fp (word primes) · Zn · GF(p^k) · CRT · rational reconstruction
        ▲               deterministic prime registry · bulk GF(p) vector kernels
        │
    resolvent-int       Integer / Natural / Rational over dashu — the bignum wall
        ▲               dashu appears in exactly one Cargo.toml and in no signature
        │
    resolvent-base      trait tower · Sign / Verdict · Error / Unsupported / Budget
                        Certified / Certainty · canonical serializer.  No bignum.

    resolvent           facade: re-exports, feature plumbing, docs.  No algorithms.

    publish = false     resolvent-oracles (subprocess drivers, rug dev-oracle)
                        resolvent-bench · resolvent-fuzz · xtask
```

**Why `resolvent-calculus` is its own crate rather than more of `resolvent-expr`.** A
Tier-1(b) zero-test reduction — `sin(π/6)` into an `AlgebraicReal` — is an L4→L3 movement.
Putting it in `resolvent-expr` would give L4 a dependency on L3 and reinstate exactly the
coupling ADR-017 §3 removed so that the expression trunk could never hold the algebraic-number
lane hostage. Placing the tiers one layer up keeps L4 buildable without L3, keeps the
two-trunk fan-out, and keeps the largest new surface out of the one crate every L4 consumer
depends on.

Two structural facts that shape everything:

- **`UPoly<C>` is defined first and standalone.** It knows nothing about monomials, orders,
  or the `Ring` context. That is what lets the univariate/algebraic-number track run as a
  parallel trunk that never waits on the multivariate one-way doors (ADR-007). The first
  consumer touches no multivariate machinery at all, so the release that lifts the degree
  ceiling is the **elimination** milestone, not the Gröbner one.
- **The Layer-1 one-way door is the interning/id/key structure, not the packing width.**
  Measured, bit-packing exponents is worth ~15%; sparse GF(p) linear algebra is 73–91% of
  an F4 run and the divisor-query index is worth 10–20×. The source specification's claim
  that packed monomials are "most of your Gröbner performance" is false as stated, and a
  lane brief that says "optimize monomial comparison" buys 15% and misses a 20×
  (ADR-008 §Context; `docs/research/algorithms-and-representation.md` §1.6).

## Verification

**Self-certifying results are the whole development model.** resolvent is intended to be
built primarily by AI agents graded by oracles, which only works if every unit of work has
a verdict function that runs without a human in the loop. So each operation emits data
alongside its answer that proves the answer, and checking that data is cheaper than
recomputing it: factorization multiplies back and exhibits a modular irreducibility
certificate where one exists; gcd checks divisibility both ways *plus* a Bézout witness
`u·A + v·B = H`; Gröbner checks ideal membership via stored cofactors `g_j = Σ h_ij f_i`;
resultants are cross-checked by three implementations sharing almost no code; root
isolation is graded by exact Sturm counts; `AlgebraicReal` is graded by trichotomy,
transitivity, sort stability, and a step budget under which "did not finish" counts as
**wrong** — because a wrong implementation of exact comparison hangs, it does not return a
wrong answer. Three rules keep this from degenerating: a certificate may never invoke the
operation it certifies (otherwise `gcd ≡ 1` certifies itself); every certificate ships with
a committed set of deliberately-wrong implementations it must be observed *rejecting*; and
every API with a "don't know" or "probably" outcome carries a tracked rate with a committed
ceiling, because a maximally conservative implementation passes every soundness certificate
and is worthless. The full specification is `plans/verification.md`; the corrections above
come from `docs/research/critique-plan.md` C1, C3, and C8 and supersede it where they
disagree.

Where a lane's success criterion is a *number to optimize* rather than a certificate to
check — F4 row reduction, ANewDsc, modular resultant throughput, bulk GF(p) kernels — that
is said explicitly. Those lanes converge over months, are non-monotone, need a pinned
machine and a frozen reference implementation, and are sequenced differently. A performance
lane's CI job does not exist until the oracle lane it is graded against is green and
frozen: Sturm before Descartes, Buchberger before F4, Zassenhaus before van Hoeij.

## License

**MIT OR Apache-2.0.** Both texts are in the repository root (`LICENSE-MIT`,
`LICENSE-APACHE`). Every non-dev dependency must offer an MIT-or-equivalent arm — Apache-2.0
alone silently voids GPLv2 compatibility downstream, which is a material part of why the
MIT arm exists. Copyleft appears only as a subprocess oracle or a `publish = false`
dev-dependency, never linked into a published crate. A `cargo-deny` gate over the published
graph enforces this from day one, and it is itself regression-tested against three planted
cases it must reject (ADR-001).

Independent reimplementation informed by architectural study of the GPL/LGPL sources —
**not "clean-room"**; that term means the authors never saw the original, and we do read
Singular, FLINT, PARI, and msolve at the level needed to understand *what* they do.
Algorithms and ideas are not copyrightable and the published literature covers the
substance. The reading discipline and its mechanical gates are in ADR-001 and in
`CLAUDE.md`.

## Status

Nothing is implemented. Design, verification planning, and two rounds of adversarial
critique are complete, and the critiques' findings are being folded back into the ADR set as
amendments — an in-flight process, not a finished one.

**Do not read an ADR's body without reading its header.** Each carries machine-readable
fields, and they are the source of truth for whether the body can be built against:

```
**Status:** Ratified <date>        — or — Proposed (<date>)
**Reversibility:** one-way | costly | cheap
**Amended:** <date> — what changed, and which critique finding drove it
**Gates lanes:** <lane ids>
```

A `Proposed` ADR is not a freeze, and no lane may start against one. An ADR carrying an
`Amended:` line supersedes the critique finding it cites; an ADR without one has not yet
absorbed its findings.

Three findings were fatal in the specific sense that an agent starting on them writes code
that must be thrown away:

1. **`Ring::zero()` / `one()` were receiverless associated functions**, so they were
   unimplementable for `Fp`, `Zn`, `GF(p^k)` and every other ring in the closed
   instantiation set that carries its modulus by value; `Liftable`'s declared signature did
   not compile; `Reducible::Image: Field` is false over algebraic extensions.
   *Corrected in ADR-006:* `type Ctx` on `Ring`, `Liftable: Reducible`,
   `Reducible::Image: CommutativeRing` with a fallible `reduce`, `BulkOps` replaced by
   `BatchField::inv_batch`.
2. **Two normative API specifications disagreed at signature level on eleven items.**
   `API.md` is now canonical for the public surface and ADR-005 declares the crate graph
   normative. The full arbitration record — `docs/decisions/RECONCILIATION.md`, referenced
   by `API.md` and ADR-019 — is not yet present.
3. **The two Gröbner modes cannot share a reducer** — one reduces over ℚ, the other is a
   `u32` GF(p) kernel — which removes the fast mode's only claimed internal oracle, and the
   cofactor prototype measures the wrong number. ADR-010 is still `Proposed`; this is not
   yet corrected.

`docs/research/critique-engineering.md` §22 and `docs/research/critique-plan.md` §20 carry
the full triage, twenty and twenty-two findings respectively. **The critiques are
authoritative wherever they contradict a plan document that has not yet been amended
against them.**

## Documents

Read in this order if you are new. Every document is decisions-first: the load-bearing
choice, then the alternatives rejected and why.

| Document | Read it when |
|---|---|
| `CLAUDE.md` | **Before writing anything.** The working agreement: certificates, the frozen layer, fail-closed and determinism discipline, license discipline, honest verification, commit conventions. |
| `API.md` | You need the public surface. **Canonical**, supersedes `plans/api-shape.md`. Consumer-facing decisions, the numeric seam, certificates, invariants INV-1…INV-18, adapter sketches. |
| `plans/architecture.md` | You need the crate shape, the layering gates, the generics boundary, or the deferred-integration position. Where it disagrees with an ADR, the ADR wins. |
| `plans/verification.md` | You are implementing anything. §2 is the certificate catalogue, §3 is where certificates run out (read this one first), §7 is the CI gates. |
| `plans/roadmap.md` | You are scheduling work, or want to know which lanes can run in parallel and which cannot. §4.1 names the four lanes that are *bad* agent targets and why. |
| `docs/decisions/ADR-001…020` | You want to change a decision, or want to know why one was made. One-way doors are marked. Changing one means writing a new ADR that supersedes it. |
| `docs/research/prior-art-and-licensing.md` | Choosing a dependency, or deciding whether a source is safe to read. Licenses verified against crates.io, not against GitHub badges. |
| `docs/research/consumer-requirements.md` | You are about to add an API. It counts what a real 17k-LOC consumer actually calls, and finds it is ~25 functions. |
| `docs/research/algorithms-and-representation.md` | You are implementing a Layer-1/2/3 algorithm. Measured performance decomposition, the failure-mode enumeration for exact comparison (§8.2 F1–F10), benchmark calibration. |
| `docs/research/consumer-{sinbad,cadabra2,solverang}.md` | You want the evidence behind a consumer-derived requirement. |
| `docs/research/challenge-{generality,evidence}.md` | You are widening the API beyond the surveyed consumers. |
| `docs/research/critique-engineering.md` | **Before implementing any Layer-0/1 type.** Twenty findings against the engineering design, three fatal. |
| `docs/research/critique-plan.md` | **Before writing a certificate or a lane brief.** Twenty-two findings against the verification story and the fan-out plan, two fatal. |
| `plans/api-shape.md` | Historical only. Superseded by `API.md`; retained as the working notes it was. |

Consumer repositories are named in this README and in `docs/research/` as **context and
evidence only**. resolvent does not depend on them, and no published crate may name them.
