# RV9 - Packages, Providers, Bindings and Scale

## Goal

Turn the mature Resolvent kernel into a sustainable external ecosystem without weakening determinism, exactness or ownership boundaries.

RV9 is not a single final release and does not wait for all earlier RV phases. Its lanes begin from their actual prerequisites: provider semantics from RV3, package/session semantics from RV4, protocol/bindings from RV8, and algorithm-specific correctness baselines from the owning algebra family.

## Principles

- One canonical public mathematical semantic model regardless of provider.
- Providers accelerate/extend algorithms but do not define public wire identity.
- Package loading is explicit and versioned.
- Do not create a second generic executable-plugin framework: use Outboard optionally when external executable discovery/worker lifecycle is needed.
- Do not create a second general durable artifact lifecycle: use Artifactum optionally for durable large artifacts/provenance/lineage when needed.
- Rust dynamic-library ABI is not a stable plugin ABI.
- Cache keys include all semantic context that can change a result.
- Parallelism preserves deterministic mathematical output and records unavoidable nondeterministic scheduling metadata separately.
- Large objects may be streamed/spilled/content-addressed rather than forcing all data into RAM.

## Work packages

### RV9-A - Package and module system

#### RV9-A1 - Package manifest

Define a versioned Resolvent package manifest containing:

- package name/version;
- Resolvent API/schema compatibility;
- dependencies/features;
- exported symbols/domains/operations/rules/algorithms;
- documentation/completion metadata;
- optional native/WASM/provider components;
- license/source metadata;
- reproducibility digest.

Package identity is separate from session-local import aliases and from executable-provider identity.

#### RV9-A2 - Package loading and namespace isolation

Support:

- deterministic dependency resolution;
- explicit imports;
- namespaced symbols/functions;
- isolated registration of rules/algorithms;
- version conflict diagnostics;
- no implicit process-global mutation from merely linking a crate.

#### RV9-A3 - Documentation and discovery

Generate searchable documentation from operation/domain/rule/algorithm descriptors.

Expose the same metadata to:

- CLI help;
- REPL completion;
- Jupyter inspection;
- native notebook browser;
- agent protocol clients.

### RV9-B - Provider architecture

Resolvent owns the **mathematical provider contract**. Generic external-process lifecycle remains an optional Outboard concern.

#### RV9-B1 - Mathematical provider contract

A provider advertises:

- provider ID/version;
- supported operation/algorithm IDs;
- domain/value import/export capabilities;
- exactness/evidence guarantees;
- deterministic behavior guarantees;
- resource/cancellation capabilities relevant to algebra planning;
- licensing/runtime requirements.

Provider selection flows through RV3 planning and appears in plans/receipts.

The contract says what mathematical operation a provider can perform and how its result is verified. It does not define a second executable discovery/worker framework.

#### RV9-B2 - In-process providers

Support statically linked or feature-selected provider implementations using internal Rust interfaces where semver coupling is acceptable.

No provider-private object appears in canonical serialized `Term`/`Element` identity.

#### RV9-B3 - External executable providers through Outboard

When a provider is separately installed or process isolation is desirable, implement a thin Resolvent/Outboard adapter rather than new discovery/worker infrastructure.

Outboard already owns reusable:

- executable naming/discovery and shadow diagnostics;
- manifest/interface/framework version negotiation;
- typed argv invocation;
- one-shot process handling;
- persistent worker lifecycle;
- progress events;
- cooperative cancellation;
- process isolation;
- worker framing and compatibility checks.

Resolvent defines the small application/domain interface describing algebra request/result artifacts and maps provider identity/capabilities into RV3 descriptors.

If a future requirement is not supported by Outboard, first determine whether it is generic plugin-framework work that belongs in Outboard or genuinely algebra-specific protocol work that belongs here.

Outboard remains optional; pure/core Resolvent does not depend on it.

#### RV9-B4 - FLINT-class optimized provider

Evaluate FLINT as a high-value optional provider for:

- integer/rational arithmetic;
- finite fields;
- polynomial arithmetic/factorization;
- matrices;
- algebraic numbers;
- series;
- rigorous real/complex arithmetic through available Arb/FLINT facilities.

Requirements:

- canonical Rust reference/evidence path exists for every promoted operation or an independent verifier is available;
- provider-version identity is recorded;
- differential corpus establishes semantic normalization conventions;
- LGPL/provider licensing remains isolated from the permissive core according to the chosen linking/distribution policy.

An in-process FLINT provider and an out-of-process provider are separate deployment choices under one mathematical provider contract.

### RV9-C - Broad bindings

Mature and publish bindings for:

- Rust;
- Python;
- C;
- JavaScript/WASM;
- Julia if demand justifies it;
- language-server/editor protocol clients.

Bindings expose structured Terms/domains/outcomes/plans rather than primarily strings.

Ownership/lifetime rules are documented explicitly for arena-relative handles and dynamic values.

### RV9-D - Deterministic parallel algorithms

Parallelize suitable workloads while preserving deterministic semantic output:

- independent modular primes;
- polynomial multiplication/factorization stages;
- Gröbner matrix operations;
- expression traversals;
- independent branches of algebraic solving;
- arbitrary-precision evaluation subproblems.

Requirements:

- deterministic ordering/tie breaking;
- stable canonical output across configured thread counts where the operation promises exact deterministic output;
- explicit seed streams for randomized algorithms;
- receipts distinguish semantic identity from performance scheduling details;
- correctness baseline remains available.

### RV9-E - Mathematical memoization and caching

Resolvent owns cache identity for deterministic algebra operations:

```text
operation/request digest
input digests
domain descriptors
assumption/context digest
plan/algorithm semantic version
provider/version when provider output semantics differ or validation depends on it
requested exactness/precision
relevant package environment
```

Do not key on local arena handles or wall-clock state.

Core/local cache classes may include:

- exact immutable result memoization;
- verified certificate memoization;
- expensive domain-conversion memoization;
- render cache;
- notebook cell cache;
- provider result cache with validation metadata.

This is **not** a general artifact lifecycle system. Resolvent cache entries are implementation/productivity data unless explicitly promoted to durable artifacts.

#### Artifactum boundary

When results need durable content-addressed storage, cross-repository lineage, reproducible transformations, remote mirroring/distribution, leases/GC, action history, or long-lived large evidence artifacts, use Artifactum through an optional adapter if its contract fits.

The adapter maps Resolvent semantic digests/receipt/certificate metadata to Artifactum artifacts/attestations/actions without making Artifactum identity the mathematical identity of a Term or Element.

### RV9-F - Large-expression and large-object scale

Add bounded **algebra-specific** infrastructure for:

- streaming canonical serialization/deserialization;
- bounded chunk iteration over large internal algebra structures;
- disk spill for algorithm intermediates where the algorithm permits it;
- memory accounting per request;
- resumable/checkpointable long algorithms only when the mathematical algorithm has a safe checkpoint contract;
- content-addressed references in kernel protocol responses.

Do not build a second generic blob/tree CAS, remote artifact server, action executor, provenance graph or distribution system. Durable versions of those capabilities belong to Artifactum.

A resource-limited operation returns a typed outcome, not an OOM-driven process failure as ordinary control flow.

### RV9-G - Agent-facing algebra API

Expose machine-oriented discovery and execution:

- list/search operations/domains/algorithms/rules;
- inspect exact schemas and examples;
- request a plan without executing;
- execute under explicit budgets;
- retrieve receipts/certificates;
- verify artifacts;
- minimize/canonicalize expressions for context-efficient exchange;
- structured capability errors with suggested applicable alternatives.

The agent API is the ordinary structured kernel API optimized for discoverability, not an LLM-specific hidden backdoor.

### RV9-H - Federation extension packages

Provide independently versioned extension packages/adapters rather than adding upper-layer dependencies to core Resolvent.

Expected integrations:

- Scientia-owned adapter for scientific algebra projection/inspection;
- CADabra-owned adapter/provider for exact algebra interoperability and notebook geometry display;
- Methodus-owned adapter for symbolic Jacobian/preprocessing inputs where useful;
- Solverang-owned adapter for algebraic constraint normalization/witnesses where useful;
- Sinbad-owned notebook/product extension for simulation campaigns;
- optional Resolvent-Outboard adapter for external algebra providers;
- optional Resolvent-Artifactum adapter for durable algebra artifacts/evidence.

Naming can differ, but dependency direction remains from extension/consumer down to Resolvent or laterally through explicit adapter packages; core Resolvent does not depend on upper federation repositories.

### RV9-I - Compatibility and release discipline

Define pre-1.0 and 1.0 stability classes:

- canonical Term wire schemas;
- domain descriptors;
- public Rust facade;
- protocol messages;
- package manifests;
- mathematical provider contract;
- receipt/certificate schemas.

Outboard worker protocol and Artifactum artifact schemas remain owned/versioned by those projects and are consumed through adapters rather than copied into Resolvent schemas.

Breaking schema changes require explicit version transitions and fixtures. Internal algorithm implementations can evolve freely when mathematical behavior and evidence contracts remain compatible.

### RV9-J - Performance program

Maintain benchmark families separated by workload shape:

- integer bit size;
- polynomial degree/variables/sparsity/coefficient height;
- Gröbner benchmark family;
- matrix dimensions/domain;
- expression nodes/sharing;
- rewrite/e-graph size;
- special-function precision;
- root isolation difficulty;
- notebook/kernel protocol overhead;
- provider adapter overhead where relevant.

Score lanes use pinned baselines/change-point tracking. Performance work never weakens exactness checks to win a benchmark.

## Exit gate

RV9 reaches mature status when:

- package/module loading is explicit, versioned and deterministic;
- mathematical providers use one Resolvent contract and are visible in RV3 plans/receipts;
- external executable providers reuse Outboard rather than duplicating generic plugin lifecycle;
- durable general artifact/provenance needs reuse Artifactum rather than duplicating its lifecycle;
- broad language bindings expose structured CAS values;
- major exact workloads parallelize without changing canonical results;
- caches are content/context addressed rather than process-history addressed;
- large requests have bounded algebra-specific memory/streaming strategies;
- agents can discover, plan, execute and verify algebra through structured APIs;
- federation integrations remain adapter/extension dependencies rather than reverse dependencies from Resolvent;
- schema/release compatibility policy is enforced mechanically.

## Parallelism

RV9 lanes are dependency-local rather than globally late:

- B begins once RV3 provider identity/plans are stable;
- A begins once RV4 package/session namespace semantics are stable enough;
- C/G build on the relevant RV8 dynamic/protocol surfaces;
- D/J proceed per algorithm only after correctness baselines are frozen;
- E/F can begin when stable request/Term/domain digests exist;
- H is naturally federated across repositories.

No RV9 lane waits for the RV9 phase exit, and most do not wait for all RV6/RV7 breadth.

## Non-goals

- making every external CAS a linked backend;
- building another Outboard-style executable plugin host;
- building another Artifactum-style durable artifact/provenance system;
- depending on upper federation repositories from the core CAS;
- allowing provider registration order to change mathematical semantics;
- hiding proprietary/copyleft providers inside the permissive core distribution;
- sacrificing deterministic exact output for parallel throughput;
- treating a native notebook or one language binding as the canonical API.
