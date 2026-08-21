# Prior art and design lessons

This document records the architectural precedents behind the Resolvent Vision roadmap. It is not a compatibility target and it does not authorize copying implementation code from systems with incompatible licenses.

## Wolfram Language / Mathematica

References:

- Expressions: https://reference.wolfram.com/language/tutorial/Expressions.html
- Evaluation: https://reference.wolfram.com/language/tutorial/Evaluation.html
- Patterns and transformations: https://reference.wolfram.com/language/guide/RulesAndPatterns.html
- WSTP: https://reference.wolfram.com/language/guide/WSTPAndExternalProgramCommunication.html
- Notebook structure: https://reference.wolfram.com/language/guide/NotebookStructure.html

### Useful ideas

- Uniform symbolic structure makes formulas, rules, code and many data objects inspectable through one vocabulary.
- Patterns and rules are first-class rather than hidden behind special-case simplifiers.
- The notebook front end is separable from the kernel.
- Rich outputs are structured values, not terminal text.

### What not to copy directly

- Pervasive implicit evaluation as the only semantic model is difficult for library embedding and reproducible compiler use.
- Global symbol/evaluation state is a poor fit for multiple independent embedded sessions in one process.
- A broad `Simplify`-style surface without inspectable planning and bounded strategies makes behavior hard to reason about for compiler/CAD consumers.

### Resolvent conclusion

Adopt a uniform `Term` surface, first-class rules and kernel/frontend separation. Keep construction, evaluation, canonicalization, rewriting and algorithm execution explicit and bounded.

## Maple and OpenMaple

References:

- Maple product/help: https://www.maplesoft.com/products/Maple/
- OpenMaple: https://www.maplesoft.com/support/help/Maple/view.aspx?path=OpenMaple
- OpenMaple Python project: https://github.com/Maplesoft/openmaple

### Useful ideas

- A full interactive CAS can also expose an embeddable engine API.
- Worksheet clients and programmatic clients can share one mathematical kernel.
- Foreign bindings should expose structured values rather than requiring command-line subprocess parsing.

### Resolvent conclusion

Embedding is a primary product, not a wrapper added after the notebook. Rust must remain the authoritative API; Python/C/Jupyter/native notebook clients sit above the same kernel protocol.

## SageMath

References:

- Categories primer: https://doc.sagemath.org/html/en/reference/categories/sage/categories/primer.html
- Coercion model: https://doc.sagemath.org/html/en/reference/coercion/index.html
- Parents/elements tutorial material: https://doc.sagemath.org/html/en/thematic_tutorials/coercion_and_categories.html

### Useful ideas

- Mathematical values belong to explicit parents.
- Parent/category/capability information is semantic, not incidental type metadata.
- Canonical coercions between compatible parents are modeled explicitly.
- Specialized backends can coexist behind a coherent mathematical surface.

### Risks to avoid

- Coercion systems become difficult to reason about when noncanonical conversions are admitted implicitly.
- A large dynamic object model can impose significant overhead if used on every hot path.

### Resolvent conclusion

Make `Domain`/`Element` first-class, use canonical embeddings for automatic coercion, require explicit maps for noncanonical/lossy conversion, and preserve statically typed Rust paths for performance-critical consumers.

## FriCAS / Axiom tradition

References:

- FriCAS documentation: https://fricas.github.io/
- FriCAS source/documentation repository: https://github.com/fricas/fricas

### Useful ideas

- Categories describe algebraic capabilities separately from concrete domains.
- Algorithms can be expressed against mathematical capabilities rather than a closed list of number types.

### Resolvent conclusion

The dynamic domain system needs a capability vocabulary corresponding to mathematical claims: ring, field, ordered field, Euclidean domain, exact division, factorization, algebraic closure properties, enclosure capability, and so on. Avoid pretending an operations-only scalar trait carries those claims.

## Nemo / AbstractAlgebra

References:

- Nemo: https://nemocas.github.io/Nemo.jl/latest/
- AbstractAlgebra: https://nemocas.github.io/AbstractAlgebra.jl/latest/

### Useful ideas

- High-level generic algebra and specialized high-performance representations can coexist.
- Parent objects encode coefficient rings, variable sets and structural choices.
- Backends such as FLINT can accelerate operations without determining the public mathematical model.

### Resolvent conclusion

Use specialized representations for domain elements and permit optional optimized providers after a canonical Rust path and evidence contracts exist.

## SymPy

References:

- Core: https://docs.sympy.org/latest/modules/core.html
- Polynomial domains: https://docs.sympy.org/latest/modules/polys/domainsintro.html
- Assumptions: https://docs.sympy.org/latest/guides/assumptions.html

### Useful ideas

- Immutable symbolic objects are straightforward to compose and cache.
- The polynomial subsystem's domain model demonstrates why dedicated algebra representations matter even in a symbolic system.
- Three-valued assumptions are necessary because many propositions are undecidable from current knowledge.

### Risks to avoid

- Parallel historical assumptions mechanisms are a warning against allowing semantic context to accrete without one explicit model.
- General expression operations can become slow when specialized domains are not used aggressively enough.

### Resolvent conclusion

One assumption-context model, explicit `Unknown`, and a strong bridge between generic terms and specialized domains.

## GiNaC and SymEngine

References:

- GiNaC: https://www.ginac.de/
- SymEngine: https://github.com/symengine/symengine

### Useful ideas

- Symbolic algebra can be designed as a language-native library rather than only as an interactive environment.
- Compact C++ expression representations and direct embedding are useful precedents for a Rust-first CAS.

### Resolvent conclusion

The Rust library surface is the primary artifact. Interactive clients should never be required to access core algebra.

## Symbolica

References:

- Product/docs: https://symbolica.io/
- Documentation: https://symbolica.io/docs/

### Useful ideas

- Modern CAS performance depends heavily on specialized polynomial/rational-function representations, fast pattern matching, compiled evaluation and explicit structural transformations.
- Rust can support a serious symbolic-algebra API and high-performance manipulation.
- Symbolic compilation and numerical evaluation are useful first-class workflows.

### Resolvent conclusion

Treat Symbolica as an important performance and ergonomics oracle. Keep Resolvent's own semantics, deterministic session model and permissive implementation independent.

## FLINT and Arb-style certified numerics

References:

- FLINT: https://flintlib.org/
- python-flint documentation: https://python-flint.readthedocs.io/

### Useful ideas

- A serious CAS needs broad high-performance support for integers, rationals, finite fields, polynomials, matrices, algebraic numbers, power series and rigorous real/complex approximation.
- Algorithm selection based on degree, coefficient size and sparsity is a normal requirement rather than an optimization afterthought.
- Ball arithmetic provides a practical certified-numerics model for transcendental and numerical work.

### Resolvent conclusion

FLINT is a strong eventual optional provider and differential oracle. The canonical public semantics and wire format remain Resolvent-owned. Provider identity, version and exactness must appear in receipts.

## Jupyter

References:

- Messaging: https://jupyter-client.readthedocs.io/en/stable/messaging.html
- Notebook format: https://nbformat.readthedocs.io/
- Kernel docs: https://docs.jupyter.org/en/latest/projects/kernels.html

### Useful ideas

- The kernel is a separate process/protocol participant.
- Rich display is a structured MIME bundle rather than plain text.
- A common protocol unlocks JupyterLab, VS Code and remote execution without building a frontend first.

### Resolvent conclusion

Ship a Jupyter kernel before a native notebook, but define a smaller Resolvent-owned transport-neutral kernel protocol first so Jupyter semantics do not become internal semantics.

## Pluto.jl

Reference: https://plutojl.org/

### Useful ideas

- Reactive notebooks expose dependency relationships and reduce hidden stale-state problems.
- A notebook can treat cells as a program graph rather than only as a mutable transcript.

### Resolvent conclusion

Support both ordinary sequential/session cells and explicitly pure reactive cells. Expensive CAS computations must not rerun unexpectedly, so reactivity is opt-in and budget-aware.

## OpenMath and MathML

References:

- OpenMath: https://openmath.org/
- MathML: https://www.w3.org/Math/

### Useful ideas

- Presentation syntax and semantic mathematical interchange are different concerns.
- Stable symbol vocabularies/content dictionaries are useful for external interchange.

### Resolvent conclusion

Use an internal versioned canonical wire format for identity and replay. Provide MathML for browser-facing presentation and OpenMath interoperability where useful. Do not make either external standard the internal arena representation.

## egg / egglog and equality saturation

References:

- egg paper: https://arxiv.org/abs/2004.03082
- egg: https://github.com/egraphs-good/egg
- egglog: https://github.com/egraphs-good/egglog

### Useful ideas

- Equality saturation is valuable when many equivalent representations must be explored under an extraction cost model.
- E-graphs are particularly useful for code optimization, CSE-like transformations, arithmetic reassociation and bounded identity search.

### Risks to avoid

- The representation can grow rapidly.
- Extraction policy is part of semantics/performance.
- An e-graph is a poor persistent user-facing expression identity model.

### Resolvent conclusion

Keep a Resolvent-owned immutable term store. Equality saturation is an optional transient algorithm behind the rewrite/optimization interface, not the canonical representation.

## Cross-system synthesis

The architecture selected by RV0-RV9 follows these combined lessons:

1. **Uniform syntax, specialized values.** User-facing symbolic structure is uniform; algebraically meaningful domains use specialized storage and algorithms.
2. **Explicit parents/domains.** Mathematical context is first-class and coercions are deliberate.
3. **Library first.** Interactive use is a client of the same embeddable kernel.
4. **No hidden loss of exactness.** Exact, conditional, certified approximate, approximate, unknown and resource-limited outcomes are distinguishable.
5. **Planning is observable.** Algorithm choice, provider, assumptions, resource budgets and verification identity are artifacts.
6. **Rewriting is bounded and explicit.** Equality saturation is optional rather than ambient.
7. **Frontend state is not algebraic truth.** Sessions own definitions/assumptions; canonical term/domain identity does not depend on UI history.
8. **Real consumers drive genericity.** CADabra and Scientia provide immediate hard requirements while standalone CAS breadth grows behind the same contracts.