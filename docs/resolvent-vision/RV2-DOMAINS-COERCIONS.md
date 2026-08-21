# RV2 - Domains, Elements, Capabilities and Coercions

## Goal

Give Resolvent an explicit mathematical object model so specialized algebraic values are not forced through the generic symbolic term representation.

RV2 is the Sage/FriCAS/Nemo side of the architecture: parents/domains are first-class, capabilities are mathematical claims, storage is specialized, and automatic coercion follows only canonical embeddings.

## Design principles

1. Every domain element belongs to exactly one mathematical parent/domain.
2. Domain identity includes all data that affects algebra: characteristic, coefficient domain, variables, monomial order, precision, truncation order, matrix shape and extension data.
3. Automatic coercion follows a deterministic graph of canonical embeddings.
4. Lossy, branch-dependent or noncanonical conversions are explicit maps/operations.
5. Hot Rust users have statically typed APIs; notebook/binding users have a dynamic type-erased API over the same domain descriptors.
6. Domain capability claims are not inferred from the presence of similarly named methods.

## Work packages

### RV2-A1 - Domain identity and descriptor schema

Define stable descriptors and digests for domains.

The schema must represent at least:

- integers `ZZ`;
- rationals `QQ`;
- residue rings `Z/nZ`;
- prime finite fields and finite extensions;
- univariate/multivariate polynomial rings;
- rational-function fields;
- real algebraic numbers;
- simple algebraic extensions;
- real/complex precision-bearing approximate/ball domains;
- dense/sparse matrix spaces;
- truncated power/formal series.

Descriptors are serializable mathematical identities. Runtime caches/implementation handles are not serialized as identity.

Exit: equivalent independently constructed domains have identical descriptors/digests; semantically distinct domains cannot collide through omitted configuration.

### RV2-A2 - Capability vocabulary

Define dynamic/static capability concepts corresponding to mathematical statements, for example:

- additive commutative group;
- ring / commutative ring;
- integral domain;
- field;
- ordered field;
- Euclidean domain;
- unique factorization capability;
- exact division;
- gcd domain;
- finite field;
- polynomial/factorization support;
- exact comparison;
- semi-decidable enclosure comparison;
- square/nth-root capability;
- transcendental enclosure capability;
- linear-algebra capability;
- series capability.

Do not make every capability a deep Rust trait inheritance tower if runtime/domain dispatch becomes unwieldy. The static and dynamic representations may differ while sharing one semantic vocabulary.

Exit: operation applicability checks can ask explicit capability questions without type-name switches.

### RV2-B1 - Core exact domains

Implement/standardize first-class domain wrappers for:

- `ZZ`;
- `QQ`;
- modular/residue arithmetic needed by planned algorithms;
- prime finite fields;
- `UPoly<C>` over supported coefficient domains;
- initial sparse `MPoly<C>` representation with explicit variable and monomial order;
- rational functions over supported polynomial domains;
- `RealAlgebraic`/certified real-root domain built on RV0 exact roots.

Where RV0 imported specialized implementations, RV2 wraps them in the domain model rather than rewriting them gratuitously.

### RV2-B2 - Approximate and certified scalar domains

Add explicit domains for:

- machine real/complex values;
- arbitrary-precision approximate real/complex values;
- interval or ball real/complex values suitable for certified evaluation.

Requirements:

- precision is part of dynamic domain/value semantics where relevant;
- approximate values are never confused with exact rationals admitted from binary bits;
- enclosure operations carry their guarantee explicitly;
- exact-to-approximate conversion states requested precision/rounding/enclosure behavior.

### RV2-B3 - Matrices and series

Implement parent-aware containers:

- dense matrix spaces over a domain;
- sparse matrix spaces when needed by exact algebra consumers;
- truncated power/formal series with variable and truncation order in the parent;
- exact matrix operations already present from RV0 routed through the domain interface.

Do not turn Methodus's large-system numerical operator model into a Resolvent matrix abstraction. These are algebraic matrix/domain values and exact/symbolic linear algebra objects.

### RV2-C1 - Static Rust API

Provide strongly typed paths for hot embedding consumers.

Representative goals:

```text
Polynomial<Rational>
Polynomial<Fp>
Matrix<Rational>
AlgebraicReal
```

or equivalent domain-parameterized APIs with explicit contexts where runtime domain data is required.

Requirements:

- no dynamic dispatch in inner loops unless measurement justifies it;
- no process-global domain state;
- domain context ownership remains explicit;
- generic algorithms can be written against meaningful algebraic traits/capabilities.

### RV2-C2 - Dynamic `Domain` / `Element` API

Provide the type-erased surface for sessions and foreign bindings.

A dynamic element exposes:

- domain descriptor/digest;
- canonical serialization;
- operation applicability;
- conversion to a symbolic `Term` when representable;
- formatting/rendering;
- explicit downcast/typed borrow hooks for extension/provider code.

Dynamic equality means mathematical equality inside one compatible domain or an explicitly resolved common domain. It is not pointer identity.

### RV2-D1 - Canonical coercion graph

Define canonical embeddings such as:

```text
ZZ -> QQ
ZZ -> ZZ[x]
QQ -> QQ[x]
QQ[x] -> Frac(QQ[x])
GF(p) -> GF(p^k) when the embedding is part of the field construction
base domain -> matrix space over that domain
base domain -> power-series ring over that domain
```

The graph must be:

- deterministic;
- inspectable;
- cycle checked;
- able to explain a chosen common parent;
- independent of registration order;
- resistant to ambiguous diamonds.

An operation on mixed parents either finds one unique canonical common target or returns a typed coercion ambiguity/failure.

### RV2-D2 - Explicit noncanonical maps

Provide explicit conversion/map objects for operations such as:

- exact value -> approximate value;
- rational/real -> integer under rounding/floor/ceiling;
- polynomial variable substitution;
- quotient/projection maps;
- choosing an algebraic embedding;
- precision changes that lose information;
- interpreting a term in a domain under assignments.

These never enter the automatic coercion graph merely because they are convenient.

### RV2-E1 - Term/domain bridge

Implement both directions carefully:

- recognize when a symbolic term belongs to a specialized domain (`as_polynomial`, `as_rational_function`, matrix/series recognition);
- construct a canonical symbolic term from an element for display/rewrite interoperability;
- preserve domain identity when the same printed expression could belong to different parents;
- return a witness/reason when specialization fails.

This bridge is a major performance and correctness boundary: algorithms should move into specialized domains as early as possible and return to generic terms only when needed.

### RV2-E2 - Domain law and coercion testing

For every domain/canonical embedding:

- algebraic law properties;
- canonical serialization round trips;
- descriptor determinism;
- coercion homomorphism properties;
- ambiguous/noncanonical cases that must be refused;
- cross static/dynamic equality;
- randomized term <-> domain recognition round trips where applicable.

## Exit gate

RV2 exits when:

- core exact and approximate mathematical domains are explicit first-class objects;
- dynamic and static APIs agree on semantics;
- automatic coercion uses only canonical, deterministic embeddings;
- noncanonical/lossy conversions require explicit operations;
- generic terms can enter specialized polynomial/rational/matrix/series domains without hidden semantic loss;
- no consumer needs to inspect concrete backend type names to ask whether an algebraic operation applies.

## Parallelism

A1/A2 define shared semantics. B1/B2/B3 can then fan out by domain family. C1/C2 are interface lanes over those implementations. D coercion work can begin once at least three parent families exist. E1 should be exercised early with `QQ`, `QQ[x]` and `Frac(QQ[x])` rather than deferred until all domains are complete.

## Non-goals

- broad factorization/Groebner/solving algorithms (RV6);
- session definitions/assumptions (RV4);
- numerical solver algorithms owned by Methodus;
- constraint graph semantics owned by Solverang;
- scientific units or physical meaning;
- geometry/topology.