# Resolvent federation ownership

This document is authoritative for ownership boundaries used by the Resolvent Vision roadmap.

## Principle

Resolvent owns mathematics that remains meaningful when every current consumer name is removed.

A capability belongs in Resolvent when its contract can be stated entirely in mathematical terms such as rings, fields, expressions, equations, assumptions, exactness, approximation, matrices, polynomials, ideals, series or certificates.

A capability does not belong in Resolvent merely because another repository needs mathematics to implement it.

## Repository ownership

### Resolvent

Owns:

- retained symbolic term representation;
- exact and arbitrary-precision scalar mathematics;
- interval/ball/certified scalar evaluation;
- algebraic domains and canonical coercions;
- polynomial, rational-function, matrix, ideal and series algorithms;
- generic symbolic differentiation/calculus;
- algebraic equation solving;
- assumptions and generic mathematical rewriting;
- operation/rule/algorithm catalogs;
- budgets, plans, outcomes, receipts and mathematical certificates;
- neutral symbolic/numeric-program preparation before backend-specific lowering;
- CAS session and kernel protocol semantics.

Does not own:

- physical units or dimensions;
- scientific source/model semantics;
- CAD geometry or topology;
- finite-element/mesh realization;
- general numerical solver policy;
- constraint-system semantics;
- coupled simulation state;
- product campaigns or promotion policy.

### Quantitas

Owns dimensions, units, quantity kinds, standards snapshots and consumer-neutral measurement metadata.

A Resolvent scalar may be used inside a Quantitas quantity, but Resolvent does not learn what temperature, pressure or length means.

### Scientia

Owns:

- `.res` syntax/module semantics;
- scientific fields, parameters, sources, properties and constitutive laws;
- equations and forms;
- differential/spatial scientific operators and scientific tensor meaning;
- method-family programs;
- scientific derivative requests, objectives, controls, ports and verification obligations;
- moving-domain/contact/event meaning at the scientific-model level.

Scientia may reference Resolvent terms for scalar algebra and delegate generic algebraic transformations. Source spans and scientific identities stay in Scientia sidecars/artifacts.

Resolvent must not acquire a second `.res` parser or a scientific semantic IR.

### CADabra3

Owns:

- CAD geometry and topology;
- checked-number policy at the geometry boundary;
- geometric predicates and certification policy;
- curve/surface carriers, sheets and branch semantics;
- SSI completeness/coverage semantics;
- p-curves, trims and arrangements;
- B-rep topology and persistent entity identity;
- geometric event classification and topology-affecting decisions;
- design-velocity meaning attached to geometry entities.

R1 already moved CADabra's generic exact/scalar machinery into Resolvent and deleted the old `cadabra-exact` and `cadabra-scalar` crates. `cadabra-number` and other geometry-facing policy remain in CADabra and consume public Resolvent APIs. RV0 hardens this landed boundary; it does not repeat the migration.

A resultant or algebraic-root operation used by an SSI algorithm may belong in Resolvent. The interpretation of those roots as intersection branches does not.

### Malleus

Owns finite-precision local kernel IR, scheduling, backend-oriented lowering, AD products and generated numerical kernels.

Resolvent may emit a neutral symbolic/numeric program or optimized expression graph. Malleus owns target-oriented executable lowering.

### Methodus

Methodus is the consumer-neutral numerical-methods library split from Solverang.

Owns:

- matrix-free linear operator and preconditioner contracts;
- nonlinear residual/JVP contracts;
- least-squares residual/Jacobian contracts;
- DAE operator contracts and numerical event stepping;
- block layouts and numerical coupling/preconditioning policies;
- linear, nonlinear, least-squares, time-integration and other numerical algorithms as they are added.

Resolvent may provide exact small-matrix baselines, symbolic Jacobians, expression optimization or certified scalar subroutines useful to Methodus. It does not own Methodus solve policy, convergence criteria, iteration histories or large-system numerical execution.

General-purpose numerical ODE/DAE, nonlinear, eigen, optimization, sampling and related method families belong in Methodus as that library grows, not in Solverang and not in Resolvent.

### Solverang

Solverang has returned to its constraint-solving roots and consumes Methodus for numerical methods.

Owns:

- generic variable/constraint graphs;
- equality and inequality activation;
- candidate constraint solving orchestration;
- rank and degree-of-freedom analysis;
- conflict diagnostics;
- constraint-specific derivative checking;
- reusable geometric constraint vocabulary;
- `solverang-geometry-2d` primitives/constraints;
- `solverang-geometry-3d` primitives/constraints.

Resolvent may provide polynomial normalization, exact algebraic subproblems, symbolic elimination or certificates that are genuinely generic. It does not own constraint graph semantics, geometric constraint vocabulary, candidate acceptance policy or conflict/DOF meaning.

CADabra remains authoritative for whether a Solverang candidate is geometrically valid and may independently certify or refuse it before committing topology.

### Finitum

Owns meshes, finite spaces, DOFs, topology, basis/quadrature realization, constraints, assembly, matrix-free global operators, transfer, transpose realization, geometry association and adaptivity.

### Krasis

Owns coupled runtime state, transactions, history, events, checkpoints and coupled operator composition.

### Sinbad

Owns simulation cases, studies, campaigns, catalogs, orchestration, results, comparison, support claims, evidence promotion and product UX.

Sinbad should normally receive symbolic/scientific artifacts through Scientia. It should not grow a parallel CAS expression representation.

## Consumer matrix

| Capability | Resolvent | Scientia | CADabra | Methodus | Solverang |
|---|---:|---:|---:|---:|---:|
| exact rational/algebraic scalar | owner | consume | consume | optional consume | optional consume |
| symbolic term store | owner | consume/reference | optional consume | optional consume | optional consume |
| units/dimensions | no | consume Quantitas | consume Quantitas as needed | no semantic ownership | targets only as consumer data |
| symbolic differentiation | generic owner | scientific request/interpretation | geometry-specific interpretation | numeric derivatives remain Methodus | constraint derivative semantics remain Solverang |
| polynomial/resultant/root algebra | owner | consume | consume | optional baseline | optional exact subproblem |
| numerical linear/nonlinear/DAE algorithms | no | no | no | owner | consume |
| generic constraint graph | no | no | no | no | owner |
| 2D/3D geometric constraint vocabulary | no | no | CAD entity mapping/authority | no | owner |
| CAD topology/events | no | no | owner | no | no |
| scientific forms/method semantics | no | owner | no | numerical realization only | no |

## Placement tests

Before moving a new capability into Resolvent, answer:

1. Can the public contract be written without naming a consumer domain?
2. Is the result meaningful independently of geometry, physics, constraints and simulation runtime?
3. Does the operation have a mathematical exactness/approximation contract Resolvent can own?
4. Can its resource behavior be bounded or explicitly reported?
5. Can correctness be graded independently of the consumer's semantic acceptance?
6. Is there a second consumer/use case, or is the mathematical domain itself sufficiently general to justify ownership?

If the answer to 1, 2 or 5 is no, the capability stays with the consumer.

## Extension direction

Frontend integration may make the system look monolithic while implementation ownership remains federated. A Resolvent notebook can expose commands backed by Scientia, CADabra, Methodus, Solverang or Sinbad through extension packages. That does not move their semantics into the CAS kernel.

The kernel protocol should preserve provider/extension identity in results and receipts so notebook convenience never obscures which repository made an authoritative decision.