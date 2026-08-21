# RV1 - Immutable Term Kernel and Canonical Wire Format

## Goal

Replace the current recursive symbolic tree with the durable symbolic identity model that every later CAS feature and frontend will share.

The resulting term layer must support compiler embedding, long-lived notebook sessions, deterministic replay, pattern matching, rich source/provenance sidecars, and conversion into specialized algebraic domains without turning every value into a generic tree.

## Design requirements

A term is retained symbolic structure. It is not a lazy exact numeric recipe and it is not a domain element optimized for arithmetic.

The store is caller/session owned. There is no process-global or thread-local mutable symbol/interner state.

Local handles may be compact and store-relative. Stable identity across sessions/processes is a content digest over a versioned canonical structural encoding.

## Work packages

### RV1-A1 - Canonical atom and literal specification

Specify before implementing the new store:

- small/big integers;
- exact rationals;
- exact decimal literals with lexical/canonical semantics;
- exact IEEE-754 bit-pattern ingress as a distinct host API;
- approximate machine/precision-bearing numeric literals;
- strings and byte strings where needed by symbolic data;
- symbols with namespace/package identity;
- booleans and distinguished symbolic constants;
- stable canonical serialization for each atom.

A textual exact decimal such as `0.1` must not become the exact binary rational represented by `0.1_f64`.

Exit: golden vectors cover canonical bytes/digests for all atom classes.

### RV1-A2 - Term node and binder specification

Define the minimal general node vocabulary. It must be capable of representing, without consumer-specific opcodes:

- generic function application/head-and-arguments form;
- associative arithmetic surface (`Add`, `Mul`) where useful for canonical construction;
- arbitrary powers;
- relations (`Eq`, `Ne`, ordering relations);
- boolean connectives;
- conditions and piecewise expressions;
- tuples/lists/maps/arrays as symbolic values;
- indexing/slicing forms;
- rules/pattern forms reserved for RV4;
- binders for lambda/function, sums, products, integrals, limits and local scopes;
- held/unevaluated forms.

Binder identity must avoid accidental capture. Choose and document one explicit model such as de Bruijn-style bound variables, scoped symbol IDs or alpha-normalized binder identities.

Exit: the wire schema can represent all currently required Scientia scalar expressions without semantic erasure, plus generic CAS binders and conditions.

### RV1-A3 - Canonical structural encoding and digest

Define a versioned encoding independent of:

- serde implementation details;
- hash-map iteration order;
- local arena indices;
- pointer addresses;
- process/thread history;
- source spans;
- pretty-print formatting.

Requirements:

- deterministic canonical bytes;
- cryptographic content digest;
- explicit schema/version identifier;
- canonical ordering rules for structurally commutative containers only where mathematically justified;
- no hidden simplification during serialization.

Exit: independent stores built in different insertion orders produce identical bytes/digests for equivalent constructed terms under the same construction-normalization rules.

### RV1-B1 - Caller-owned hash-consed `TermStore`

Implement:

- compact `TermId` handles;
- symbol/function-name interning within the store/session;
- structural hash-consing;
- iterative traversal;
- topological walk;
- node/term count and memory accounting;
- safe cross-store rebuild/import;
- stable digest computation;
- thread-safe read access after construction or a documented concurrent-construction model;
- no recursive drop/evaluation path that can overflow on deep terms.

Construction normalization is deliberately narrow: canonical atom construction, optional flattening of explicitly associative node kinds, obvious identities where the constructor contract promises them, and hash-consing. General algebraic simplification belongs to RV4.

### RV1-B2 - Store lifetime and notebook-scale retention

A notebook kernel may live for days and create millions of transient terms. Design bounded retention explicitly.

Evaluate at least:

- strong arena ownership for session-persistent terms;
- weak interning for reclaimable derived terms;
- generation/epoch stores;
- snapshot/rebuild into compact stores;
- content-addressed external artifacts for very large terms.

Requirements:

- outstanding public handles can never silently change referent;
- reclaim/compaction cannot invalidate live handles without a typed generation failure;
- deterministic digests do not depend on reclamation strategy.

Exit: committed stress test creates and releases large transient workloads without unbounded retained-node growth under the selected policy.

### RV1-B3 - Structural query API

Provide the substrate required by later consumers/frontends:

- node/head/children inspection;
- symbols/free symbols;
- bound/free variable analysis;
- topological traversal;
- term size/depth/shared-node metrics;
- substitution that is binder safe;
- structural replacement by exact location/path;
- content digest and canonical bytes;
- cross-store rebuild.

Do not add general rewriting yet.

### RV1-C1 - Source/provenance sidecars

Define an origin-map abstraction outside the term identity:

```text
TermDigest or TermId -> zero/one/many Origin records
```

Origins may include source file/module, byte span, authored/generated status, parent transformation and consumer-owned IDs.

Resolvent defines the generic sidecar shape if useful, but Scientia remains the owner of source spans and scientific declaration identities.

Exit: one canonical term can be associated with multiple distinct Scientia source origins without changing term identity.

### RV1-C2 - Lossless Scientia bridge

Replace the current temporary algebra conversion limitations.

Requirements:

- preserve exact integer/rational/decimal literal intent;
- preserve comparisons instead of lowering them to zero;
- preserve supported function calls structurally;
- preserve piecewise/conditional scalar structure as it is introduced;
- retain Scientia source spans and semantic IDs outside the term;
- explicitly refuse scientific constructs that are not scalar algebra rather than fabricating a term;
- support direct references from Scientia's semantic arena to Resolvent terms or a deterministic bridge artifact without maintaining two algebraically authoritative scalar trees.

Exit:

- current differentiation cases remain green;
- new regression cases cover comparisons and exact decimal literals;
- round trip through the bridge does not pass through `f64` for authored exact literals;
- no `.res` type moves into Resolvent.

### RV1-D1 - Core renderers

Ship deterministic renderers sufficient for library/CLI/frontend development:

- canonical/debug structural representation;
- plain text;
- LaTeX;
- MathML presentation form;
- JSON tree/debug form distinct from the canonical binary identity encoding.

Rendering never changes term identity and does not become a parser-round-trip requirement except where an explicit surface syntax promises one.

### RV1-D2 - Stress/fuzz/determinism gate

Test:

- very deep unary/binary chains;
- very wide sums/products;
- highly shared DAGs;
- millions of constructed/released transient terms;
- random binder/substitution cases;
- cross-store rebuild;
- serialization fuzzing;
- identical digests across insertion order/process/thread-count matrices;
- malformed wire data fails closed.

## Exit gate

RV1 exits when:

- one caller-owned immutable term-store API is public;
- stable identity is digest/wire based rather than local handle based;
- numeric literal exactness is preserved explicitly;
- relations/conditions/binders are structurally representable;
- Scientia uses a lossless generic algebra bridge;
- source spans/provenance remain sidecar data;
- deep/shared/session-scale stress tests pass;
- the kernel wire identity is stable enough for RV8 protocol work to start.

## Parallelism

A1-A3 are architecture-first and should be reviewed together. B1 can begin once A1-A3 schemas are sufficiently frozen. B2 stress/lifetime prototypes can run alongside B1. C1/C2 can be developed against a minimal B1 slice. D renderers and fuzzing can proceed in parallel once canonical traversal exists.

## Non-goals

- full domain/coercion system (RV2);
- general evaluator and definitions (RV4);
- pattern matcher/rule engine (RV4);
- broad simplification;
- e-graph implementation;
- notebook UI;
- scientific tensor/form semantics;
- geometry topology.