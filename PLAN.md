# Resolvent Vision implementation plan

**Status:** proposed future program  
**Updated:** 2026-08-21  
**Program authority:** [`docs/resolvent-vision/README.md`](docs/resolvent-vision/README.md) and the RV0-RV9 capability files  
**Cross-roadmap sequencing:** [`docs/resolvent-vision/CROSS-ROADMAP-CONTRACT.md`](docs/resolvent-vision/CROSS-ROADMAP-CONTRACT.md)  
**Landed truth:** [`STATUS.md`](STATUS.md)
**Workspace execution order:** `/projects/sinbad/PLAN.md` in the sibling checkout

Resolvent is evolving from the newly consolidated consumer-neutral exact algebra crate into a full embeddable computer algebra system. R1 has already moved CADabra's generic exact/scalar implementation into Resolvent, migrated CADabra directly, and deleted the old duplicate crates. That landed foundation is the starting point for this program, not future work.

The target is not a clone of any one existing CAS. The target is a library-first mathematical kernel with Mathematica-class breadth and interactivity, Sage-class explicit mathematical domains, Rust-native embedding and performance, deterministic bounded execution, inspectable algorithm selection, and independently checkable results.

The same kernel must serve:

- Rust applications embedding algebra directly;
- Scientia's scientific semantic compiler through operation-specific algebra projections;
- CADabra's exact and symbolic algebra substrate;
- Methodus and Solverang where reusable algebra is useful without moving numerical or constraint semantics into Resolvent;
- command-line, Jupyter, native-notebook, Python, C, JavaScript/WASM, and agent clients.

No frontend, consumer, or optional provider is allowed to become the semantic authority for the core CAS.

## Architectural thesis

Resolvent combines three useful prior-art traditions:

1. **Wolfram/Mathematica:** a uniform symbolic term surface, first-class patterns/rules, and a strict kernel/frontend separation.
2. **Sage/FriCAS/Nemo:** explicit domains, parents/categories/capabilities, canonical coercions, and specialized internal representations instead of representing every value as a generic syntax tree.
3. **GiNaC/SymEngine/Symbolica-style embedding:** a native library API first, with language, notebook, and foreign bindings layered on the same kernel.

The resulting design is:

> Uniform at the symbolic surface, strongly domain-aware internally, deterministic and evidence-bearing at execution boundaries, and natively embeddable everywhere.

See [`PRIOR-ART.md`](docs/resolvent-vision/PRIOR-ART.md) for the detailed survey and design takeaways.

## Federation ownership

Resolvent must get much broader without absorbing the semantics of its consumers.

| Repository | Owns |
|---|---|
| `resolvent` | symbolic/exact mathematics, domains/coercions, algebraic algorithms, assumptions, rewriting, algebra plans/receipts/certificates, certified scalar evaluation |
| `scientia` | `.res`, one canonical scientific semantic arena and expression identity, equations/forms/method families, physical/scientific derivative requests and verification obligations |
| `quantitas` | dimensions, units, quantity kinds, standards metadata |
| `cadabra3` | geometry, topology, predicates, branches/sheets, intersections, p-curves/trims, persistent geometric identity, geometric events and certification policy |
| `malleus` | finite-precision local structured computation/IR, AD products, scheduling, backend lowering, generated numerical kernels |
| `methodus` | consumer-neutral numerical operator contracts and algorithms; future physics-neutral Krylov/nonlinear/DAE/eigen/optimization/sampling/reduction methods as demanded |
| `solverang` | generic constraint graphs, equality/inequality activation, rank/DOF/conflict semantics, candidate solving orchestration, reusable 2-D/3-D constraint vocabulary; Methodus supplies numerical algorithms |
| `finitum` | meshes/spaces/DOFs, global realization, geometry association, discretization constraints, transfer, transpose realization and adaptivity |
| `krasis` | coupled simulation state, transactions, histories, events, checkpoints and coupled operator composition |
| `sinbad` | simulation product cases, studies, campaigns, orchestration, results and evidence promotion |

Detailed boundaries are in [`FEDERATION-OWNERSHIP.md`](docs/resolvent-vision/FEDERATION-OWNERSHIP.md). The live cross-repository sequencing interpretation is in [`CROSS-ROADMAP-CONTRACT.md`](docs/resolvent-vision/CROSS-ROADMAP-CONTRACT.md).

## Core object model

Resolvent separates four concepts that small CAS implementations often conflate:

1. **`Term`** - immutable retained symbolic structure in a caller/session-owned hash-consed store. Term identity is structural, not a claim of mathematical equivalence.
2. **`Domain` / `Element`** - canonical mathematical values with optimized representations: integers, rationals, polynomial rings, rational-function fields, algebraic numbers, balls, matrices, series, and later ideals/modules.
3. **`Scalar`** - the lower-level numeric-kernel seam already consolidated from CADabra for writing the same kernel over `f64`, certified exact reals and dual numbers. It is not the universal CAS abstraction.
4. **`Value` / `Outcome`** - session-facing results that can carry terms, domain elements, collections, graphics, plans, receipts and explicit exactness/conditional/unknown/resource-limit states.

The existing lazy exact `Real` DAG is a backend for exact-real computation. It is not the general symbolic term store: lazy exact recipes may be pruned after forcing, while symbolic terms must remain inspectable and serializable.

RV1 construction does not perform algebraic reordering/flattening merely because a head is printed as `Add` or `Mul`. Algebraic canonicalization requires an explicit operation/domain contract and belongs in later domain/rewrite layers. This keeps structural hashing sound for noncommutative and extension-defined operations.

## Exactness model

Resolvent never silently changes an exact request into an approximate answer. Public operations return typed outcomes such as:

```text
Exact(value, optional certificate)
Conditional(value, conditions, optional certificate)
Certified(value, enclosure/error bound, certificate)
Approximate(value, precision, error estimate)
Unknown(reason, optional residual)
Unsupported(capability)
ResourceLimit(consumed, optional partial result)
```

Text literals also preserve intent. `0.1` parsed by a Resolvent surface syntax as an exact decimal is `1/10`; exact IEEE-754 ingress and approximate machine-float ingress are separate APIs.

For `.res`, Scientia owns preserving authored literal semantics because it owns source parsing. Resolvent cannot recover lexical exactness after Scientia has reduced a literal to `f64`; the eventual exact-literal cut is therefore coordinated but Scientia-owned at the source/schema boundary.

## Algebra planning model

Resolvent adopts Scientia's artifact discipline for algorithms, without turning algorithms into a declarative scientific-law DSL.

```text
AlgebraRequest
    -> applicability and feature extraction
    -> AlgebraPlan
    -> algorithm/provider execution
    -> AlgebraOutcome
    -> AlgebraReceipt
    -> optional independently checkable Certificate
```

Three catalogs remain distinct:

- **operation catalog:** mathematical meaning, domains, branches, arity/binders, exactness and formatting;
- **rule catalog:** guarded mathematical identities and transformations with provenance and verification strategy;
- **algorithm catalog:** implementations, applicability classifiers, guarantees, budgets, provider identity, fallback order and certificate type.

Users and agents should eventually be able to call `plan`, `explain`, force an algorithm, replay a receipt, and independently verify a result.

## Program graph: capability gates, not phase barriers

RV numbers identify capability programs. They do **not** impose a blanket `RV0 -> RV1 -> ... -> RV9` implementation sequence.

In the sibling ecosystem checkout, the workspace master plan selects one active
integration batch and may delegate only its non-overlapping leaves. That
coordination policy serializes merges without turning RV numbers into blanket
consumer prerequisites.

```text
R1 exact/scalar consolidation (landed)
  |
  +--> RV0 post-R1 hardening -------------------------------+
  |                                                         |
  +--> CADabra R2 + demand-driven RV5-C algebra ------------+
  |                                                         |
  v                                                         |
RV1 structural Term identity -------------------------------+
  |             |                                           |
  |             +--> RV8 parser/formatter/CLI prototypes    |
  |                                                         |
  +--> RV2 domains/coercions --------+                       |
  |                                  |                       |
  +--> RV3 requests/plans/evidence --+--> RV4 assumptions/evaluation/rewriting
  |                                  |
  |                                  +--> stable RV8 protocol/Jupyter
  |
  +--> RV5-S Scientia algebra projections/compiler helpers

RV6 exact-algebra families start independently when their required RV2/RV3
contracts exist. RV7 calculus families start from their actual Term/domain/
evaluation/function prerequisites. RV9 lanes likewise start from local gates.
```

A phase exit records maturity of that capability family. It is not permission for every higher-numbered work package to begin.

## Phase index

| Phase | Title | Primary maturity exit |
|---|---|---|
| [RV0](docs/resolvent-vision/RV0-EXACT-FOUNDATION.md) | Exact Foundation Stabilization | audited post-R1 contracts, budgets, serialization, performance/evidence baseline and downstream consumer gate |
| [RV1](docs/resolvent-vision/RV1-TERM-KERNEL.md) | Immutable Term Kernel and Wire Format | deterministic caller-owned structural term store, stable digests/wire representation, exact literals and generic retained symbolic structure |
| [RV2](docs/resolvent-vision/RV2-DOMAINS-COERCIONS.md) | Domains, Elements, Capabilities and Coercions | explicit mathematical parents/domains with canonical coercion graphs and specialized storage |
| [RV3](docs/resolvent-vision/RV3-PLANS-EVIDENCE.md) | Algebra Plans, Outcomes, Receipts and Certificates | deterministic algorithm planner and replayable evidence-bearing outcomes wrap the existing algebra vertical slice |
| [RV4](docs/resolvent-vision/RV4-EVALUATION-REWRITING.md) | Evaluation, Assumptions and Rewriting | bounded evaluator, assumption contexts, definitions, guarded rewrite packs and optional equality saturation |
| [RV5](docs/resolvent-vision/RV5-CONSUMER-ALGEBRA.md) | Consumer-Critical Algebra | demand-driven CADabra and Scientia/Sinbad algebra lanes mature through common Resolvent contracts |
| [RV6](docs/resolvent-vision/RV6-GENERAL-ALGEBRA.md) | General Algebra and Equation Solving | broad standalone exact-algebra and solving capability across polynomial, ideal, linear and discrete algebra families |
| [RV7](docs/resolvent-vision/RV7-CALCULUS-NUMERICS.md) | Calculus, Special Functions and Certified Numerics | staged symbolic calculus plus arbitrary-precision/certified scalar numerical evaluation |
| [RV8](docs/resolvent-vision/RV8-FRONTENDS-NOTEBOOKS.md) | Language, Kernel Protocol and Notebooks | CLI/REPL, protocol, Jupyter, bindings and native notebook all exercise the same kernel |
| [RV9](docs/resolvent-vision/RV9-ECOSYSTEM-SCALE.md) | Packages, Providers, Bindings and Scale | versioned package/provider ecosystem, broad bindings, deterministic caching/parallelism and federation extension packages |

## Immediate implementation lanes

1. **RV0 hardening and RV1 design run in parallel.** RV0 is intentionally short; RV1's one-way public identity changes merge only after the minimum RV0 readiness invariants are frozen.
2. **CADabra R2 is already unblocked.** Reusable algebra discovered by R2 starts the narrow `RV5-C*` package it needs immediately; it does not wait for RV1/RV4/RV5 phase exits.
3. **RV1 specifies structural Terms and exact Resolvent literals.** Do not put scientific roles, units, shapes, axes, field operators, or semantic `ExprId` identity into the Term store.
4. **Scientia integration is a projection, not an IR replacement.** Scientia remains authoritative for its `SemanticModel`/`ExprId`; supported scalar subexpressions project into Resolvent Terms/domain elements for algebra and are re-embedded/attached under Scientia-owned semantics.
5. **RV2 and RV3 begin from RV1 identity but may overlap.** RV3's first vertical slice can use the existing exact types before the entire RV2 domain catalog exists; domain-aware planning adopts RV2 descriptors incrementally.
6. **RV4 starts per subfeature when the needed RV1/RV2/RV3 pieces exist.** It is not a reason to block consumer algebra that does not need rewriting/assumptions.
7. **RV8 parser/formatter/simple CLI prototypes may start after RV1.** The stable kernel protocol and Jupyter compatibility promise wait until initial RV2 dynamic values and RV3 outcome/plan/receipt schemas are coherent. Stateful definitions/assumptions/rules need RV4.
8. **RV6/RV7 fan out by algorithm family, not by phase completion.** Correctness baselines precede optimized score lanes.
9. **RV9 provider/package/cache lanes use local prerequisites.** Outboard may supply executable-plugin lifecycle; Artifactum may supply durable external artifacts/lineage. Neither becomes a mandatory core dependency.

## Parallel work streams

### Core semantics

RV1, RV2, RV3, and RV4 have real dependencies but are not one monolithic serial chain. Freeze only one-way data-model contracts; allow independent certificate, domain, parser and consumer lanes to proceed against stable lower surfaces.

### CADabra

CADabra continues R2-R7 against the landed exact substrate while Resolvent hardening/term/domain work proceeds. Geometry/topology semantics never move down into Resolvent. Algebra requested by CADabra follows the minimum-prerequisite rule in the cross-roadmap contract.

### Scientia and Sinbad

Scientia retains its canonical scientific semantic arena. Resolvent is an algebra service over projected mathematical subproblems, not the owner of Scientia's full expression identity. Sinbad's SV programs consume scientific artifacts through Scientia and do not wait for unrelated RV capability exits.

### Methodus and Solverang

Methodus remains the owner of numerical algorithms. Solverang remains the owner of generic constraint solving and its 2-D/3-D constraint vocabulary, using Methodus for finite-precision numerical work. Resolvent may provide exact algebra, symbolic preprocessing, polynomial subproblems or algebraic witnesses; Solverang retains the semantics of conflict/redundancy/DOF/activation diagnostics.

### Malleus

Resolvent may optimize algebraic expressions and produce explicit CSE/let schedules. It does not create a competing executable numeric SSA/kernel IR. Malleus remains the owner of structured finite-precision local computation, AD execution products, iteration/index/effect semantics and backend lowering.

### Frontends and providers

Protocol/Jupyter work is dependency-sliced as described above. Resolvent owns mathematical provider semantics and provider identity in plans/receipts; Outboard may own external executable discovery/workers and Artifactum may own durable large artifacts/provenance when concrete use requires them.

## Agent implementation protocol

Every implementation work package has a stable ID such as `RV1-A2` or `RV5-C3`. PRs should:

1. name the work-package ID in the title;
2. state the owning repository and exact downstream consumer(s);
3. name the minimum typed prerequisite(s), not just a lower phase number;
4. name the input artifact/API and output artifact/API;
5. include a deterministic correctness baseline before optimized execution where applicable;
6. include at least one real consumer fixture for consumer-pulled generic machinery;
7. include a dissimilar second consumer/use case or a mathematical-domain justification before generalizing a consumer-specific mechanism;
8. preserve explicit budgets for potentially expensive operations;
9. emit or extend receipts/certificates when the operation is evidence-bearing;
10. update the corresponding phase ledger only after implementation lands;
11. update `STATUS.md` only with landed truth, never planned capability.

Do not create compatibility crates, duplicate scientific expression IRs, duplicate executable numeric IRs, consumer-specific dispatch in Resolvent, or hidden approximate fallbacks.

## Validation strategy

A broad CAS cannot be validated primarily with expected pretty-printed strings. Each family uses mathematical verdicts appropriate to it:

- algebraic law/property tests for each domain;
- cross-representation and metamorphic tests;
- independent simple implementations that grade optimized algorithms;
- subprocess oracles such as SymPy/Sage/Maple/Mathematica/FLINT/Symbolica where licensing and availability permit;
- multiply-back for factorization;
- substitution for equation solutions;
- differentiation of antiderivatives;
- independent root counts for isolators;
- exact or certified numerical evaluation under explicit assumptions;
- mutation tests for certificate checkers;
- deterministic seeds for randomized modular algorithms;
- parser/wire-format fuzzing;
- performance corpora classified by degree, sparsity, coefficient height, matrix size and expression sharing;
- live Scientia and CADabra integration fixtures.

Oracle output is evidence, not semantic authority. Commercial/copyleft systems remain differential or subprocess oracles unless their licenses permit a deliberate optional-provider integration.

## Non-negotiable guardrails

- No `.res` or scientific/physical meaning in Resolvent.
- No replacement of Scientia's canonical scientific `SemanticModel`/`ExprId` arena.
- No geometry, topology, mesh or CAD policy in Resolvent.
- No Methodus numerical-solver policy or Solverang constraint semantics in Resolvent.
- No second executable numeric/kernel IR competing with Malleus.
- No silent exact-to-approximate fallback.
- No process-global mutable symbol semantics.
- No algebraic reordering in the structural Term store without an explicit domain/operation contract.
- No unbounded automatic simplification.
- No e-graph as the canonical term store.
- No Python, Tokio, GUI or notebook dependency in the core library.
- No generic expression-tree implementation for every specialized algebraic value.
- No serialization of dependency-internal representations as the public wire contract.
- No consumer-specific compatibility facade during migrations.
- No algorithm promotion without a stable mathematical contract, resource policy and verification story.
- No frontend behavior becomes kernel semantics by accident.
- No duplicate Outboard-style executable-plugin host or Artifactum-style durable artifact lifecycle in the core CAS.

## Documentation authority

- [`STATUS.md`](STATUS.md) contains landed implementation truth only.
- This file defines Resolvent program intent and capability groups.
- [`docs/resolvent-vision/CROSS-ROADMAP-CONTRACT.md`](docs/resolvent-vision/CROSS-ROADMAP-CONTRACT.md) is authoritative when phase numbering could be misread as consumer sequencing.
- [`docs/resolvent-vision/README.md`](docs/resolvent-vision/README.md) defines architecture and work-package conventions.
- [`FEDERATION-OWNERSHIP.md`](docs/resolvent-vision/FEDERATION-OWNERSHIP.md) is authoritative for repository ownership.
- [`PRIOR-ART.md`](docs/resolvent-vision/PRIOR-ART.md) records design precedents.
- Each RV file is authoritative for its own work packages and non-goals, subject to the cross-roadmap minimum-prerequisite rule.
- Sinbad's Simulation Vision and CADabra's Recovery Plan remain authoritative for simulation-product and geometry-provider ordering respectively.

Historical Resolvent CAS planning remains available in Git history. Useful earlier decisions may be reintroduced only when reconciled with current consumers and this program; old documents are not automatically normative.
