# Resolvent science-stack architecture

**Status:** normative for the scientific-compiler/refinement architecture. The existing
`DESIGN.md` and `API.md` remain normative for exact algebra/CAS details unless this document
explicitly supersedes their project-boundary assumptions.

## Thesis

Resolvent is the single public Rust mathematical/scientific compiler crate. It spans exact
algebra, symbolic expressions, scientific equation systems, structural transformations,
continuum/variational forms, discretization, discrete operators, and validation-contract
generation. It does **not** own physics packages, geometry kernels, numerical solver product
APIs, machine-code optimization, theorem proving, experimental campaigns, or empirical
truth.

The load-bearing abstraction is not one universal `Expr`; it is an auditable **refinement
chain**. Each stage has its own typed semantic dialect and every transition records the
relation claimed between source and target, assumptions, scope change, obligations, evidence,
and provenance.

## End-to-end system

```text
Ferris-Howard -> Lean kernel -> checked Resolvent SpecIR
                                  |
                                  v
                              RESOLVENT
                 exact algebra / symbolic mathematics
                         scientific model IR
                    structural/DAE compilation
                       field + form dialect
                        discretization IR
                         operator program
                     refinement + obligations
                                  |
                     +------------+-----------+
                     |                        |
                   Anvil                  Solverang
              executable kernels       numerical methods
                     +------------+-----------+
                                  |
                                Sinbad
                                  |
                         observable prediction
                                  |
                              Validator
                          /                 \
                   formal/numeric        experiment
                          \                 /
                           evidence bundle
                                  |
                               Pi Lab
```

Lean Atlas remains a read-only intelligence layer over kernel-checked mathematical objects;
it may propose relevant theorems, representations, or correspondences, but a proposal only
enters a refinement chain after the relevant statement/proof/certificate is checked.

## Semantic dialects

Resolvent deliberately keeps these distinct:

1. **Exact/algebraic values** — integers, rationals, finite fields, polynomials, algebraic
   numbers, ideals, exact linear algebra, certificates.
2. **Expression IR** — mathematical scalar/vector/tensor expressions and explicit symbolic
   transformations.
3. **Scientific model IR** — variables, parameters, equations, derivatives, events,
   components, connectors, assumptions, scope, observables, validation contracts.
4. **Structural IR/views** — incidence, matching, BLT, tearing, Pantelides/index reduction,
   alias elimination and causalization. The algorithms currently living in Sinbad's Plexus
   belong here in the long-term architecture.
5. **Field/form IR** — domains, fields, function spaces, grad/div/curl, traces, measures,
   test/trial functions and variational forms. The semantic role currently named Residua is
   absorbed here.
6. **Discrete IR** — restrictions, basis actions, quadrature, orientation, pointwise physics,
   integration/scatter and other discretization structure. This layer should retain enough
   structure for assembled, partially assembled and matrix-free realizations.
7. **Operator IR** — residual blocks, mass/damping, Jacobian/JVP/VJP semantics, events,
   eigen-operators, sparsity, nullspaces and validation metadata.
8. **Executable IR** — intentionally outside Resolvent. Anvil owns finite-precision machine
   computation, scheduling, vectorization and code generation.

A problem may skip stages. Algebraic constraints need not become weak forms; circuits need
not become finite elements; PDEs need not be assembled into matrices.

## ScientificSpec

`ScientificSpec` is the semantic center. A PDE alone is not a scientific specification.
The specification states:

- what state and parameters exist;
- which laws are claimed;
- assumptions and exact applicability scope;
- initial and boundary conditions;
- observables and their measurement/uncertainty interpretations;
- invariants and validation contracts;
- a stable locator back to the formal source when one exists.

Solver choices, timesteppers, meshes, GPU targets, empirical datasets, and research-campaign
state are explicitly excluded.

## Refinements, not implicit lowering

Every transformation emits a `Refinement` receipt. Important relation classes include:

- definitional equality;
- mathematical equivalence;
- logical consequence;
- specialization/reformulation;
- strong-to-weak formulation;
- index reduction;
- discretization with stated consistency/convergence claims;
- bounded approximation;
- finite-precision implementation with an error model;
- compiled implementation;
- observable interpretation.

`source_scope` and `target_scope` are explicit. A scope change is never silently promoted.
A restricted-orbit result cannot satisfy a global claim without a `ScopeTransport`
obligation and evidence discharging it.

## Evidence is multi-axis

Resolvent does not maintain one confidence score. Formal, numerical, and empirical warrant
answer different questions and remain independently visible.

Formal evidence may include kernel proofs or checked certificates. Numerical evidence may
include convergence studies, reference cross-checks, adjoint identities and mutation tests.
Empirical evidence belongs to the validation/research layer and may establish model adequacy
under a declared measurement model; it cannot be relabeled as a theorem.

## Lean bridge

The trusted bridge is **not arbitrary Lean AST -> Rust**. Resolvent provides a small deep IR
on the Lean side with a denotational interpretation. Domain formalizations reify into that IR
and prove the reification sound/equivalent. Rust consumes the checked IR plus declaration and
statement hashes.

Long-term shape:

```lean
def heatSpec : Resolvent.Spec := ...

theorem heatSpec_sound :
  Resolvent.Spec.denote heatSpec <-> HeatTheory.heatEquation := by
  ...
```

The exporter's correctness is therefore not part of the mathematical trusted base: a bad
export either changes the checked hash or produces an artifact whose certificate cannot be
validated.

## Certificate strategy

Resolvent should prefer fast untrusted producers plus small checkers over attempting to
formally verify the entire optimized Rust implementation at once. Candidate certificate
families include factorization, root isolation, row-dependency, Gröbner/cofactor,
rewrite/normalization, exact assembly identities and floating-point error bounds.

A certificate may be rechecked by a small Lean implementation and ultimately by the Lean
kernel. Failure to produce/check a certificate is a typed unresolved obligation, not a
reason to silently downgrade exactness.

## Ecosystem ownership

- **Resolvent:** mathematical semantics, exact algebra/CAS, systems, forms, discretization,
  operator semantics, refinement receipts, generated verification obligations.
- **Sinbad:** simulation product, physics composition, runtime, studies, results and product
  provenance. Physics packages describe equations through Resolvent.
- **Solverang:** batteries-included geometric/general constraint solver and numerical solver
  package. It stays independently useful and selectively consumes Resolvent algebra/symbolic
  capabilities.
- **Anvil:** executable finite-precision graph/compiler and AD/JVP/VJP realization.
- **Ferris-Howard:** ergonomic Lean frontend and proof-agent workbench; the Lean kernel remains
  authoritative.
- **Lean Atlas:** read-only exact relation/intuition engine over checked Lean corpora.
- **Pi Lab / project labs:** research/evidence control plane deciding what to investigate,
  reproduce, falsify and promote.
- **Validator:** conceptual bridge joining formal obligations, numerical behavior and
  experimental observations. Resolvent generates contracts; project/campaign layers own
  empirical data and promotion decisions.

## Migration from Residua and Plexus

There is no flag-day rewrite.

1. Freeze and inventory all existing Residua/Plexus capabilities and tests.
2. Introduce Resolvent semantic types and refinement receipts without changing numerical
   behavior.
3. Wrap the existing P1 stiffness/mass/evolution/DAE code as the first reference lowering
   backend. Existing algorithms remain authoritative until the new path reproduces them.
4. Move structural algorithms from Plexus behind a Resolvent-system adapter; keep the crate as
   a compatibility facade while downstream code migrates.
5. Add form/discrete/operator dialects only as two dissimilar physics cases demand each seam.
6. Differential-test old and new paths, including transpose/adjoint and corpus mutation
   gates, before deleting any compatibility surface.
7. Delete the old architectural boundary only after the capability inventory has no orphaned
   entries.

The rule is **move semantics first; delete code last**.

## Falsification cases for the architecture

The same typed pipeline must support, without downward domain hacks:

- nonlinear transient heat;
- incompressible Stokes/Navier-Stokes mixed systems;
- high-index RLC/mechanical DAE models;
- Solverang geometric constraint clusters;
- H(curl) Maxwell as the first orientation/exact-sequence stress test;
- a 3HDM polynomial/certificate workflow that never needs Sinbad.

If the lower algebra layers learn about meshes/physics, if geometric constraints are forced
through weak forms, or if empirical validation can silently promote formal scope, the
architecture has failed.
