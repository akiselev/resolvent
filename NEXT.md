# NEXT — start here

This is the smallest sequence of work that produces a **running, self-checking artifact
before any real algorithm exists**. It is meant to be actionable without reading anything
else; every claim links out to the decision that justifies it.

**The thesis, and it is the whole reason for the ordering below.** The oracle loop must
exist before the algorithms do, so that every subsequent line of code arrives with a verdict
function already waiting for it. Founding constraint #3 says resolvent is built primarily by
agents graded by oracles; that only works if the grader is older than the code.

**By the end of week 1 there is:** a workspace where two independently written
implementations grade each other automatically, on generated adversarial input, under a step
budget, with a minimizer that reduces any disagreement to its smallest form; a license gate
that has been *observed rejecting* three real-world hazards; a determinism check that every
future artifact depends on; and a coefficient-ring trait tower that has been through a
compiler. **No algebra of consequence exists yet, and that is correct.**

---

## Day 0 — ratification (human, half a day, and it blocks everything)

Nothing in `docs/decisions/` is in force until someone reads it and merges it. Ratification
is defined as exactly that act (ADR-021 §2) — not a discussion, not an agent's assessment,
not the file existing.

1. Read `docs/decisions/README.md`, then the ADRs in its **Reading order** list. The three
   that repay the most attention are **ADR-021** (how these documents relate to each other
   and to `API.md` and `ROADMAP.md`), **ADR-006** (the trait tower — a one-way door that was
   wrong until 2026-07-31 and is the subject of Day 5), and **ADR-023** (every certificate
   ships a mutant set, which changes what "done" means for roughly fifty operations).
2. For each ADR you accept, confirm its `**Status:** Ratified 2026-07-31` line. For each you
   do not, change it to `**Status:** Proposed` and write why in the file. A `Proposed` ADR
   blocks its lanes automatically once Day 1 lands `lanes.toml`, which is the point.
3. Merge. **That commit is the ratification.**

Wave 0's lanes gate only on ADR-001, ADR-005, ADR-016, ADR-021 and ADR-024, so if the rest
take longer, Day 1–4 can proceed regardless.

---

## Day 1 — the gates that cost nothing now and everything if skipped

### Files

```
Cargo.toml                    # workspace, lockstep version, resolver = "2"
deny.toml                     # explicit [licenses] allow list; every copyleft SPDX denied
lanes.toml                    # lane -> gating ADRs, grade, oracle list   (ADR-021 §3)
sharpness-ceilings.toml       # empty. `TBD` is not a ceiling (ADR-024 §3)
tuning-thresholds.toml        # empty. Every threshold is measured, never copied
rust-toolchain.toml           # pinned stable. No nightly, ever (ADR-006)
.github/workflows/gate0.yml
crates/
  resolvent-base/             # publish = true.  Empty except lib.rs today.
  resolvent-int/              # publish = true.  Empty.
  resolvent-modular/          # publish = true.  Empty.
  resolvent-poly/             # publish = true.  Empty.
  resolvent-algebra/          # publish = true.  Empty.
  resolvent-real/             # publish = true.  Empty.
  resolvent-expr/             # publish = true.  Empty.
  resolvent-calculus/         # publish = true.  Empty.  (ADR-005 am. 2026-08-08)
  resolvent-display/          # publish = true.  Empty.
  resolvent/                  # publish = true.  Facade. Empty.
  resolvent-oracles/          # publish = false. Property tests, differential oracles, rug.
  resolvent-bench/            # publish = false.
  resolvent-fuzz/             # publish = false.
  xtask/                      # publish = false. CI helper commands.
tests/license-gate/           # the three planted cases
```

The crate graph is normative (ADR-005; the alternative sketched in `plans/api-shape.md` §1.4
is superseded — there is no `resolvent-seam` and no `resolvent-lazy`).

### What to write

- **Every published crate's `lib.rs` starts with**
  `#![forbid(unsafe_code)] #![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic,
  clippy::indexing_slicing, clippy::arithmetic_side_effects)]`.
  The single exception is `resolvent-modular::simd`, which does not exist yet and is scoped
  by ADR-022 when it does.
- **Every published crate's `[dev-dependencies]` table is empty** — gate L6a. This is not
  stylistic: without it, the `rug` oracle lands in `resolvent-int/tests/` and a published MIT
  crate carries an LGPL-3.0+ dev-dependency that `cargo deny` cannot see (ADR-002
  §Decision 5). Property tests and differential oracles live in `resolvent-oracles` and test
  only the public surface, which the newtype wall makes sufficient by design. In-crate
  `#[cfg(test)]` unit tests stay, and may use only workspace crates.
- **`lanes.toml`**, seeded with the Wave-0 lanes:

  ```toml
  [lane.H1]
  crate = "xtask"
  gates = ["ADR-001", "ADR-005", "ADR-016", "ADR-021"]
  grade = "certificate"
  oracle = []

  [lane.H2a]
  crate = "resolvent-base"
  gates = ["ADR-012", "ADR-021"]
  grade = "certificate"
  oracle = []
  ```

  *Amended 2026-08-08.* The schema also carries the three `conformance` keys —
  `self_certifying`, `oracle_systems`, `divergence_ceiling` — and CI enforces them
  (ADR-021 §3 as amended, ADR-030 §1). **`oracle` holds lane ids; `oracle_systems` holds
  external system names.** Wave 0 has no conformance lane, so no entry uses them yet; write
  the three checks anyway, because the first conformance lane arrives in Wave 2 and a schema
  check added after the first user is a schema check that was never observed rejecting
  anything.

- **The three planted license cases**, each a tiny crate in `tests/license-gate/` that the
  gate must reject: one depending on `malachite` (LGPL-3.0-only, hiding behind a
  permissive-looking pure-Rust crate), one on `polynomen` (GPL-3.0-only with an innocuous
  name), and one synthetic Apache-2.0-only crate depending on `rug`.

### First tests

| Test | Asserts |
|---|---|
| `xtask::license_gate::rejects_planted_cases` | `cargo deny check licenses` **fails** on all three. *A gate that has never been observed to fail is not known to work* |
| `xtask::layering::l6a_no_dev_dependencies` | `cargo metadata` shows an empty `[dev-dependencies]` for every `publish = true` crate |
| `xtask::layering::l1_graph_matches` | `cargo tree --edges normal` equals the checked-in expected graph |
| `xtask::grep_gates::l4_no_geometry_vocabulary` | No `Point`, `Curve`, `Segment`, `Vertex`, `Face`, `tolerance`, `epsilon`, `snap` in any published crate |
| `xtask::grep_gates::l5_no_consumer_names` | No consumer repository name in source, `Cargo.toml`, feature name or doc example |
| `xtask::grep_gates::l13_no_ambient_state` | *Added 2026-08-08 (ADR-029 §2).* No `static mut`, no `thread_local!`, no lazily-initialized global cache in any published crate — **and a planted violation in a scratch crate is observed being rejected** |
| `xtask::grep_gates::l14_no_env_or_allocator_config` | No environment-variable read in a decision path, no process-global allocator assumption. Planted case observed rejected |
| `xtask::grep_gates::l15_no_process_identity` | No `std::process::id`, working directory, or filesystem dependency in any published crate. Planted case observed rejected |
| `xtask::lanes::conformance_schema` | `grade = "conformance"` ⟹ `self_certifying = false`, `oracle_systems` non-empty, `divergence_ceiling` present and not `TBD`; and no lane names a conformance lane in its `oracle` list. Asserted against a planted bad entry, since Wave 0 has no real conformance lane yet |
| `xtask::ratification::blocks_unratified_lane` | A scratch commit setting one gating ADR to `Proposed` removes that lane's crate from the workspace members list and skips its tests; setting it back to `Ratified` restores both |

The last one matters more than it looks. It is the freeze, and the freeze has never been
observed working either.

**Do not skip the observation step on any of these.** ADR-023 exists because the plan
previously applied "observe the gate failing" to exactly one gate out of fifty.

---

## Day 2 — the canonical serializer, then determinism

**Order matters here and the previous plan got it wrong.** The determinism harness, the
corpus format and every oracle adapter all serialize polynomials, and one implementation is
shared by all of them (ADR-012 §9). Three agents writing three serializers is a merge that
rewrites two of them, so **H2a is a blocking half-lane and H2b, H3, H4 depend on it.**

### H2a — `resolvent-base/src/canonical.rs`

The form, fixed before any oracle exists, because a SHA-256 certificate only works if
normalization is byte-identical:

- **Polynomials**: content removed; leading coefficient positive; terms **descending** in the
  ring's declared order; coefficients as decimal integers with an explicit `-` and no `+`;
  exponent vectors as full-length comma-separated non-negative integers.
- **Gröbner bases**: each element canonicalized, then the list sorted by leading monomial
  descending.
- **Algebraic numbers**: minimal polynomial plus a 0-based ascending root index.
- **`SCHEMA_VERSION`**, and a CI rule that a golden-file change without a version bump in the
  same commit fails.
- **Excluded, and this is load-bearing:** certificates, `Certainty`, `ProbableReason`,
  `Telemetry`. Only the mathematical value is serialized. If evidence were inside the bytes,
  ADR-012 §8's tuning-matrix value-equality gate could never be green, because the batch
  width `N` is a tuning knob and it changes `primes_used`.

Today there are no polynomials to serialize. Write the serializer against a stand-in
`Vec<i64>` "polynomial" and the golden-file machinery around it; the real types slot in on
Day 6–7 without touching the format.

### H2b — `xtask/src/determinism.rs`

Run any registered instance twice in-process, twice cross-process, at
`RAYON_NUM_THREADS ∈ {1, 2, 8}`, across feature combinations; compare canonical bytes. Any
difference is a failure.

**Tier it now, before the corpus has entries** (ADR-024 §1), because the alternative is
discovering in month three that Gate 0 takes 40 minutes and that the determinism matrix is
the cheapest thing to cut:

| Tier | Runs in | Budget |
|---|---|---|
| `fast` | Gate 0, every commit — 1 and 8 threads, in-process, certificates sampled at 10 % | **90 s, hard** |
| `full` | Gate 1, every PR — the complete matrix, certificates on | 25 min |
| `slow` | Gate 2, nightly — Mignotte / Swinnerton–Dyer / Hexapod, the overflow sweep | hours |

CI **prints the tier census** and **fails if `fast` exceeds its budget**, so promotion out of
`fast` is a deliberate, visible act.

### Start lane Z2 in the background today

It gates `resolvent-int`, it takes a day of wall-clock, and its result may add a lane to M1.
Clone `tczajka/bigint-benchmark-rs`, pin `dashu` 0.5.2, run locally. Then the ladder that
actually matters: **`gcd` and `gcd_ext` at 64 / 256 / 1k / 4k / 16k / 64k / 256k bits against
`rug`, plus a `rational_reconstruct` microbenchmark at ≈70 kbit** (Hexapod's modulus size:
1102 primes × 63 bits).

The two top rungs and the reconstruction benchmark are the whole point. The original ladder
stopped at 16 kbit — an order of magnitude below the regime that matters — so it could not
have detected the problem it exists to detect (ADR-002 §Context). Output is
`docs/research/bignum-ladder.toml` with `(dashu_ns, rug_ns, ratio)` medians of `k` runs with
IQR, **and the pre-committed verdict line**: if `gcd_ext` at 64 kbit is worse than 8× `rug`,
a half-GCD lane inside `resolvent-int` is scheduled in M1.

---

## Day 3 — the corpus, the generators, and the score

### `crates/resolvent-bench/src/corpus.rs`, `generators/`, `minimizer.rs`

Three corpus layers with different lifecycles:

| Layer | Contents | Gate |
|---|---|---|
| **Regression** | Every minimized counterexample ever found, plus hand-authored known-answer instances. Append-only | **100 % pass, always.** A gate, not a score |
| **Generator fleet** | Versioned, seeded generators | Feeds the score |
| **Benchmark** | Pinned, degree-checked instances of the standard families | Feeds the performance scoreboard, never the correctness gate |

**Every corpus entry carries a `provenance` field** (ADR-024 §2) —
`constructive-generator` / `oracle-consensus` (with systems and versions) / `hand-computed`
(with author and method) / `minimized-counterexample` (with class and origin commit). Without
it, a wrong expected answer that entered from a mis-triaged disagreement becomes a permanent
gate that a *correct* future implementation fails, and append-only means it can never leave.
`oracle-consensus` entries are re-derived nightly and drift is flagged.

**The score:**

> the number of CPU-seconds of adversarial generation resolvent survives with zero invariant
> violations, on a fixed machine, against a fixed versioned generator fleet, with a fixed
> seed schedule. Reported always as **`(fleet_version, seconds_survived)`**.

A pass-rate is gamed by weakening tests; a survival time is not, because weakening a
generator is a fleet-version bump and shows up in the reported pair.

**The minimizer**, delta-debugging in cheapest-structural-first order: drop terms → halve
coefficient bit-length → reduce degree → reduce variable count → reduce generator count →
shrink the query interval. It yields a **1-minimal** form — no single further step preserves
the disagreement — and says so; delta-debugging does not find a global minimum and the exit
gate must not claim it does.

**Declines are classified before they are scored** (ADR-024 §4): a decline is a *failure*
only if the instance is in the must-complete sub-corpus or the operation's budget came from a
proven bound. Otherwise it is a survived instance counted against a committed ceiling. The
blanket "any decline fails" rule is what pushes an agent to raise budgets until declines
become hangs.

### First test

`resolvent_bench::harness::falsifies_planted_stub` — the harness runs against a deliberately
buggy stub, falsifies it within `⟨B⟩` CPU-seconds at fleet version 1, minimizes the
counterexample to ≤ `⟨k⟩` terms, and reports `survived` once the stub is fixed. Commit `B`
and `k` in the test. Two runs at the same `(fleet_version, commit)` must be byte-identical.

---

## Day 4 — the first oracle, and it must be calibrated

### `crates/resolvent-oracles/src/sympy.rs`

`sympy` 1.14.0 is importable on this machine today via `python3`; nothing else (Singular,
PARI, Sage, Maxima, FLINT, Macaulay2) is installed. Tier 0 is therefore sympy, driven as a
**subprocess over a text protocol**. Nothing links — every capable oracle is copyleft
(ADR-016 §1).

*(Note for whoever writes the detector: `command -v gp` succeeds on this machine because `gp`
is a shell alias for `git push`. Detect PARI by invoking `gp -q -f` on a known expression and
checking the output, not by probing `$PATH`.)*

Two things ship together, and the second is the one that was missing:

1. **The adapter** — emits the canonical S-expression form from Day 2's serializer, parses a
   canonical form back. Oracle-specific parsing never appears in a test body.
2. **The calibration corpus** (ADR-016 §5) — a dozen instances per operation whose answers
   are **hand-computed and committed**, with the *oracle's* answer asserted against them:
   `Res(x²−2, x²−3)`, `gcd(x²−1, x³−1)`, `factor(x⁴+1)` over ℚ and over `GF(3)`,
   `isolate_roots` of a Chebyshev polynomial, `subresultants` of a pair with a known degree
   sequence.

   A round-trip is resolvent → S-expression → resolvent: it tests resolvent's own encoder and
   decoder and **never establishes that sympy read the same polynomial**. An adapter that
   emits variables in the wrong order round-trips perfectly and then produces confident
   agreement about the wrong object. The calibration corpus is the only test that can say so,
   and it is also what catches an oracle version bump changing a convention.

### The triage classifier — `resolvent-oracles/src/triage.rs`

Re-run resolvent's own certificate on the disagreeing instance. Self-certificate **also**
fails → **Class A: resolvent bug, certain**, straight to the regression corpus.
Self-certificate passes → **Class B: normalization, convention, or oracle limitation**,
minimize and re-classify.

**A missing oracle is a counted, loud SKIP — never a pass.** The harness prints a skip census
and a job declaring Tier 0 fails if sympy is absent.

---

## Day 5 — `resolvent-base`, and the afternoon that matters most

**Lane Z0 is the single most-depended-on deliverable in the project, and it appeared in no
lane, no wave and no milestone in the previous plan.** Everything above Layer 0 inherits its
signature, and ADR-006 marks that signature a one-way door.

### `crates/resolvent-base/src/ring.rs`

Write the trait tower from ADR-006 §Decision **verbatim**, and then — this is the whole point
of the day — **make it compile with two real implementations.**

```rust
pub trait Ring: Clone + PartialEq + Send + Sync + 'static {
    const LANES: usize;
    type Scalar: Ring;
    type Ctx: Clone + PartialEq + Send + Sync + 'static;
    fn zero(ctx: &Self::Ctx) -> Self;
    fn one(ctx: &Self::Ctx) -> Self;
    fn ctx(&self) -> &Self::Ctx;
    fn add(&self, r: &Self) -> Self;  fn sub(&self, r: &Self) -> Self;
    fn mul(&self, r: &Self) -> Self;  fn neg(&self) -> Self;
    fn is_zero(&self) -> bool;
    fn add_assign(&mut self, r: &Self) { *self = self.add(r); }
    fn sub_assign(&mut self, r: &Self) { *self = self.sub(r); }
    fn mul_assign(&mut self, r: &Self) { *self = self.mul(r); }
}
pub trait Reducible: Ring {
    type Image: CommutativeRing;                       // NOT Field
    fn reduce(&self, m: &Modulus) -> Result<Self::Image, BadPrime>;
}
pub trait Liftable: Reducible {                        // NOT Ring
    fn crt_lift(images: &[Self::Image], moduli: &[Modulus]) -> Result<Self>;
}
pub trait BatchField: Ring {
    fn inv_batch(&self) -> Result<Self, LaneMask>;
}
// There is no BulkOps. Bulk kernels are free functions over concrete types.
```

Then write, in the same crate, a throwaway `struct Fp { v: u32, params: FpParams }` and a
`struct Z(i128)` and `impl Ring` for both. If both compile, the tower is real. If either does
not, **stop and fix the ADR before anyone starts Wave 1.**

Why this is worth a whole day's attention: the previous version of this block had
`fn zero() -> Self` with no receiver and no context, which **five of the seven rings in its
own instantiation set cannot implement** — `Fp` carries `p` by value, so `Fp::zero()` must
answer "zero of which prime field?" from nothing — and `Liftable: Ring` with `Self::Image` in
its signature, which does not compile at all. A one-way-door signature that has never been
through a compiler is not a settled decision, and both defects were `cargo check`-visible in
seconds.

### Also in `resolvent-base` today

- `Sign { Negative, Zero, Positive }` and `Verdict<T> { Certain(T), Unknown }`, with the rule
  that decides which: **a function returns a bare `Sign` iff it is total and exact; a function
  that can be indeterminate returns `Verdict<Sign>` and never `Sign`.** `Verdict` is produced
  only by enclosure and filter APIs, never by an algebraic-decision API (ADR-011 §5).
- `Error`, `Unsupported` (a structured enum, **never a string**), `Budget`, `Decline`, with
  `is_decline()` distinguishing declines from faults.
- `Certified<T>`, `Certainty`, `ProofKind`, `Certificate<C: Claim>` — private fields,
  `pub(crate)` mint, public read accessors, `certifies(&C) -> bool`, `verify(Budget)`
  (ADR-010 §2). Unforgeable means no public constructor; checkable means public read.
- H2a's canonical serializer moves here if it was written elsewhere on Day 2.

### First tests — in `resolvent-oracles`, per L6a

| Test | Asserts |
|---|---|
| `ring_laws::associativity/commutativity/distributivity/identity/inverse` | For every instantiation, over generated elements |
| `ring_laws::ctx_is_free_for_context_free_rings` | `<Z as Ring>::Ctx == ()` and `Z::zero(&())` compiles and is correct |
| `verdict::sign_and_verdict_are_distinct_types` | A compile-fail test: an algebraic-decision signature returning `Verdict<Sign>` does not compile |
| `certificate::no_public_mint` | A compile-fail test: constructing a `Certificate` outside the crate does not compile |
| `canonical::golden_files_stable` | Byte-identical across processes, thread counts and feature combinations |

Commit `cargo public-api`'s snapshot for `resolvent-base`. It is the crate a consumer depends
on to implement a ring without pulling a bignum, so its surface is a promise.

---

## Day 6 — `resolvent-int` and `Fp`

### `crates/resolvent-int/`

`Integer` and `Rational` newtypes over `dashu`, with **`dashu` in no public signature and no
re-export**, and appearing in exactly one `Cargo.toml` (gate L2). The conversion surface is
over primitives and slices, never over a third-party bignum: `From<{i8..i128, u8..u128}>`,
`TryFrom<&Integer>`, `FromStr`, `from_le_limbs64`/`to_le_limbs64`,
`from_signed_bytes_be`/`to_signed_bytes_be`.

`Rational::try_from_f64` is **exact dyadic and `Err` on NaN/±∞**. There is no "nice rational"
heuristic and there never will be: silently turning a baked `sin(30°)` into `1/2` analyses a
different system than the one the caller authored.

`num_bits()` / `den_bits()` and an explicit `round_to_f64_grid()` — resolvent exposes size
and rounding and applies neither; the policy stays with the caller.

### `crates/resolvent-modular/`

`Fp` for word primes with Barrett/Shoup `mulmod`, `Copy`, carrying `p` and its precomputed
reciprocal **by value**. Arithmetic is `#[inline]` inherent methods, never a method on a ring
object.

### First tests — in `resolvent-oracles`

| Test | Asserts |
|---|---|
| `int::differential_vs_rug` | `rug` as the in-process dev-oracle (precedent: `dashu`'s own `fuzz/` found an `nth_root` bug this way), against the **public** surface only |
| `int::word_boundary_generators` | Carry bugs cluster at word boundaries; the generator targets them |
| `int::gcd_bezout_certificate` | `g\|a`, `g\|b`, **and** `u·a + v·b == g`. **Not** `gcd(a/g, b/g) == 1`, which is circular: `fn gcd(_,_) -> ONE` passes it, by calling itself (ADR-023 §2) |
| `fp::exhaustive_small_p` | For every prime `p < 2^10`, **every pair `(a,b)` with `a,b < p`** against an `i128` reference. **The loop bound is `p·p`, not `p`** — the off-by-one that silently tests `p` pairs is in the mutant set |
| `fp::inverse_of_every_unit` | `a·a⁻¹ == 1` for every unit |

### Mutant sets start today (ADR-023 §1)

Every certificate ships at least one deliberately wrong implementation under `#[cfg(test)]`
in the same module, plus a test asserting the certificate **rejects** it. A mutant rejected by
the type system does not count — it must compile and produce a plausible wrong value. Start
with the two cheapest and most instructive:

- `gcd → 1` (the **identity** mutant) — must be rejected by the Bézout witness.
- `Fp::mul` with the reduction step removed (the **silent wrap** mutant) — must be rejected by
  the exhaustive small-`p` test.

This is the difference between a self-certifying library and a library that says it is one,
and it is cheapest at the moment the operation is written, because the mutant is the wrong
version the author already had in their head.

---

## Day 7 — `UPoly` and the naive reference: close the loop

### `crates/resolvent-poly/src/upoly.rs`

`UPoly<C>` — dense, low-to-high, trailing zeros trimmed, carrying one `C::Ctx`. **No monomial
type, no order, no `Ring` context.** This standalone-ness is the decision that makes the
two-trunk fan-out possible, and inverting it (defining `UPoly` as a 1-variable `MPoly`) is the
specific mistake ADR-007 exists to prevent.

Today: `add`, `sub`, `mul`, `div_rem`, `eval_horner`, `derivative`, `content`,
`primitive_part`, `canonical_associate`, `reverse` (`xⁿ·p(1/x)` — **public**, because it is
private in the prior art and consumers need it to move the point at infinity to zero).

**And, in the same crate, the naive `O(n²)` reference implementation of each.** It is part of
the deliverable, not a follow-up: it is the oracle, and it ships with the code.

### The verdict that makes the week worth it

```
resolvent_oracles::poly::fast_agrees_with_naive
resolvent_oracles::poly::product_divides_back          //  (a·b)/b == a
resolvent_oracles::poly::degree_is_additive            //  deg(a·b) == deg a + deg b
resolvent_oracles::poly::evaluation_is_a_homomorphism  //  eval(a·b, x) == eval(a,x)·eval(b,x)
```

The last one is a Schwartz–Zippel argument with failure probability `deg/p` — **but only if
the point varies.** At a single committed seed it is a golden test at one point, and an error
that happens to vanish at `prime(0)` is certified forever. So: **randomized certificates are
evaluated across the fleet seed schedule, never at the default seed alone**, and the number of
distinct seeds used is reported alongside the score (ADR-023 §3). Inside the library the
default seed stays fixed and deterministic; the *harness* is what varies it.

**This is the loop closing.** Two independently written implementations of the same
mathematics, graded against each other on generated adversarial input, under a step budget,
with a minimizer that reduces any disagreement, and a determinism check that the whole thing
is reproducible. Report the first score: `(fleet_version = 1, seconds_survived = N)`.

---

## Week 2 — what comes next, briefly

The full plan is `ROADMAP.md` §2. The immediate continuation:

- **U4 (Sturm) then U5 (Descartes/VCA).** Sturm gives the *exact* count of distinct real
  roots, so it grades every Descartes output automatically, and
  `count_sturm(f,a,b) == len(isolate_descartes(f,a,b))` on the whole fleet is the strongest
  certificate in Layer 2. Build Sturm knowing it will never be the production isolator. Commit
  its **permitted-import set** in `lanes.toml` on the day it lands, so it cannot later be
  "fixed" by routing it through the Ducos PRS it is supposed to grade — at which point the
  strongest certificate in Layer 2 silently becomes a check of a component against itself and
  no test changes colour (ADR-023 §6).
- **Z3 in parallel** — `Fp` / `Zn` / `GF(p^k)`. The best agent lane in the project, and it
  decomposes cleanly across two or three agents.
- **Z6** — CRT and rational reconstruction, with the **prime-registry sieve cross-check**.
  That check is not optional: a composite in the registry breaks `Fp` silently while CRT and
  rational reconstruction keep certifying, because both certify statements about `M` rather
  than about `M`'s factorization. It is the modular architecture's root of trust and the one
  assumption with no downstream detector.
- **X1 and X3** (the L4 `Store`, node set, `walk_topological`, canonical bytes) can start any
  time after Day 5. They are blocked by nothing and block nothing. X2 and X4 are *not* free —
  X2's exit gate is agreement with `UPoly::derivative`, and X4 returns an `MPoly`.
- **E-MUT** at the tail of U1/U2 — the four `AlgebraicReal` mutability prototypes, ≈300 lines
  over `UPoly<Integer>` with roots built as `Π(x−rᵢ)`. It needs `cmp`, `refine` and polynomial
  sign evaluation, **not** the production isolator, and it gates all of M3.

---

## What NOT to do in week 1

- **Do not start the multivariate trunk.** It gates on E-MONO (ADR-008), which needs a
  recorded S-pair trace, which needs a throwaway Buchberger. None of that is week-1 work and
  the term type is a one-way door.
- **Do not optimize anything.** Every score lane's CI job is defined not to exist until its
  oracle lane is green and frozen. There is nothing frozen yet.
- **Do not add a tolerance, epsilon, or "close enough" parameter anywhere, at any layer, under
  any name.** Grep gate L4 catches it; treat it as a defect in review if it survives. The
  first consumer refuses tolerance by construction and would be unable to use one.
- **Do not expose a float interval type** (ADR-015). Bounds are rationals; the float
  information is an outward-correct `(f64, f64)` pair.
- **Do not name a consumer repository anywhere** — not in a feature flag, not in a doc example,
  not in a comment (gate L5).
- **Do not copy a tuning threshold from any reference implementation.** Every threshold is
  re-derived by measurement on resolvent's own corpus with the measurement committed. This is
  simultaneously a licensing rule and a correctness rule, which is why it holds under pressure.
- **Do not read Symbolica, or any repository with no declared license.** Tier C is a
  blocklist, not a preference, and it is *stricter* than the GPL tiers — "source-available"
  sounds safer than GPL and the inference runs backwards (ADR-001).
- **Do not write a `Derivation:` line citing only a paper.** It cites a paper **and** a path
  into `docs/research/`, CI resolves the path, and the note needs a `Sources:` block with a
  tier tag per reference. A citation to a paper nobody opened is exactly what an agent working
  from a source tree would write.

---

## Links

| For | Read |
|---|---|
| How these documents relate, and what "ratified" means | [ADR-021](docs/decisions/ADR-021-document-precedence-and-ratification.md) |
| The full milestone and lane plan | [ROADMAP.md](ROADMAP.md) |
| The public surface, capability by capability | [API.md](API.md) |
| Every decision, indexed | [docs/decisions/README.md](docs/decisions/README.md) |
| The verdict functions in detail | [plans/verification.md](plans/verification.md) — working notes, non-normative where an ADR speaks |
| Why the trait tower looks like that | [ADR-006](docs/decisions/ADR-006-generics-boundary.md) |
| What "done" means for a lane | [ADR-023](docs/decisions/ADR-023-certificates-are-adversarially-validated.md), [ADR-024](docs/decisions/ADR-024-corpus-tiering-and-gate-budgets.md) |
