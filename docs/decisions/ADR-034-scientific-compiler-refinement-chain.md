# ADR-034 — Resolvent is the scientific compiler; refinements are first class

**Status:** Proposed 2026-08-16

## Context

Resolvent was originally planned as a consumer-independent exact algebra/CAS engine. Sinbad
independently grew Residua (field/operator/assembly/DAE semantics) and Plexus (structural DAE
analysis). That split creates multiple mathematical IRs and adapter boundaries exactly where
nonlinear physics, automatic differentiation, formal specifications and generated validation
need shared identity and provenance.

The wider stack also now includes Ferris–Howard/Lean for checked specifications, Lean Atlas
for read-only structural intelligence, Anvil for executable finite-precision graphs,
Solverang for batteries-included constraint/numerical solving, Sinbad for simulation, and Pi
Lab/project laboratories for evidence-controlled research.

## Decision

1. `resolvent` is the single public Rust mathematical/scientific compiler crate.
2. Lower exact/CAS layers remain consumer-independent and may not import physics, mesh,
   Sinbad, Solverang, Lean Atlas, or campaign concepts.
3. Resolvent adds typed scientific-model, structural, field/form, discrete and operator
   dialects above the symbolic substrate.
4. Residua's long-term semantic role is absorbed by Resolvent's field/form/discrete/operator
   layers. Plexus's long-term semantic role is absorbed by Resolvent structural compiler
   passes. Existing implementations remain compatibility/reference paths until differential
   gates prove parity.
5. Resolvent does not adopt one universal AST. Algebraic expressions, systems, forms,
   discrete programs and operators use typed handles/stores and may reference lower-stage
   objects without becoming interchangeable.
6. Every lowering returns/records a `Refinement` receipt describing the claimed semantic
   relation, assumptions, source/target scope, open/discharged obligations, evidence and
   provenance.
7. Formal, numerical and empirical evidence are independent axes. No global confidence
   ordering may silently promote empirical agreement into proof or formal proof into model
   adequacy.
8. Lean integration uses a checked deep `ScientificSpec`/refinement representation plus
   soundness theorems, not arbitrary surface-AST translation. Ferris–Howard is an ergonomic
   frontend only; the Lean kernel remains the formal authority.
9. Optimized Rust algorithms may be untrusted producers. Where formal status is required,
   prefer compact certificates checked by small Lean checkers rather than placing the whole
   Rust compiler in the trusted base.
10. Sinbad remains the simulator, Solverang remains an independent batteries-included solver
    product, and Anvil remains the executable-code/finite-precision compiler.

## Consequences

- Existing algebraic ADRs remain valid unless they depended on Resolvent being *only* an
  algebra library rather than a layered scientific compiler.
- `SCIENCE-STACK.md` is normative for the new upper layers and ecosystem boundaries.
- No deletion of Residua/Plexus implementation is authorized by this ADR. Migration requires
  an explicit capability inventory and old-vs-new differential validation.
- Scientific claims become traceable through formal source -> model -> form -> discretization
  -> operator -> executable -> observable prediction, with scope and obligations preserved.
- Research campaigns can consume the same certificate/refinement vocabulary without making
  Resolvent the owner of experimental data or evidence promotion.

## Rejected alternatives

### Keep Resolvent and Residua as peer mathematical stacks

Rejected because nonlinear constitutive expressions, manufactured solutions, variational
linearization, parameter derivatives and formal provenance would cross a foreign-AST adapter.

### Make weak forms the universal representation

Rejected because circuits, algebraic constraints, particle systems and many DAEs do not have
natural weak-form semantics.

### Make one generic `Expr` represent everything

Rejected because it erases useful type distinctions and encourages premature lowering of
field/domain/discretization semantics.

### Put numerical solving into Resolvent

Rejected because Solverang has an independent high-level product identity: geometry/general
constraints, diagnosis, globalization, optimization, continuation and batteries-included
solve workflows.

### Require a formally verified compiler before integration

Rejected as an adoption blocker. Proof-producing/certificate-producing lowerings and small
checkers provide an incremental trust path.
