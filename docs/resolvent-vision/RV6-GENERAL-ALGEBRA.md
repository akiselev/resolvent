# RV6 - General Algebra and Equation Solving

## Goal

Expand Resolvent from consumer-critical algebra into a broad standalone exact CAS without abandoning the RV1-RV5 domain, planning, budget and evidence contracts.

RV6 is deliberately family-based. No single headline algorithm such as Gröbner bases is allowed to become the definition of a "complete CAS". Each family grows through explicit capability levels and independent acceptance gates.

## Algorithm-family structure

Every family should provide, where meaningful:

1. operation definitions and domain requirements;
2. a simple deterministic correctness baseline;
3. optimized realization(s);
4. applicability/selection descriptors;
5. resource budget model;
6. exactness/completeness guarantees;
7. certificate/independent oracle strategy;
8. corpus/performance classification.

## Work packages

### RV6-A - Integer and modular algebra

#### RV6-A1 - Integer arithmetic infrastructure

Harden arbitrary-precision infrastructure needed by higher algorithms:

- gcd/gcd-ext;
- modular exponentiation;
- exact roots/perfect-power checks;
- valuation/content helpers;
- rational reconstruction;
- CRT;
- deterministic prime streams/good-prime predicates;
- bit-size/resource accounting.

Backend-specific bignum details remain hidden behind Resolvent public types.

#### RV6-A2 - Primality and integer factorization

Add staged capabilities:

- deterministic small-range primality;
- probable-prime tests with explicit evidence grade;
- proof-producing primality methods where implemented;
- trial/Pollard-style baseline factors;
- ECM or other scalable methods only after correctness/evidence contracts are clear.

Integer factorization is not allowed to silently label probable completion as proved complete factorization.

### RV6-B - Polynomial factorization

#### RV6-B1 - Finite-field factorization

Implement square-free/distinct-degree/equal-degree factorization over finite fields with complete multiply-back and irreducibility checks.

#### RV6-B2 - Integer/rational univariate factorization

Implement a staged path such as:

- modular factor selection;
- Hensel lifting;
- Zassenhaus baseline recombination;
- LLL-based/van-Hoeij-style recombination when justified by measured workloads.

Correctness and performance lanes remain separate. The simpler baseline grades optimized recombination wherever feasible.

#### RV6-B3 - Algebraic-extension factorization

Extend to supported simple algebraic/number-field coefficient domains after RV5-C3/RV2 provides reliable parent/coercion semantics.

### RV6-C - Multivariate polynomial infrastructure

#### RV6-C1 - Monomial/ring representation

Implement:

- explicit variable/order-aware polynomial-ring parent;
- sparse distributed terms;
- monomial ordering and divisibility;
- overflow-safe exponent representation;
- deterministic term identity/order independent of insertion history;
- heap-based multiply/division baseline.

Representation decisions must be benchmarked on realistic reduction workloads rather than chosen for compactness alone.

#### RV6-C2 - Multivariate GCD and factorization

Add generic algorithms incrementally, starting with correctness-oriented routes and promoting modular/sparse methods only with independent grading.

### RV6-D - Gröbner and ideal theory

#### RV6-D1 - Buchberger correctness baseline

Implement Buchberger plus complete criterion checks as the internal correctness oracle.

Required outputs include enough data to verify both:

- the generated basis satisfies Buchberger's criterion;
- the ideal relationship to the input is correct, using cofactors or independent reduction checks where practical.

#### RV6-D2 - F4/F5-style optimized bases

Introduce optimized sparse finite-field reduction only after the baseline and dense row-reduction oracle are frozen.

Performance lanes are score-based and should not be parallelized blindly against a moving baseline.

#### RV6-D3 - Modular Gröbner over characteristic zero

Add deterministic prime selection, tracing/stabilization, CRT/reconstruction and explicit probabilistic/completeness status until a complete verification route is available.

#### RV6-D4 - Ideal operations

Add, as demanded by solving/elimination workflows:

- elimination ideals;
- ideal membership;
- intersection;
- quotient/saturation;
- dimension/standard-monomial information;
- FGLM/order conversion;
- syzygy/module machinery only when downstream operations require it.

### RV6-E - Algebraic system solving

#### RV6-E1 - Univariate exact solving

Return structured root objects with multiplicities and domain information rather than untyped expression lists.

#### RV6-E2 - Zero-dimensional polynomial systems

Implement one or more exact routes such as:

- rational univariate representation;
- triangular decomposition;
- Gröbner + FGLM + isolating intervals.

Results state completeness and multiplicity semantics explicitly.

#### RV6-E3 - Real polynomial conditions

Build staged real-algebra support:

- sign conditions;
- univariate inequalities;
- semialgebraic cells/partial CAD-style projection only when justified;
- exact decision procedures under explicit variable/degree/resource budgets.

Do not conflate this mathematical CAD (cylindrical algebraic decomposition) with CADabra's computer-aided-design geometry semantics in naming/docs.

### RV6-F - Exact and symbolic linear algebra

Add domain-generic:

- determinant;
- rank;
- row echelon/RREF;
- exact solve/nullspace;
- characteristic/minimal polynomial;
- eigenvalue algebraic representation for exact domains;
- Smith/Hermite normal forms where relevant;
- polynomial/rational-function matrix operations.

Large finite-precision iterative solves remain Methodus-owned.

### RV6-G - Rational functions, recurrences, sums and products

#### RV6-G1 - Rational functions

Provide canonical normalization, partial fractions and denominator/factor-aware operations.

#### RV6-G2 - Recurrences

Start with linear constant/polynomial-coefficient recurrences where solutions and evidence are well understood.

#### RV6-G3 - Symbolic finite sums/products

Implement staged classes rather than claiming universal summation:

- polynomial/rational sums;
- geometric/hypergeometric classes;
- telescoping/Gosper-style routes;
- explicit residual symbolic sums when unsupported.

### RV6-H - Formal series and asymptotics foundation

Broaden RV5-S5:

- formal power/Laurent series;
- composition/reversion where valid;
- algebraic series;
- asymptotic series objects with explicit expansion point/direction/order;
- series arithmetic in specialized RV2 domains.

Analytic validity claims belong in RV7 and require conditions/evidence beyond a formal expansion.

### RV6-I - Generic `solve` result model

Define structured solution-set values:

```text
FiniteSolutionSet
ParametricSolutionSet
ConditionalSolutionSet
AlgebraicSet
Interval/CertifiedSet
ResidualUnsolved
Unknown/ResourceLimited
```

A high-level `solve` dispatches through the RV3 planner and never hides omitted cases by returning a partial vector as though it were complete.

## Acceptance corpus

Use a layered corpus:

- tiny hand-computed examples;
- randomized generated algebra with construction-known answers;
- adversarial coefficient-growth/sparsity families;
- classical benchmark systems (Katsura, Cyclic, Eco, etc.) where relevant;
- external oracle comparisons;
- consumer-derived but domain-neutral cases from CADabra/Scientia/Solverang;
- mutation cases that target incompleteness, multiplicity and normalization bugs.

## Exit gate

RV6 exits at the program level when Resolvent has usable, evidenced capability in all major exact-algebra families listed above. Individual families may continue to improve after RV6; performance maturity is tracked separately from correctness support.

The standalone CAS must at minimum be able to:

- construct and convert core domains;
- factor nontrivial integer/rational/finite-field polynomials;
- compute multivariate bases/ideal operations on practical benchmark sizes;
- solve useful univariate and zero-dimensional polynomial systems exactly;
- perform exact symbolic linear algebra;
- normalize/decompose rational functions;
- manipulate formal series and common discrete sums/recurrences;
- explain algorithm selection and result completeness through RV3 artifacts.

## Parallelism

A/B/F/G/H can proceed largely in parallel once the needed RV2 domains exist. C precedes optimized D. D1 must be frozen before D2 score work. E depends on appropriate B/D foundations. Algorithm families should be split into correctness lanes and score lanes; correctness lanes can fan out aggressively, score lanes generally cannot.

## Non-goals

- unrestricted transcendental solving;
- general theorem proving;
- numerical large-system solution owned by Methodus;
- generic constraint graph solving owned by Solverang;
- physical/scientific model semantics;
- computer-aided-design geometry/topology;
- notebook UI completeness.