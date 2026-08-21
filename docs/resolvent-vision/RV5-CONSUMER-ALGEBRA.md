# RV5 - Consumer-Critical Algebra

## Goal

Use real consumers to pull Resolvent toward practical algebra capability while preventing consumer-specific semantics from leaking into the library.

RV5 is a **cross-cutting consumer-pull namespace**, not a barrier that starts only after RV4. Each work package starts when its actual lower-level prerequisite exists:

- **RV5-C:** CADabra exact-geometry algebra; some packages may start immediately from the landed R1/RV0 exact substrate.
- **RV5-S:** Scientia/Sinbad compiler algebra; packages start from the required RV1/RV2/RV3/RV4 pieces individually.
- **RV5-E:** evidence/oracle infrastructure shared by both; packages may start as soon as the operation under test exists.

Methodus and Solverang are secondary consumers where generic algebra proves useful, but they do not redefine the phase around numerical or constraint semantics.

The authoritative cross-roadmap interpretation is [`CROSS-ROADMAP-CONTRACT.md`](CROSS-ROADMAP-CONTRACT.md).

## Shared placement rule

A consumer request moves into Resolvent only when:

- the API can be stated in generic mathematical terms;
- correctness can be graded independently of the consumer's semantic acceptance;
- it does not require geometry, physics, constraints or simulation runtime types;
- it has a second use case or a sufficiently general mathematical-domain justification;
- it does not create a second scientific semantic IR, executable numeric IR, plugin host, or artifact lifecycle system already owned elsewhere.

## RV5-C - CADabra lane

CADabra R2 is already unblocked by the landed R1 exact substrate. An RV5-C package starts when a concrete CADabra predicate or geometry workflow exposes a generic algebra need; it does not wait for the RV5 phase exit or unrelated RV1/RV4 work.

### RV5-C1 - Subresultant and polynomial remainder infrastructure

**Minimum prerequisite:** landed exact rational/polynomial substrate. RV2/RV3 wrappers may be adopted incrementally but do not block the correctness implementation.

Implement production-quality:

- polynomial remainder sequences;
- subresultant PRS;
- principal subresultant coefficients;
- discriminants and gcd/multiplicity utilities;
- square-free decomposition;
- exact coefficient/content normalization.

Use simpler existing routes as correctness oracles before optimizing coefficient growth.

Acceptance:

- independent low-degree determinant/resultant agreement;
- external CAS differential checks on generic polynomials;
- known multiplicity/discriminant fixtures;
- explicit degree/coefficient-bit budgets.

### RV5-C2 - Algebraic real identity, ordering and sign

**Minimum prerequisite:** landed exact-root substrate. RV2 domain identity and RV3 evidence integrate as they become available.

Promote/harden:

- immutable serialized algebraic-real certificates;
- exact equality/ordering;
- rational comparison;
- sign of a polynomial at an algebraic root;
- interval refinement;
- computable separation/refinement budgets where practical;
- canonical identity rules that do not depend on mutable cached intervals.

Acceptance includes trichotomy/transitivity/sort stability and explicit no-hang budgets.

### RV5-C3 - Algebraic extensions and radicals

Generalize current square-root/radical machinery where consumer cases require it:

- `a + b*sqrt(r)` style extensions;
- towers/simple number-field elements where required;
- exact sign in supported radical towers;
- root/domain coercion with explicit embedding choice;
- extension collapse when a radicand or defining polynomial reduces to the base field.

Do not build arbitrary number-field breadth before a concrete CAD or RV6 consumer justifies it.

### RV5-C4 - Bernstein and certified range algebra

Promote the generic Bernstein machinery as domain-independent polynomial range infrastructure.

Use cases include:

- root exclusion;
- interval subdivision;
- range certification;
- exact geometric predicate support in CADabra without moving geometric meaning into Resolvent.

### RV5-C5 - Bivariate elimination on demand

Promote bivariate resultants/elimination only when an actual CADabra R2-R6 predicate supplies:

- a generic mathematical input contract;
- a direct consumer fixture;
- an independent oracle;
- a performance target based on measured degree/sparsity/coefficient size.

This package may start as soon as those requirements and its exact-polynomial prerequisites exist. It does not wait for general multivariate RV6 or unrelated CAS frontend work.

Avoid a generic multivariate-elimination project merely because CAD uses low-degree resultants today. Broader multivariate work belongs in RV6 and should share the same eventual domain/evidence contracts.

### RV5-C6 - Geometry consumer gate

For every promoted algebraic operation, maintain:

1. Resolvent-domain property tests;
2. a consumer-neutral generated corpus;
3. CADabra regression fixtures;
4. the relevant Parasolid or independent geometry oracle when available;
5. explicit proof that CADabra remains the owner of branch/sheet/topology interpretation.

## RV5-S - Scientia/Sinbad lane

### RV5-S1 - Lossless scalar algebra projection

**Minimum prerequisite:** RV1 structural Term identity plus the coordinated Scientia source/semantic projection contract.

Scientia retains the one canonical `SemanticModel`/`ExprId` arena. Supported scalar mathematical subproblems project into Resolvent Terms/domain elements for generic algebra and results are re-embedded or attached under Scientia-owned semantics.

Support:

- exact authored numeric values once Scientia preserves them at the source/schema boundary;
- arithmetic/functions;
- comparisons and conditions where meaningful to the requested algebra operation;
- piecewise scalar expressions;
- indexed/tensor scientific leaves only through explicit projection variables/opaque leaves where generic algebra does not own their meaning;
- sidecar source/scientific identities retained by Scientia.

Do not make a Resolvent `TermId` the durable identity of a scientific expression.

### RV5-S2 - Symbolic differentiation and derivative expressions

Expand generic differentiation beyond the current small function set:

- arbitrary symbolic powers with domain/branch conditions;
- multivariate gradients;
- Jacobians;
- Hessians;
- directional derivatives;
- piecewise differentiation;
- user/extension function derivative rules;
- common special functions needed by scientific source models.

Scientia owns which scientific variables are active/frozen and what a derivative means for fields, forms and moving domains. Resolvent owns the generic scalar calculus once the active algebraic projection is specified.

### RV5-S3 - Rational normalization and exact symbolic linear algebra

Deliver compiler-critical algebra:

- rational-expression normalization;
- denominator/content management;
- exact small symbolic matrices;
- determinant/rank/solve over exact and rational-function domains;
- symbolic block/algebra utilities useful for compiler transformations.

Large finite-precision numerical linear solves remain Methodus-owned.

### RV5-S4 - CSE, Hornerization and algebraic evaluation schedules

Provide deterministic algebraic optimization:

- common-subexpression elimination;
- algebraic strength reduction where exact semantics permit it;
- Horner form / polynomial evaluation planning;
- constant folding;
- explicit temporary/let-binding schedules;
- cost metadata for downstream lowering.

**Do not create a second general numeric SSA/kernel IR.** Resolvent's output remains an optimized algebraic Term/domain expression plus optional let schedule. Scientia/Malleus lower that algebra into Malleus-owned finite-precision structured computation.

Malleus owns loop/iteration domains, operands/index maps, effects/reductions, AD execution products, backend lowering and generated kernels.

### RV5-S5 - Series and local expansions

Implement enough exact/formal series machinery for compiler/response workflows:

- Taylor expansion of supported functions;
- multivariate local series where practical;
- order/truncation as explicit domain data;
- remainder/validity metadata when claiming more than a formal series.

This supports Sinbad response/model-reduction work without moving simulation semantics or numerical reduction algorithms into Resolvent. Methodus owns physics-neutral numerical reduction algorithms.

### RV5-S6 - Scientia/Sinbad acceptance

Acceptance cases should include generic algebra projected from:

- nonlinear heat/property expressions;
- elasticity/Stokes/Maxwell scalar coefficients;
- DAE/index-reduction symbolic time derivatives where applicable;
- manufactured-solution forcing terms;
- Jacobian/JVP reference expressions;
- response/local expansion fixtures.

The scientific compiler remains authoritative for source spans, units, shapes, axes, forms, methods, active sets and derivative conventions.

## RV5-E - Evidence lane

### RV5-E1 - Oracle adapters

Build subprocess/differential adapters for available systems such as:

- SymPy;
- Sage;
- Maple;
- Mathematica/Wolfram Engine;
- FLINT/python-flint;
- Symbolica;
- specialized polynomial/root tools where useful.

Each adapter records:

- tool/version;
- exact command/input;
- normalization rules;
- timeout/resource policy;
- whether the result is complete/certified/heuristic for the queried operation.

No external system is a production semantic dependency merely because it grades tests.

External executable discovery/worker lifecycle may use Outboard through an optional adapter rather than creating a second plugin framework.

### RV5-E2 - Certificate/mutant corpora

Extend RV3's evidence system with deliberately incorrect variants for:

- root multiplicity;
- algebraic comparison;
- cancellation under zero denominators;
- branch-sensitive differentiation;
- CSE/algebraic optimization that changes semantics;
- resultant convention/sign errors.

### RV5-E3 - Consumer promotion gate

A consumer-pulled operation is marked stable only when:

- generic tests pass;
- consumer fixture passes;
- relevant independent oracle/certificate passes;
- resource ceilings are known;
- the public API contains no consumer-domain nouns;
- ownership documentation remains correct.

Durable large oracle outputs/certificates may use Artifactum when cross-repository lineage/distribution is needed; ordinary core tests do not require Artifactum.

## Optional Methodus/Solverang use

Resolvent may be used by Methodus or Solverang for:

- exact small-system baselines;
- symbolic Jacobians/Hessians supplied to numerical least-squares/nonlinear methods;
- exact polynomial constraint subproblems;
- generic algebraic witnesses consumed by constraint diagnostics;
- expression simplification before finite-precision evaluation.

But:

- Methodus owns iterative numerical solution, convergence, time integration, optimization/sampling/reduction algorithms and related numerical policy;
- Solverang owns constraint graphs, activation, DOF/rank/conflict/redundancy semantics and 2-D/3-D constraint vocabulary;
- CADabra owns authoritative geometric acceptance of constraint candidates.

## Exit gate

RV5 reaches mature status when:

- CADabra has a materially stronger generic algebra substrate without duplicate exact math;
- Scientia can project/re-embed scalar algebra losslessly while retaining canonical scientific semantic identity;
- generic algebra demanded by consumers adopts RV3 planning/evidence contracts as those contracts become available;
- at least one operation in each consumer lane is validated by both a real consumer fixture and an independent generic oracle/certificate;
- no geometry/scientific/numerical-solver/constraint semantics have migrated into Resolvent;
- no second executable numeric IR has been introduced.

RV5 maturity is **not** a prerequisite for starting unrelated RV6 algorithm families.

## Parallelism

RV5-C starts immediately from the landed R1/RV0 exact substrate where possible. RV5-S starts package-by-package from the required RV1/RV2/RV3/RV4 pieces. RV5-E can fan out across oracle/certificate families. Within each operation, correctness baselines precede performance optimization.

## Non-goals

- making RV5 a global phase barrier;
- complete general CAS breadth (RV6/RV7);
- CAD branch/topology algorithms;
- scientific method-family selection;
- replacing Scientia's expression arena;
- executable numeric/kernel IR;
- Methodus numerical solver implementation;
- Solverang generic or geometry constraint engine implementation;
- native notebook UI.
