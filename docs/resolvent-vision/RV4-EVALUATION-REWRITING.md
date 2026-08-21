# RV4 - Evaluation, Assumptions and Rewriting

## Goal

Add the semantic machinery required for a serious symbolic language without collapsing all symbolic behavior into one ambient `evaluate until stable` loop.

RV4 separates definitions, assumption reasoning, canonicalization, directed rewriting, simplification strategies and optional equality saturation. Every potentially explosive transformation is bounded and inspectable through RV3 plans/receipts.

## Principles

- Term construction is not general evaluation.
- Definitions and assumptions belong to an explicit session/context.
- Algebraic identities are guarded by domains, assumptions and branch conventions.
- A rule that is valid over positive reals is not automatically valid over complex numbers.
- `Unknown` is a valid result of assumption reasoning.
- Simplification is strategy-dependent and must expose the strategy/plan used.
- Equality saturation is optional, transient and bounded.

## Work packages

### RV4-A1 - Immutable `AssumptionContext`

Define a persistent/immutable assumption context with stable digest.

It should represent and query facts such as:

- domain membership (`x in Reals`, `n in Integers`);
- equality/inequality;
- zero/nonzero;
- positive/nonnegative/negative;
- finite/infinite where relevant;
- parity and integrality;
- interval/range bounds;
- simple algebraic relations.

Queries return proved true, proved false or unknown, optionally with a proof/evidence reference.

Context combination detects direct contradiction rather than silently selecting one assumption.

### RV4-A2 - Assumption inference tiers

Implement staged reasoning so simple queries remain cheap.

Suggested order:

1. direct facts;
2. domain-derived facts;
3. structural sign/range propagation;
4. polynomial/rational exact decision where supported by RV2/RV5;
5. optional heavier decision procedures behind explicit budgets.

Every inference tier is an RV3 algorithm descriptor so `explain` can say why a proposition is known or unknown.

### RV4-B1 - Session definitions

Define explicit session-local definitions separate from immutable term identity.

Support eventually:

- symbol values;
- function definitions;
- delayed vs immediate definition semantics if the language needs both;
- local scopes;
- attributes relevant to evaluation/patterns;
- package/module namespaces.

Do not make definitions process-global. Two sessions in one process must be independent.

### RV4-B2 - Held and controlled evaluation

Add held/unevaluated forms and explicit evaluation controls sufficient for:

- macros/rules;
- inspecting arguments before evaluation;
- delayed definitions;
- symbolic code/data manipulation;
- notebook pedagogy/debugging.

Evaluation is implemented as a bounded plan over explicit definitions/attributes, not hidden constructor recursion.

### RV4-C1 - Pattern representation and matcher

Implement typed structural patterns with:

- single-node variables;
- sequence variables;
- literal/head constraints;
- domain/capability guards;
- binder-safe matching;
- optional associative/commutative matching for operations that declare those properties;
- deterministic match ordering;
- match budgets.

Pattern identity/serialization is versioned so rule packs can be content-addressed and replayed.

### RV4-C2 - Rule catalog and provenance

A rule contains:

```text
rule id/version
left pattern
right/template
assumption/domain guard
branch convention
orientation
priority/strategy tags
complexity hint
verification strategy
source/provenance
```

Initial built-in packs should be small and high-confidence:

- ring identities;
- rational normalization;
- elementary arithmetic powers;
- selected elementary-function identities with correct branch/domain guards;
- differentiation cleanup rules.

Do not begin by importing thousands of integration/simplification rules.

### RV4-C3 - Directed rewrite engine

Provide explicit strategies such as:

- once/top-down;
- once/bottom-up;
- repeat-to-fixed-point under a step/node budget;
- rule-pack sequence;
- innermost/outermost;
- targeted subterm rewrite.

Outputs include a rewrite trace or digest-linked trace artifact recording rule IDs and locations.

Cycle detection must distinguish a genuine fixed point from oscillation.

### RV4-D1 - Public transformation surface

Define clear separate operations:

- `canonicalize`;
- `rewrite`;
- `simplify` with explicit strategy/profile;
- `expand`;
- `factor` (delegating specialized algebra to domains);
- `cancel`/rational normalization;
- `refine` under assumptions;
- `approximate`;
- `optimize` for code-generation cost models later.

Avoid one generic function whose behavior depends on hidden global settings.

### RV4-D2 - Branch and condition semantics

Centralize conventions for branch-sensitive operations:

- logarithm;
- roots/powers;
- inverse trigonometric functions;
- complex arguments;
- piecewise/conditional identities.

Representative acceptance cases:

- `sqrt(x^2)` under unknown `x` does not become `x`;
- under real `x`, it may become `abs(x)`;
- under `x >= 0`, it may become `x`;
- cancellation does not remove a denominator without recording `denominator != 0` when the result would otherwise change domain;
- power/log identities preserve complex branch conditions.

### RV4-E1 - Optional equality-saturation backend

Add an internal/provider seam for bounded equality saturation only after the directed rule system is stable.

Use cases:

- Horner-form exploration;
- arithmetic reassociation under an exact domain;
- CSE/code-size optimization;
- alternative derivative forms;
- bounded identity search.

Requirements:

- transient e-graph built from `Term`/domain data;
- explicit rule pack;
- node/iteration/time/cost budget;
- extraction cost model recorded in the plan;
- output re-imported as ordinary Resolvent terms/elements;
- no persistent `TermId` identity tied to e-graph IDs.

`egg`/`egglog` may be evaluated as optional implementations, but RV4's public semantics do not depend on either crate's representation.

### RV4-E2 - Simplification and rewrite verification

Testing must include:

- exact evaluation equality over finite fields/rationals where applicable;
- domain-guard mutation tests;
- branch-cut regression corpus;
- rule-cycle corpus;
- deterministic match/extraction order;
- size-growth budget regressions;
- comparison with external CAS simplification on a classified corpus without assuming textual identity.

## Exit gate

RV4 exits when:

- assumptions are explicit immutable context data;
- simple assumption queries return proved true/false/unknown;
- session definitions are isolated and deterministic;
- held forms and evaluation controls exist;
- patterns/rules are first-class versioned data;
- directed rewriting is bounded and traced;
- branch/condition correctness is regression-tested;
- public `simplify` is an explicit strategy/profile rather than an unbounded ambient evaluator;
- any equality-saturation implementation is optional and transient.

## Parallelism

A1/A2 assumption work and B1/B2 session/evaluation work can proceed in parallel after RV1/RV2 basics. C1 pattern infrastructure can begin from RV1 terms. C2/C3 follow. D2 branch semantics should be specified early and used as a gate for rule packs. E1 is deliberately late within the phase so it does not dictate the core term representation.

## Non-goals

- complete theorem proving;
- unrestricted transcendental equality decision;
- full integration/ODE solving;
- scientific assumptions owned by Scientia;
- geometry event policy owned by CADabra;
- numerical convergence policy owned by Methodus;
- constraint activation/DOF semantics owned by Solverang.