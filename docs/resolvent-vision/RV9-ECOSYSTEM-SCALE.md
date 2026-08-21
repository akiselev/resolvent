# RV9 - Packages, Providers, Bindings and Scale

## Goal

Turn the mature Resolvent kernel into a sustainable external ecosystem without weakening determinism, exactness or ownership boundaries.

RV9 is not a single final release. It is the packaging, provider, distribution and scale program that begins once RV1-RV3 contracts are stable and matures as RV6/RV7 breadth lands.

## Principles

- One canonical public semantic model regardless of provider.
- Providers accelerate/extend algorithms but do not define public wire identity.
- Package loading is explicit and versioned.
- Dynamic/plugin ABIs use stable protocol/C handles, not Rust trait-object ABI.
- Cache keys include all semantic context that can change a result.
- Parallelism preserves deterministic mathematical output and records unavoidable nondeterministic execution metadata separately.
- Large objects may be streamed/spilled/content-addressed rather than forcing all data into RAM.

## Work packages

### RV9-A - Package and module system

#### RV9-A1 - Package manifest

Define a versioned package manifest containing:

- package name/version;
- Resolvent API/schema compatibility;
- dependencies/features;
- exported symbols/domains/operations/rules/algorithms;
- documentation/completion metadata;
- optional native/WASM/provider components;
- license/source metadata;
- reproducibility digest.

Package identity is separate from session-local import aliases.

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

#### RV9-B1 - Provider contract

A provider advertises:

- provider ID/version;
- supported operation/algorithm IDs;
- domain/value import/export capabilities;
- exactness/evidence guarantees;
- deterministic behavior guarantees;
- resource/cancellation capabilities;
- licensing/runtime requirements.

Provider selection flows through RV3 planning.

#### RV9-B2 - In-process providers

Support statically linked or feature-selected provider implementations using internal Rust interfaces where semver coupling is acceptable.

No provider-private object appears in canonical serialized `Term`/`Element` identity.

#### RV9-B3 - Stable out-of-process/dynamic providers

For separately distributed providers, use:

- the kernel/provider protocol; or
- a stable C-compatible handle ABI.

Do not expose Rust trait-object/vtable ABI across shared-library boundaries.

Process isolation is preferred for tools with incompatible licenses, fragile runtimes or crash risk.

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

Other providers can be added using the same contract rather than one-off integration APIs.

### RV9-C - Broad bindings

Mature and publish bindings for:

- Rust;
- Python;
- C;
- JavaScript/WASM;
- Julia if demand justifies it;
- language-server/editor protocol clients.

Bindings expose structured terms/domains/outcomes/plans rather than primarily strings.

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

### RV9-E - Content-addressed caching

Define cache keys over:

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

Cache classes:

- exact immutable result cache;
- verified certificate cache;
- expensive domain-conversion cache;
- render cache;
- notebook cell cache;
- provider result cache with validation metadata.

### RV9-F - Large-expression and large-object scale

Add bounded infrastructure for:

- streaming serialization/deserialization;
- chunked/content-addressed large terms and polynomial data;
- disk spill for large intermediates where algorithms permit it;
- memory accounting per request;
- resumable or checkpointable long algorithms only when the mathematical algorithm has a safe checkpoint contract;
- remote artifact references in kernel protocol responses.

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

- `resolvent-scientia` or Scientia-owned adapter for scientific scalar terms/inspection;
- CADabra-owned adapter/provider for exact algebra interoperability and notebook geometry display;
- Methodus-owned adapter for symbolic Jacobian/preprocessing inputs where useful;
- Solverang-owned adapter for algebraic constraint normalization/inspection where useful;
- Sinbad-owned notebook/product extension for simulation campaigns.

Naming/ownership can differ, but dependency direction remains from extension/consumer down to Resolvent.

### RV9-I - Compatibility and release discipline

Define pre-1.0 and 1.0 stability classes:

- canonical wire schemas;
- domain descriptors;
- public Rust facade;
- protocol messages;
- package manifests;
- provider protocol;
- receipt/certificate schemas.

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
- notebook/kernel protocol overhead.

Score lanes use pinned baselines/change-point tracking. Performance work never weakens exactness checks to win a benchmark.

## Exit gate

RV9 reaches mature status when:

- package/module loading is explicit, versioned and deterministic;
- external providers use one stable contract and are visible in RV3 plans/receipts;
- broad language bindings expose structured CAS values;
- major exact workloads parallelize without changing canonical results;
- caches are content/context addressed rather than process-history addressed;
- large requests have bounded memory/artifact strategies;
- agents can discover, plan, execute and verify algebra through structured APIs;
- federation integrations remain adapter/extension dependencies rather than reverse dependencies from Resolvent;
- schema/release compatibility policy is enforced mechanically.

## Parallelism

A/C/G can begin after RV8 dynamic/protocol surfaces stabilize. B begins once RV3 provider identity/plans are stable. D/J proceed per algorithm only after correctness baselines are frozen. E/F can develop alongside provider work because they operate at artifact/protocol boundaries. H is naturally federated across repositories.

## Non-goals

- making every external CAS a linked backend;
- depending on upper federation repositories from the core CAS;
- allowing provider registration order to change mathematical semantics;
- hiding proprietary/copyleft providers inside the permissive core distribution;
- sacrificing deterministic exact output for parallel throughput;
- treating a native notebook or one language binding as the canonical API.