# Agent instructions

Resolvent is a consumer-neutral algebraic CAS. It may own structural symbolic
Term representation, exact and certified scalar mathematics, mathematical
domains and coercions, bounded rewriting/canonicalization, generic symbolic
calculus, polynomial/resultant/root/ideal/series operations, algebra plans,
receipts, and mathematical certificates.

Do not add `.res` syntax or scientific semantics, CAD geometry/topology,
mesh/discretization semantics, runtime simulation state, Methodus numerical
solver policy, Solverang constraint semantics, Malleus executable numeric IR,
compatibility facades, or consumer-specific dispatch.

## Federation boundaries

- Scientia owns one canonical scientific `SemanticModel` / `ExprId` arena and
  `.res` source-literal meaning. Project supported scalar algebra into Resolvent
  for a specific operation; do not replace Scientia expression identity with a
  Resolvent `TermId`.
- CADabra owns geometry/topology truth, branch/sheet/trim meaning, persistent
  entities, geometry events, and certification policy.
- Malleus owns finite-precision structured computation, iteration/index/effect
  semantics, AD execution products, backend lowering, and emitted kernels.
  Resolvent may return optimized algebraic Terms/domain values and explicit
  CSE/let schedules, not a competing general numeric SSA/kernel IR.
- Methodus owns consumer-neutral numerical algorithms and operator contracts,
  including future physics-neutral Krylov/nonlinear/DAE/eigen/optimization/
  sampling/reduction methods as consumers require them.
- Solverang owns generic constraint graphs, candidate solve orchestration, and
  conflict/redundancy/DOF/activation semantics plus reusable 2-D/3-D constraint
  vocabulary, using Methodus for numerical work.
- Outboard owns reusable external executable discovery/worker lifecycle.
  Artifactum owns reusable durable artifact/provenance lifecycle. Use optional
  adapters when those capabilities are needed instead of cloning them into the
  CAS core.

## Resolvent Vision work

[`PLAN.md`](PLAN.md) and `docs/resolvent-vision/` define the proposed RV0-RV9
capability program. [`docs/resolvent-vision/CROSS-ROADMAP-CONTRACT.md`](docs/resolvent-vision/CROSS-ROADMAP-CONTRACT.md)
is authoritative for cross-repository sequencing. `STATUS.md` is landed truth
and must never claim planned capability.

RV phase numbers are **not** blanket prerequisites. For each work package:

1. name the work-package ID (for example `RV1-A2`) in the PR title;
2. state the owning repository and downstream consumer(s);
3. name the minimum typed prerequisites, not just a lower RV number;
4. preserve one semantic owner and avoid duplicate scientific/executable IRs;
5. add a deterministic correctness baseline or independent grader before
   performance optimization when practical;
6. include a real consumer fixture for consumer-pulled generic machinery;
7. include a dissimilar second consumer/use case or a general mathematical
   justification before promoting consumer-specific machinery;
8. keep expensive work explicitly budgeted and evidence-bearing operations
   receipt/certificate aware;
9. update a phase ledger only after implementation actually lands.

CADabra-pulled generic algebra may start as soon as its actual exact/domain
prerequisites exist; it does not wait for the RV5 phase exit. RV8 parser/CLI
prototypes may start with RV1, but stable protocol/Jupyter schemas wait for the
initial RV2 dynamic-value and RV3 outcome/plan/receipt contracts.

## Term-store rule

RV1 Term identity is structural, not mathematical equivalence. Do not silently
sort, commute, flatten, factor, cancel, reassociate, or otherwise algebraically
canonicalize nodes merely to stabilize hashing. Preserve structural child order
unless the node's structural schema itself declares order irrelevant. Apply
algebraic laws only through explicit domain/rewrite operations.

Every potentially expensive operation needs an explicit budget or bounded input
contract. An unavailable decision returns a typed error/outcome; it never becomes
an approximate exact answer. Exact-to-approximate conversion must be explicit.

Do not treat historical planning documents in Git history as automatically
normative. Reintroduce earlier decisions only after reconciling them with the
current consolidated exact substrate, live consumers, and active RV plan.

Before handoff run formatting, locked checks, clippy with warnings denied, all
tests, rustdoc with warnings denied, doctests, and `git diff --check`. Keep
`STATUS.md` concise and truthful.
