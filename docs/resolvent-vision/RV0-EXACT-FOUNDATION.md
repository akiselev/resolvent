# RV0 - Exact Foundation Consolidation

## Goal

Complete the remaining CADabra R1 ownership cut so Resolvent has one mature exact/scalar foundation before its general symbolic representation changes.

RV0 deliberately does **not** redesign the existing `Expr` model. Mixing a cross-repository numeric migration with a new symbolic identity model would make regressions difficult to classify and would block CADabra R2 unnecessarily.

## Starting point

Resolvent currently has a small `num-bigint`/`num-rational` exact expression and polynomial implementation. CADabra still contains the mature generic machinery in `cadabra-exact` and `cadabra-scalar`: exact rationals, interval filters, error-free transforms, lazy exact reals, certified roots, radicals, Bernstein forms, exact matrices and dual numbers.

The active CADabra recovery plan already requires this machinery to move directly into Resolvent and the duplicate crates to be deleted. RV0 is the Resolvent side of that gate.

## Work packages

### RV0-A1 - Migration census and API freeze

Before code movement:

- enumerate public APIs and all stable/maintained CADabra call sites for `cadabra-exact` and `cadabra-scalar`;
- classify each type/function as generic algebra, geometry policy or experiment-only;
- record the existing exact/scalar tests, doctests and benchmark commands;
- identify public assertions/panics that must become validated constructors, typed errors or private proved preconditions;
- keep the current narrow Scientia `Rational` conversion API available during the cut.

Exit: one checked migration table accounts for every public item and consumer.

### RV0-A2 - Canonical rational and stable serialization

Adopt the mature CADabra arbitrary-precision rational implementation as Resolvent's canonical rational representation rather than preserving two rational stacks.

Requirements:

- dependency backend is hidden behind Resolvent-owned public types;
- exact finite `f64` admission remains available for host interop;
- exact decimal ingress is designed here or reserved explicitly for RV1 rather than conflated with binary-float ingress;
- wire serialization uses Resolvent-defined numerator/denominator canonical data, not dependency-internal serde structure;
- denominator/sign normalization is canonical;
- conversion to outward-correct float intervals is preserved and tested at normal/subnormal/overflow boundaries.

Exit: round-trip golden fixtures and the existing Scientia conversion tests pass against the new canonical type.

### RV0-B1 - Exact/filter tower transplant

Move generic modules and tests into Resolvent:

- sign and uncertainty values;
- interval arithmetic;
- error-free transforms and expansions;
- exact ring/field traits required by the imported algorithms;
- certification/filter ladder and metrics;
- lazy exact `Real` DAG and tuple formulas;
- square-root extensions and radical-sign helpers;
- exact root types, isolation/refinement and sign-at-root operations;
- Bernstein polynomial range machinery.

Geometry-specific classification, branch meaning and event policy remain in CADabra.

Exit: imported property tests and deep-DAG concurrency/teardown tests pass inside Resolvent without importing CADabra vocabulary.

### RV0-B2 - Polynomial and exact matrix consolidation

Move and reconcile:

- mature univariate rational polynomial implementation;
- root/isolation algorithms;
- exact rational matrices;
- polynomial matrices;
- resultants;
- any generic coefficient-growth metrics used by the exact stack.

Remove weaker duplicate Resolvent polynomial/root paths instead of retaining two implementations.

Bivariate/multivariate elimination remains experimental until RV5/RV6 consumers and acceptance cases justify promotion.

Exit: one public polynomial/root implementation remains.

### RV0-C1 - Scalar and differentiability seam

Move the generic scalar seam and `Dual<S>` to Resolvent.

Preserve the key property: one numeric kernel may instantiate over fast `f64`, certified exact `Real`, and dual values without cloning algorithm text.

Do not claim this seam is the complete CAS domain abstraction. RV2 introduces explicit domains/parents for rings, fields, polynomials, matrices, series and dynamic notebook values.

Exit:

- float-only code can use the seam without forcing exact computation at runtime;
- exact kernels run at `Real`;
- forward-mode exact derivatives using `Dual<Real>` retain the current certification tests.

### RV0-C2 - Budgets and typed indeterminacy

Extend `AlgebraBudget` or its successor so every potentially explosive imported operation has an explicit bound or a bounded input contract.

Examples:

- expression/work nodes;
- root refinement/isolation steps;
- coefficient bit growth;
- polynomial degree/terms;
- matrix dimension/elimination work;
- forced lazy-exact nodes.

Operations that exhaust a budget return a typed resource result. They do not guess, silently fall back to `f64`, or hang indefinitely.

### RV0-D1 - Scientia revalidation

Keep Scientia's existing narrow adapter behavior stable during RV0.

Acceptance:

- complete Scientia test suite passes;
- Sinbad's current corpus compiler checks remain green;
- no scientific semantics migrate into Resolvent;
- no second scientific expression representation is introduced in RV0.

The lossy structural limitations of the adapter are fixed in RV1, not here.

### RV0-D2 - Direct CADabra migration and duplicate deletion

Migrate stable CADabra consumers directly to public Resolvent types, including at least:

- `cadabra-number`;
- `cadabra-predicates`;
- `cadabra-arrangements`;
- `cadabra-ssi`;
- maintained algebra/geometry experiments that remain part of the recovery gate.

Then delete `cadabra-exact` and `cadabra-scalar`, remove workspace entries and remove stable direct backend dependencies that bypass Resolvent.

No adapter crate, type-alias compatibility facade or dual backend is allowed.

### RV0-E1 - Cross-repository evidence gate

Run and record:

- Resolvent formatting/check/clippy/tests/rustdoc/doctests;
- all tests/doctests migrated from the two CADabra crates (the current recovery plan records 111 as the pre-migration count);
- Scientia full suite and corpus checks;
- CADabra ordinary workspace gates;
- exact/filter benchmarks;
- arrangement filter-rate regression cases;
- licensed Parasolid oracle/integration tests when available because exact geometric decisions changed;
- downstream Finitum/Krasis/Sinbad gates required by the active federation plan.

Existing diagnostic performance references such as filtered determinant speedup and filtered predicate rates remain regression context, not claims that must be preserved byte-for-byte if the algorithm changes for a justified reason.

## Exit gate

RV0 exits only when:

- Resolvent is the sole owner of the generic exact/scalar implementation;
- one canonical rational/polynomial/root stack remains;
- stable serialized exact values do not depend on backend-private formats;
- Scientia consumes Resolvent and remains behaviorally green;
- CADabra consumes Resolvent directly;
- `cadabra-exact` and `cadabra-scalar` no longer exist;
- no compatibility layer or duplicate backend replaces them;
- CADabra R2 has a stable public algebra substrate.

## Parallelism

Within RV0, API census/serialization design can proceed alongside transplant preparation and test migration. The actual consumer cutover is serialized after the relevant types land. RV1 term work may be designed in parallel but must not merge a public term-model replacement into the same cross-repository migration cut.

## Non-goals

- new general CAS language;
- new hash-consed symbolic store;
- assumptions/rewrite engine;
- broad multivariate algebra;
- notebook/protocol work;
- geometry semantics;
- Methodus numerical algorithms;
- Solverang constraint semantics.