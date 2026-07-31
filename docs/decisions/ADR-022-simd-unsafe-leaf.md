# ADR-022 — One audited `unsafe` leaf: `resolvent-modular::simd`

**Status:** Ratified 2026-07-31
**Reversibility:** cheap in one direction (delete the module, lower the published gate),
costly in the other (a `forbid(unsafe_code)` policy relaxed after publication is a visible
promise broken)
**Gates lanes:** G3, Z4, Z8, G6.
**Evidence:** `docs/research/critique-engineering.md` §13;
`docs/research/algorithms-and-representation.md` §1.6 (linear algebra is 73–91% of an F4
run); `plans/verification.md` §6.2 (the published performance ladder).

---

## Context

Two commitments were made independently and cannot both hold.

**The policy.** `#![forbid(unsafe_code)]` on every published crate, wanted by two consumers
(11 of 28 crates in one already forbid it; both `lazy-exact` and one certification crate do).
ADR-006 additionally pins stable Rust, which forecloses `portable_simd`, so the only route
to explicit vectorization is `core::arch` intrinsics — which are `unsafe`.

**The published target.** The performance ladder names a *Competitive* rung at
"Cyclic-9 < 600 s, Katsura-13 < 900 s, Eco-14 < 600 s (≈ 2× state of the art)".

The plan noted that msolve reports AVX2 halving its linear-algebra time, and that SIMD "sits
behind the `unsafe` confinement rule and needs its own audit". It never did the arithmetic.
Linear algebra is **73–91%** of an F4 run. Forgoing a 2× on 73–91% of the run is a
**~1.6–1.8× overall factor**. Against a target set at 2× SOTA, the policy and the published
number are within noise of each other. Publishing a target the policy forbids reaching is
the failure; either half is defensible alone.

There is a second, quieter cost that decides the shape of the fix. Auto-vectorization of a
sparse GF(p) `axpy` with Barrett reduction — a widening multiply plus a conditional subtract
— is vectorized inconsistently by LLVM across versions. A performance series whose level
shifts on a compiler upgrade **with no code change** will trip the change-point detector,
and the fastest way to teach everyone to ignore an alerting system is to have it fire on
nothing. Relying on auto-vectorization is therefore not the conservative option; it is the
unpredictable one.

---

## Decision

**Name exactly one `unsafe`-permitted leaf: the module `resolvent-modular::simd`. Keep
`#![forbid(unsafe_code)]` everywhere else, including every other module of
`resolvent-modular`.**

The exception carries five conditions, all mechanical:

1. **Scope.** `#![allow(unsafe_code)]` appears on that module and nowhere else. The
   workspace `unsafe` inventory in Gate 0 is a checked-in allowlist of exactly one path; a
   new entry is a diff that fails CI without an explicit review trailer.
2. **`SAFETY:` on every block**, naming the invariant discharged (alignment, length,
   target-feature availability).
3. **Runtime feature detection.** No `target_feature` assumed at compile time; dispatch is
   `is_x86_feature_detected!` once per *phase*, never per element, and the chosen path is
   recorded in the run's `Tuning` so it is visible in the benchmark record.
4. **A scalar fallback that CI asserts is bit-identical.** This is the condition that makes
   the exception genuinely auditable rather than nominally so, and it is *available* here
   for a reason worth stating: **these are exact integer operations.** A SIMD path is a pure
   speed change and cannot alter a value. So the assertion is not aspirational — it is a
   componentwise equality test over random vectors including tails and misaligned lengths,
   which is the same verdict function lane Z4 already has.
5. **Scope of contents: bulk GF(p) vector kernels only** — `axpy`, `scale`, `normalize`,
   `dot`, and the sparse-row variants F4 reduction calls. Nothing algorithmic. No control
   flow that a certificate depends on. If a proposed addition is not "the same arithmetic,
   four lanes at a time", it does not belong in this module.

**The determinism story is unaffected**, and this is the reason the exception is admissible
at all. ADR-012 requires bit-identical output across thread counts and build profiles; a
kernel that computes the same integers faster changes timing and nothing else. Condition 4
is what turns that from an argument into a test.

**A compiler bump is a re-baseline event**, recorded in the benchmark metadata alongside the
resolvent commit and the fleet version, and treated exactly as a fleet-version bump is: the
series legitimately shifts and is labelled as having shifted.

**If this ADR is ever reversed**, the published *Competitive* rung moves from ≈2× SOTA to
**≈3–4× SOTA in the same commit**, with AVX2 named as the reason. The two are one decision
and must never drift apart again.

---

## Consequences

- **The published target becomes reachable in principle.** It was not, and the plan did not
  know it.
- **One module in the workspace requires a different review standard.** That is the whole
  cost, and it is bounded: bulk vector kernels over `u32`/`u64` slices are the most
  reviewable `unsafe` in existence — no lifetimes, no aliasing beyond `&mut [T]` vs `&[T]`,
  no ownership transfer, no FFI.
- **Two consumers' `forbid(unsafe_code)` expectation is met in substance and not in
  letter.** Stated plainly in the crate docs rather than buried: `resolvent-modular` contains
  one `unsafe` module, its contents are exact-integer SIMD kernels, and a bit-identical
  scalar fallback is asserted in CI. A consumer that cannot accept any `unsafe` at all
  disables the `simd` feature and gets the scalar path, which is the default until lane G3
  measures a win.
- **`simd` is a feature, default-off until measured.** It cannot change a value, so it
  cannot change a corpus outcome; it is default-off only so that the first published release
  ships the path that has the most test-hours on it.
- **Miri cannot run the SIMD module.** Gate 2's Miri job covers the monomial arena and the
  packing crate and explicitly excludes this module; the bit-identity assertion is what
  covers it instead.

---

## Alternatives considered and why rejected

**Keep `forbid(unsafe_code)` everywhere and lower the published gate to ≈3–4× SOTA.** Fully
defensible, and the honest fallback if the exception is ever withdrawn. Rejected as the
default because the gap is not decoration: 1.6–1.8× on the library's flagship benchmark is
the difference between "competitive with a real F4" and "in the OpenF4 band", and the
exception's cost is one small, exhaustively testable module.

**Rely on auto-vectorization and write the scalar loop carefully.** Rejected. It is not the
conservative choice: the sequence LLVM must recognize (widening multiply, conditional
subtract) vectorizes inconsistently across compiler versions, so the performance series
level-shifts with no code change. That is worse than an explicit intrinsic, because it is
both slower *and* unpredictable, and it poisons the change-point detector.

**Wait for `portable_simd` to stabilize.** Rejected as a plan. It may stabilize; a
foundation library cannot schedule against it, and ADR-006's stable-Rust pin exists so that
consumers are never asked to adopt a toolchain. If it stabilizes, this module's *contents*
are replaced and the allowlist entry is deleted — which is the cheapest possible reversal
and is a reason to keep the module small.

**Vendor a third-party SIMD crate.** Rejected on ADR-001 grounds first (most are Apache-only
or worse) and on scope second: the kernels here are five functions over `u32` slices with a
Barrett constant. A dependency would not be earning its keep, and it would move the `unsafe`
out of review rather than out of existence.

**Put the SIMD kernels in a `publish = false` crate.** Rejected — they are on the hot path
of a published crate, so they cannot be. The two-category rule (ADR-016) is about *oracles
and benchmarks*, not about hiding production code from its own license gate.

---

## What would reverse this

- **Lane G3 measuring the AVX2 win at materially less than the ~1.6× the arithmetic
  predicts** on resolvent's own corpus — e.g. because memory bandwidth, not arithmetic,
  bounds sparse row reduction at the sizes that matter. Response: delete the module, restore
  `forbid(unsafe_code)` workspace-wide, and lower the published *Competitive* rung to 3–4×
  SOTA in the same commit.
- **A consumer with a hard no-`unsafe`-anywhere policy becoming a priority consumer.**
  Response: the default-off feature already serves it; if a *dependency-graph-level* audit
  is required, the kernels move behind a second crate that the facade does not pull by
  default. Additive.
- **`portable_simd` stabilizing.** Response: replace the module's contents, delete the
  allowlist entry, restore `forbid(unsafe_code)`. Keep the bit-identity test.
