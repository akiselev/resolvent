# Resolvent Vision architecture and execution

This directory defines the RV0-RV9 capability program described in the root [`PLAN.md`](../../PLAN.md).

Cross-repository implementation order is governed by [`CROSS-ROADMAP-CONTRACT.md`](CROSS-ROADMAP-CONTRACT.md). RV numbers are capability namespaces and maturity targets, not a blanket global sequence.

## Product target

Resolvent becomes a full embeddable CAS with one mathematical kernel shared by library users, algebra projections from Scientia, CADabra, agents, and interactive frontends. It should eventually provide Mathematica-class symbolic breadth and notebook ergonomics without adopting global evaluator coupling, opaque algorithm choice, or frontend-defined kernel semantics.

The core design combines:

- a uniform **structural** symbolic `Term` surface for retained symbolic structure;
- specialized mathematical `Domain`/`Element` representations for efficient algebra;
- a lower-level `Scalar` seam for reusable numeric kernels over approximate/exact/differentiable values;
- typed `Value`/`Outcome` results that preserve exactness, conditions, uncertainty, refusals and resource limits;
- explicit algorithm requests, plans, receipts and certificates;
- stateless library APIs where possible and explicit session state where definitions/assumptions require it.

## Architecture

```text
                         frontends and consumers

  Rust API    CLI/REPL    Python/C/WASM    Jupyter    native notebook    agents
      \          |             |              |              |             /
       +---------+-------------+--------------+--------------+------------+
                                     |
                          versioned kernel protocol
                                     |
                              Resolvent session
                    definitions / assumptions / packages
                                     |
          +--------------------------+--------------------------+
          |                          |                          |
          v                          v                          v
     structural Terms          dynamic values             evidence
       TermStore            Domain / Element       plans / receipts / certs
          |                          |                          |
          +--------------------------+--------------------------+
                                     |
                          algorithm catalog / planner
                                     |
                 +-------------------+-------------------+
                 |                                       |
                 v                                       v
          canonical Rust paths                    optional providers
      exact/general algebra                   in-process or external

Federation consumers remain above this boundary:

Quantitas -----> Scientia -----> Malleus -----> Finitum -----> Krasis -----> Sinbad
                    ^
                    |
          operation-specific
          algebra projections
                    |
                 Resolvent
                    ^
                    |
                 CADabra

Methodus: consumer-neutral numerical algorithms
Solverang: generic constraint engine + 2-D/3-D constraint vocabularies, numerics via Methodus
```

The arrows express ownership/consumption, not permission to replace another repository's semantic IR.

R1 already consolidated CADabra's generic exact/scalar implementation into Resolvent. RV0 hardens that landed substrate; it does not repeat the migration and does not block CADabra R2 absent a concrete correctness problem.

## Five identity boundaries

### 1. Resolvent `Term`

A Term is retained **structural** symbolic syntax stored in a caller/session-owned hash-consed `TermStore`.

Properties:

- immutable nodes;
- local handles plus stable content-derived digests;
- no process-global interner;
- structural identity independent of source spans;
- exact literals as first-class nodes;
- binders, relations, logic, conditions, arrays, rules and generic calls;
- deterministic canonical structural bytes;
- iterative traversal and teardown;
- bounded store-growth mechanisms suitable for long notebook sessions.

Structural identity is not mathematical equivalence. RV1 must not sort, commute, flatten, cancel, factor, or reassociate expressions merely to make hashes canonical. Those are explicit domain/rewrite operations in later layers.

### 2. Scientia scientific identity

Scientia keeps one canonical `SemanticModel`/`ExprId` arena. Resolvent Terms do not replace it.

Generic algebra integration is an operation-specific projection:

```text
Scientia ExprId
  -> supported scalar algebra projection
Resolvent Term / Domain Element
  -> generic algebra operation
result + receipt/certificate
  -> re-embed or attach under Scientia-owned semantics
```

Scientia therefore retains source spans, units, roles, shapes, axes, field/differential meaning, and scientific expression identity.

### 3. `Domain` and `Element`

Mathematical values carry an explicit parent/domain. Examples:

```text
ZZ
QQ
GF(7)
QQ[x, y; grevlex]
Frac(QQ[x, y])
QQ(sqrt(2))
RealAlgebraic
RealBall(precision = 256)
ComplexBall(precision = 256)
MatrixSpace(QQ[x], 4, 4)
PowerSeries(QQ, x, order = 32)
```

`Element` storage is specialized per domain. A sparse multivariate polynomial is not encoded as a generic `Add` tree merely because it can be printed as one.

The API provides both statically typed Rust paths and a type-erased dynamic path for sessions/bindings.

### 4. `Scalar`

R1 consolidated CADabra's exact/approximate scalar seam and dual numbers into Resolvent. The seam serves numeric kernels that legitimately need the same implementation over `f64`, certified exact values, intervals and dual numbers.

It is not the parent/domain abstraction and not a substitute for Methodus numerical-method contracts.

### 5. Malleus executable computation identity

Resolvent may optimize algebraic Terms/domain expressions and emit explicit CSE/let schedules. It does not own a second executable numeric SSA/kernel IR.

Malleus owns finite-precision structured operation semantics, iteration/index maps, reductions/effects, AD execution products, scheduling, backend lowering and portable generated kernels.

## Numeric literal semantics

Resolvent surface syntax distinguishes:

- integers;
- rational literals;
- exact decimal literals;
- exact IEEE-754 binary values admitted from host APIs;
- approximate machine floats;
- arbitrary-precision approximate values;
- interval/ball values.

A Resolvent-authored exact `0.1` is `1/10`.

For `.res`, however, Scientia owns preserving authored literal meaning because Scientia owns parsing. Its current source expression representation stores numbers as `f64`, so a coordinated Scientia schema/parser change is required before exact decimal intent can reach Resolvent. Resolvent cannot reconstruct source lexical exactness after it has been discarded upstream.

## Catalog model

### Operation catalog

Defines mathematical meaning and surface behavior:

- operation identity;
- arity/binders;
- accepted domains;
- result-domain rules;
- branch conventions;
- exactness possibilities;
- derivative/series hooks;
- formatting and documentation.

### Rule catalog

Defines guarded mathematical transformations:

```text
id
left pattern
right pattern
required domain/capabilities
assumption guard
branch conditions
orientation
expected complexity effect
verification strategy
provenance/version
```

Rules may be grouped into explicit packs. There is no unbounded global `simplify` applying hidden heuristics indefinitely.

### Algorithm catalog

Defines realizations of operations:

```text
operation
algorithm id/version
domain/capability requirements
structural applicability classifier
guarantee/exactness
resource budget schema
certificate kind
provider identity
fallback algorithms
performance classification
```

Selection starts deterministic and rule-based. Learned selection may be added later only as an inspectable hint whose model identity is recorded and whose exact result remains independently verified when required.

## Evaluation model

Construction, evaluation, canonicalization, rewriting, simplification, expansion, factorization, approximation and code optimization are distinct operations.

The session may own definitions and assumptions, but Term construction itself does not repeatedly invoke an ambient evaluator or silently invoke algebraic laws.

## Assumption model

Assumptions are immutable contexts and are included in operation identity where they affect meaning. Queries may be proved true, proved false or remain unknown.

Rules never silently assume:

- real-valued variables;
- nonzero denominators;
- principal branches beyond the operation's declared convention;
- convergence;
- positivity/nonnegativity;
- integer-valued exponents.

Conditional answers are first-class outcomes.

## Rewriting and e-graphs

Resolvent owns an ordinary bounded rewrite engine with structural/typed patterns, associative/commutative matching only where a mathematical operation/domain contract justifies it, guards, explicit strategies, traces, cycle detection and budgets.

Equality saturation is optional and transient. It is suitable for bounded code optimization, Hornerization, CSE, stability-oriented transformations and identity exploration. It is not the canonical Term representation and not the default evaluator.

## Evidence model

An `AlgebraReceipt` evolves from a pair of hashes into an execution record containing, where applicable:

```text
schema
operation
input/output digests
context and assumption digest
domain descriptors
requested exactness
algorithm id/version
provider id/version
requested budget
consumed resources
deterministic seed
conditions and warnings
plan digest
certificate kind/digest
checker version
```

Certificates are operation-specific. Examples include Bezout witnesses for GCD, multiply-back plus irreducibility witnesses for factorization, Sturm/Descartes data for root isolation, refinement/separation data for algebraic comparison, and applied-rule traces for rewriting.

Not every result has a compact proof. The outcome states whether evidence is a proof/certificate, independent verification, certified enclosure, probabilistic check or heuristic validation.

## Provider and artifact boundaries

Resolvent owns mathematical provider semantics: operation/algorithm identity, supported domains, exactness/evidence guarantees and provider identity in plans/receipts.

It does not need a second generic process-plugin framework. External executable discovery, manifest compatibility, persistent workers, progress, cancellation and process isolation may use **Outboard** through an optional adapter.

Resolvent owns ephemeral/local mathematical memoization. Durable large certificates, provider outputs, cross-repository lineage, distribution and long-lived content-addressed evidence may use **Artifactum** through an optional adapter.

Neither project is a mandatory dependency of core Resolvent.

## Kernel and frontend boundary

The internal kernel protocol is transport-neutral. Expected mature operations include:

- evaluate;
- cancel;
- complete;
- inspect;
- format/render;
- request/explain plan;
- verify receipt/certificate;
- retrieve/store artifact reference;
- snapshot session;
- replay cell/request;
- package/environment inspection.

Jupyter is an adapter to this protocol, not the protocol itself.

## Capability-start graph

- R1 exact/scalar consolidation is landed.
- RV0 is a short stabilization/baseline program and does not block CADabra R2 absent a discovered correctness issue.
- CADabra-pulled `RV5-C*` algebra may start immediately from the landed exact substrate when its own minimum prerequisites are present.
- RV1 freezes structural Term identity, not scientific or algebraic semantic identity.
- RV2 and RV3 overlap after the relevant RV1 identity decisions; RV3 may wrap existing exact typed operations before the full RV2 catalog exists.
- RV4 subfeatures begin when the needed RV1/RV2/RV3 contracts exist.
- RV5-S is demand-driven from Scientia and uses projection/re-embedding rather than a second canonical scientific IR.
- RV6 families begin individually when their RV2/RV3 primitives exist; they do not wait for RV5 maturity.
- RV7 families begin individually from their actual Term/domain/evaluation/function prerequisites; they do not wait for all RV6 families.
- RV8 parser/formatter/simple CLI prototypes can start with RV1; stable protocol/Jupyter needs coherent initial RV2 dynamic values and RV3 outcome/plan/receipt schemas; stateful definitions/assumptions/rules need RV4.
- RV9 package/provider/cache/scale lanes start from local prerequisites rather than a final global gate.

## Work-package convention

Each phase file contains stable IDs:

```text
RV<phase>-<lane><number>
```

Examples: `RV1-A1`, `RV3-C2`, `RV5-S4`.

A work package is complete only after its stated acceptance gate lands. Plans are not status. [`../../STATUS.md`](../../STATUS.md) remains the record of implemented truth.

Agents must name the **minimum typed prerequisites** for a package rather than relying on phase-number ordering. [`CROSS-ROADMAP-CONTRACT.md`](CROSS-ROADMAP-CONTRACT.md) is authoritative when a phase file could otherwise be read as a blanket sequencing rule.

## Parallel implementation rules

1. Keep one owner for every semantic type.
2. Introduce shared generic machinery from a real consumer requirement or a mathematically general domain requirement, not aesthetic deduplication alone.
3. New optimized algorithms ship with a deterministic correctness baseline or independent grader whenever practical.
4. Score/performance lanes begin only after their correctness oracle is stable.
5. Resource-heavy operations expose budgets before broadening capability.
6. Schema/wire changes are explicitly versioned and have round-trip and migration tests.
7. Cross-repository cuts record the exact consumer fixture and preserve downstream gates.
8. Optimized provider paths never become the only way to verify correctness.
9. Do not duplicate Scientia scientific IR, Malleus executable IR, Outboard plugin lifecycle or Artifactum durable artifact lifecycle.

## Definition of the first useful standalone CAS milestone

Resolvent is a coherent early standalone CAS when:

- the RV0 stabilization gate is complete;
- RV1 structural Term identity is complete;
- initial RV2 domains/dynamic values exist;
- RV3's existing-operation vertical slice is complete;
- RV8 has a usable stable CLI/protocol/Jupyter subset over those schemas.

That milestone does not require Mathematica-class algorithm breadth. It proves that the architecture scales from embedded Rust through real algebra consumers to an interactive environment before large breadth work finishes.
