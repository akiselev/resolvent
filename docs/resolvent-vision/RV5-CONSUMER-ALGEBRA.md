# RV5 - Consumer-Critical Algebra

## Goal

Use the first real consumers to drive Resolvent from a sound CAS kernel into a practically valuable algebra system, while preventing consumer-specific semantics from leaking into the library.

RV5 runs as parallel lanes over the same RV1-RV4 contracts:

- **RV5-C:** CADabra exact geometry algebra;
- **RV5-S:** Scientia/Sinbad symbolic compiler algebra;
- **RV5-E:** evidence/oracle infrastructure shared by both.

Methodus and Solverang are secondary consumers where generic algebra proves useful, but they do not redefine the phase around numerical or constraint semantics.

## Shared placement rule

A consumer request moves into Resolvent only when:

- the API can be stated in generic mathematical terms;
- correctness can be graded independently of the consumer's semantic acceptance;
- it does not require geometry, physics, constraints or simulation runtime types;
- it has a second use case or a sufficiently general mathematical-domain justification.

## RV5-C - CADabra lane

### RV5-C1 - Subresultant and polynomial remainder infrastructure

Implement production-quality:

- polynomial remainder sequences;
- subresultant PRS;
- principal subresultant coefficients;
- discriminants and gcd/multiplicity utilities;
- square-free decomposition;
- exact coefficient/content normalization.

Use the simpler existing routes as correctness oracles before optimizing coefficient growth.

Acceptance:

- independent low-degree determinant/resultant agreement;
- external CAS differential checks on generic polynomials;
- known multiplicity/discriminant fixtures;
- explicit degree/coefficient-bit budgets.

### RV5-C2 - Algebraic real identity, ordering and sign

Promote the RV0 exact-root implementation into the RV2 domain/evidence model.

Provide:

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

Avoid a generic multivariate-elimination project merely because CAD uses low-degree resultants today. Broader multivariate work belongs in RV6 and should use the same API when it arrives.

### RV5-C6 - Geometry consumer gate

For every promoted algebraic operation, maintain:

1. Resolvent-domain property tests;
2. a consumer-neutral generated corpus;
3. CADabra regression fixtures;
4. the relevant Parasolid or independent geometry oracle when available;
5. explicit proof that CADabra remains the owner of branch/sheet/topology interpretation.

## RV5-S - Scientia/Sinbad lane

### RV5-S1 - Lossless scalar term integration

Complete the RV1 bridge so Scientia's semantic arena can refer to Resolvent scalar algebra without reimplementing its own algebraically authoritative expression tree.

Support:

- exact authored numeric values;
- arithmetic/functions;
- comparisons and conditions;
- piecewise scalar expressions;
- indexed/tensor scalar leaves via explicit opaque/scientific references where generic algebra cannot own their meaning;
- sidecar source/scientific identities.

### RV5-S2 - Symbolic differentiation and derivative programs

Expand generic differentiation beyond the current small function set:

- arbitrary symbolic powers with domain/branch conditions;
- multivariate gradients;
- Jacobians;
- Hessians;
- directional derivatives;
- piecewise differentiation;
- user/extension function derivative rules;
- common special functions needed by scientific source models.

Scientia owns which scientific variables are active/frozen and what a derivative means for fields, forms and moving domains. Resolvent owns the generic scalar calculus once the active symbolic problem is specified.

### RV5-S3 - Rational normalization and symbolic linear algebra

Deliver compiler-critical algebra:

- rational-expression normalization;
- denominator/content management;
- exact small symbolic matrices;
- determinant/rank/solve over exact and rational-function domains;
- symbolic block/algebra utilities useful for compiler structural transformations.

Large numerical linear solves remain Methodus-owned.

### RV5-S4 - CSE, Hornerization and neutral numeric programs

Provide deterministic symbolic optimization:

- common-subexpression elimination;
- algebraic strength reduction where exact semantics permit it;
- Horner form / polynomial evaluation planning;
- constant folding;
- temporary scheduling;
- a neutral scalar/tensor-free numeric SSA/program artifact suitable for downstream lowering.

Malleus owns backend-oriented finite-precision lowering and kernel execution. Resolvent must not acquire target-specific GPU/CPU backend policy.

### RV5-S5 - Series and local expansions

Implement enough exact/formal series machinery for compiler/response workflows:

- Taylor expansion of supported functions;
- multivariate local series where practical;
- order/truncation as explicit domain data;
- remainder/validity metadata when claiming more than a formal series.

This becomes an early substrate for Sinbad response/model-reduction work without moving simulation semantics into Resolvent.

### RV5-S6 - Scientia/Sinbad acceptance

Acceptance cases should include generic algebra extracted from:

- nonlinear heat/property expressions;
- elasticity/Stokes/Maxwell scalar coefficients;
- DAE/index-reduction symbolic time derivatives where applicable;
- manufactured-solution forcing terms;
- Jacobian/JVP reference expressions;
- response/local expansion fixtures.

The scientific compiler remains authoritative for units, shapes, axes, forms and method programs.

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

### RV5-E2 - Certificate/mutant corpora

Extend RV3's evidence system with deliberately incorrect variants for:

- root multiplicity;
- algebraic comparison;
- cancellation under zero denominators;
- branch-sensitive differentiation;
- CSE/code transformation that changes evaluation order where semantics forbid it;
- resultant convention/sign errors.

### RV5-E3 - Consumer promotion gate

A consumer-pulled operation is marked stable only when:

- generic tests pass;
- consumer fixture passes;
- relevant independent oracle/certificate passes;
- resource ceilings are known;
- the public API contains no consumer-domain nouns;
- ownership documentation remains correct.

## Optional Methodus/Solverang use

Resolvent may be used by Methodus or Solverang for:

- exact small-system baselines;
- symbolic Jacobians/Hessians supplied to numerical least-squares/nonlinear methods;
- exact polynomial constraint subproblems;
- generic algebraic conflict certificates;
- expression simplification before finite-precision evaluation.

But:

- Methodus owns iterative numerical solution, convergence and time integration;
- Solverang owns constraint graphs, activation, DOF/rank/conflict semantics and 2D/3D constraint vocabulary;
- CADabra owns authoritative geometric acceptance of constraint candidates.

## Exit gate

RV5 exits when:

- CADabra has a materially stronger generic algebra substrate without duplicate exact math;
- Scientia uses lossless shared scalar algebra for differentiation/optimization workflows;
- generic algebra demanded by consumers goes through RV3 planning/evidence contracts;
- at least one operation in each consumer lane is validated by both a real consumer fixture and an independent generic oracle/certificate;
- no geometry/scientific/numerical-solver/constraint semantics have migrated into Resolvent.

## Parallelism

RV5-C and RV5-S are intentionally independent except where they share RV2 domain primitives. RV5-E can fan out across oracle/certificate families. Within each lane, correctness baselines precede performance optimization.

## Non-goals

- complete general CAS breadth (RV6/RV7);
- CAD branch/topology algorithms;
- scientific method-family selection;
- Methodus numerical solver implementation;
- Solverang generic or geometry constraint engine implementation;
- native notebook UI.