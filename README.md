# Resolvent

Resolvent is the consumer-neutral algebraic CAS used directly by Scientia and
CADabra. It owns exact rational expressions, scalar and dual-number vocabulary,
bounded deterministic canonicalization, exact symbolic differentiation,
interval and expansion filters, univariate polynomial arithmetic and
resultants, certified real-root isolation, radical and lazy-exact numbers,
exact matrices, exact sign queries, and algebra receipts.

It does not own `.res`, scientific fields/forms/methods, geometry/topology,
meshes, runtime state, or solver policy. The FC0-FC11 scientific compiler moved
to the standalone Scientia repository during R1; Git history preserves its
earlier residence here.

The current crate is intentionally one cohesive package. Split packages only
when real consumers require independently versioned capability boundaries.
The R1 consolidation moved CADabra's former generic exact/scalar implementation
and tests here and deleted the old crates. There is no adapter, compatibility
facade, or second algebra backend.
