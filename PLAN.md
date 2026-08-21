# Resolvent Vision implementation plan

**Status:** proposed future program  
**Updated:** 2026-08-21  
**Program authority:** [`docs/resolvent-vision/README.md`](docs/resolvent-vision/README.md) and the RV0-RV9 phase files  
**Landed truth:** [`STATUS.md`](STATUS.md)

Resolvent is evolving from the newly separated consumer-neutral exact algebra crate into a full embeddable computer algebra system. The target is not a clone of any one existing CAS. The target is a library-first mathematical kernel with Mathematica-class breadth and interactivity, Sage-class explicit mathematical domains, Rust-native embedding and performance, deterministic bounded execution, inspectable algorithm selection, and independently checkable results.

The same kernel must serve:

- Rust applications embedding algebra directly;
- Scientia's scientific semantic compiler;
- CADabra's exact and symbolic algebra substrate;
- Methodus and Solverang where reusable algebra is useful without moving their numerical or constraint semantics into Resolvent;
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

The current ownership correction remains fundamental. Resolvent must get much broader without absorbing the semantics of its consumers.

| Repository | Owns |
|---|---|
| `resolvent` | symbolic/exact mathematics, domains/coercions, algebraic algorithms, assumptions, rewriting, algebra plans/receipts/certificates, certified scalar evaluation |
| `scientia` | `.res`, scientific meaning, equations/forms/method families, physical/scientific derivative requests and verification obligations |
| `quantitas` | dimensions, units, quantity kinds, standards metadata |
| `cadabra3` | geometry, topology, predicates, branches/sheets, intersections, p-curves/trims, persistent geometric identity, geometric events and certification policy |
| `malleus` | finite-precision local kernel IR, backend-oriented lowering, AD products, scheduling, generated numerical kernels |
| `methodus` | consumer-neutral numerical methods: linear/nonlinear/least-squares/DAE/block operator contracts and numerical algorithms |
| `solverang` | generic constraint graphs, equality/inequality activation, rank/DOF/conflict analysis, candidate solving orchestration, and reusable 2D/3D constraint vocabulary; Methodus supplies its numerical algorithms |
| `finitum` | meshes/spaces/DOFs, global realization, geometry association, constraints, transfer, transpose realization and adaptivity |
| `krasis` | coupled simulation state, transactions, histories, events, checkpoints and coupled operator composition |
| `sinbad` | simulation product cases, studies, campaigns, orchestration, results and evidence promotion |

Detailed boundaries and extension rules are in [`FEDERATION-OWNERSHIP.md`](docs/resolvent-vision/FEDERATION-OWNERSHIP.md).

## Core object model

Resolvent should explicitly separate four concepts that small CAS implementations often conflate:

1. **`Term`** - immutable retained symbolic structure in a caller/session-owned hash-consed store. Terms preserve structure for rewriting, printing, pattern matching, provenance and serialization.
2. **`Domain` / `Element`** - canonical mathematical values with optimized representations: integers, rationals, polynomial rings, rational-function fields, algebraic numbers, balls, matrices, series, and later ideals/modules.
3. **`Scalar`** - the lower-level numeric-kernel seam inherited from CADabra for writing the same numerical kernel over `f64`, certified exact reals and dual numbers. It is not the universal CAS abstraction.
4. **`Value` / `Outcome`** - session-facing results that can carry terms, domain elements, collections, graphics, plans, receipts and explicit exactness/conditional/unknown/resource-limit states.

The CADabra lazy exact `Real` DAG becomes a backend for exact-real computation. It is not the general symbolic term store: lazy exact recipes may be pruned after forcing, while symbolic terms must remain inspectable and serializable.

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

Text literals also preserve intent. `0.1` parsed as an exact decimal is `1/10`; exact IEEE-754 ingress and approximate machine-float ingress are separate APIs.

## Algebra planning model

Resolvent adopts Scientia's artifact discipline for algorithms, without turning algorithms into a declarative scientific-law DSL.

The runtime model is:

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

## Program graph

```text
RV0 exact/scalar consolidation
  |
  v
RV1 term kernel + canonical wire format -------------------------+
  |                                                             |
  +--> RV2 domains/coercions --> RV3 plans/outcomes/evidence --> RV4 assumptions/evaluation/rewriting
  |                                  |                          |
  |                                  +-------> RV5 consumer-critical algebra
  |                                                     |
  |                          +--------------------------+------------------+
  |                          v                          v                  v
  |                      CADabra lane              Scientia lane       evidence lane
  |                          |                          |
  |                          +------------+-------------+
  |                                       v
  |                             RV6 general algebra/solving
  |                                       |
  |                             RV7 calculus/certified numerics
  |
  +----------------------------> RV8 language/protocol/notebooks
                                            |
                                            v
                                 RV9 ecosystem/providers/scale
```

RV8 begins incrementally after RV1 rather than waiting for RV6/RV7 breadth. CADabra R2 may proceed immediately after RV0's consolidation gate; it does not wait for the general CAS surface.

## Phase index

| Phase | Title | Primary exit |
|---|---|---|
| [RV0](docs/resolvent-vision/RV0-EXACT-FOUNDATION.md) | Exact Foundation Consolidation | CADabra exact/scalar machinery has one owner in Resolvent; Scientia and CADabra consume it directly; duplicate crates are gone |
| [RV1](docs/resolvent-vision/RV1-TERM-KERNEL.md) | Immutable Term Kernel and Wire Format | deterministic caller-owned term store, stable digests/wire representation, exact literals, full structural expression surface |
| [RV2](docs/resolvent-vision/RV2-DOMAINS-COERCIONS.md) | Domains, Elements, Capabilities and Coercions | explicit mathematical parents/domains with canonical coercion graphs and specialized storage |
| [RV3](docs/resolvent-vision/RV3-PLANS-EVIDENCE.md) | Algebra Plans, Outcomes, Receipts and Certificates | deterministic algorithm planner and replayable evidence-bearing outcomes wrap the existing algebra vertical slice |
| [RV4](docs/resolvent-vision/RV4-EVALUATION-REWRITING.md) | Evaluation, Assumptions and Rewriting | bounded evaluator, assumption contexts, definitions, guarded rewrite packs and optional equality saturation |
| [RV5](docs/resolvent-vision/RV5-CONSUMER-ALGEBRA.md) | Consumer-Critical Algebra | CADabra and Scientia/Sinbad get the reusable algebra they need through common Resolvent contracts |
| [RV6](docs/resolvent-vision/RV6-GENERAL-ALGEBRA.md) | General Algebra and Equation Solving | broad standalone exact-algebra and solving capability across polynomial, ideal, linear and discrete algebra families |
| [RV7](docs/resolvent-vision/RV7-CALCULUS-NUMERICS.md) | Calculus, Special Functions and Certified Numerics | staged symbolic calculus plus arbitrary-precision/certified scalar numerical evaluation |
| [RV8](docs/resolvent-vision/RV8-FRONTENDS-NOTEBOOKS.md) | Language, Kernel Protocol and Notebooks | CLI/REPL, protocol, Jupyter, bindings and native notebook all exercise the same kernel |
| [RV9](docs/resolvent-vision/RV9-ECOSYSTEM-SCALE.md) | Packages, Providers, Bindings and Scale | versioned package/provider ecosystem, broad bindings, deterministic caching/parallelism and federation extension packages |

## First implementation sequence

1. **RV0-A: finish the active CADabra R1 consolidation.** Move the mature exact/scalar implementation and tests into Resolvent, adopt stable serialization, migrate consumers directly and delete `cadabra-exact`/`cadabra-scalar`. Do not redesign the expression model in the same cut.
2. **RV1-A: specify canonical values and term wire format before implementing the new store.** Exact integers/rationals/decimals, symbols, heads, binders, relations, conditions, canonical bytes and digests must be fixed together.
3. **RV1-B/C: implement the caller-owned hash-consed term store and coordinated Scientia bridge.** Replace lossy `f64` round-tripping and preserve relations rather than lowering comparisons to zero.
4. **RV3-A may start as soon as RV1 identity is stable.** Wrap current canonicalization, differentiation, sign, resultant and root-isolation operations in request/plan/outcome/receipt v2 contracts. This exercises the final execution architecture early.
5. **RV2 and RV4 continue in parallel.** Domain machinery and assumption/rewrite machinery should not block protocol prototyping once term identity is stable.
6. **RV8-A/B start after RV1 wire-format freeze.** Ship the CLI/kernel protocol/Jupyter path while the algebra breadth lanes proceed.
7. **CADabra R2 starts after RV0.** New reusable algebra discovered by R2 enters RV5-C instead of being copied back into CADabra.
8. **Scientia moves onto the lossless term bridge during RV1 and consumes RV5-S incrementally.** Sinbad continues to consume Scientia rather than acquiring its own symbolic layer.

## Parallel work streams

### Core semantics

RV1 -> RV2/RV3 -> RV4 is the critical API path. One-way data-model decisions need tight review and cross-consumer fixtures before broad algorithm fan-out.

### CADabra

After RV0, CADabra continues R2-R7 independently. Resolvent work is pulled by generic algebra requirements only. Geometry/topology semantics never move down into Resolvent.

### Scientia and Sinbad

After the RV1 bridge, Scientia can use generic symbolic differentiation, CSE, exact algebra, assumptions and symbolic programs while retaining all scientific semantics. Sinbad's SV0/SV1/SV3/SV4 programs consume those results through Scientia.

### Methodus and Solverang

Methodus remains the owner of numerical algorithms. Solverang remains the owner of generic constraint solving and its 2D/3D constraint vocabulary, using Methodus for finite-precision numerical work. Resolvent may provide exact algebra, symbolic preprocessing, polynomial subproblems, certification helpers or notebook-visible extension commands, but it does not absorb either repository's solver semantics.

### Frontends

The protocol/Jupyter stream starts after RV1's wire representation is fixed. The native notebook waits until the kernel protocol has been exercised by simpler clients so UI implementation cannot accidentally define kernel semantics.

## Agent implementation protocol

Every implementation work package has a stable ID such as `RV1-A2` or `RV5-C3`. PRs should:

1. name the work-package ID in the title;
2. state the owning repository and exact downstream consumer(s);
3. name the input artifact/API and output artifact/API;
4. include a deterministic correctness baseline before optimized execution where applicable;
5. include at least one real consumer fixture for consumer-pulled generic machinery;
6. include a dissimilar second consumer/use case or a mathematical-domain justification before generalizing a consumer-specific mechanism;
7. preserve explicit budgets for potentially expensive operations;
8. emit or extend receipts/certificates when the operation is evidence-bearing;
9. update the corresponding phase ledger only after implementation lands;
10. update `STATUS.md` only with landed truth, never planned capability.

Do not create compatibility crates, duplicate expression IRs, consumer-specific dispatch in Resolvent, or hidden approximate fallbacks.

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
- No geometry, topology, mesh or CAD policy in Resolvent.
- No Methodus numerical-solver policy or Solverang constraint semantics in Resolvent.
- No silent exact-to-approximate fallback.
- No process-global mutable symbol semantics.
- No unbounded automatic simplification.
- No e-graph as the canonical term store.
- No Python, Tokio, GUI or notebook dependency in the core library.
- No generic expression-tree implementation for every specialized algebraic value.
- No serialization of dependency-internal representations as the public wire contract.
- No consumer-specific compatibility facade during migrations.
- No algorithm promotion without a stable mathematical contract, resource policy and verification story.
- No frontend behavior becomes kernel semantics by accident.

## Documentation authority

- [`STATUS.md`](STATUS.md) contains landed implementation truth only.
- This file defines program order and phase relationships.
- [`docs/resolvent-vision/README.md`](docs/resolvent-vision/README.md) defines architecture, dependency rules, work-package conventions and parallel execution.
- [`FEDERATION-OWNERSHIP.md`](docs/resolvent-vision/FEDERATION-OWNERSHIP.md) is authoritative for repository ownership and Methodus/Solverang boundaries.
- [`PRIOR-ART.md`](docs/resolvent-vision/PRIOR-ART.md) records the design precedents behind the roadmap.
- Each RV phase file is authoritative for its own work packages, exit gate and non-goals.
- CADabra's active R1 consolidation gate remains authoritative for the cross-repository exact/scalar migration until RV0 exits.

Historical Resolvent CAS planning remains available in Git history. Useful earlier decisions may be reintroduced only when reconciled with the current consumers and this program; old documents are not automatically normative.