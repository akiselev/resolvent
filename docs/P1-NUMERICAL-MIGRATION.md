# P1 numerical migration: Resolvent ↔ Residua

This document is the numerical companion to ADR-035 and `STACK-LOSS-AUDIT.md`.
It records exactly what the first continuum migration wave proves, and just as importantly,
what it does **not** prove.

## Canonical boundary after this wave

Resolvent owns the mathematical meaning of the first scalar finite-element vertical:

```text
FormProgram
    ↓ validate scalar Lagrange P1 / H1 semantics
P1DiscretizationRequest
    ↓ deterministic reference lowering
DiscreteProgram (stiffness + optional mass)
    ↓
OperatorProgram
    ↓
ScalarEllipticAssembly / MassAssembly / EvolutionAssembly
```

The concrete reference lowering covers:

- conforming 2-D triangular P1 elements;
- scalar H1 diffusion with piecewise-constant per-region coefficients;
- deterministic vertex-order DOF numbering;
- constant Dirichlet data and static condensation;
- piecewise-constant volumetric source loading;
- constant Neumann edge flux loading;
- full-space and condensed deterministic CSR stiffness matrices;
- consistent P1 capacity mass;
- row-sum lumped capacity mass;
- the semi-discrete linear evolution `M·u_dot + K·u = f`;
- residual, stiffness JVP/VJP, mass action, and shifted implicit matrices;
- semantic `DiscreteProgram` / `OperatorProgram` artifacts and a versioned
  `RefinementRelation::Discretization` receipt.

Resolvent is **not** the linear solver or time integrator. Solverang still owns solve/time/
optimization policy. Resolvent is **not** a machine-code backend. Anvil still owns executable
lowering/JIT.

## Determinism and matrix semantics

The reference CSR builder performs a stable `(row,column)` sort. Duplicate element
contributions therefore remain in assembly insertion order and are summed in that order.
This is intentionally stricter than merely defining the same real-valued operator: it gives
migration tests a reproducible floating-point artifact to compare.

Zero entries are omitted after summation. Matrix-vector application walks rows and columns in
stored CSR order. `scaled_sum` re-scatters inputs in term order, so shifted operators such as
`(c0/h) M + K` also have deterministic accumulation semantics.

## Definiteness claims are conservative

The operator metadata must never infer a stronger mathematical property merely because a
matrix happened to look well behaved on one fixture.

For the diffusion block:

- a negative or non-finite coefficient causes Resolvent to make **no** positive-definiteness
  claim;
- non-negative coefficients with any zero value are labelled positive-semidefinite at most;
- strictly positive coefficients are labelled positive-definite only when every connected mesh
  component contains at least one constrained Dirichlet vertex;
- otherwise the block is labelled positive-semidefinite.

The disconnected-component and negative-coefficient cases have regression tests.

## Differential gate in Sinbad

Sinbad PR #6 contains a dev-only differential oracle against the incumbent Residua
implementation. A single shared `MeshTopology` fixture is converted to Resolvent's portable
`P1Mesh` without changing ordering. It contains four triangles, two material regions, two
Dirichlet boundaries, volumetric forcing, and top/bottom Neumann flux.

The gate compares **bitwise f64 results**, not tolerance-based summaries, for:

1. full stiffness matrix;
2. free/free stiffness matrix;
3. Dirichlet static-condensation RHS;
4. volumetric source full/free vectors;
5. Neumann full/free vectors;
6. complete condensed RHS;
7. incumbent algebraic residual;
8. JVP;
9. VJP;
10. consistent full/free mass matrices;
11. lumped full/free mass matrices;
12. BDF shifted iteration matrix `(c0/h) M + K`.

At the time this document was written all three differential test groups pass bitwise.

Because both repositories are private and GitHub's repository-scoped Actions token cannot
fetch a sibling private repository, Sinbad CI carries a frozen **dev-only** snapshot of the
three numerical source files (`mesh.rs`, `matrix.rs`, `assembly.rs`). The snapshot records the
green Resolvent commit and each upstream git-blob ID. Production Sinbad does not link the
snapshot. The semantic form/compiler path remains tested in Resolvent's own CI.

## What remains incumbent in Residua

Passing this gate is evidence for one well-defined vertical. It is not permission to delete the
entire Residua crate. The following capabilities remain incumbent or unproved:

- arbitrary/state-dependent/nonlinear material coefficients;
- time-dependent forcing and nonconstant Dirichlet data;
- Robin/interface terms;
- reaction/advection operators;
- singular-mass DAE cases and index semantics beyond the nonsingular parabolic ODE case;
- 3-D tetrahedral discretization;
- vector/tensor H1 spaces and linear elasticity;
- mixed spaces and incompressible Stokes/Navier–Stokes blocks;
- H(curl) / curl-curl electromagnetics;
- matrix-free/quadrature-point execution;
- differentiated coefficient assembly (`∂R/∂p`);
- full transient adjoint march/checkpoint/event saltation;
- field extractors and production result projection;
- existing physics-corpus convergence/MMS gates.

## Deletion / ownership-flip rule

The first ownership flip may replace duplicated **scalar P1 assembly primitives** with the
Resolvent implementation while leaving a compatibility implementation available for
differential testing. Before deleting an incumbent path, reviewers must have artifacts for all
of the following:

- same frozen inputs exercised by old and new implementations;
- value parity, including RHS sign conventions and boundary contributions;
- equivalent failure/degeneracy behavior;
- deterministic ordering and sparsity behavior;
- JVP/VJP parity;
- transient shifted-operator parity where relevant;
- at least one manufactured-solution convergence test through the new production path;
- at least one real Sinbad physics consumer using the new production path;
- no downstream imports of an incumbent-only API.

Until those gates exist, Residua remains a compatibility/reference implementation rather than
being deleted on architectural grounds alone.

## Next witnesses

The next two useful witnesses are intentionally dissimilar:

1. **nonlinear transient heat** — state-dependent conductivity/capacity, automatic residual
   differentiation, MMS generation, JVP/VJP validation, and Solverang time integration;
2. **mixed Stokes or linear elasticity** — stresses vector/tensor fields, block structure,
   mixed function spaces, and the point at which scalar-P1 assumptions must stop leaking into
   the general form/operator IR.

A successful architecture should add genuinely new primitives for these witnesses without
creating a second symbolic/form/operator universe.
