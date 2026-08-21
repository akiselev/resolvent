# Resolvent federation ownership

This document is authoritative for ownership boundaries used by the Resolvent Vision roadmap. Sequencing across consumer roadmaps is defined by [`CROSS-ROADMAP-CONTRACT.md`](CROSS-ROADMAP-CONTRACT.md).

## Principle

Resolvent owns mathematics that remains meaningful when every current consumer name is removed.

A capability belongs in Resolvent when its contract can be stated entirely in mathematical terms such as rings, fields, expressions, equations, assumptions, exactness, approximation, matrices, polynomials, ideals, series or certificates.

A capability does not belong in Resolvent merely because another repository needs mathematics to implement it.

## Repository ownership

### Resolvent

Owns:

- retained structural symbolic term representation;
- exact and arbitrary-precision scalar mathematics;
- interval/ball/certified scalar evaluation;
- algebraic domains and canonical coercions;
- polynomial, rational-function, matrix, ideal and series algorithms;
- generic symbolic differentiation/calculus;
- algebraic equation solving;
- assumptions and generic mathematical rewriting;
- operation/rule/algorithm catalogs;
- budgets, plans, outcomes, receipts and mathematical certificates;
- algebraic expression optimization, including CSE/Horner/factorization and explicit temporary/let schedules;
- CAS session and kernel protocol semantics.

Does not own:

- physical units or dimensions;
- `.res` source semantics or Scientia's canonical scientific expression/semantic arena;
- CAD geometry or topology;
- finite-element/mesh realization;
- executable numeric/kernel IR or backend lowering;
- general numerical solver policy;
- constraint-system semantics;
- coupled simulation state;
- product campaigns or promotion policy;
- generic external-executable plugin lifecycle;
- general durable artifact/provenance lifecycle.

### Quantitas

Owns dimensions, units, quantity kinds, standards snapshots and consumer-neutral measurement metadata.

A Resolvent scalar may be used inside a Quantitas quantity, but Resolvent does not learn what temperature, pressure or length means.

### Scientia

Owns:

- `.res` syntax/module semantics and authored source-literal meaning;
- one canonical `SemanticModel` arena and its scientific `ExprId` identity;
- scientific fields, parameters, sources, properties and constitutive laws;
- equations and forms;
- differential/spatial scientific operators and scientific tensor meaning;
- method-family programs;
- scientific derivative requests, objectives, controls, ports and verification obligations;
- moving-domain/contact/event meaning at the scientific-model level.

Scientia delegates generic algebra through **operation-specific projections** of supported scalar subexpressions into Resolvent Terms/domain elements. Resolvent results are then re-embedded or attached as evidence under Scientia-owned semantic identity.

Resolvent Terms do not replace Scientia's canonical expression arena, and Resolvent arena-local handles never become scientific declaration/expression identity.

Because Scientia owns `.res` parsing, Scientia also owns preserving exact authored numeric literals at the source/schema boundary. Resolvent supplies exact decimal/rational atoms and algebra after projection; it cannot reconstruct lexical exactness once the compiler has reduced a literal to `f64`.

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

A reusable algebra operation discovered by CADabra R2-R7 can start its `RV5-C*` work package as soon as that operation's actual lower-level prerequisites exist. CADabra does not wait for unrelated RV phase exits.

### Malleus

Owns:

- finite-precision structured local computation/IR;
- iteration domains and affine operand/index maps;
- reductions/effects;
- AD execution products;
- scheduling and backend-oriented lowering;
- portable generated numerical kernels.

Resolvent may emit optimized algebraic Terms/domain elements and explicit CSE/let schedules. It does **not** create a second general numeric SSA/kernel IR below Scientia. If an algebraic optimization must cross into code generation, Scientia/Malleus lower the optimized algebra into Malleus-owned structured computation.

### Methodus

Methodus is the consumer-neutral numerical-methods library split from Solverang.

Owns:

- matrix-free linear operator and preconditioner contracts;
- nonlinear residual/JVP contracts;
- least-squares residual/Jacobian contracts;
- DAE operator contracts and numerical event stepping;
- block layouts and numerical coupling/preconditioning policies;
- linear, nonlinear, least-squares, time-integration and other numerical algorithms as they are added;
- future physics-neutral Krylov, eigen, optimization, sampling, model-reduction and related numerical method families when concrete consumers require them.

Resolvent may provide exact small-matrix baselines, symbolic Jacobians, expression optimization or certified scalar subroutines useful to Methodus. It does not own Methodus solve policy, convergence criteria, iteration histories, numerical event stepping or large-system numerical execution.

### Solverang

Solverang has returned to its constraint-solving roots and consumes Methodus for numerical methods.

Owns:

- generic variable/constraint graphs;
- equality and inequality activation;
- candidate constraint solving orchestration;
- rank and degree-of-freedom analysis;
- redundancy/conflict diagnostics;
- constraint-specific derivative checking;
- reusable geometric constraint vocabulary;
- `solverang-geometry-2d` primitives/constraints;
- `solverang-geometry-3d` primitives/constraints.

Resolvent may provide polynomial normalization, exact algebraic subproblems, symbolic elimination, or generic algebraic witnesses consumed by Solverang. Resolvent does not decide that a witness constitutes a constraint conflict, redundancy, remaining DOF, or activation state; those are Solverang semantics.

CADabra remains authoritative for whether a Solverang candidate is geometrically valid and may independently certify or refuse it before committing topology.

### Finitum

Owns meshes, finite spaces, DOFs, mesh/topology realization, basis/quadrature execution, **discretization/DOF constraints**, assembly, matrix-free global operators, transfer, transpose realization, geometry association and adaptivity.

Finitum's discretization constraints are distinct from Solverang's user/model constraint graphs.

### Krasis

Owns coupled runtime state, transactions, history, events, checkpoints and coupled operator composition.

### Sinbad

Owns simulation cases, studies, campaigns, catalogs, orchestration, results, comparison, support claims, evidence promotion and product UX.

Sinbad should normally receive symbolic/scientific artifacts through Scientia. It should not grow a parallel CAS expression representation.

### Outboard

Outboard is the existing generic external executable-plugin framework. It owns reusable executable discovery, manifest/version compatibility, typed invocation, persistent-worker lifecycle, progress/cancellation and process isolation.

Resolvent owns the mathematical provider contract and may use Outboard through an optional adapter when an out-of-process algorithm/provider needs those generic lifecycle capabilities. Resolvent should not rebuild an Outboard-equivalent plugin host in the CAS core.

### Artifactum

Artifactum is the existing local-first durable artifact lifecycle system. It owns reusable content-addressed storage, immutable large artifacts, action/execution history, lineage, verification, distribution and remote artifact handling.

Resolvent may own ephemeral/local mathematical memoization and cache keys. Durable large certificates, provider outputs, cross-repository lineage and distributable evidence may use Artifactum through an optional adapter when needed. Resolvent should not recreate a general durable artifact lifecycle system in the CAS core.

Neither Outboard nor Artifactum is a mandatory dependency of core Resolvent.

## Consumer matrix

| Capability | Resolvent | Scientia | CADabra | Methodus | Solverang |
|---|---:|---:|---:|---:|---:|
| exact rational/algebraic scalar | owner | consume via algebra projection | consume | optional consume | optional consume |
| structural symbolic Term store | owner | project to/from; not semantic identity | optional consume | optional consume | optional consume |
| scientific `SemanticModel` / `ExprId` | no | owner | no | no | no |
| units/dimensions | no | consume Quantitas | consume Quantitas as needed | no semantic ownership | targets only as consumer data |
| symbolic differentiation | generic owner | scientific request/interpretation | geometry-specific interpretation | numeric derivatives remain Methodus | constraint derivative semantics remain Solverang |
| polynomial/resultant/root algebra | owner | consume | consume | optional baseline | optional exact subproblem |
| executable local numeric IR | no | lower toward Malleus | no | numerical operators only | no | 
| numerical linear/nonlinear/DAE algorithms | no | no | no | owner | consume |
| generic constraint graph | no | no | no | no | owner |
| conflict/redundancy/DOF semantics | no | no | no | no | owner |
| 2-D/3-D geometric constraint vocabulary | no | no | CAD entity mapping/authority | no | owner |
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
7. Would implementing it create a second scientific semantic IR, executable numeric IR, plugin host, or durable artifact system already owned elsewhere?

If the answer to 1, 2 or 5 is no, the capability stays with the consumer. If 7 is yes, define only the Resolvent-specific semantic adapter and reuse the existing owner.

## Extension direction

Frontend integration may make the system look monolithic while implementation ownership remains federated. A Resolvent notebook can expose commands backed by Scientia, CADabra, Methodus, Solverang or Sinbad through extension packages. That does not move their semantics into the CAS kernel.

The kernel protocol should preserve provider/extension identity in results and receipts so notebook convenience never obscures which repository made an authoritative decision.
