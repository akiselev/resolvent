# Agent instructions

Resolvent is a consumer-neutral exact algebra library. It may own expression
representation, bounded rewriting/canonicalization, exact differentiation,
polynomial/resultant/root operations, exact comparison, and algebra receipts.

Do not add `.res` syntax, scientific semantics, geometry/topology, meshes,
runtime state, solver policy, compatibility facades, or consumer-specific
dispatch. Scientia and CADabra must consume the same public algebra rather than
copying it.

Every potentially expensive operation needs an explicit budget or bounded
input contract. An unavailable decision returns a typed error; it never becomes
an approximate exact answer.

Before handoff run formatting, locked checks, clippy with warnings denied, all
tests, rustdoc with warnings denied, doctests, and `git diff --check`. Keep
`STATUS.md` concise and truthful.
