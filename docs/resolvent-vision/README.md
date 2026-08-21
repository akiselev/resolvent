# Resolvent Vision architecture and execution

This directory is the implementation authority for the RV0-RV9 program described in the root [`PLAN.md`](../../PLAN.md).

## Product target

Resolvent becomes a full embeddable CAS with one kernel shared by library users, Scientia, CADabra, agents and interactive frontends. It should eventually provide Mathematica-class symbolic breadth and notebook ergonomics without adopting Mathematica's most difficult architectural coupling: global evaluation semantics, opaque algorithm choice, and frontend/kernel behavior that is hard for embedding consumers to reason about.

The core design combines:

- a uniform symbolic `Term` surface for retained symbolic structure;
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
     symbolic terms            dynamic values             evidence
  TermStore / patterns       Domain / Element       plans / receipts / certs
          |                          |                          |
          +--------------------------+--------------------------+
                                     |
                          algorithm catalog / planner
                                     |
                 +-------------------+-------------------+
                 |                                       |
                 v                                       v
          canonical Rust paths                    optional providers
    exact/algorithms/certified numerics        FLINT/etc. when justified

Federation consumers remain above this boundary:

Quantitas -----> Scientia -----> Malleus -----> Finitum -----> Krasis -----> Sinbad
                    ^                                  
                    |                                  
                 Resolvent <---------------------------+
                    ^                                  
                    |                                  
                 CADabra                               

Methodus: consumer-neutral numerical algorithms
Solverang: generic constraint engine + 2D/3D constraint vocabularies, numerics via Methodus
```

The arrows are capability dependencies, not a demand that every repository directly import every lower repository. In particular, Sinbad should continue to receive scientific algebra through Scientia rather than acquiring an alternate symbolic IR.

## Four core abstractions

### 1. `Term`

A term is retained symbolic structure stored in a caller/session-owned hash-consed `TermStore`.

Properties:

- immutable nodes;
- local handles plus stable content-derived digests;
- no process-global interner;
- structural identity independent of source spans;
- exact literals as first-class nodes;
- binders, relations, logic, conditions, arrays, rules and generic calls;
- arbitrary symbolic powers;
- deterministic canonical bytes;
- iterative traversal and teardown;
- bounded store growth mechanisms suitable for long notebook sessions.

Source locations belong in sidecar origin maps because the same interned term may occur at multiple source locations.

### 2. `Domain` and `Element`

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

The API provides both:

- statically typed Rust paths for hot consumers and generic algorithms;
- a type-erased dynamic path for sessions, notebooks and foreign bindings.

Dynamic values still expose their precise domain and capability set.

### 3. `Scalar`

RV0 inherits CADabra's exact/approximate scalar seam and dual numbers. It serves numeric kernels that legitimately need the same implementation over `f64`, certified exact values, intervals and dual numbers.

It is deliberately not the parent/domain abstraction. Polynomial rings, quotient fields, matrices, power series and symbolic terms need stronger mathematical structure.

### 4. `Value` and `Outcome`

A session-facing `Value` can contain a term, domain element, matrix, collection, table, plot/graphic, plan, receipt, certificate or opaque extension value.

An operation returns an outcome that states its epistemic status explicitly. Exact requests never degrade invisibly to approximation.

## Numeric literal semantics

The parser and wire format distinguish:

- integers;
- rational literals;
- exact decimal literals;
- exact IEEE-754 binary values admitted from host APIs;
- approximate machine floats;
- arbitrary-precision approximate values;
- interval/ball values.

This distinction fixes the current Scientia bridge behavior where an authored decimal may be converted through `f64` and then admitted as the exact binary rational represented by that float.

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

Rules may be grouped into explicit packs. There is no unbounded global `simplify` that applies hidden heuristics indefinitely.

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

Construction, evaluation, canonicalization, rewriting, simplification, expansion, factorization, approximation and code optimization are different operations.

The session may own definitions and assumptions, but term construction itself does not repeatedly invoke an ambient evaluator. This is a deliberate departure from traditional global-evaluation CAS designs and is important for embedding, reproducibility and consumer-owned semantics.

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

Resolvent owns an ordinary bounded rewrite engine with structural/typed patterns, associative/commutative matching where justified, guards, explicit strategies, traces, cycle detection and budgets.

Equality saturation is optional and transient. It is appropriate for bounded code optimization, Hornerization, CSE, stability-oriented transformations and identity exploration. It is not the canonical term representation and not the default evaluation engine.

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

## Kernel and frontend boundary

The internal kernel protocol is transport-neutral. Expected operations include:

- evaluate;
- cancel;
- complete;
- inspect;
- format/render;
- request/explain plan;
- verify receipt/certificate;
- retrieve/store artifact;
- snapshot session;
- replay cell/request;
- package/environment inspection.

Jupyter is an adapter to this protocol, not the protocol itself.

The native notebook eventually supports code, Markdown, typeset math, tables, plots/scenes, algorithm inspection, dependency history, local/remote kernels, receipts/certificates and deterministic replay. Sequential cells are the default; explicitly pure cells may participate in a reactive dependency graph.

## Phase dependency summary

- RV0 is the only hard initial blocker for CADabra R2.
- RV1's wire identity is the blocker for durable Scientia integration and frontend protocol work.
- RV2 and RV3 can overlap after the RV1 identity decisions.
- RV4 builds on RV1-RV3 semantics.
- RV5 contains parallel CADabra, Scientia/Sinbad and evidence lanes.
- RV6/RV7 broaden the standalone CAS after the execution/evidence contracts are exercised by real consumers.
- RV8 starts early at the CLI/protocol/Jupyter layers and matures throughout the program.
- RV9 packages the mature system for broad external use.

## Work-package convention

Each phase file contains stable IDs:

```text
RV<phase>-<lane><number>
```

Examples: `RV1-A1`, `RV3-C2`, `RV5-S4`.

A work package is complete only after its stated acceptance gate lands. Plans are not status. [`../../STATUS.md`](../../STATUS.md) remains the only concise record of implemented truth.

## Parallel implementation rules

1. Keep one owner for every semantic type.
2. Introduce shared generic machinery from a real consumer requirement or a mathematically general domain requirement, not aesthetic deduplication alone.
3. New optimized algorithms ship with a deterministic correctness baseline or independent grader whenever practical.
4. Score/performance lanes begin only after their correctness oracle is stable.
5. Resource-heavy operations expose budgets before broadening capability.
6. Schema/wire changes are explicitly versioned and have round-trip and migration tests.
7. Cross-repository cuts record the exact consumer fixture and preserve downstream gates.
8. Optimized provider paths never become the only way to verify correctness.

## Definition of the first useful standalone CAS milestone

Resolvent is a coherent early standalone CAS when RV0, RV1, the first RV2 domains, RV3's existing-operation vertical slice and RV8's CLI/Jupyter path are complete. That milestone does not require Mathematica-class algorithm breadth. It proves that the architecture scales from embedded Rust through scientific/CAD consumers to an interactive environment before large breadth work begins.