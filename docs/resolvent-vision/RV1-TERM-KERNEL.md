# RV1 - Immutable Term Kernel and Canonical Wire Format

## Goal

Replace the current recursive symbolic tree with the durable **structural** symbolic identity model that later CAS features and frontends share.

The resulting term layer must support library embedding, long-lived notebook sessions, deterministic replay, pattern matching, source/provenance sidecars, and conversion into specialized algebraic domains without turning every mathematical value or consumer semantic expression into a generic tree.

RV1 is a CAS representation program. It is **not** a migration of Scientia's canonical scientific expression arena.

## Design requirements

A Term is retained symbolic structure. It is not:

- a lazy exact numeric recipe;
- a specialized domain element optimized for arithmetic;
- Scientia's scientific `ExprId`/`SemanticModel` identity;
- a Malleus executable numeric/kernel IR.

The store is caller/session owned. There is no process-global or thread-local mutable symbol/interner state.

Local handles may be compact and store-relative. Stable identity across sessions/processes is a content digest over a versioned canonical structural encoding.

### Structural identity is not algebraic canonicalization

RV1 canonical bytes answer: **"is this the same retained symbolic structure?"** They do not answer: **"is this mathematically equivalent?"**

Therefore RV1 construction must not sort, commute, flatten, cancel, factor, reassociate or otherwise algebraically normalize operations unless the operation's structural contract itself explicitly guarantees that representation independent of a mathematical domain.

In particular, a printed head such as `Mul` does not justify assuming commutativity. Matrices, operators and extension-defined products make that unsound. Domain-aware canonicalization belongs to RV2/RV4 and is recorded as an explicit transformation.

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

A textual exact decimal such as `0.1` in a Resolvent-authored syntax must not become the exact binary rational represented by `0.1_f64`.

This does not by itself fix `.res` literals. Scientia owns `.res` parsing and must preserve authored exact literal data before projection into Resolvent.

Exit: golden vectors cover canonical bytes/digests for all atom classes.

### RV1-A2 - Term node and binder specification

Define the minimal general node vocabulary. It must be capable of representing, without consumer-specific opcodes:

- generic function application/head-and-arguments form;
- arithmetic syntax without assuming domain-level algebraic laws;
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

Exit: the wire schema can structurally represent the consumer-neutral scalar algebra projected from current Scientia expressions without semantic erasure, plus generic CAS binders and conditions.

### RV1-A3 - Canonical structural encoding and digest

Define a versioned encoding independent of:

- serde implementation details;
- hash-map iteration order;
- local arena indices;
- pointer addresses;
- process/thread history;
- source spans;
- pretty-print formatting;
- algebraic rewrite/canonicalization history.

Requirements:

- deterministic canonical bytes;
- cryptographic content digest;
- explicit schema/version identifier;
- child order preserved wherever it is part of structural syntax;
- any structurally order-insensitive container explicitly declares that fact in its own representation;
- no hidden mathematical simplification during serialization.

Exit: independent stores that construct the same structural term produce identical bytes/digests regardless of insertion order/process history.

#### Frozen A1-A3 contract (schema version 1)

The reviewed A1-A3 boundary is now concrete:

- atoms have explicit tags for integer, rational, exact decimal, exact IEEE-754
  bits, machine-float bits, precision-bearing decimal real, UTF-8 string,
  bytes, namespaced symbol, boolean, symbolic constant, and de Bruijn variable;
- exact decimals are canonical `coefficient * 10^-scale` values and never pass
  through `f64`; exact IEEE-bit ingress and approximate machine-float intent
  remain different atom classes even when their payload bits match;
- all node vectors, piecewise cases, array elements, application arguments, and
  ordered-map entries retain their authored order. There is no unordered node
  container in schema version 1;
- binders use de Bruijn indices, where zero names the nearest bound variable.
  Bounds are evaluated outside the binder's newly introduced scope. Stores may
  hold open fragments during construction, but a canonical wire root must be
  closed; escaping indices fail with a typed binder error;
- canonical bytes begin with `RESOLVENT-TERM`, a NUL separator, and schema byte
  `1`. Reachable nodes follow deterministic child-first postorder and refer only
  to earlier canonical node indices. Counts and indices use shortest-form
  unsigned LEB128; signed decimal exponents/scales use zigzag encoding;
- node tags are `01` for atoms and `10` through `1b` for application, relation,
  boolean, condition, piecewise, collection, ordered map, index, slice, rule,
  binder, and held syntax. Atom tags are `01` through `0c` in the order listed
  above. Enum variants use explicit frozen sub-tags, never Rust discriminants;
- the root canonical index terminates the stream. Decoding rejects unknown
  tags, forward references, duplicate canonical nodes, overlong varints,
  trailing bytes, noncanonical representations, and all configured budget
  overruns, then requires byte-for-byte re-encoding equality;
- the stable digest is BLAKE3 over these canonical bytes. Local handles, local
  symbol IDs, insertion history, source spans, allocation layout, Serde, and
  rendering do not enter stable identity.

Construction, traversal, import, encoding, and decoding all enforce the
applicable `TermBudget` node/depth/child/atom/wire limits. Interning computes and
stores depth as each node is admitted. An exact hash-cons hit is checked before
charging `max_nodes`; cross-store import first plans the actual deduplicated
target DAG and commits only after the complete plan fits.

`StoreMetrics::logical_bytes` is a portable saturating `u64` schema measure, not
resident memory. Each node/atom/enum/optional tag is one byte, each logical
length and term reference is eight bytes, an interned symbol reference is four
bytes, and integer-width scalar fields use their declared 1/4/8-byte widths.
Variable payloads charge an eight-byte logical length plus payload bytes.
Distinct symbol namespace/name blobs are charged once in the symbol table and
not copied into every symbolic atom's charge. Hash-table buckets, allocator
capacity, pointers, and Rust `size_of` never enter this metric.

Schema evolution requires a new version byte and new golden vectors. Version 1
must not be reinterpreted in place. Golden coverage freezes bytes and BLAKE3
digests for every atom, node, enum, collection, binder, and optional-field
subtag, while independent shared-DAG construction permutations verify that
local insertion order does not leak into identity.

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

Construction normalization is deliberately narrow: atom representation, arity/schema validation, binder alpha/capture invariants, and exact structural interning. Algebraic identities belong to explicit RV2/RV4 operations.

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

Do not add general algebraic rewriting yet.

### RV1-C1 - Source/provenance sidecars

Define an optional generic origin-map abstraction outside Term identity:

```text
TermDigest or TermId -> zero/one/many Origin records
```

Origins may include source file/module, byte span, authored/generated status, parent transformation and consumer-owned IDs.

Resolvent may define a reusable sidecar shape, but Scientia remains the owner of source spans, scientific declaration IDs and scientific expression IDs.

Exit: one canonical Resolvent Term can be associated with multiple consumer origins without changing Term identity.

### RV1-C2 - Lossless Scientia algebra projection

Replace the current temporary algebra conversion limitations **without replacing Scientia's semantic arena**.

The contract is operation-specific:

```text
Scientia SemanticModel / ExprId
    -> project supported scalar algebra
Resolvent Term and/or Domain Element
    -> generic algebra operation
algebra result + receipt/certificate
    -> re-embed or attach under Scientia-owned semantics
```

Requirements:

- Scientia retains its canonical `SemanticModel` and `ExprId` identity;
- preserve exact integer/rational/decimal literal intent once Scientia exposes it;
- preserve supported arithmetic/functions structurally;
- preserve comparisons/conditions for operations where those structures are meaningful;
- do not claim that every representable relation is a valid scalar differentiation input;
- preserve piecewise/conditional scalar structure as it is introduced;
- retain Scientia source spans and semantic IDs on the Scientia side;
- explicitly refuse scientific constructs that are outside the requested generic algebra operation;
- no Resolvent `TermId` becomes durable scientific identity;
- no second algebraically authoritative copy of a transformed scientific model is maintained silently.

### RV1-C3 - Coordinated exact `.res` literal preservation

Scientia currently stores source numeric values as `f64`. Resolvent cannot repair authored decimal exactness after that conversion.

This is a coordinated integration change with clear ownership:

- **Scientia** owns parser/source/schema changes required to preserve exact authored decimal/rational data;
- **Resolvent** owns exact decimal/rational Term atoms, exact conversion APIs and subsequent algebra;
- schema changes are versioned and current 50-model compiler behavior remains covered;
- unit-bearing numeric literals retain Quantitas/Scientia meaning rather than becoming Resolvent-owned quantities.

Exit: an authored `.res` decimal such as `0.1` can reach a Resolvent algebra projection as exact `1/10` when the scientific operation requests exact scalar algebra, without changing Scientia's ownership of the source expression.

### RV1-D1 - Core renderers

Ship deterministic renderers sufficient for library/CLI/frontend development:

- canonical/debug structural representation;
- plain text;
- LaTeX;
- MathML presentation form;
- JSON tree/debug form distinct from canonical binary identity encoding.

Rendering never changes Term identity and does not become a parser-round-trip requirement except where an explicit surface syntax promises one.

### RV1-D2 - Stress/fuzz/determinism gate

Test:

- very deep unary/binary chains;
- very wide applications;
- highly shared DAGs;
- structurally distinct but mathematically equivalent expressions retaining different Term digests until an explicit algebra operation transforms them;
- noncommutative/order-sensitive heads preserving argument order;
- millions of constructed/released transient terms;
- random binder/substitution cases;
- cross-store rebuild;
- serialization fuzzing;
- identical digests across insertion order/process/thread-count matrices for structurally identical terms;
- malformed wire data fails closed.

## Exit gate

RV1 exits when:

- one caller-owned immutable structural Term-store API is public;
- stable identity is digest/wire based rather than local handle based;
- structural identity is explicitly separated from mathematical canonicalization;
- numeric literal exactness is explicit on the Resolvent side;
- relations/conditions/binders are structurally representable;
- Scientia has a lossless operation-specific algebra projection while retaining canonical scientific `ExprId` ownership;
- the coordinated exact `.res` literal path is specified and tested when the Scientia schema change lands;
- source spans/provenance remain sidecar/consumer data;
- deep/shared/session-scale stress tests pass;
- the Term wire identity is stable enough for RV8 parser/CLI work to start.

RV1 completion alone does **not** freeze the full RV8 kernel protocol; stable dynamic values and plan/outcome schemas additionally require the initial RV2/RV3 surfaces.

## Parallelism

A1-A3 are architecture-first and should be reviewed together. B1 can begin once A1-A3 schemas are sufficiently frozen. B2 stress/lifetime prototypes can run alongside B1. C1/C2 can be developed against a minimal B1 slice. C3 is a coordinated Scientia/Resolvent lane and may proceed as soon as its source-schema change is agreed. D renderers and fuzzing can proceed in parallel once structural traversal exists.

## Non-goals

- replacing Scientia's scientific `SemanticModel`/`ExprId` arena;
- scientific tensor/form/differential semantics;
- algebraic canonicalization/rewrite policy (RV2/RV4);
- full domain/coercion system (RV2);
- general evaluator and definitions (RV4);
- pattern matcher/rule engine (RV4);
- broad simplification;
- e-graph implementation;
- notebook UI;
- executable numeric/kernel IR;
- geometry topology.
