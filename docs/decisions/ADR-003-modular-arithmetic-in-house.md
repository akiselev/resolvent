# ADR-003 — GF(p), ℤ/n and GF(p^k) are written in-house, not adopted

**Status:** Ratified 2026-07-31
**Reversibility:** cheap
**Gates lanes:** Z3, Z4, Z5, Z6, Z8.
**See also:** ADR-022 (the one audited `unsafe` SIMD leaf lives in `resolvent-modular::simd`
and is scoped there, not here).
**Evidence:** `docs/research/prior-art-and-licensing.md` §2; `docs/research/algorithms-and-representation.md` §3.5.

---

## Context

Under ADR-010 (modular methods everywhere), word-size GF(p) arithmetic is the single
hottest code in the library: F4's row reduction is 73–91% of an F4 run, and every gcd,
resultant, factorization and Gröbner computation over ℤ or ℚ routes through it. Getting the
dependency wrong here is expensive.

The temptation is to adopt a maintained crypto crate. R1 §2.1 shows why the mismatch is
structural rather than a matter of taste:

| Axis | Cryptography wants | A CAS wants |
|---|---|---|
| Modulus | **One**, fixed, known at compile time | **Many**, chosen at *runtime*, changed per reduction round |
| Size | 256–768 bit | **Machine word**, ≤ 63 bits, so `mulmod` is one `mulx` + a reduction |
| Timing | Constant-time is mandatory | Constant-time is a pure tax; data-dependent early exit is a *feature* |
| Batch shape | One element at a time | **Vectors of thousands** of residues, row-reduced in bulk |
| Extension | Towers over a fixed base | GF(p^k) for Cantor–Zassenhaus |
| Composite modulus | Rare | Required — Hensel lifting to `p^k` |

`ark-ff` takes the modulus as a **proc-macro attribute**:
`#[derive(MontConfig)] #[modulus = "18446744069414584321"]`. A compile-time string literal
cannot express "the 37th word prime, chosen at runtime because the previous 36 were
rejected as bad". The existence of the `ark-feanor` bridge crate is itself evidence the
mismatch is real and painful.

`crypto-bigint` *does* support runtime moduli (`MontyParams`/`BoxedMontyParams`), so it is
not disqualified on capability — but its whole reason for existing is constant-time
execution, which buys resolvent nothing, and it is sized for operands an order of magnitude
larger than a word prime.

`num-modular` and `num-prime` implement pieces of what is needed, but both are
**Apache-2.0-only**, which breaks the MIT arm under ADR-001.

---

## Decision

**Write `resolvent-modular` ourselves.**

Scope:

- **`Fp`** for word primes `p < 2^63`. Default reduction is **Shoup/Barrett with a
  precomputed reciprocal**; a Montgomery path is implemented alongside and benchmarked
  against it (see *Open* below). `Fp` is `Copy` and carries `p` plus its precomputed
  reciprocal **by value**; arithmetic is `#[inline]` inherent methods, never a method on a
  ring object (ADR-006).
- **`Zn`** for composite modulus — required by Hensel lifting to `p^k`.
- **`GF(p^k)`** as `Fp[x]/(f)`, for Cantor–Zassenhaus equal-degree splitting.
- **Bulk vector operations are first class**, not an afterthought: `axpy`, `scale`,
  `normalize`, `dot`, and sparse-row variants over `&mut [u32]` / `&[u32]`. F4's inner loop
  is bulk row reduction, not scalar arithmetic.
- **A deterministic prime registry**: `prime(i)` is a pure function of `i` over a
  checked-in generator of word primes, plus a "good prime" predicate per algorithm (does
  not divide the leading coefficient, does not collapse the degree, does not divide the
  discriminant). Never "pick a random prime" (ADR-012).
- **CRT accumulation and rational reconstruction**, with both bound-driven and
  stabilization-driven stopping rules, typed per ADR-010.
- **The `LANES` door stays open**: the coefficient-ring trait admits a tuple ring
  `Fp4 = [u32; 4]` so that Groebner.jl's batched multi-modular trick (up to ~2.7×
  amortized, `N = 4` in their production build) remains available later (ADR-006).

**Lane split, because these are different kinds of problem (constraint #3):**

- *Correctness of `resolvent-modular`* is a **certificate** lane: exhaustive small-`p`
  testing against `i128` reference arithmetic plus field-axiom property tests. Converges in
  days. Fan out immediately.
- *Speed of the bulk row-reduction kernel* is a **number** lane with no certificate.
  Converges over months, needs a tracked corpus and change-point detection, not a pass/fail
  gate. **These must be separate lanes and must not share a definition of done.**

---

## Consequences

- Small, extremely well-specified, perfectly self-certifiable code — which is the ideal
  shape for an agent lane, and a better use of the dependency budget than papering over a
  crypto crate's cost model.
- No dependency at all is added at this layer, which keeps the published graph short and
  keeps ADR-001's gate trivially satisfiable here.
- We own the tuning. Every threshold (delayed-reduction cutoff, Barrett-vs-Montgomery
  crossover, SIMD width) is measured on our corpus and checked in, per ADR-001 Tier B and
  ADR-012 §Tuning.
- We own the bugs. Mitigated by the exhaustive small-`p` oracle and by `crypto-bigint` as an
  optional differential oracle for the Montgomery path — it is permissively licensed and
  heavily audited, so it is a good dev-dependency even though it is a bad runtime one.
- **Open, and it is the first benchmark of this lane:** Barrett/Shoup vs Montgomery. R1
  argued for Barrett on the grounds that the same `p` is reused against many operands so
  Montgomery conversion is not amortized — but that is an architecture argument, not a
  measurement, and the answer may legitimately differ between the scalar path and the F4
  bulk-row path. Both are implemented; the default is chosen by measurement and recorded.

---

## Alternatives considered and why rejected

**`ark-ff`.** MIT OR Apache-2.0, well maintained, fast. Rejected: **compile-time modulus**.
Structurally incompatible with runtime prime selection, which is not a nice-to-have — it is
how modular methods work. Its `FftField`/two-adicity trait surface also assumes
SNARK-friendly primes that a CAS's prime selection cannot always satisfy.

**`crypto-bigint`.** Right capability (runtime odd moduli), wrong cost model
(constant-time tax, 256-bit+ sizing, one-element-at-a-time API). Kept as a differential
oracle.

**`num-modular` / `num-prime`.** Apache-2.0-only — breaks the MIT arm (ADR-001). Also small
enough that the dependency would not be earning its keep.

**`feanor-math`'s `zn_64` / `zn_rns`.** Genuinely good prior art (four ℤ/nℤ
implementations including a Barrett `zn_64` and an RNS variant) and MIT-licensed, so it is
a Tier-A read. Rejected as a *dependency* because the crate pins
`nightly-2026-03-01` (`rust-toolchain.toml`), which a foundation library cannot impose on
its consumers.

**Use `dashu`'s `modular` / `monty` modules directly.** Rejected as the primary path: they
are general-purpose (arbitrary-precision moduli, `Reducer` abstraction) where the hot case
is a single machine word, and using them would put `dashu` types back into hot code paths
that ADR-002's wall exists to keep them out of. They remain useful *inside* `resolvent-int`
for large-modulus cases (e.g. `p^k` beyond a word during Hensel lifting).

---

## What would reverse this

- **A permissively licensed crate appears that is shaped for runtime word-size moduli with
  first-class bulk vector operations.** Then re-evaluate — but only if it also admits the
  tuple/batched-lane shape, because losing that would cost more than the dependency saves.
- **The hand-rolled kernel fails to reach the number-lane thresholds** and a vendored SIMD
  library (not a field library) turns out to be the gap. That is a different decision —
  adopting a SIMD helper is orthogonal to adopting a field implementation.

Reversal is cheap in both directions because `Fp` is behind resolvent's own trait
vocabulary (ADR-006) and appears in no consumer-facing signature.
