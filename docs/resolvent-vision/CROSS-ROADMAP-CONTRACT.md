# Resolvent cross-roadmap contract

**Status:** authoritative sequencing/ownership supplement for the Resolvent Vision roadmap  
**Updated:** 2026-08-21

This document reconciles the Resolvent RV0-RV9 capability program with the live Sinbad, Scientia, CADabra3, Methodus, and Solverang roadmaps.

The governing rule is simple:

> RV phase numbers name capability programs. They are not a global serialization barrier. A work package starts when its actual typed prerequisites exist and a real consumer or standalone mathematical requirement justifies it.

Consumer roadmaps remain authoritative for consumer delivery order. Resolvent supplies reusable mathematics on demand without forcing CADabra, Scientia, Sinbad, Methodus, or Solverang to wait for unrelated CAS breadth.

## 1. Current federation baseline

The live ownership split is:

- **Resolvent:** consumer-neutral symbolic/exact mathematics, mathematical domains/coercions, generic calculus, algebra algorithms, assumptions/rewriting, and algebra evidence.
- **Scientia:** `.res`, one canonical scientific semantic arena, scientific expression identities, equations/forms/methods, scientific derivative/verification/response/event meaning.
- **CADabra3:** CAD geometry/topology, checked-number policy at the geometry boundary, predicates, intersections/trims/arrangements, persistent geometric identity, geometry events, and candidate certification.
- **Malleus:** finite-precision local structured computation, AD products, scheduling, backend lowering, and portable local-kernel emission.
- **Finitum:** concrete mesh/space/DOF realization, geometry association, discretization constraints, transfer, global transpose actions, and adaptivity.
- **Krasis:** coupled/world state, transactions, histories, events, checkpoints, and coupled operator composition.
- **Methodus:** consumer-neutral numerical operator contracts and numerical algorithms. Future physics-neutral Krylov, nonlinear, DAE, eigen, optimization, sampling, reduction, and related methods belong here as product demand grows.
- **Solverang:** domain-neutral variable/constraint graphs, candidate solve orchestration, rank/DOF/conflict diagnostics, and reusable 2-D/3-D constraint vocabulary over Methodus.
- **Sinbad:** simulation cases, studies, campaigns, support claims, product orchestration, comparison, and evidence promotion.

R1 already consolidated CADabra's generic exact/scalar implementation into Resolvent and deleted the old duplicate crates. CADabra R2 is therefore unblocked.

## 2. Phase numbers are capability namespaces

The broad dependency shape is:

```text
R1 exact/scalar consolidation (landed)
  |
  +--> RV0 post-R1 hardening -------------------------------+
  |                                                         |
  +--> CADabra R2 and early RV5-C consumer algebra ---------+
  |                                                         |
  v                                                         |
RV1 structural Term identity -------------------------------+
  |             |                                           |
  |             +--> RV8 parser/CLI prototypes              |
  |                                                         |
  +--> RV2 domains/coercions --------+                       |
  |                                  |                       |
  +--> RV3 requests/plans/evidence --+--> RV4 assumptions/evaluation/rewriting
  |                                  |
  |                                  +--> stable RV8 protocol/Jupyter
  |
  +--> RV5-S Scientia algebra projections and compiler helpers

RV6 exact-algebra families start family-by-family when their RV2/RV3
prerequisites exist; they do not wait for every RV5 lane.

RV7 calculus families start when the required Term/domain/evaluation/function
semantics exist; they do not wait for all RV6 families.

RV9 package/provider/cache/scale lanes start from their individual prerequisites.
```

A phase exit is a maturity statement for that capability family, not permission for all higher-numbered work to begin.

## 3. CADabra contract

CADabra's R0-R7 roadmap is the geometry-provider implementation authority under Sinbad SV0-SV9.

### Immediate rule

CADabra R2 may use the landed Resolvent exact foundation now. If R2 exposes a missing reusable algebra operation, the corresponding `RV5-C*` work package may start immediately against the minimum lower-level prerequisites it actually needs.

Examples:

- polynomial remainder/subresultant work can start on the landed polynomial/exact substrate without waiting for RV1 Term, RV4 rewriting, or the RV5 phase exit;
- algebraic-root ordering/sign hardening can start on the landed root substrate and later adopt RV2/RV3 wrappers when those exist;
- bivariate elimination starts only from a real CADabra predicate plus an independent generic oracle, but it does not wait for unrelated notebook/calculus work.

### Authority boundary

Resolvent may decide algebraic facts about polynomials, roots, signs, resultants, and exact values. CADabra decides what those facts mean for intersection branches, sheets, trims, topology, persistent entities, geometry events, and commit/refusal policy.

Solverang may produce finite-precision parametric-constraint candidates. CADabra independently certifies any candidate that can affect authoritative geometry/topology.

## 4. Scientia contract

Scientia's current `SemanticModel` and `ExprId` arena remain the single canonical scientific semantic representation.

Resolvent **does not replace Scientia's scientific expression arena**. Integration is an operation-specific algebra projection:

```text
Scientia SemanticModel / ExprId
        |
        | project supported scalar algebra
        v
Resolvent Term and/or Domain Element
        |
        | generic algebra operation + receipt/certificate
        v
algebra result
        |
        | re-embed/attach under Scientia-owned semantics
        v
Scientia artifact / transformed scientific expression / evidence
```

This avoids two failure modes:

1. making Resolvent understand scientific roles, shapes, axes, units, traces, fields, or differential operators;
2. making Scientia's semantic identity depend on Resolvent arena-local handles or session state.

### Exact authored literals

Resolvent can provide exact decimal/rational Term atoms and exact host-ingress APIs, but it cannot recover lexical exactness already lost upstream.

Scientia currently owns `.res` parsing and therefore owns preserving authored literal semantics. Any change from a source `f64` literal representation to exact decimal/rational source data is a coordinated **Scientia-owned source/schema change**. Resolvent consumes that exact value after projection.

### Operation-specific projection

Not every scientific expression must project to a Resolvent Term for every operation. For example, a relation may be representable structurally for simplification or condition reasoning while still being invalid as the direct input to scalar differentiation. Unsupported projections return a typed capability refusal rather than inventing semantics.

## 5. Malleus boundary

Resolvent must not create a second executable numeric IR beneath Scientia.

Resolvent may produce:

- optimized algebraic Terms/domain elements;
- CSE decisions and explicit let-binding/temporary schedules;
- Horner/factorized algebraic forms;
- symbolic derivative/Jacobian/Hessian expressions;
- expression-level cost/optimization plans.

Malleus owns:

- finite-precision structured operation semantics;
- loop/iteration domains;
- operand/index maps;
- effects/reductions;
- AD execution products;
- backend lowering and portable emitted kernels.

If a Resolvent optimization needs an exchange artifact, it is an algebraic expression/let plan, not a competing general SSA/kernel IR.

## 6. Methodus and Solverang boundary

Methodus is the numerical-method layer. Resolvent can provide exact/symbolic inputs and small algebraic verification cases, but does not own convergence, iterative methods, numerical event stepping, optimization/sampling policy, or large-system execution.

Solverang is the constraint-semantic layer. Resolvent can provide generic algebraic witnesses consumed by Solverang, but **Solverang owns whether those witnesses imply redundancy, conflict, remaining DOF, or constraint activation status**.

Neither Methodus nor Solverang is a required dependency of the Resolvent core.

## 7. Sinbad SV0-SV9 crosswalk

Resolvent capabilities are supporting machinery, not new product-level prerequisites unless an SV work package explicitly needs them.

| Sinbad program | Resolvent relationship |
|---|---|
| SV0 Trustworthy Simulation Factory | RV3 algebra receipts/certificates can strengthen algebra evidence, but SV0 campaign/evidence schemas do not wait for RV3 |
| SV1 Differentiability | landed differentiation plus demand-driven RV5-S improvements support Scientia; CADabra R2/R3/R4 shape-gradient lane does not wait for RV4/RV6 |
| SV2 Methods/Realizations | Methodus/Malleus/Finitum own the execution platform; Resolvent supplies only generic algebra used during compilation |
| SV3 Portable Codegen | Scientia owns scientific export and Malleus owns executable/kernel IR; Resolvent may provide expression optimization only |
| SV4 Response/Reduction | demand-driven RV5-S/RV6 algebra may support symbolic response/reduction; Methodus owns numerical reduction algorithms |
| SV5 Optimization/UQ/Inference | Methodus owns physics-neutral numerical algorithms; Resolvent supplies generic symbolic/exact transformations only |
| SV6 Learned Models | no direct Resolvent ownership beyond optional algebraic structure/evidence |
| SV7 Dynamic Worlds | CADabra/Scientia/Finitum/Krasis/Methodus own geometry/scientific/realization/state/numerics; Resolvent only generic algebra |
| SV8 Physics-Embedded Models | Resolvent may express generic algebraic structure; Scientia owns scientific structure/validity |
| SV9 Full-System Campaigns | no ownership changes; Resolvent remains an algebra provider |

## 8. Frontend sequencing

RV8 is intentionally split by prerequisite:

- **parser + structural formatter + simple batch CLI:** may start once RV1 Term syntax/identity stabilizes;
- **dynamic value inspection:** needs the relevant RV2 `Domain`/`Element` surface;
- **plan/explain/receipt protocol messages:** need the relevant RV3 schemas;
- **stateful REPL definitions/assumptions/rules:** need RV4 session/evaluation semantics;
- **stable transport-neutral protocol and Jupyter compatibility promise:** freezes only after the RV1 Term, initial RV2 dynamic-value, and RV3 outcome/plan/receipt schemas are coherent;
- **native notebook:** follows exercised protocol/Jupyter behavior.

Protocol prototyping may occur earlier. Protocol stability may not.

## 9. Providers, plugins, artifacts, and caches

Resolvent owns the **mathematical provider contract**: operation/algorithm identity, domain import/export semantics, exactness/evidence guarantees, and provider identity in RV3 plans/receipts.

It should not rebuild generic executable-plugin lifecycle machinery. When an external executable provider needs discovery, manifest compatibility, persistent workers, progress, cancellation, or process isolation, use **Outboard** through an optional adapter if its contract fits.

Resolvent also should not rebuild a durable general artifact lifecycle system. It may own in-process/local ephemeral memoization keyed by mathematical identity. Durable large certificates, external-provider outputs, lineage, reproducible transformations, distribution, and cross-repository evidence may use **Artifactum** through an optional adapter when needed.

Neither Outboard nor Artifactum becomes a mandatory dependency of the core CAS.

## 10. Change protocol

When a future roadmap change affects these boundaries:

1. update the owning consumer roadmap if product/provider sequencing changes;
2. update this cross-roadmap contract and `FEDERATION-OWNERSHIP.md` if ownership changes;
3. keep `STATUS.md` limited to landed truth;
4. do not make a higher RV phase number into a blanket prerequisite when a narrower typed dependency is sufficient;
5. record exact cross-repository commits in integration evidence when a coordinated cut lands.
