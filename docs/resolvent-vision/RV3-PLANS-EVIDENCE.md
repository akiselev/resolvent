# RV3 - Algebra Plans, Outcomes, Receipts and Certificates

## Goal

Make algorithm choice and result evidence first-class before Resolvent accumulates a large body of independently implemented CAS functions.

The existing operations provide the first vertical slice. RV3 does not wait for broad algebra: it wraps canonicalization, differentiation, exact sign, polynomial GCD/resultant and root isolation in the final request -> plan -> execution -> outcome -> receipt architecture.

## Principles

- Mathematical operation identity is separate from algorithm identity.
- An operation may have several implementations/providers with different applicability/performance but the same mathematical contract.
- Planner selection is deterministic under a fixed request/context/provider set.
- Resource limits are explicit inputs and explicit outcomes.
- Exactness/conditionality/uncertainty are represented in the return type.
- Receipts record execution facts; certificates prove or check mathematical claims where practical.
- A certificate checker must not simply invoke the algorithm it is meant to certify.

## Work packages

### RV3-A1 - `AlgebraRequest` and operation identities

Define a generic request envelope and stable operation IDs.

The request captures, where applicable:

- operation ID;
- input term/element digests;
- domain descriptors;
- assumption/context digest;
- requested exactness/precision;
- resource budget;
- explicit algorithm/provider override;
- deterministic seed policy;
- desired evidence level;
- output-shape/options that affect mathematics.

Operation IDs are semantic and versioned only when mathematical behavior changes. Algorithm IDs are separately versioned.

### RV3-A2 - Typed `AlgebraOutcome`

Implement a common outcome vocabulary without forcing every API to erase useful typed result data.

Required statuses:

- exact;
- conditional;
- certified enclosure/approximation;
- approximate;
- unknown/indeterminate;
- unsupported;
- resource limit;
- invalid-domain/input.

Typed convenience APIs may unwrap narrower guaranteed outcomes when their domain contract makes other states impossible.

### RV3-B1 - `AlgebraPlan` schema

A plan records the selected realization before/while execution:

```text
operation id
algorithm id/version
provider id/version
applicability reasons/features
required domain capabilities
normalization/preprocessing steps
fallback chain
budget allocation
expected guarantee/certificate
child/subplans
```

Plans have stable digests and are serializable/replayable artifacts.

### RV3-B2 - Deterministic planner

Implement an initial rule-based planner.

The planner may inspect:

- domain/capabilities;
- degree;
- term count/sparsity;
- coefficient bit size;
- matrix dimensions;
- expression structure/sharing;
- requested exactness/precision;
- available providers;
- resource budget.

Selection must not depend on hash-map order, wall clock, global process history or nondeterministic provider enumeration.

An explicit `algorithm = ...` request bypasses selection but still runs applicability checks.

### RV3-B3 - Algorithm descriptor registry

Each implementation registers a descriptor containing:

- operation family;
- algorithm/version;
- domain/capability predicate;
- structural classifier;
- guarantee;
- budget model;
- certificate/evidence kind;
- fallback relationship;
- deterministic/nondeterministic metadata;
- provider identity.

Registration is session/kernel construction data, not uncontrolled global mutable state.

### RV3-C1 - Receipt schema v2

Replace the current digest-only receipt with a versioned record containing, when relevant:

```text
schema
operation
input digest(s)
output digest(s)
context/assumption digest
domain descriptors
requested exactness/precision
plan digest
algorithm id/version
provider id/version
requested budget
consumed resources
seed/randomness identity
conditions/warnings
certificate kind/digest
checker version
```

Receipts remain small metadata. Large traces/certificates are separate content-addressed artifacts referenced by digest.

### RV3-C2 - Certificate interface and checkers

Define a generic certificate envelope plus operation-specific certificate types.

Initial certificate/checker targets:

- polynomial GCD: divisibility plus Bezout relation where available;
- resultant: independent evaluation/subresultant witness appropriate to the implemented route;
- real-root isolation: isolating intervals plus independent root-count certificate;
- exact comparison/sign: refinement/separation witness when nontrivial;
- differentiation: structural derivation trace and/or independent polynomial-domain agreement;
- canonicalization: value-preserving transformation trace for rules that are not constructor identities.

Checkers have explicit versions and import restrictions where necessary to prevent circular verification.

### RV3-C3 - Evidence grades

Define a small explicit evidence taxonomy, for example:

- `Proof` / mathematically complete certificate;
- `Certified` / rigorous enclosure or exact verifier;
- `IndependentlyVerified` / structurally independent implementation/oracle agreement;
- `Differential` / external system agreement;
- `Probabilistic` / randomized verification with stated parameters;
- `Heuristic` / useful but not sufficient for promotion.

Do not label multiply-back alone as proof of irreducibility or completeness.

### RV3-D1 - Existing-operation vertical slice

Move the existing public operations through the request/plan/evidence path:

- expression canonicalization;
- symbolic differentiation;
- exact sign/evaluation;
- polynomial GCD;
- resultant;
- real-root isolation.

Keep ergonomic direct Rust functions as thin typed entry points over the same execution machinery where appropriate.

Exit: CLI/debug examples can print `plan -> outcome -> receipt -> certificate check` for each operation.

### RV3-D2 - Replay and `explain`

Provide APIs that can:

- replay a stored plan against the same input/context/provider identities;
- explain why an algorithm was selected;
- explain why alternatives were inapplicable;
- show fallback decisions and resource-limit causes;
- verify a receipt/certificate without rerunning the production algorithm where the certificate permits it.

### RV3-E1 - Resource accounting

Expand budgets and track actual consumption categories appropriate to algorithms:

- term/node visits;
- coefficient bit growth;
- polynomial terms/degree;
- modular primes;
- matrix dimensions/elimination work;
- root subdivisions/refinements;
- rewrite steps/e-graph nodes later;
- wall-clock cancellation checkpoints as a runtime concern, not a source of deterministic mathematical identity.

Budget exhaustion returns a `ResourceLimit` outcome with consumed counts and a safe partial/residual result when mathematically meaningful.

### RV3-E2 - Planner/evidence mutation suite

Plant deliberately wrong implementations/checkers and prove the system rejects them, for example:

- GCD always returning one;
- root isolator dropping one interval;
- resultant with a sign/convention bug;
- differentiation omitting one product term;
- certificate checker that accidentally calls the production algorithm.

## Exit gate

RV3 exits when:

- the existing algebra surface executes through deterministic plans;
- result exactness/resource status is explicit;
- receipt v2 records semantically relevant choices;
- at least GCD/root isolation/differentiation have useful independent certificate or oracle checks;
- plan replay and explanation are public APIs;
- algorithm/provider selection order is deterministic;
- mutation cases demonstrate that evidence checkers catch plausible wrong answers.

## Parallelism

A1/A2 and B1 can proceed together after RV1 identity exists. B2/B3 depend on descriptors but can be exercised against mock algorithms. C1 can be drafted early. Operation-specific certificate/checker lanes are highly parallel once the common envelope lands. D1 should integrate one operation at a time so the architecture is exercised before bulk migration.

## Non-goals

- learned planner selection as a production dependency;
- general rewrite/rule engine (RV4);
- full provenance archive/store such as Artifactum integration unless a concrete cross-repository evidence need requires it;
- broad algorithm catalog (RV6/RV7);
- frontend transport protocol (RV8).