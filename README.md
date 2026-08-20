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

- a recovering parser and canonical formatter for `.res` modules;
- one `ScientificModel` and `Expr` representation with source spans and stable semantic digests;
- Quantitas-backed authored quantities and unit validation;
- property, constitutive, coupling, time/state, and evidence semantics;
- structural incidence, matching, SCC/BLT, tearing, alias, and DAE planning over the same model;
- compilation of authored forms into `VariationalForm` while retaining the canonical `Expr` type;
  and
- realization-neutral `LocalFormProgram` factorization and explicit scalar pointwise lowering into
  `malleus::StructuredKernel`.

Tensor factorization, strong-equation-to-form derivation, integration-by-parts receipts, basis
requirements, and derivative kernel generation remain active compiler work. Unsupported tensor or
differential operations fail at the Malleus boundary instead of becoming opaque opcodes.

## Command line

```console
cargo run --bin resolvent -- check examples/nonlinear_heat.res
cargo run --bin resolvent -- fmt examples/nonlinear_heat.res
cargo run --bin resolvent -- parse examples/nonlinear_heat.res
cargo run --bin resolvent -- coupling examples/nonlinear_heat.res
cargo run --bin resolvent -- structural examples/nonlinear_heat.res
cargo run --bin resolvent -- form path/to/model.res form_name
```

The library is the authoritative API. See [SCIENTIFIC-COMPILER.md](SCIENTIFIC-COMPILER.md) for the
artifact boundaries and [STATUS.md](STATUS.md) for the exact checked state and next work.

## Development

```console
cargo fmt --all -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```

Resolvent is dual-licensed under MIT or Apache-2.0.
