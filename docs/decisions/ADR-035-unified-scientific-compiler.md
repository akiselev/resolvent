# ADR-035 — Resolvent is the unified scientific compiler

**Status: Ratified 2026-08-15 by repository owner**

## Decision

Resolvent is one public Rust crate and one conceptual mathematical system. It remains a
computer-algebra system, but its scope now extends upward through symbolic scientific model
compilation:

```text
exact algebra
  -> generic symbolic expressions
  -> scientific System IR
  -> structural equation compilation
  -> continuum/variational Form IR (when applicable)
  -> structured Discrete IR
  -> solver-facing OperatorProgram
```

These are **distinct typed dialects**, not one universal `Expr` enum and not independent
mathematical products connected by serialization adapters. Higher dialects reference lower
objects by stable handles so mathematical identity and provenance survive lowering.

The public Cargo surface is `resolvent`. Internal package splitting is permitted later for
compile-time or implementation reasons, but internal crates are not an ecosystem API and
should default to `publish = false`.

## Refinement is the compiler invariant

Every semantic lowering emits a `RefinementRecord` containing:

- source and target artifact digests;
- the exact relation claimed (equivalence, consequence, strong-to-weak, discretization,
  finite-precision implementation, etc.);
- source and target scope;
- an explicit `ScopeTransition`;
- assumptions and proof/validation obligations;
- orthogonal formal, numerical and empirical evidence;
- producer/version/parent provenance.

Scope broadening is invalid unless the record names an obligation that justifies it. This is
a permanent defense against restricted-family -> global-claim laundering.

Formal, numerical and empirical warrant are separate axes. They are never collapsed to one
confidence number: a Lean theorem does not establish physical adequacy, and experimental fit
does not prove a theorem.

## Lean boundary

Ferris–Howard is not trusted and is not a Resolvent frontend. Ferris–Howard elaborates to
ordinary Lean. Resolvent consumes a kernel-visible Lean declaration through a reification
receipt. A receipt claiming `KernelProved` must name a Lean soundness theorem connecting the
formal declaration to the Resolvent deep-IR artifact and must pass the declared axiom
whitelist.

The intended long-term companion Lean package defines a deep `Resolvent` Spec IR and its
semantics. The Rust exporter is therefore not part of the mathematical trusted base: Lean
checks the reification theorem; Rust consumes the checked artifact.

## Ownership boundaries

Resolvent does not absorb product identities that remain useful:

- **Solverang** owns batteries-included numerical and geometric/general constraint solving.
- **Anvil** owns finite-precision execution graphs, AD at the executable level, scheduling,
  vectorization and target code generation.
- **Sinbad** owns simulator composition, physics packages, runtime, studies and results.
- **Lean/Ferris–Howard** own formal specification/proof authoring and kernel checking.
- **Lean Atlas** owns read-only structural intelligence over checked formal corpora.
- **Pi Lab / project labs** own research judgment, evidence promotion and empirical campaign
  state.

Residua's mathematical mission and Plexus's structural-compiler mission migrate into
Resolvent. Their existing Sinbad crates remain compatibility/reference implementations until
the dependency can flip without losing tests or behavior.

## Amendments to earlier decisions

This ADR amends the architectural consequences of ADR-005, ADR-018 and ADR-029 where they
assume the long-term public product is a federation of CAS-only crates or that scientific
consumer integration remains outside Resolvent. Their exact-algebra representation and
soundness decisions remain in force.

In particular, this ADR does **not** reopen the separate Cadabra/arrangements scalar-seam
question. Geometry consumers may use the lower Resolvent modules without importing the
scientific dialects.

## Migration rule

No working implementation is deleted merely to satisfy the diagram. For each migrated
capability:

1. establish the Resolvent type/behavior and tests;
2. differential-test it against the incumbent implementation;
3. make the incumbent crate a compatibility facade or adapter;
4. only then remove duplicate implementation code.

This makes "we lost nothing" an executable gate rather than a review impression.
