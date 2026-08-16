# Scientific stack no-loss audit

**Reviewed:** 2026-08-15 / 2026-08-16 UTC  
**Architecture branch:** `agent/scientific-refinement-stack` across the participating repos.

This is a responsibility audit, not merely a line-diff review. The migration rule is that a
capability may change owners only after the new implementation has a parity/differential gate;
working incumbents are retained until then.

## Executive result

No production implementation was deleted in this architecture wave.

- Resolvent: implementation/docs are additive to the previous design-only repository.
- Sinbad: existing frozen result/cache schemas, Residua, Plexus, Anvil and physics crates are
  untouched; scientific provenance is an additive envelope.
- Solverang: no solver/constraint implementation was removed; symbolic export is an optional
  defaulted capability.
- Ferris–Howard: one integration document added; existing design/agent anti-cheat contract is
  untouched.
- Lean Atlas: external-artifact relations are additive; the existing declaration relation
  schema is unchanged.
- Pi Lab: old evidence required fields and `strength` vocabulary remain valid; new evidence
  axes/scope/artifact fields are optional.
- 3dhm-lab: no promoted evidence or legacy run was changed; the architecture fixture lives
  under `seed/`, not `evidence/`.

## Responsibility matrix

| Capability | Before | New canonical owner | Current migration state | Loss gate |
|---|---|---|---|---|
| exact integers/rationals/modular/poly/root algebra | planned Resolvent / local consumers | Resolvent | existing Resolvent algebra design preserved; implementation still staged | old CAS ADR/API contracts + consumer differential oracles |
| generic symbolic expressions/calculus | planned Resolvent L4 | Resolvent `expr` | foundational DAG/identity/store now implemented; full calculus still staged | canonical serialization + derivative/oracle tests |
| scientific equations/system semantics | split across Plexus/physics/consumer code | Resolvent `model` | common System + ScientificSpec implemented | RLC/DAE + constraint + continuum verticals |
| incidence + maximum matching | Plexus | Resolvent `structural` | migrated onto System projection | compare Plexus/Resolvent matchings on frozen corpus |
| SCC / BLT | Plexus | Resolvent `structural` | iterative Tarjan + BLT migrated | compare block partitions/order on frozen corpus |
| deterministic tearing + schedule | Plexus | Resolvent `structural` | greedy tearing + explicit/torn/loop schedule migrated | compare schedule block kind/equations/vars on frozen corpus |
| Pantelides / dummy derivatives / alias elimination | Plexus planned/deferred | Resolvent `structural` | intentionally deferred; semantic derivative node now exists | literature implementation + Plexus/DAE witnesses |
| fields/function spaces/weak forms | Residua + physics code | Resolvent `form` | typed dialect implemented; incumbent assembly retained | nonlinear heat, Stokes, H(curl) witnesses |
| P1 stiffness/mass/source/BC/DOF assembly | Residua | Resolvent `discrete` reference lowering | structured discrete dialect exists; numeric lowering not moved yet | bit/ULP differential tests against Residua |
| evolution/DAE operator composition | Residua | Resolvent `operator` | operator contracts exist; incumbent execution retained | existing transient/DAE corpus parity |
| adjoint/JVP/VJP semantic contract | Residua + Anvil | Resolvent operator semantics + Anvil executable AD | semantic capability declared; old implementations retained | dot-product/FD/form-vs-discrete derivative gates |
| finite-precision graph/JIT/codegen | Anvil in Sinbad | Anvil | unchanged | existing Anvil tests/benchmarks |
| nonlinear/linear/DAE/eigen/optimization solve policy | Solverang | Solverang | unchanged | existing Solverang suites |
| geometric/general constraint product | Solverang | Solverang | unchanged; optional CAS-neutral symbolic sink added | all existing constraints + symbolic/numeric differential tests |
| simulator composition/coupling/studies/results | Sinbad | Sinbad | unchanged | existing Sinbad corpus/gates |
| formal authoring UX | Ferris–Howard | Ferris–Howard | unchanged | FH statement freeze + clean elaboration |
| proof authority | Lean kernel | Lean kernel | unchanged | axiom audit + kernel recheck |
| formal corpus mining/relations | Lean Atlas | Lean Atlas | unchanged; external artifact links additive | relation warrant category tests |
| research judgment/evidence promotion | Pi Lab | Pi Lab | unchanged; evidence schema made more explicit | append-only promotion/state invariants |
| 3HDM campaign truth | 3dhm-lab/Pi Lab | 3dhm-lab/Pi Lab | unchanged | campaign authority hierarchy + independent replay |

## Schema and provenance review

### Sinbad frozen schemas

The existing `ResultSnapshot`, `VerificationReport`, cache profile and stage-cache types were
not modified. `ScientificResultSnapshot` wraps the old snapshot and stores foreign artifact
references rather than importing Resolvent's schema into Sinbad. Old readers/cache hashes
therefore do not silently reinterpret old bytes.

### Resolvent caller-owned stores

A post-CI semantic review found and fixed a serialization hazard: skipped hash-cons/symbol
indexes originally needed a manual `rebuild_indexes()` after deserialization. `ExprStore` and
`SymbolTable` now rebuild those indexes during `Deserialize`, with a round-trip identity test.

A second review found evidence summaries were insertion-order dependent. Grades are now
ordered **within** each evidence axis and profiles retain the strongest grade on that axis;
formal/numerical/empirical axes remain non-comparable to one another.

### Scope

Both Resolvent and Pi Lab now retain explicit scope obligations. Resolvent rejects a
`Broadened`/`Changed` scope transition whose named obligation is missing. Pi Lab flags records
whose tested/proved scope differs from target scope without a transport/generalization
obligation. This is the generalized 3HDM anti-scope-laundering gate.

## Validation status at review time

- **Resolvent:** GitHub Actions runs format, clippy with warnings denied, and Rust tests. The
  core architecture was green before the SCC/BLT/tearing migration; the structural migration
  receives a fresh run before handoff.
- **Lean Atlas:** full Rust tests and formatting green after one formatting-only correction.
- **Pi Lab:** targeted strict TypeScript check for the new evidence contract plus repository
  tests green. Full-repository `npm run check` is independently blocked on pre-existing
  `TeacherSession` + `exactOptionalPropertyTypes` assignments outside this change.
- **Solverang:** repository CI cannot load Cargo metadata on a clean GitHub runner because the
  base repository contains absolute private path dependencies to `/home/dev/sinbad` for
  `sinbad-anvil` and `numeric-contracts`. This predates the symbolic-seam PR. The architecture
  PR neither removes those capabilities nor claims a green full build.
- **Sinbad:** current base has no GitHub workflow directory. New result/provenance code is
  additive and was API-reviewed against `ContentHash::of`/`ResultSnapshot`; full federation
  execution remains an on-machine gate.
- **Ferris–Howard:** current `master` is architecture-only for this integration. An older
  large `main -> master` reconciliation PR remains open; merge/rebase ordering must be handled
  explicitly.
- **3dhm-lab:** architecture/configuration only; no evidence mutation.

## Open blockers intentionally not hidden

1. **Solverang portability:** replace absolute Sinbad path dependencies with a reproducible
   federation/package arrangement without deleting DAE/JIT functionality.
2. **Ferris–Howard branch topology:** reconcile the older `main -> master` PR with the
   additive Resolvent integration document.
3. **Sinbad older compiler ADR PR:** the older tentative architecture must be reconciled with
   ADR-035 rather than merged as a second normative architecture.
4. **Pi Lab global TypeScript baseline:** fix `TeacherSession` optional-field mutations in a
   separate focused change; do not weaken `exactOptionalPropertyTypes`.
5. **Residua numerical migration:** the new dialects are not evidence of numeric parity.
   Preserve Residua until differential P1/evolution/adjoint gates pass.
6. **Plexus deletion:** even after structural algorithms exist in Resolvent, keep Plexus until
   a frozen corpus demonstrates identical matching/BLT/tearing schedules and diagnostics.

## Deletion checklist

Before deleting or reducing a compatibility implementation, reviewers must be able to answer
all of these from tests/artifacts rather than prose:

- What exact incumbent responsibilities are being removed?
- Which Resolvent type/pass now owns each one?
- What frozen inputs exercise both implementations?
- Are result values, failure modes, ordering and determinism compared?
- Are source spans/provenance/diagnostics retained?
- Does any downstream crate still import an incumbent-only API?
- Can the old corpus be replayed without changing scientific verdicts?

Until all answers are affirmative, the compatibility implementation stays.
