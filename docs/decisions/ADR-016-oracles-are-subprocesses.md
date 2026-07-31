# ADR-016 — Oracles are subprocesses; the workspace has exactly two crate categories

**Status:** Ratified 2026-07-31
**Reversibility:** cheap
**Amended:** 2026-07-31 — every adapter ships an **oracle calibration corpus**; benchmark
families need a Tier-A citation or are dropped; published crates carry zero dev-dependencies
(critique-plan C16, C22; critique-engineering §18).
**Gates lanes:** H4, and every DIFF-graded lane.
**Evidence:** `docs/research/prior-art-and-licensing.md` §5, §7;
`docs/research/algorithms-and-representation.md` §3.2–§3.4, §6.3, §7.4, §10;
`docs/research/critique-plan.md` C16, C22.

---

## Context

Differential testing against mature systems is one of the two verification pillars (the
other, self-certification, is primary). Every system worth testing against is copyleft:
Singular (GPL-2/3), PARI/GP (GPL-2+), FLINT (LGPL-3+), msolve (GPL-2), Macaulay2 (GPL-2/3),
SageMath (GPL-2+), Maxima (GPL-2+), CoCoALib (GPL-3). The one permissive option, sympy
(BSD-3), is already importable on this machine.

Linking a GPL library into a permissive library is not permitted at all. FLINT is LGPL, so
linking is *conditionally* permitted — and `flint-sys` (MIT/Apache **bindings**) would give
in-process speed that is materially better than subprocess round-trips for high-volume
property testing. But `flint-sys`'s repository (`alex-ozdemir/flint-rs`) has **no license
file on GitHub** despite its crates.io metadata, which would need resolving first, and
LGPL §4's relinking condition is unsettled for statically linked Rust anyway (ADR-001).

Availability on this machine, verified 2026-07-31 with `pacman -Si` (**none currently
installed**): `singular` 4.4.1 (11.4 MiB download / 59.3 MiB installed), `pari` 2.17.4
(small), `flint` 3.6.0, `sagemath` 10.9 (56.4 MiB / 371.2 MiB, ~60 transitive deps),
`maxima`, `python-sympy` 1.14.0 (already present). `macaulay2` is AUR-only and needs a
source build. msolve is not packaged for Arch.

---

## Decision

### 1. Subprocess-only, uniformly

**Every external oracle is driven as a subprocess over a text protocol.** No oracle code
enters resolvent's binary or its dependency graph. One rule, no exception process.

If FLINT's in-process speed ever proves necessary: it goes in a `publish = false` crate,
behind a non-default feature, not in any published crate's dependency graph, with the LGPL
obligation documented in that crate's README — **and** the `flint-sys` license file must be
resolved upstream first. Prefer the uniform rule.

### 2. Exactly two crate categories, and no third

- **`publish = true`** — gated by `cargo-deny` against a permissive allowlist over the
  published graph (ADR-001 §gates).
- **`publish = false`** — `resolvent-oracles`, `resolvent-bench`, `resolvent-fuzz`, `xtask`.
  May carry LGPL dev-dependencies (`rug` as the bignum oracle) and may shell out to GPL
  binaries.

There is no third **category** and no per-crate exception; the crate list above may grow
within the `publish = false` category. Gate L6 asserts that no published crate depends on an
unpublished one, including through dev-dependencies, and **gate L6a asserts that every
published crate's `[dev-dependencies]` table is empty** — which is what makes L6 enforceable
rather than merely stated (ADR-005). The consequence, stated so nobody rediscovers it: **all
property tests and differential oracles live in `resolvent-oracles`** and exercise only the
public surface of the crate under test. For `resolvent-int` that is sufficient by design,
because the newtype wall means the public surface is the whole point.

### 3. Self-certification is the primary gate; oracles are secondary

Per constraint #4 and R3 §10, and this ordering matters operationally:

- **A self-certifying failure is a bug in resolvent with certainty.** Factorization
  multiplies back; gcd checks divisibility both ways **plus the degree condition**; Gröbner
  checks membership via stored cofactors; resultants satisfy three structural invariants;
  root isolation is graded by exact Sturm counts.
- **An oracle disagreement might be a normalization difference** — leading-coefficient sign,
  monomial-order convention, unit factors, root ordering — and needs triage.

**These are different signals and an agent lane must not treat them as equivalent.** A lane
brief that says "test against Singular" without saying "and self-certify first" will
generate triage work that looks like bugs.

### 4. Tiering by install cost, so a fresh clone still tests something

| Tier | Contents | When it runs |
|---|---|---|
| **0** — zero install | `sympy` via `python3` | Always. Already available. |
| **1** — cheap, developer-recommended | `singular` + `pari` (~70 MiB combined) | Local dev, and CI. |
| **2** — CI box only | `sagemath`, `macaulay2` (AUR), `msolve` (source build) | Nightly / release gates. |

**A missing oracle SKIPs loudly and is counted. It never passes.** A test run reports
`N passed, M skipped (oracles absent: …)` and CI fails if the skip count exceeds a
per-tier budget. Silently green because nothing was installed is the failure mode this
rule exists to prevent.

### 5. Text protocol, canonicalized on both sides

Each oracle gets an adapter that emits a canonical form of the input and parses a canonical
form back — using the **same serializer** as ADR-012 §9, which lives in `resolvent-base`.
This makes disagreements triageable and keeps oracle-specific parsing out of test bodies.

**Round-tripping an adapter proves nothing about the oracle, so every adapter also ships a
calibration corpus.** *Added 2026-07-31.* A round-trip is resolvent → S-expression →
resolvent: it exercises resolvent's own encoder and decoder and never establishes that the
oracle read the same polynomial. An adapter that emits variables in the wrong order, or
emits a polynomial in `x` where resolvent meant `y`, round-trips perfectly and then produces
confident agreement or confident disagreement **about the wrong object** — and §4.3's own
note that "order convention is the number-one false disagreement" puts the burden on the
comparison rather than on the adapter.

> **Calibration corpus:** per operation, a dozen instances whose answers are hand-computed
> and committed, with the *oracle's* answer asserted against them. `Res(x²−2, x²−3)`,
> `gcd(x²−1, x³−1)`, `factor(x⁴+1)` over ℚ and over `GF(3)`, `isolate_roots` of a Chebyshev
> polynomial, `subresultants` of a pair with a known degree sequence. If the oracle's answer
> to a known-answer instance is wrong, the **adapter** is wrong, and this is the only test
> that can say so. It is also what detects an oracle version bump changing a convention.

The calibration corpus is part of lane H4's deliverable, not a follow-up.

### 6. Best oracle per operation

Architecture claims about what each tool specializes in, not benchmark claims:

| Operation | Primary | Secondary |
|---|---|---|
| Gröbner (drl, lex, elimination) | **Singular** (`std`/`groebner`) | Macaulay2, msolve |
| Multivariate factorization over ℚ/ℤ | **Singular** (`factorize`) | sympy, PARI |
| Univariate factorization over ℤ/ℚ/GF(p)/number fields | **PARI/GP** (`factor`, `nffactor`) | FLINT, Singular |
| Resultants / subresultant PRS | **PARI/GP** (`polresultant`) | sympy `subresultants` — gives the **whole PRS chain**, which is the actual intermediate data the lane must match |
| Real root isolation | **PARI/GP** (`polrootsreal`, certified intervals) | sympy `real_roots`, `CRootOf` |
| Algebraic-number comparison | **sympy** (`CRootOf`, `minimal_polynomial`) | PARI, Sage `QQbar` |
| Bignum integer arithmetic | **`rug`** (dev-dependency, in-process — the only oracle worth linking, and precedent from `dashu`'s own `fuzz/`) | PARI |
| Fallback for everything | **SageMath** | — |

### 7. Internal oracles are built deliberately, and gate their performance lanes

Three algorithms are built **as oracles**, knowing they will never be the production path:

- **Sturm** gives the *exact* count of distinct real roots in an interval, which grades
  every Descartes output automatically. Descartes only gives an upper bound congruent
  mod 2; Sturm turns it into an equality assertion.
- **Buchberger** grades F4.
- **Zassenhaus** grades van Hoeij for `r ≤ 20`.

Plus two independent resultant implementations (Ducos PRS and modular
evaluation-interpolation) that share almost no code, which makes them a strong differential
pair at zero marginal cost.

**A performance lane may not start until its oracle passes.** Do not start F4 before
Buchberger passes; do not start ANewDsc before plain Descartes passes; do not start van
Hoeij before Zassenhaus passes. Without a frozen baseline, a number lane has no verdict
function at all, and constraint #3 requires one.

### 8. Benchmark instances are generated and *asserted*, not assumed

Conventions differ by an index shift, and that silently changes which instance is being
timed. Guards:

- **Katsura-`n`**: the ideal degree is a checkable invariant — `2^(n-1)` under msolve's
  naming (Katsura-9 → 256, …, Katsura-14 → 8192). **A generator that does not reproduce the
  published degree is generating a different system.** Assert it in the harness, do not
  comment it.
- **Cyclic-`n`**: variables `x_0..x_{n-1}`; `f_k = Σ_i Π_{j=i}^{i+k-1} x_{j mod n}` for
  `k = 1..n-1`, and `f_n = x_0⋯x_{n-1} − 1`.
- **Eco-`n`, Noon-`n`, Reimer-`n`** — *amended 2026-07-31.* The original instruction was
  "pin them to the Groebner.jl benchmark repo generators and record the file hashes". That
  is an instruction to transcribe a generator out of a **GPL-2.0** test suite into an MIT
  repository — the exact Tier-B transcription ADR-001 forbids without exception, arrived at
  by following the verification plan literally, in the one lane nobody would look for a
  licensing problem. Instead: **every benchmark family carries a Tier-A citation** (the
  original paper) in its generator's metadata, checked by the same CI rule as `Derivation:`
  (ADR-001 gate 5). The defining recurrences for Eco-`n` and Noon-`n` are published and are
  short. **A family with no Tier-A source is dropped, not transcribed.** The system itself
  is a published mathematical object; the *generator source file* is someone's copyrighted
  expression of it, and those are different things.
- **Every family additionally commits the SHA-256 of the generated system** at each `n` used,
  so a generator edit is a visible diff rather than a silent change of instance.

---

## Consequences

- **License risk at the oracle boundary is zero**, because nothing links. That is worth more
  than the subprocess round-trip cost for a testing path.
- **Subprocess overhead is real** — a Singular invocation is tens of milliseconds — which
  caps property-test volume against external oracles. Mitigated by: self-certification
  carrying the high-volume load (it is in-process and free), batching many instances per
  oracle invocation, and `rug` being in-process for the bignum lane specifically.
- **The oracle harness is itself a lane with an automatic verdict**: adapters must round-trip
  a canonical corpus, and a missing oracle is a counted skip.
- **Building three throwaway algorithms (Sturm, Buchberger, Zassenhaus) costs real time.**
  It is the price of every downstream number lane having a baseline, and it is cheaper than
  the alternative, which is number lanes with no verdict.
- **The Katsura/Cyclic assertions will fail on first write.** That is the point — index
  conventions differ, and finding it at harness-authoring time is free.

---

## Alternatives considered and why rejected

**Link FLINT via `flint-sys` for speed.** Tempting for high-volume property testing.
Rejected as the default: LGPL §4 relinking is unsettled for statically linked Rust, the
bindings repo has no license file despite its metadata, and a single uniform rule is
enforceable where a conditional one is not. Available as a `publish = false` non-default
feature if measurement ever demands it.

**Vendor a permissive oracle instead** (e.g. use sympy exclusively). Rejected: sympy is the
only permissive option and it is the weakest oracle for exactly the hard cases — no van
Hoeij, no LLL, Buchberger-only Gröbner. It is Tier 0 because it is free, not because it is
sufficient.

**Skip external oracles entirely; rely on self-certification.** Rejected. Self-certification
does not test irreducibility (a coarse factorization passes the multiply-back check), does
not test the `⟨G⟩ ⊆ I` half of Gröbner without cofactors, and does not catch a *systematic*
misunderstanding shared by both of resolvent's own implementations.

**A per-crate license exception process for oracle dependencies.** Rejected — exception
processes are how `alkahest-cas` ships an Apache-2.0 crate with mandatory LGPL
dependencies. Two categories, no exceptions.

**Treating a missing oracle as a pass.** Rejected explicitly, because it is the default
behaviour of most test harnesses and it produces a green CI that tests nothing.

---

## What would reverse this

- **Subprocess overhead measurably capping the property-test volume needed to find a class
  of bug.** Response: the `publish = false` FLINT path, with the license file resolved
  upstream first. That is a scoped exception inside the existing two-category rule, not a
  new category.
- **An oracle becoming unavailable** (Macaulay2's AUR package breaking, msolve's build
  bit-rotting). Response: it drops to a counted skip in Tier 2 and the harness reports it.
  No code change.
- **A permissively licensed CAS of real capability appearing.** It would become a linkable
  Tier-0 oracle and a very good one. Nothing about the two-category rule changes.
