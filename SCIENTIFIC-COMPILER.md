# Resolvent scientific compiler

## Purpose

Resolvent turns authored scientific meaning into inspectable mathematical artifacts. It stops at
the local structured-kernel boundary. Concrete discretization, coupled runtime state, and solver
policy are separate concerns with separate owners.

The intended artifact flow is:

```text
.res source
  -> ScientificModule / ScientificModel
  -> formulation and structural analysis
  -> VariationalForm
  -> indexed tensor and local-operator factorization
  -> Malleus StructuredKernel
```

Each arrow is a compiler pass with diagnostics, source provenance, and an evidence receipt when it
changes mathematical form. These are successive artifacts, not interchangeable frontends or
temporary representations.

## Repository boundaries

### Quantitas

Owns exact dimensions, rational exponents, quantity-kind identity, units, registry provenance, and
canonical quantities. `.res` declarations carry Quantitas types directly. Resolvent does not wrap
or mirror them.

### Resolvent

Owns:

- the `.res` grammar, source spans, diagnostics, formatting, module resolution, and semantic hash;
- fields, equations, authored forms, measures, properties, constitutive laws, events, observables,
  and verification annotations;
- dimension/kind checking using Quantitas;
- formulation derivation and variational semantics;
- dependency/coupling, incidence, alias, matching, SCC/BLT, tearing, and DAE analysis;
- tensor/QFunction/local-operator factorization; and
- evidence for symbolic and semantic transformations.

Resolvent does not own meshes, global degrees of freedom, finite-element tables, assembly, time
integration, nonlinear solves, device code generation, or product orchestration.

### Malleus

Owns the schedule-independent local kernel IR, validation, derivative products, schedule choices,
and executable backends. Resolvent constructs Malleus types directly; Malleus never imports
Resolvent and has no scientific vocabulary.

### Downstream

Finitum binds form requirements and kernels to concrete meshes, spaces, basis data, quadrature,
constraints, and global operators. Krasis combines those operators with transactional coupled
state. Solverang consumes physics-neutral residual/operator traits. Sinbad selects cases, runs,
policies, and artifacts.

## Canonical semantic model

`ScientificModule` and `ScientificModel` are the only source/semantic model. The same `Expr` nodes
are referenced by equations, properties, forms, conditions, coupling analysis, differentiation,
and form compilation. Structural passes project incidence from this model; they do not translate
through a graph-compiler model.

Authored forms compile to `VariationalForm`, which retains canonical expressions and adds only
form-specific organization:

- the selected model and form identity;
- fields actually referenced by the form;
- other referenced symbols;
- measures, integrands, and their source spans.

Strong-form derivation will target the same artifact. It must emit explicit integration-by-parts,
boundary-term, sign, and assumption receipts. Until that pass exists, the compiler rejects requests
to infer a weak form rather than guessing.

## Malleus boundary

`factor_local_integral` produces a realization-neutral `LocalFormProgram`; `lower_local_program`
is the first narrow implementation of the Malleus boundary. It accepts scalar pointwise arithmetic
and common scalar functions, then constructs `malleus::StructuredKernel` directly. It rejects
`grad`, `div`, `curl`, `dot`, indexed tensors, and vector expressions.

Those operations belong to the next compiler layer:

1. classify arguments, coefficients, domains, measures, sides, and traces;
2. expand differential operators into typed indexed tensor expressions;
3. select transformations and basis evaluation requirements;
4. factor restriction, basis, geometry, pointwise QFunction, and accumulation work;
5. lower only the local numerical regions to Malleus.

No named-physics opcode is permitted. A heat, elasticity, Maxwell, or flow form must decompose into
general mathematical operations and explicit data dependencies.

## Evidence and validation

The compiler treats these as different claims:

- source accepted and resolved;
- semantic model internally valid;
- quantity and kind constraints satisfied;
- structural schedule valid;
- mathematical transformation justified;
- local kernel structurally valid;
- numerical realization verified downstream.

An earlier implementation is not an oracle. Compiler tests use grammar invariants, independent
small exhaustive oracles, analytical identities, manufactured solutions, and external fixtures as
appropriate. Removed code remains available in Git history but has no active runtime or acceptance
role.

## Immediate work

1. Split the large semantic module into coherent parser, model, property, and analysis modules
   without duplicating types.
2. Add typed expression elaboration with dimensions, value shapes, field roles, and source-rich
   diagnostics.
3. Derive variational forms from strong equations with explicit transformation receipts.
4. Define indexed tensor/QFunction IR and basis/transformation requirements.
5. Lower Poisson from `.res` through Malleus, then bind it in Finitum and solve through Solverang.
6. Add primal, JVP, VJP, and parameter-derivative kernel requests after the primal path is stable.
