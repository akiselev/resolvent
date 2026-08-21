# RV7 - Calculus, Special Functions and Certified Numerics

## Goal

Add broad symbolic calculus and rigorous scalar numerical evaluation without turning Resolvent into the owner of large-system numerical simulation methods.

RV7 focuses on mathematical functions, limits, integration, transforms, series/asymptotics, exact/symbolic equation classes and arbitrary-precision/certified scalar numerics. General linear/nonlinear/least-squares/ODE/DAE/eigen/optimization/sampling algorithms remain Methodus-owned.

## Principles

- Function semantics include domains, branch conventions and singularities.
- Exact symbolic answers may be conditional.
- Unsupported classes remain explicit residual expressions or `Unknown`; there is no requirement to fake a closed form.
- Arbitrary-precision approximate results carry precision/error semantics.
- Rigorous numerical results use enclosures/certificates rather than ordinary floating-point comparisons.
- Symbolic and numerical routes are planned independently and may cooperate through explicit fallback plans.

## Work packages

### RV7-A - Elementary and special function semantics

#### RV7-A1 - Function catalog

Expand the RV3/RV4 operation catalog with explicit semantics for:

- exp/log;
- trigonometric/hyperbolic and inverse functions;
- powers/roots;
- abs/sign/floor/ceiling;
- gamma/beta and related functions;
- common orthogonal polynomials;
- error functions;
- Bessel families;
- polylog/zeta families;
- other functions only as implementation/evidence becomes available.

Each entry records:

- domains and singularities;
- principal branch convention;
- conjugation/parity properties;
- exact special values;
- derivative rules;
- series/asymptotic hooks;
- arbitrary-precision evaluation hook;
- simplification identities with guards.

### RV7-A2 - Certified elementary evaluation

Implement arbitrary-precision real/complex evaluation with either native ball arithmetic or an optional validated provider behind Resolvent semantics.

Acceptance includes:

- precision escalation;
- enclosure containment against high-precision independent references;
- branch-cut fixtures;
- cancellation/near-singularity cases;
- deterministic precision planning.

### RV7-B - Limits and local analysis

#### RV7-B1 - Algebraic/rational limits

Provide exact limits for polynomial/rational/algebraic classes, including one-sided limits and infinity where mathematically defined.

#### RV7-B2 - Series-based elementary limits

Use RV6 formal series plus function series to decide broader limits.

Conditions and direction are first-class. Failure to prove required dominance/order returns a residual/unknown result.

#### RV7-B3 - Asymptotic expansions

Add asymptotic scales and expansion direction to RV6's series objects.

Do not treat a formal local expansion as an analytic asymptotic statement without the additional validity contract.

### RV7-C - Symbolic integration

Integration grows in capability tiers.

#### RV7-C1 - Rational integration

Implement Hermite reduction/partial-fraction-based rational integration with the strongest straightforward verification gate: differentiate the returned antiderivative and compare in the rational-function domain.

#### RV7-C2 - Algebraic-function integration

Add supported algebraic extensions/classes as RV6 domains mature.

#### RV7-C3 - Rule-based elementary integration

Introduce a versioned guarded integration-rule catalog for common elementary forms.

Each successful result is graded by differentiation under its stated conditions. Rule provenance and applied-rule traces are recorded.

#### RV7-C4 - Risch-style staged integration

Pursue increasingly complete elementary integration as separate subfamilies rather than one all-or-nothing milestone.

Capability reporting should identify the supported differential-field class. A residual unevaluated integral is preferable to a false elementary answer.

#### RV7-C5 - Definite integration

Add definite integrals for supported symbolic classes with:

- endpoint/singularity analysis;
- convergence conditions;
- branch conventions;
- conditional results;
- certified numerical fallback only when explicitly requested or when the high-level operation permits it as a distinct outcome.

### RV7-D - Symbolic transforms

Build staged families for:

- Laplace/inverse Laplace;
- Fourier/inverse Fourier;
- z transforms;
- Mellin transforms;
- convolution/transform identities.

Results preserve regions/conditions of convergence and do not erase distributional/generalized-function conditions.

### RV7-E - Symbolic differential equations

Resolvent may solve symbolic ODE/recurrence classes whose answers are mathematical expressions/solution sets.

Start with:

- separable first-order ODEs;
- first-order linear ODEs;
- constant-coefficient linear ODEs;
- Euler/Cauchy equations;
- selected exact/Bernoulli/Riccati-reducible classes;
- linear systems where exact symbolic matrix machinery applies;
- series solutions around ordinary/regular singular points.

General numerical ODE/DAE integration belongs to Methodus. Scientia owns the interpretation of scientific field equations and time/state semantics.

### RV7-F - Certified numerical scalar solving

Provide mathematically local certified numerical tools:

- interval/ball root isolation/refinement for scalar real functions under enclosable derivatives;
- arbitrary-precision complex root refinement with explicit certification level;
- certified quadrature for scalar integrands;
- rigorous extrema/range bounds for bounded scalar domains where feasible.

These are scalar mathematical operations. They are not substitutes for Methodus's large nonlinear systems/time integrators.

### RV7-G - Symbolic-numeric equation solving

High-level scalar/small algebraic `solve` may combine:

- exact preprocessing;
- domain decomposition;
- certified numeric isolation;
- arbitrary-precision refinement;
- residual symbolic conditions.

The RV3 plan exposes whether each result is exact, certified numerical, approximate or unresolved.

### RV7-H - Differentiation breadth

Complete common symbolic calculus support:

- higher derivatives;
- multivariate differential operators over scalar algebra;
- implicit differentiation as an algebraic transformation with explicit equations/variables;
- parameter derivatives;
- series coefficient differentiation;
- special-function derivative identities;
- differentiation under sums/products/integrals only under explicit sufficient conditions.

Scientia remains the owner of field/tensor/spatial scientific operator semantics and moving-domain derivative conventions.

### RV7-I - External validation matrix

For each function/calculus family, use suitable independent systems:

- exact algebra comparison in SymPy/Sage/Maple/Mathematica;
- high-precision value comparison against multiple implementations;
- ball/enclosure containment against independent high-precision samples;
- branch-cut/singularity corpora;
- differentiation/substitution verification of symbolic results;
- convergence-condition mutation tests.

Textual equivalence is never the only oracle for mathematically equivalent forms.

## Exit gate

RV7 exits when Resolvent provides a useful broad calculus surface with explicit capability boundaries:

- elementary/special function metadata and arbitrary-precision evaluation;
- common exact/series-based limits;
- robust rational integration plus meaningful elementary integration coverage;
- transform support for common classes;
- common symbolic ODE classes;
- certified scalar root/quadrature operations;
- conditions/branches/exactness exposed in outcomes and receipts;
- no numerical Methodus functionality duplicated inside the CAS.

## Parallelism

A function catalog is shared infrastructure. After that, A2/B/C/D/E/F lanes can proceed by family. C1 rational integration should precede rule-heavy integration because it provides a strong exact baseline. Certified numerical work may proceed alongside symbolic integration after RV2 approximate/ball domains exist.

## Non-goals

- claiming a complete decision procedure for transcendental equality;
- making full Risch completion a gate for an otherwise useful CAS;
- large sparse numerical linear solvers;
- numerical nonlinear/least-squares algorithms;
- production numerical ODE/DAE integrators;
- optimization/UQ/sampling algorithms;
- constraint-system solving;
- scientific PDE semantics.