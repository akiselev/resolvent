# Resolvent

Resolvent is a consumer-neutral algebraic CAS currently used by Scientia. It
owns exact rational expressions, bounded deterministic canonicalization, exact
symbolic differentiation, univariate polynomial arithmetic, univariate
resultants, real-root isolation, exact sign queries, and algebra receipts.

It does not own `.res`, scientific fields/forms/methods, geometry/topology,
meshes, runtime state, or solver policy. The FC0-FC11 scientific compiler moved
to the standalone Scientia repository during R1; Git history preserves its
earlier residence here.

The current crate is intentionally one cohesive package. Split packages only
when real consumers require independently versioned capability boundaries.
