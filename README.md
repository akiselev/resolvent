# Resolvent

Resolvent is the `.res` language and mathematical form compiler for Sinbad. It owns source
parsing, scientific semantics, variational meaning, structural equation analysis, property and
constitutive declarations, and evidence attached to semantic transformations.

The repository deliberately has a narrow dependency direction:

```text
Quantitas -> Resolvent -> Malleus
                  |
                  +-> downstream realization in Finitum
```

- Quantitas supplies the shared identity of dimensions, quantity kinds, units, and canonical
  quantities.
- Resolvent lowers local numerical work directly into Malleus-owned structured kernel types.
- Finitum owns meshes, basis tabulations, degrees of freedom, constraints, quadrature execution,
  assembly, and matrix-free global operators.
- Krasis owns coupled runtime state. Solverang owns numerical algorithms. Sinbad owns the product.

There is no alternate expression language, discrete runtime, or reference FEM backend in this
repository. Git history is the archive for the removed implementation.

## Current compiler surface

Resolvent currently provides:

- a recovering parser, canonical formatter, and deterministic module resolver for `.res` modules;
- a source-syntax tree with byte-precise expression/reference spans;
- one typed `SemanticModel` arena with resolved domain/symbol/expression identities, value shapes,
  axes, Quantitas dimensions and quantity kinds, domain frames, and scientific roles;
- stable structured diagnostics for malformed syntax, units, kinds, roles, names, axes, and frames;
- property, constitutive, coupling, time/state, and evidence semantics;
- structural incidence, matching, SCC/BLT, tearing, alias, and DAE planning over the same model,
  including field dependencies hidden behind model-defined values, properties, and constitutive
  laws;
- compilation of authored forms and derivation of strong equations into `VariationalForm`, with
  `DeclarationId`, `SymbolId`, `DomainId`, `RegionId`, and `ExprId` identities plus transformation
  and boundary-term receipts;
- typed differential, contraction, tensor/facet trace, jump, average, normal-component, and
  conjugation operations with explicit cell, exterior, interior, interface, and point sides;
- a deterministic form interpreter over caller-supplied point values, derivatives, traces,
  normals, and weights, plus `required_evaluations` for discovering the binding contract; and
- FC3 `FormRequirements` inference for mesh-free H1, L2, product/mixed, Hcurl, Hdiv, DG, and trace
  spaces, including element, pullback, orientation, quadrature, geometry/basis preprocessing,
  essential-constraint, boundary-partition, and canonically grouped integral requirements;
  constitutive/value chains expose their physical-field spaces and distinguish basis,
  model-defined, and external inputs; and
- FC4 `TensorProgram`, `QFunctionProgram`, and `OperatorFactorization` artifacts with explicit
  shapes, free/reduction axes, sides, scalar semantics, restriction, basis, geometry, quadrature,
  transpose, scatter, and constraint stages; symbolic test differentiation produces basis-dual
  point outputs and symbolic directional differentiation produces digest-linked JVP programs;
- deterministic QFunction and element-factorization interpreters, validated for Poisson against
  an independent analytic P1 triangle residual, element tensor, and directional finite
  difference; and
- the retained narrow `LocalFormProgram` path for scalar point arithmetic into
  `malleus::StructuredKernel`.

Lowering the indexed FC4 programs into complete Malleus structured modules and derivative kernel
bundles is FC5 work. Unsupported tensor primitives return stable capability diagnostics instead
of becoming opaque or named-physics opcodes.

Derived forms record that declared exterior regions are assumed to partition the domain boundary;
Finitum must validate that assumption against mesh topology. A Neumann value is substituted at
most once per region and field because FC2 has no per-flux boundary correspondence. Complex
conjugation is always explicit—derivation never silently changes a bilinear contraction into a
sesquilinear one.

## Command line

```console
cargo run --bin resolvent -- check examples/nonlinear_heat.res
cargo run --bin resolvent -- fmt examples/nonlinear_heat.res
cargo run --bin resolvent -- parse examples/nonlinear_heat.res
cargo run --bin resolvent -- elaborate examples/nonlinear_heat.res
cargo run --bin resolvent -- coupling examples/nonlinear_heat.res
cargo run --bin resolvent -- structural examples/nonlinear_heat.res
cargo run --bin resolvent -- form path/to/model.res form_name
cargo run --bin resolvent -- derive-form path/to/model.res equation_name
cargo run --bin resolvent -- requirements path/to/model.res form_name
cargo run --bin resolvent -- derive-requirements path/to/model.res equation_name
cargo run --bin resolvent -- operator path/to/model.res form_name
cargo run --bin resolvent -- derive-operator path/to/model.res equation_name
cargo run --bin resolvent -- requirements path/to/multi.res ModelName:form_name
cargo run --bin resolvent -- structural path/to/multi.res ModelName
```

All commands other than `parse` and `fmt` require successful typed elaboration. `parse` is
intentionally syntax-only, while `elaborate` prints the canonical typed arena. An external
scientific function with no declared signature receives an explicit `deferred` result constraint;
it is never guessed to be scalar or dimensionless.

Commands that select a form or equation accept `ModelName:item_name`. Coupling and structural
commands accept `ModelName`. An unqualified item remains valid for a single-model module; a
multi-model module requires an explicit model so the CLI never silently chooses the first one.

The library is the authoritative API. See [SCIENTIFIC-COMPILER.md](SCIENTIFIC-COMPILER.md) for the
artifact boundaries and [STATUS.md](STATUS.md) for the exact checked state and next work.
The quadrature/Finitum/Malleus boundary is fixed in
[ITERATION-OWNERSHIP.md](ITERATION-OWNERSHIP.md).

## Development

```console
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Resolvent is dual-licensed under MIT or Apache-2.0.
