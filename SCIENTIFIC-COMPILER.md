# Resolvent scientific compiler

## Purpose

Resolvent turns authored scientific meaning into inspectable mathematical artifacts. It stops at
the local structured-kernel boundary. Concrete discretization, coupled runtime state, and solver
policy are separate concerns with separate owners.

The intended artifact flow is:

```text
.res source
  -> ScientificModule / ScientificModel
  -> SemanticModule / SemanticModel arena
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

## Source and canonical semantic model

`ScientificModule`, its contained source models, and `Expr` retain authored syntax and provenance.
They are parser output, not a second type system. `compile_semantics` resolves that syntax into one
`SemanticModule` / `SemanticModel` arena. Every semantic expression has an arena identity, precise
span, resolved symbol identity, value shape and axes, a Quantitas `Dimension` and
`QuantityKindId` when constrained, a domain frame, and a distinct scientific role.

Unspecified external scientific-function signatures remain explicitly `deferred`. Known facts are
still enforced around them; no scalar, unit, axis, or frame meaning is invented. Stable diagnostic
codes and byte spans cover parsing, module imports, names, domains, units, quantity kinds, roles,
axes, shapes, dimensions, and frames. Semantic arena digests exclude presentation spans.

Authored forms and derived strong equations compile to `VariationalForm`, which retains canonical
expressions and adds only form-specific organization:

- the selected semantic declaration and parent semantic digest;
- test/trial arguments and captures keyed by `SymbolId`, with roles, types, spaces, and domains;
- cell, exterior/interior-facet, interface, and point measures keyed by `DomainId` or `RegionId`;
- integrands keyed by `ExprId`; and
- an artifact digest and receipt recording the source declaration and transformation history.

Strong-form derivation residualizes equations, selects or accepts an explicit physical field for
the test space, applies integration by parts only when that space supports the resulting
derivative, and records boundary terms as retained, substituted, or eliminated by an essential
condition. The receipt explicitly assumes that its resolved exterior regions partition the domain
boundary; Finitum must discharge that assumption against topology. A Neumann value may substitute
only one integrated flux per region and field until the language can express per-term
correspondence. Ambiguous test spaces or fluxes, missing boundary partitions, invalid derived
differentials, curl orientation requirements, and unsupported Robin flux laws return stable
capability errors rather than guessed forms.

Differential, contraction, tensor/facet trace, jump, average, normal-component, and conjugation
operations are typed semantic nodes rather than string opcodes. Two-sided facets require explicit
minus/plus restrictions (or jump/average). The deterministic form interpreter evaluates these
nodes over caller-supplied point values, derivatives, traces, normals, and weights; constructing
quadrature remains a Finitum responsibility. `required_evaluations` exposes the expression and
evaluation-site bindings without requiring consumers to traverse the semantic arena. Form receipts
record an explicit-conjugation-only convention; derivation inserts no implicit complex conjugate.

## Malleus boundary

`factor_local_integral` produces a realization-neutral `LocalFormProgram`; `lower_local_program`
is the first narrow implementation of the Malleus boundary. It accepts scalar pointwise arithmetic
and common scalar functions, then constructs `malleus::StructuredKernel` directly. Local inputs
retain test-basis, trial-basis, physical-field, parameter, constant, source, property, and
constitutive-law roles plus required value/gradient/time-derivative/trace evaluations. It rejects
`grad`, `div`, `curl`, `dot`, indexed tensors, and vector expressions until the basis/tensor pass
has expanded them.

The local artifact is explicitly a one-quadrature-point QFunction. Finitum owns quadrature choice
and traversal; Malleus's empty iteration domain therefore means one invocation, never an omitted
symbolic loop. See [ITERATION-OWNERSHIP.md](ITERATION-OWNERSHIP.md).

The remaining tensor/QFunction work belongs to the next compiler layer:

1. expand differential operators into typed indexed tensor expressions;
2. select basis, mapping, orientation, and quadrature requirements;
3. factor restriction, basis, geometry, pointwise QFunction, and accumulation work;
4. lower only the local numerical regions to Malleus.

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

1. Define indexed tensor/QFunction IR and basis/transformation requirements.
2. Lower Poisson from `.res` through Malleus, then bind it in Finitum and solve through Solverang.
3. Add primal, JVP, VJP, and parameter-derivative kernel requests after the primal path is stable.
