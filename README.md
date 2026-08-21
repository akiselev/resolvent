# Resolvent

Resolvent is the consumer-neutral algebraic CAS used directly by Scientia and
CADabra. It owns exact rational expressions, scalar and dual-number vocabulary,
bounded deterministic canonicalization, exact symbolic differentiation,
interval and expansion filters, univariate polynomial arithmetic and
resultants, certified real-root isolation, radical and lazy-exact numbers,
exact matrices, exact sign queries, and algebra receipts.

It does not own `.res`, scientific fields/forms/methods, geometry/topology,
meshes, runtime state, numerical solver policy, or constraint-system semantics.
The FC0-FC11 scientific compiler moved to the standalone Scientia repository
during R1; Git history preserves its earlier residence here.

The current crate is intentionally one cohesive package. Split packages only
when real consumers require independently versioned capability boundaries.
The R1 consolidation moved CADabra's former generic exact/scalar implementation
and tests here and deleted the old crates. There is no adapter, compatibility
facade, or second algebra backend.

## Resolvent Vision

[`PLAN.md`](PLAN.md) defines the proposed RV0-RV9 program to evolve this exact
algebra substrate into a full embeddable CAS with:

- a caller-owned immutable symbolic term kernel and stable wire identity;
- explicit mathematical domains, capabilities and canonical coercions;
- deterministic algorithm planning, budgets, receipts and certificates;
- assumptions, definitions, patterns and bounded rewriting;
- consumer-critical algebra for CADabra and Scientia/Sinbad;
- broad exact algebra, equation solving, calculus, special functions and
  certified scalar numerics;
- CLI/REPL, a transport-neutral kernel protocol, Jupyter and a native notebook;
- packages, optional providers, broad bindings, content-addressed caching and
  agent-facing structured APIs.

The roadmap keeps federation ownership explicit. Methodus owns generic numerical
methods. Solverang owns generic constraint solving and its reusable 2D/3D
constraint vocabulary, using Methodus for numerical algorithms. Resolvent may
supply reusable algebra to either but does not absorb their solver semantics.

See [`docs/resolvent-vision/README.md`](docs/resolvent-vision/README.md) for the
architecture and phase execution model, and [`STATUS.md`](STATUS.md) for landed
implementation truth.