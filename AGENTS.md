# Agent instructions

Resolvent is a consumer-neutral algebraic CAS. It may own expression/term
representation, exact and certified scalar mathematics, mathematical domains and
coercions, bounded rewriting/canonicalization, generic symbolic calculus,
polynomial/resultant/root/ideal/series operations, algebra plans, receipts and
mathematical certificates.

Do not add `.res` syntax, scientific semantics, geometry/topology, meshes,
runtime simulation state, Methodus numerical-solver policy, Solverang constraint
semantics, compatibility facades, or consumer-specific dispatch. Scientia and
CADabra must consume the same public algebra rather than copying it.

Methodus owns consumer-neutral numerical algorithms and operator contracts.
Solverang owns generic constraint solving and its reusable 2D/3D constraint
vocabulary, using Methodus for numerical work. Resolvent may provide generic
algebraic subroutines to either without taking over those semantics.

Every potentially expensive operation needs an explicit budget or bounded input
contract. An unavailable decision returns a typed error/outcome; it never becomes
an approximate exact answer. Exact-to-approximate conversion must be explicit.

## Resolvent Vision work

[`PLAN.md`](PLAN.md) and `docs/resolvent-vision/` define the proposed RV0-RV9
implementation program. `STATUS.md` is landed truth and must never claim planned
capability.

For RV implementation work:

1. name the work-package ID (for example `RV1-A2`) in the PR title;
2. state the owning repository and downstream consumer(s);
3. preserve one semantic owner and avoid duplicate expression/domain IRs;
4. add a deterministic correctness baseline or independent grader before
   performance optimization when practical;
5. include a real consumer fixture for consumer-pulled generic machinery;
6. keep expensive work budgeted and evidence-bearing operations receipt-aware;
7. update the phase ledger only after implementation actually lands.

Do not treat historical planning documents in Git history as automatically
normative. Reintroduce earlier decisions only after reconciling them with the
current consolidated exact substrate and the active RV plan.

Before handoff run formatting, locked checks, clippy with warnings denied, all
tests, rustdoc with warnings denied, doctests, and `git diff --check`. Keep
`STATUS.md` concise and truthful.