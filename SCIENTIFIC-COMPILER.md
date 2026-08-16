# Resolvent scientific compiler

This document is the operational architecture introduced by ADR-035. The older algebra
DESIGN/API documents remain authoritative for their exact-algebra decisions; this document
owns the new cross-stack pipeline.

## The loop, not merely the stack

```text
literature / theory / experiment
           |
        Pi Lab
           |
   Ferris-Howard syntax
           |
         Lean  <---------------- Lean Atlas
           |
   checked reification theorem
           |
       Resolvent ScientificSpec
           |
   +-------+-------------------------------+
   | exact/symbolic  structural  forms     |
   | algebra          passes      (PDE)    |
   +--------------------+------------------+
                        |
                  refinement chain
                        |
                 Discrete / Operator
                   /             \
                Anvil          Solverang
                   \             /
                       Sinbad
                         |
                    observables
                         |
                     Validator
                    /         \
                formal       empirical
                  |             |
                  +---- Pi Lab -+
```

The same architecture also supports exact campaigns that never need Sinbad:

```text
Lean claim -> Resolvent polynomial/semialgebraic problem -> certificate/counterexample
          -> Lean checker -> promoted Pi Lab evidence
```

## Dialects

### `expr`

Generic mathematical terms, exact literals, symbols, functions and semantic derivatives.
No mesh, solver or physics vocabulary belongs here.

### `model`

Equations, variables, parameters, events, hierarchical systems, assumptions, scope,
observables and theorem/property contracts. This is the common language for acausal DAEs,
circuits, algebraic constraints and continuum models.

### `structural`

Read-only projections of `model::System`: incidence, matching and eventually the full
BLT/tearing/Pantelides/dummy-derivative pipeline. Structural analysis does not own a second
equation AST.

### `form`

The continuum/variational dialect: fields, function spaces, grad/div/curl, traces, trial/test
roles, measures and integrals. Only models that need continuum discretization visit it.

### `discrete`

A structured finite-dimensional dialect inspired by the restriction/basis/pointwise/integrate
factorization: element restriction, basis interpolation/derivatives, quadrature-point
physics, transpose basis action and scatter. Keeping these pieces explicit permits assembled,
partial-assembly and matrix-free realizations of one discretization.

### `operator`

Solver-facing residual/mass/damping/stiffness/constraint blocks; derivative capabilities;
sparsity; nullspaces; conservation/symmetry declarations. Solver strategy is intentionally
absent.

### Anvil (external)

Machine computation: `f32/f64`, memory, target instructions, FMA choices, vectorization,
scheduling, executable JVP/VJP. Exact mathematical rewriting and floating-point performance
rewriting remain different systems.

## The differentiation ladder

Resolvent deliberately has several derivative meanings:

1. expression differentiation: `df/dx`;
2. system differentiation: semantic `d/dt`, derivative variables, index reduction;
3. form differentiation: Gateaux/variational derivative and weak-form linearization;
4. discrete differentiation: Jacobian/JVP/VJP of the finite-dimensional operator;
5. Anvil AD: executable computational derivative.

Adjacent levels should be cross-checked rather than collapsed.

## Scientific specification

A `ScientificSpec` contains the system plus its assumptions, declared scope, observables and
property contracts. It is deliberately larger than a PDE or an equation list. In particular,
experimental validation occurs against an `Observable` and its measurement model, not an
arbitrary internal field.

## Evidence and promotion

Formal, numerical and empirical evidence are orthogonal:

- formal: unchecked -> asserted -> certificate checked -> kernel proved;
- numerical: untested -> replayed -> differential -> convergence -> bounded;
- empirical: no data -> retrospective -> independently replicated -> prospective.

The enums are deliberately not one ordered type. Consumers may impose campaign-specific
promotion policies, but cannot claim that one axis substitutes for another.

## Compatibility migration

### Residua

Current Residua assembly/evolution/adjoint code remains Sinbad's reference backend while the
`form -> discrete -> operator` compiler matures. First migration target: scalar elliptic +
mass/evolution, differential-tested bit-for-bit against current P1 behavior.

### Plexus

Current Plexus matching/SCC/BLT/tearing remains the reference implementation. Resolvent now
derives incidence directly from its common `System` IR and has deterministic maximum
matching. SCC/BLT/tearing migrate next, followed by symbolic `d/dt`, Pantelides and dummy
derivatives. Sinbad only becomes a compatibility facade after differential tests pass.

### Solverang

Solverang remains a high-level product. A dependency-neutral symbolic sink/diagnostic seam
lets an optional Resolvent adapter expose symbolic residuals and exact generic-rank/
certificate capabilities without forcing every Solverang user to compile the CAS.

## Required vertical falsification cases

The architecture is not frozen merely because the types compile. It must survive:

1. nonlinear transient heat;
2. incompressible Stokes/Navier-Stokes;
3. high-index RLC/mechanical DAE;
4. Solverang geometric constraint cluster;
5. Maxwell H(curl) immediately after the first four;
6. an exact algebraic campaign such as 3HDM BFB that never enters the simulator.

A new domain-specific lower-level escape hatch is evidence that the architecture needs
revision, not a reason to add a permanent special case.
