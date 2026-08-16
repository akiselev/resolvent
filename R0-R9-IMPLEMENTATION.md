# R0–R9 implementation tranche

This branch turns the scientific-stack architecture into an executable, agent-facing compiler path.

## Implemented

- **R0:** frozen migration cases and deterministic differential comparison.
- **R1:** RSL, constrained scientific LaTeX, source spans, stable diagnostics and CLI inspection/freeze commands.
- **R2:** typed SI dimensions, domains, fields and H1/H(curl)/H(div)/L2 function spaces.
- **R3:** System-native structural incidence, deterministic matching, SCC/BLT/tearing parity tests, alias analysis, derivative profiles and index-reduction planning.
- **R4:** Form → Discrete → Operator lowering plus deterministic scalar H1 reference execution.
- **R5:** symbolic differentiation and scalar reference evaluation.
- **R6:** generated manufactured-solution forcing, dimension analysis, derivative/adjoint gates and convergence-order utilities.
- **R7:** reference execution and machine-readable authoring/inspection interfaces.
- **R8:** semantic physics locks and Rust promotion macros.
- **R9:** backend-neutral frozen execution plans consumed by standalone Malleus adapters.

## Generality gates

The branch also contains reference implementations for structures absent from the old Residua compiler: vector H1 elasticity, mixed Stokes saddle blocks, and oriented lowest-order Nedelec H(curl). These are intentionally deterministic correctness oracles rather than performance backends; Sinbad Lab uses them to test whether an agent can author new physics from semantic source without writing fresh assembly/JVP/VJP plumbing.

## Trust boundary

RSL and LaTeX are source languages. They are never proof-producing and are not semantic authority. The typed Resolvent IR, explicit refinement records, formal Lean boundary, numerical evidence, and empirical evidence remain distinct layers.