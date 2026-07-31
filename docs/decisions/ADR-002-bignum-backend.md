# ADR-002 — Layer-0 bignum: `dashu`, behind a resolvent-owned newtype wall

**Status:** Ratified 2026-07-31
**Reversibility:** costly
**Amended:** 2026-07-31 — the "modular methods make megabit integers irrelevant" claim is
corrected; the gating measurement ladder is extended; half-GCD is promoted from a reversal
trigger to a planned M1 contingency; published crates are declared dev-dependency-free
(critique-engineering §8, §18).
**Gates lanes:** Z1, Z2, Z6.
**Evidence:** `docs/research/prior-art-and-licensing.md` §1.1–§1.4, §7;
`docs/research/critique-engineering.md` §8, §18.

---

## Context

Everything above Layer 0 inherits the ℤ and ℚ implementation. The license constraint
(ADR-001) eliminates the fast options, and the remaining choice is narrower than it looks.

Verified against the crates.io API on 2026-07-31 (the GitHub sidebar license badge is
*not* authoritative and disagrees with `Cargo.toml` for at least five relevant crates):

| Crate | License (verified) | Verdict |
|---|---|---|
| `malachite` 0.10.0 | **LGPL-3.0-only** — README: "Parts of Malachite are derived from GMP, FLINT, and MPFR" | Forbidden. The fastest pure-Rust bignum, unavailable. |
| `rug` / `gmp-mpfr-sys` | **LGPL-3.0+** | Forbidden as a runtime dep. Dev-oracle only. |
| `num-bigint` 0.5.1 | MIT OR Apache-2.0 | Permitted. **Rejected on capability.** |
| `ibig` 0.3.6 | MIT OR Apache-2.0 | Permitted. Superseded — `dashu-int` is its fork. |
| `crypto-bigint` 0.7.5 | Apache-2.0 OR MIT | Permitted. Wrong shape (ADR-003). |
| `ramp`, `fixed-bigint` | Apache-2.0 **only** | Barred by ADR-001's MIT-arm rule; also dead / fixed-width. |
| **`dashu` 0.5.2** | **MIT OR Apache-2.0** | **Adopt.** |

`num-bigint` deserves a real reason rather than a vibe, since it is the ecosystem default.
Reading `rust-num/num-bigint/src/biguint/multiplication.rs:93-285`, its multiplication
ladder is schoolbook → half-Karatsuba → Karatsuba → **Toom-3, and stop**. There is no
FFT, NTT, or Schönhage–Strassen path. Toom-3 is Θ(n^1.465), and the measured consequence
is 482 s where GMP takes 2.8 s on the same task. It is rejected on capability, not license.

`dashu-int` has the full CAS shopping list: NTT multiplication over Proth primes combined
by Garner CRT (added in 0.4.3), Toom-3, Karatsuba, Lehmer GCD **with `gcd_ext`**,
Burnikel–Ziegler division, a Montgomery module for odd moduli, a runtime-modulus modular
module, and env-tunable thresholds under a `tuning` feature. It is actively maintained
(0.5.2 published 2026-07-31).

**The honest performance picture**, with its caveat stated first: every published
large-operand number used `dashu` **0.4.2 — one release before NTT landed**, so the widely
cited "7× behind GMP" figure is stale and is *not evidence* about 0.5.2. What is solid:

- Below ~1 kbit, `dashu` **beats** GMP, because GMP allocates and has no inline small-value
  path. One-word `ubig_add`: ~3.3 ns vs `rug`'s ~21.2 ns.
- Around two words: parity.
- Crossover near ~1 kbit; at 10 kbit `rug` wins `ubig_mul` ~4.5 µs vs ~12.8 µs (≈2.8×).
- Above ~1 Mbit: unknown post-NTT.

**Correction, 2026-07-31.** This ADR previously read the last bullet as *irrelevant if the
architecture is honoured*, on the grounds that "megabit integers appear in a CAS exactly
when someone computes over ℤ or ℚ directly instead of mod several primes and
reconstructing". **That is false, and ADR-010 refutes it three pages later.** Modular
methods do not eliminate large integers. They **concentrate** them, into two places, and
both are on the *default certified path*:

- **The CRT modulus `M = Π pᵢ`.** ADR-010 §Context: Cyclic-10 needs >2000 primes of 29 bits
  (≈58 000 bits). `plans/verification.md` §6.2: **Hexapod needs 1102 primes for a
  computation whose single modular run takes 0.00 s** — ≈70 kbit at 63-bit primes — and the
  plan deliberately puts Hexapod in the corpus from the first modular milestone precisely
  because it is reconstruction-bound.
- **Rational reconstruction**, which is `gcd_ext` on integers of size `M`. That is exactly
  the operation where the one identified structural pure-Rust deficit lives: `dashu` has
  Lehmer (quadratic worst case); GMP has a subquadratic half-GCD. At ~1100 limbs that is
  ~10⁶ word operations per reconstruction against ~10⁴–10⁵.

So the honest statement is: **modular methods keep the *bulk* of arithmetic sub-kbit and
concentrate a *small number* of very large operations in CRT accumulation and rational
reconstruction.** The bulk regime — coefficient ingress, per-prime work, Landau–Mignotte
and Hadamard bounds, multiply-back verification — is where `dashu` is competitive or
better. The concentrated regime is where it is weakest, and it is not optional.

**The bignum choice and the modular-methods choice are still the same decision seen from
two sides.** What changes is that the decision must be priced against reconstruction, not
against the average operand size.

Three mitigations are on the record so an agent does not rediscover them under pressure:

1. **Incremental (Garner) CRT.** Accumulate residue-by-residue so no single step operates
   on a full-width modulus; the accumulation cost is then linear in prime count with
   small-operand steps rather than one megabit multiply.
2. **Early-termination rational reconstruction with a doubling modulus.** Attempt
   reconstruction at `2^k` primes for increasing `k` and verify; the common case never runs
   `gcd_ext` at full modulus width.
3. **A half-GCD inside `resolvent-int`** (§What would reverse this, promoted below to a
   planned contingency).

---

## Decision

**Depend on `dashu` (MIT OR Apache-2.0) for ℤ and ℚ, and wrap it in a resolvent-owned
newtype wall in `resolvent-int`.**

1. `resolvent-int` exposes `Integer`, `Natural`, and `Rational` newtypes. **`dashu` types
   appear in no public signature and in no trait bound outside `resolvent-int`'s private
   modules.**
2. **`dashu` is not re-exported.** A public re-export would make `dashu`'s semver a hard
   part of resolvent's semver.
3. **`dashu` appears in exactly one `Cargo.toml`** — `resolvent-int`'s. CI asserts
   `cargo tree -i dashu` lists exactly one direct dependent (gate L2 in `plans/architecture.md`).
4. **The conversion surface is over primitives and slices**, not over a third-party bignum:
   `From<{i8..i128, u8..u128, isize, usize}>`, `TryFrom<&Integer>` back to those,
   `FromStr`, `from_le_limbs64(sign, &[u64])` / `to_le_limbs64()`,
   `from_signed_bytes_be` / `to_signed_bytes_be`. A consumer whose own bignum is also
   `dashu` converts through the limb slices without coupling versions.
5. **`rug` is a dev-dependency oracle only**, in `publish = false` crates. This is precedent
   from the crate being adopted: `dashu`'s own 0.5.0 changelog credits its
   `fuzz/`-based `rug::Integer` oracle with finding an `nth_root` bug.

   **Amended 2026-07-31, and this is a rule not a preference: every `publish = true` crate
   has an empty `[dev-dependencies]` table.** Gate L6 forbids a published crate depending
   on an unpublished one "including dev-dependencies", but `cargo deny` is scoped to the
   published graph minus dev-only features and will not catch it, while `cargo publish`
   records dev-dependencies in the manifest and a downstream `cargo test` then builds GMP.
   The ℤ/ℚ differential oracle therefore lives in `resolvent-oracles` and tests only
   `resolvent-int`'s **public** surface — which is sufficient by design, because the
   newtype wall means the public surface is the whole point. CI asserts the empty table
   with one `cargo metadata` query (ADR-005 gate L6a).
6. **No speculative GMP backend.** If measurement later justifies one, it is an optional,
   non-default, never-in-CI-release `backend-gmp` feature that documents loudly that
   enabling it subjects the build to LGPL-3.0+ (ADR-001 §What would reverse this). Do not
   build it now.
7. **A half-GCD inside the wall is a *planned* M1 contingency, not a hypothetical.** It has
   a numeric trigger, fixed here so the decision is not re-argued under schedule pressure:
   **if lane Z2 measures `dashu`'s `gcd_ext` at 64 kbit at more than 8× `rug`'s, the
   half-GCD lane is scheduled in M1** as a self-contained, `rug`-certifiable lane inside
   `resolvent-int` with no signature change anywhere above Layer 0. Below 8×, the
   mitigations in §Context (Garner CRT, early-termination reconstruction) are sufficient
   and the lane is not scheduled. The trigger is committed *before* the measurement runs,
   per ADR-012 §8.

---

## Consequences

- **The wall is the insurance.** If `dashu` goes unmaintained, or if the Lehmer-vs-half-GCD
  gap proves fatal at the measured workload, swapping the backend is a change inside one
  crate with no consumer-visible signature change. Without the wall it is a breaking change
  across the whole ecosystem.
- **The wall costs a thin layer of delegating methods** and, for a few operations, a
  `clone` that a direct dependency would avoid. Mitigated by `#[inline]` and by
  `#[repr(transparent)]` where the newtype is a single field.
- **Consumers never name `dashu`**, which is what makes the deferred integration decision
  (ADR-018) cheap: an adapter deals with `resolvent::Integer`, not with a version-pinned
  third-party type leaking through a public signature.
- **A measurement is now blocking.** Re-running `tczajka/bigint-benchmark-rs` with 0.5.2
  pinned must happen **before `resolvent-int` is written**, because a negative result
  strengthens the case for designing the `backend-gmp` feature seam now (cheap) rather than
  later (expensive). Half a day, self-verdicting.
- **A second measurement sets ADR-004's aggressiveness *and* decides item 7.** The ladder is
  `gcd` / `gcd_ext` at **64 / 256 / 1k / 4k / 16k / 64k / 256k bits** against the `rug`
  dev-oracle, **plus a `rational_reconstruct` microbenchmark at Hexapod's modulus size
  (1102 × 63-bit primes ≈ 70 kbit)**.

  *Amended 2026-07-31.* The original ladder stopped at 16 kbit — an order of magnitude
  below the regime §Context shows actually matters — so the gating measurement could not
  have detected the problem it exists to detect. The two added rungs and the
  reconstruction microbenchmark are the whole point of the lane. Output is a
  machine-readable `docs/research/bignum-ladder.toml` with `(dashu_ns, rug_ns, ratio)`
  medians of `k` runs with IQR on the pinned machine, plus the item-7 verdict line.

---

## Alternatives considered and why rejected

**`malachite`.** The fastest pure-Rust option, and the one I most wanted. **LGPL-3.0-only**,
derived from GMP/FLINT/MPFR source, no dual arm, no permissive subset. `malachite-bigint`
(the `num-bigint`-API shim in the same repo) inherits it. Rejected by ADR-001 without
further analysis; this is constraint #2 doing its job and it is the single most consequential
finding in the whole research phase.

**`rug`.** LGPL-3.0+ bindings to GMP/MPFR. Same rejection. Retained as a dev-oracle.

**`num-bigint`.** Permitted by license. Rejected on capability: no FFT/NTT path at all
(verified at `src/biguint/multiplication.rs:93-285`), ~170× behind GMP at megabit sizes.
Adopting it would mean the architecture's escape hatch — "if modular methods fail us, fall
back to direct bignum" — does not exist.

**`ibig`.** Permitted, and `dashu-int` is its fork. Its last release is 0.3.6 from
2022-09-17 and it predates the NTT work entirely. Strictly dominated.

**`crypto-bigint`.** Permitted and actively maintained, and it *does* support runtime
moduli. Rejected because its reason for existing is constant-time execution, which is a
pure tax here — nothing in resolvent processes secrets — and it is sized for 256-bit+
operands, not the ≤63-bit word primes where a CAS lives. Kept as a possible differential
oracle for hand-rolled Montgomery code (ADR-003).

**Write our own bignum.** Rejected. It is a multi-year project on its own, it is exactly
the kind of number-to-optimize lane with no certificate that constraint #3 says converges
slowest, and it competes for attention with the algebra that is the actual product. The
newtype wall means we can revisit this incrementally later — e.g. hand-write a
subquadratic half-GCD *inside* `resolvent-int* while delegating everything else — without
a rewrite.

**Depend on `dashu` directly without the wall.** Rejected. It makes `dashu`'s semver part
of resolvent's semver, leaks a version-pinned type into every consumer's dependency graph,
and forecloses the incremental-replacement path above. The wall is cheap insurance against
the one thing that could still go wrong.

---

## What would reverse this

- **`dashu` becomes unmaintained** (no release for ~18 months plus an unfixed correctness
  bug). Response: fork inside the wall, or replace the backend. The wall makes this a
  one-crate change.
- **The post-NTT re-measurement shows the large-operand gap is still catastrophic** *and*
  the modular architecture fails to keep work in the sub-kbit regime for a real consumer
  workload. Response: design the `backend-gmp` feature seam, keep it non-default, and keep
  the permissive path as the shipped default.
- **The GCD gap proves fatal at the measured workload** — i.e. ADR-004's ℤ-primitive
  discipline is honoured and Lehmer is *still* the bottleneck. Response: implement a
  half-GCD inside `resolvent-int`. This is a self-contained, self-certifiable lane
  (differential against `rug`), which is a good lane shape, and the wall is what makes it
  possible without touching anything above Layer 0.
