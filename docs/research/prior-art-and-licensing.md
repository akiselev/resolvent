# Research R1 — Prior art, dependency landscape, and license posture

Status: **findings, for ratification before any Layer-0 code exists** (2026-07-31).
Scope: what `resolvent` may legally depend on given a hard MIT-OR-Apache-2.0 constraint,
what already exists in Rust, and what discipline keeps a permissive reimplementation safe
when the best references are GPL.

**Verification method.** Every license and status claim below was checked against a live
source on 2026-07-31, not recalled: `https://crates.io/api/v1/crates/<name>` for the
`license` field of the newest published version, `gh api repos/<owner>/<repo>` for
maintenance signals, and the repo's own `Cargo.toml` / `LICENSE` / `README` where the two
disagreed. **They disagree often.** GitHub's license auto-detector reports a single SPDX id
and picks one file: it reports `Apache-2.0` for `rust-num/num-bigint`, `tczajka/ibig`,
`RustCrypto/crypto-bigint`, `arkworks-rs/algebra`, and `cmpute/dashu`, all of which are
actually dual MIT OR Apache-2.0 per their `Cargo.toml` and `README`. **Rule for future
agents: `Cargo.toml`'s `license` field and the `LICENSE*` files are authoritative; the
GitHub sidebar badge is not.**

---

## 0. Headline findings

1. **The fastest bignums are all LGPL, and one of them surprised me.** `rug` and
   `gmp-mpfr-sys` are LGPL-3.0+ (expected — they are GMP/MPFR bindings). **`malachite` is
   also LGPL-3.0-only**, not permissive, because it is partly *derived from GMP, FLINT and
   MPFR source*. Malachite is the fastest pure-Rust bignum by a wide margin and it is
   unavailable to us. This is the single most consequential fact in this report.
2. **`dashu` is the only viable Layer-0 bignum.** MIT OR Apache-2.0, actively maintained
   (0.5.2 published 2026-07-31), and it has the algorithm coverage a CAS needs: NTT
   multiplication, Toom-3, Karatsuba, Lehmer GCD with `gcd_ext`, Burnikel–Ziegler division,
   and a Montgomery modular module. `num-bigint` — the obvious default — **tops out at
   Toom-3 with no FFT/NTT path at all** and is catastrophically slow on large operands.
3. **The GMP gap is real but lands almost entirely outside a CAS's hot path**, *provided*
   the modular-methods thesis is honoured. Published numbers put pure-Rust `dashu` ahead of
   GMP below ~1 kbit and roughly 3× behind at 10 kbit; the 100×-scale disasters only appear
   at megabit sizes that modular methods exist to avoid. Details and caveats in §1.3.
4. **One design consequence falls out of the bignum choice immediately: `resolvent` must be
   ℤ-primitive, not ℚ-primitive.** `RBig`-style rationals renormalise with a GCD on every
   operation, and bignum GCD is the one operation where pure-Rust is structurally behind
   GMP's subquadratic half-GCD. The first consumer's `QPoly` is a dense vector of rationals
   (`arrangements/crates/lazy-exact/src/roots.rs:43-45`) — **do not copy that shape.**
5. **Symbolica — the only "blazing fast Rust CAS" — is proprietary source-available.** Its
   `License.md` reads "It is not permitted to copy or distribute any part of the Symbolica
   code without express prior permission," commercial use is €3,000/yr per developer
   machine, and the free tier is limited to one core. It cannot be a dependency, cannot be
   vendored, and its source is a *legal hazard to read* in a way that GPL source is not.
6. **The permissive niche is genuinely open, and there is a live cautionary example.**
   `alkahest-cas` 3.7.0 declares `license = "Apache-2.0"` while carrying **non-optional**
   dependencies on `rug` and `gmp-mpfr-sys` (both LGPL-3.0+). That is exactly the trap
   resolvent exists to avoid, and it is being shipped today.
7. **`feanor-math` (MIT) is real prior art and must be read.** It already has F4-style
   Buchberger, Cantor–Zassenhaus, Hensel-lifting factorisation over ℤ/ℚ and number fields,
   LLL, and multivariate polynomial rings. Its own README states the gap resolvent targets:
   *"operations with polynomials over infinite rings … are currently very slow, since
   efficient implementation require a lot of care to prevent coefficient blowup."* It pins
   `nightly-2026-03-01`, which disqualifies it as a dependency but not as a reference.
8. **Every differential oracle we want is one `pacman -S` away on this machine.** Singular
   4.4.1, PARI 2.17.4, FLINT 3.6.0, SageMath 10.9, Maxima, sympy 1.14.0 are all in Arch
   `extra`. Macaulay2 is AUR-only. None is currently installed. All are GPL/LGPL, so all
   must be driven as **subprocesses, never linked**.
9. **A subtle license-arithmetic rule that constrains the dependency table:** resolvent's
   MIT arm is only worth something if *every* non-dev dependency also offers an MIT-or-
   equivalent arm. Apache-2.0 is FSF-incompatible with GPLv2; an Apache-only crate anywhere
   in the published tree silently removes GPLv2-compatibility from downstream consumers who
   chose the MIT arm precisely to get it. This rules out `num-modular`, `num-prime`, and
   `nalgebra` (all Apache-2.0-only) as runtime dependencies.

---

## 1. Arbitrary-precision integers and rationals — the critical path

### 1.1 The license audit

Verified 2026-07-31 against the crates.io API (newest published version's `license` field).

| Crate | Version | License (verified) | Verdict |
|---|---|---|---|
| `num-bigint` | 0.5.1 (2026-07-05) | **MIT OR Apache-2.0** | Permitted. Too slow. See §1.2. |
| `dashu` / `dashu-int` / `dashu-ratio` / `dashu-float` / `dashu-base` | 0.5.2 / 0.5.1 (2026-07-26…31) | **MIT OR Apache-2.0** | **Permitted. Recommended.** |
| `ibig` | 0.3.6 (2022-09-17) | **MIT OR Apache-2.0** | Permitted. Superseded by `dashu` (which is a fork of it). |
| `malachite` / `-base` / `-nz` / `-q` | 0.10.0 (2026-07-27) | **LGPL-3.0-only** | **FORBIDDEN.** |
| `rug` | 1.30.0 (2026-04-27) | **LGPL-3.0+** | **FORBIDDEN** as a runtime dep. Dev/oracle only. |
| `gmp-mpfr-sys` | 1.7.1 (2026-04-25) | **LGPL-3.0+** | **FORBIDDEN** as a runtime dep. Dev/oracle only. |
| `crypto-bigint` | 0.7.5 (2026-06-22) | **Apache-2.0 OR MIT** | Permitted. Wrong shape — see §2. |
| `ramp` | 0.7.0 (2022-04-23) | **Apache-2.0** (only) | Dead + Apache-only + nightly asm. Reject. |
| `fixed-bigint` | 0.6.1 (2026-07-25) | **Apache-2.0** (only) | Fixed-width `no_std` embedded target. Not applicable. |

**The GPL/LGPL-encumbered fast options, called out explicitly as requested:**

- **`rug` (LGPL-3.0+)** binds GMP + MPFR + MPC. GMP itself is, from v6, *"distributed under
  the dual licenses, GNU LGPL v3 and GNU GPL v2"* (https://gmplib.org/). Neither arm is
  permissive. MPFR is LGPL-3.0+ with no dual arm.
- **`malachite` (LGPL-3.0-only)**. Its README states plainly: *"Parts of Malachite are
  derived from [GMP], [FLINT], and [MPFR]."* The LGPL here is not a stylistic preference —
  it is the inherited obligation of a derived work. There is no dual arm and no permissive
  subset. `malachite-bigint` (a `num-bigint`-API-compatible shim in the same repo) inherits
  it.

**Why LGPL is not a workable compromise for resolvent specifically**, even though LGPL
nominally permits use by non-GPL programs. LGPL §4 conditions that permission on the
recipient being able to relink the combined work against a modified library. Rust has no
stable ABI and statically links by default; discharging §4 for a Rust crate that `use`s
LGPL types in its public API is at best unsettled and at worst impossible. More decisively:
resolvent's *entire product thesis* is "an MIT-licensed computer algebra kernel." A library
whose ℤ type is LGPL cannot honestly make that offer to a downstream that wants to ship a
closed or GPLv2 binary. **Reject, without further analysis.** This is constraint #2 doing
its job.

### 1.2 Algorithm coverage — the reason `num-bigint` loses

`num-bigint` is the ecosystem default and it is permissively licensed, so it deserves a
real reason for rejection rather than a vibe. Reading
`rust-num/num-bigint/src/biguint/multiplication.rs` (lines 93-285), its multiplication
ladder is: schoolbook → Half-Karatsuba (for imbalanced operands) → Karatsuba → **Toom-3,
and stop.** There is no FFT, NTT, or Schönhage–Strassen path. Division got Burnikel–Ziegler
in 0.4.7 (2026-07-02). Small values are inlined as of 0.4.7, which fixed its worst
small-operand overhead.

Toom-3 is Θ(n^1.465). At megabit operand sizes that is a catastrophe, and the measured
numbers in §1.3 show it: 482 s where GMP takes 2.8 s.

`dashu-int` by contrast has, per its source tree and `integer/CHANGELOG.md`:

- `mul/{simple,karatsuba,toom_3,ntt}` — **NTT multiplication over Proth primes
  (`K·2^N + 1`) combined via Garner CRT**, added in 0.4.3, activating above ~4 000 words
  (~256 kbit), with specialised squaring paths at every tier.
- `div/{simple,divide_conquer}` — Burnikel–Ziegler.
- `gcd/lehmer` — Lehmer's GCD, plus `gcd_ext` (extended GCD) in the public API.
- `monty/` — Montgomery modular arithmetic for odd moduli (`MontgomeryRepr`, `Montgomery`),
  with mul/sqr/add/sub/neg/double/pow/inv.
- `modular/` — a separate runtime-modulus modular arithmetic module with a `reducer`
  abstraction.
- Runtime-tunable thresholds via `DASHU_THRESHOLD_*` env vars under the `tuning` feature —
  directly useful for a benchmark lane.

That is the full CAS shopping list: fast multiplication for large operands, exact division,
GCD and extended GCD, and modular arithmetic. `num-bigint` has everything except fast
multiplication for large operands.

`ibig` is the ancestor `dashu-int` forked from (`dashu-int/README.md`: *"The majority of the
code is based on the ibig crate"*), with a `NOTICE` modification record. `ibig`'s last
crates.io release is 0.3.6 from 2022-09-17 and it predates the NTT work. Prefer `dashu`.

### 1.3 The honest performance answer

Two independent public benchmark suites, both cited with their own caveats. **I did not run
these; I did not invent them; and both are stale in the specific way that matters most.**

**(a) `tczajka/bigint-benchmark-rs`** — large-operand tasks (Fibonacci, digits of *e*).
Versions benchmarked: `rug` 1.24.1, `malachite` 0.4.14, `dashu` 0.4.2, `ibig` 0.3.6,
`num-bigint` 0.4.6.

| Library | e 100k | e 1m | e 10m | fib 10m | fib 100m | fib_hex 100m |
|---|---|---|---|---|---|---|
| `rug` (GMP) | 0.009 s | 0.184 s | 2.788 s | 0.171 s | 2.937 s | 0.575 s |
| `malachite` | 0.012 s | 0.240 s | 3.689 s | 0.306 s | 5.192 s | 0.933 s |
| `dashu` | 0.019 s | 0.756 s | 19.943 s | 0.773 s | 26.224 s | 6.718 s |
| `ibig` | 0.020 s | 0.649 s | 20.673 s | 0.793 s | 26.705 s | 6.725 s |
| `num-bigint` | 0.058 s | 5.037 s | **482.383 s** | 7.007 s | **671.809 s** | 6.312 s |

Source: https://github.com/tczajka/bigint-benchmark-rs

**Critical caveat: this run used `dashu` 0.4.2 — one release *before* NTT multiplication
landed in 0.4.3.** The `dashu` column above is a pre-NTT number and is not evidence about
`dashu` 0.5.2. It is still valid evidence about `num-bigint` (no NTT was added since) and
about `malachite` (LGPL, so moot). **What would settle it:** re-run
`bigint-benchmark-rs` locally with `dashu` 0.5.2 pinned. That is a concrete, cheap,
self-verdicting task and should be the first item in the Layer-0 lane.

**(b) `DRMacIver/rust-bigint-benchmarks`** — small-and-medium operand micro-benchmarks
across 205 cases. Versions: `dashu-int` 0.4.2, `ibig` 0.3.6, `num-bigint` 0.4.6,
`malachite-nz` 0.9.1, `rug` 1.30.0.

- Win counts: `dashu` 54, `rug` 52, `ibig` 37, `num-bigint` 35, `malachite` 27.
- Head-to-head `dashu` vs `rug`: `dashu` faster in **123** cases, `rug` in 82.
- One-word `ubig_add`: `dashu` ~3.3 ns vs `rug` ~21.2 ns. One-word `ubig_mul`: ~3.3 ns vs
  ~18.4 ns. GMP carries fixed per-value overhead with no inline small-value path.
- Two words: roughly parity (~25 ns vs ~28 ns add).
- **Crossover around ~1 kbit**; at 10 kbit `rug` wins `ubig_mul` ~4.5 µs vs ~12.8 µs (≈2.8×).
- Stated caveat, quoted: *"These are quick, machine-specific local numbers, not
  authoritative … Treat them as a rough comparison, and re-run locally for anything
  load-bearing."*

Source: https://github.com/DRMacIver/rust-bigint-benchmarks/blob/main/benchmark-report.md

**So: is "the permissive options are materially slower than GMP" true, and does it matter?**

*Below ~1 kbit — no, `dashu` is faster than GMP*, because GMP allocates and has no inline
small-value representation while `dashu` inlines up to a double-word. This regime is where a
CAS spends most of its bignum time under the modular-methods design: coefficient ingress,
CRT accumulation over a few dozen 63-bit primes, Landau–Mignotte and Hadamard bounds,
rational reconstruction, and the final multiply-back verification of a factorisation. All
of those operate on integers in the tens-to-thousands-of-bits range.

*Between 1 kbit and ~100 kbit — yes, roughly 2–3× behind*, which is a real but survivable
constant factor, and one that shrinks as `dashu`'s NTT threshold work matures.

*Above ~1 Mbit — unknown post-NTT, and irrelevant if the architecture is honoured.* Megabit
integers appear in a CAS exactly when someone computes a resultant or a Gröbner basis
*directly over ℤ or ℚ* instead of mod several primes and reconstructing. The IDEAS spec
already calls this out — *"naive rational arithmetic gives you coefficient explosion and a
dead project"* — and it is right. **The modular-methods decision is what makes the
permissive bignum choice affordable.** They are the same decision viewed from two sides.

**The one place the gap does bite, and the design change it forces.** GMP has a subquadratic
half-GCD; `dashu` has Lehmer, which is quadratic in the worst case. GCD is not a corner case
for a CAS — it is invoked on *every single rational arithmetic operation* to renormalise
num/den. `dashu`'s own `RBig` does this. The first consumer's polynomial type is
`QPoly { coeffs: Vec<Rational> }` where `Rational` wraps `dashu::rational::RBig`
(`arrangements/crates/lazy-exact/src/roots.rs:41-45`,
`arrangements/crates/lazy-exact/src/exact/rational.rs:1-17`). For degree-≤4 geometry
predicates that is fine. **For resolvent it is not.**

> **Decision forced (Layer 0/1, one-way door):** resolvent's polynomial coefficients are
> **ℤ-primitive**. ℚ appears only as a thin façade at the API boundary that clears
> denominators to a primitive-part-over-ℤ representation on ingress and reattaches a single
> content factor on egress. No inner loop ever calls a rational GCD. This is standard CAS
> practice (Cohen; von zur Gathen & Gerhard), but it is worth writing down as a *decision*
> because the nearest reference implementation in this workspace does the opposite.

### 1.4 Layer-0 recommendation

**Depend on `dashu` (MIT OR Apache-2.0), behind a resolvent-owned newtype wall.**

- `resolvent-int` exposes `Integer` and `Rational` newtypes. `dashu` types appear in **no**
  public signature and in **no** trait bound outside that crate. Rationale: this is the
  cheapest possible insurance against the one thing that could still go wrong — `dashu`
  going unmaintained, or a measured need to swap in a hand-written half-GCD or an
  optional GMP backend for the tail. It also honours constraint #1: a consumer writing an
  adapter deals with `resolvent::Integer`, not with a version-pinned third-party type
  leaking through.
- **Do not** re-export `dashu` from the public API. A public re-export makes `dashu`'s
  semver a hard part of resolvent's semver.
- Add an **optional, non-default, never-in-CI-release `backend-gmp` feature** later *only
  if* measurement justifies it, and document loudly that enabling it subjects the build to
  LGPL-3.0+. Do not build this speculatively.
- Keep `rug` as a **dev-dependency oracle only**. `dashu`'s own repo already does exactly
  this — its 0.5.0 changelog credits *"the new `fuzz/` `rug::Integer` oracle"* for finding a
  `nth_root` bug. That is the correct pattern, and it is precedent from the crate we are
  adopting.

---

## 2. Finite fields and modular arithmetic

### 2.1 What a CAS needs vs. what cryptography crates optimise for

This is the sharpest mismatch in the whole dependency landscape, and getting it wrong would
be expensive.

| Axis | Cryptography wants | A CAS wants |
|---|---|---|
| Modulus | **One**, fixed, known at compile time (a curve order) | **Many**, chosen at *runtime* from a prime sieve, changed per reduction round |
| Modulus size | 256–768 bit | **Machine word**: ≤ 63 bits, so that `mulmod` is one `mulx` + a Barrett/Shoup reduction |
| Modulus parity | Odd (Montgomery requires it) | Odd is fine — CAS primes are odd anyway — but also needs ℤ/n for composite n (Hensel lifting to prime powers) |
| Timing | **Constant-time is mandatory** | Constant-time is a pure tax; data-dependent early exit is a feature (e.g. early-abort in modular GCD) |
| Batch shape | One element at a time | **Vectors of thousands** of residues, row-reduced in bulk (F4 is sparse linear algebra over GF(p)) |
| Extension fields | Towers over a fixed base | GF(p^k) for factorisation over finite fields (Cantor–Zassenhaus) |

### 2.2 `ark-ff` — disqualified on the modulus axis

`ark-ff` 0.6.0, MIT OR Apache-2.0, actively maintained (`arkworks-rs/algebra` pushed
2026-07-28, 882 stars). Its instantiation model, quoted from
`ff/README.md`:

```rust
#[derive(MontConfig)]
#[modulus = "18446744069414584321"]
#[generator = "7"]
pub struct F64Config;
pub type F64 = Fp64<MontBackend<F64Config, 1>>;
```

**The modulus is a proc-macro attribute — a compile-time string literal.** A CAS picks its
primes at runtime, typically dozens per computation, sized to fit `Word` and chosen to avoid
bad reduction. There is no way to express that in `ark-ff`'s `Config` model without
generating code at runtime. `ark-ff` also carries a large trait surface aimed at
FFT-friendliness for STARK/SNARK provers (`FftField`, two-adicity requirements) that a CAS's
prime selection cannot always satisfy.

**Verdict: ignore.** The `ark-feanor` bridge crate (MIT OR Apache-2.0, 0.7.1) exists to
paper over precisely this impedance mismatch, which is itself evidence the mismatch is real.

### 2.3 `crypto-bigint` — right capability, wrong cost model

`crypto-bigint` 0.7.5, Apache-2.0 OR MIT, very actively maintained. It *does* support
runtime moduli: `MontyParams` / `MontyForm` and `FixedMontyParams` / `FixedMontyForm` for
runtime odd moduli, plus `BoxedMontyParams` / `BoxedMontyForm` for heap-allocated dynamic
sizing (https://docs.rs/crypto-bigint/latest/crypto_bigint/modular/index.html). Odd modulus
is required.

But its whole reason for existing is **constant-time execution**, which for a CAS is a pure
performance tax paid on every operation with zero benefit. Nothing in resolvent processes
secrets. It is also sized for 256-bit+ operands, not for the ≤63-bit word primes where a CAS
lives, and its API is not shaped for the bulk-vector reduction that F4 needs.

**Verdict: ignore for the GF(p) core.** Possibly useful as a differential oracle for a
hand-rolled Montgomery implementation, since it is permissively licensed and heavily
audited.

### 2.4 Recommendation: hand-roll it, in resolvent

Write `resolvent-modular` ourselves. This is a small, extremely well-specified, and
**perfectly self-certifiable** piece of code — the ideal shape for an agent lane:

- `Fp` for word-size primes `p < 2^63`, with **Shoup/Barrett** precomputed-reciprocal
  `mulmod` as the default (better than Montgomery when the same `p` is used against many
  different operands and conversion in/out of Montgomery form would dominate), and a
  Montgomery path benchmarked against it.
- Runtime prime selection: a prime sieve plus a "good prime" predicate (does not divide the
  leading coefficient, does not collapse the degree, etc.). Verification of every modular
  result is what makes the whole engine Las Vegas rather than heuristic — per constraint #4.
- `Zn` for composite modulus (Hensel lifting to `p^k`).
- `GF(p^k)` as `Fp[x]/(f)` for Cantor–Zassenhaus.
- **Vectorised bulk operations first-class**, because F4's inner loop is row reduction over
  GF(p), not scalar arithmetic. This is the single hottest loop in the whole library.

**Verdict on the lane type (constraint #3):** the *correctness* of `resolvent-modular` is a
certificate (exhaustive small-`p` testing against `i128` reference arithmetic, plus
property tests for field axioms — a fully automatic verdict). The *speed* of the F4 bulk
row-reduction kernel is a number to optimise, with no certificate. **Those must be separate
lanes.** The correctness lane converges in days; the kernel lane converges over months and
needs a tracked benchmark corpus with change-point detection, not a pass/fail gate.

`num-modular` (Apache-2.0-only) and `num-prime` (Apache-2.0-only) implement pieces of this
but are ruled out by §0 item 9 as runtime dependencies, and the code is small enough that
the dependency would not be earning its keep anyway.

---

## 3. Existing Rust CAS and polynomial work

All licenses verified against crates.io on 2026-07-31.

| Project | Version / last update | License (verified) | Alive? | What it is | Verdict |
|---|---|---|---|---|---|
| **`symbolica`** | 2.2.0 (2026-07-22), repo pushed 2026-07-31, 953 ★ | **`non-standard` — proprietary source-available** | Very | High-performance CAS for Rust + Python, aimed at high-energy physics | **Do not depend. Do not read. See §3.1.** |
| **`feanor-math`** | 3.5.24 (2026-07-26), 55 ★ | **MIT** (in `Cargo.toml`; **no `LICENSE` file in repo** — 404) | Yes | Rings/fields framework, F4-style Buchberger, Cantor–Zassenhaus, Hensel factorisation over ℤ/ℚ/number fields, LLL, FFT/NTT convolutions | **Learn from, heavily. Do not depend. See §3.2.** |
| **`alkahest-cas`** | 3.7.0 (2026-07-25), repo created 2026-04-21, 9 ★ | `Apache-2.0` **but transitively LGPL — see §3.3** | New | "Symbolic expressions, polynomials, Gröbner bases, JIT, Arb ball arithmetic" | **Do not depend. Cautionary example.** |
| `groebner` | 0.2.0 (2026-07-07), 7 ★, 1004 total downloads | MIT | Barely | Buchberger | Ignore. Toy scale. |
| `lau-algebraic-geometry` | 0.1.0 (2026-05-31), 22 total downloads | MIT | No | "varieties, ideals, Gröbner bases, elimination theory" | Ignore. |
| `flint-sys` | 0.9.0 (2026-04-26) | MIT OR Apache-2.0 *(bindings only)* | Marginal | Raw FFI to FLINT | **Bindings ≠ library. Linking pulls LGPL-3.0+ FLINT.** Oracle-only, §5. |
| `rug-polynomial` | 0.2.6 (2026-01-27) | MIT OR Apache-2.0 *(wrapper only)* | Marginal | Polynomials over Rug + FLINT | Same trap. Oracle-only. |
| `polynomen` | 2.0.0 (2024-10-24) | **GPL-3.0-only** | No | Univariate polynomials | **Forbidden.** A permissive-looking name hiding a GPL crate — exactly the kind of thing an unchecked audit misses. |
| `algebraics` | 0.3.0 (2022-03-29) | **LGPL-2.1-or-later** | No | **Real algebraic numbers in Rust** — closest thing to Layer 3 | **Forbidden**, and dead anyway. Notable that the one existing L3 attempt is copyleft. |
| `polynomial` | 0.2.6 (2023-09-30) | MIT | Dormant | Dense univariate over a generic scalar | Ignore. Trivial. |
| `symbolic_polynomials` | 0.1.0 (**2016**) | Apache-2.0 | Dead | Integer polynomial manipulation | Ignore. |
| `rust-poly` | 0.4.3 (2025-09-12) | MIT | Dormant | *Numeric* (float/complex) polynomial root-finding | Ignore. Different problem — approximate, not exact. |
| `ark-poly` | 0.6.0 (2026-04-26) | MIT OR Apache-2.0 | Very | Dense/sparse polys over FFT-friendly fields for SNARKs | Ignore, for the §2.2 reason. |
| `msolve` **(crates.io)** | 0.6.0 (2020) | MIT | No | **A sudoku solver.** The name is taken. | Note: the real msolve is C. See §5. |
| `nalgebra` | 0.35.0 | **Apache-2.0** (only) | Very | Dense linear algebra | Not needed; and Apache-only bars it under §0.9. F4 needs sparse GF(p) row reduction, which is ours to write. |

### 3.1 Symbolica — the reason resolvent exists, and a hazard

`symbolica`'s crates.io `license` field is literally the string `non-standard` for every
published version back to 1.3.0. The repo's `License.md` reads, verbatim:

> The source code of Symbolica is publicly available. It is not permitted to copy or
> distribute any part of the Symbolica code without express prior permission.

Tiers, per https://symbolica.io/license/: free tier is **one core, one instance per device,
non-commercial only**; Standard is €3,000/year per interactive developer machine, with
server/CI use priced separately; *"If you use Symbolica as part of your employment, whether
in academia or in a commercial or non-commercial organization, a license is required"*;
*"redistribution of the code, whether modified or unmodified, requires prior written
permission."*

Three consequences.

1. **It cannot be a dependency at any tier.** Even the free tier's one-core limit would be a
   runtime constraint baked into resolvent's users.
2. **It validates the market.** `feanor-math`'s own README concedes that for multivariate
   polynomial work *"Symbolica does perform better than `feanor-math`."* The performance
   ceiling for a Rust CAS is demonstrably high; the permissive ceiling is unclaimed.
3. **It is the one codebase agents must be forbidden from reading.** GPL source is
   *readable* (copyright covers expression, and the GPL grants a read/study right);
   Symbolica's license grants no copying right at all and its source-availability is
   conditioned. There is no upside to reading it — the algorithms are all in the published
   literature — and a large downside. **Add it to the §6 blocklist, at a stricter tier than
   the GPL sources.**

### 3.2 `feanor-math` — the honest prior-art assessment

This is the closest thing to a permissive Rust CAS that exists, and pretending otherwise
would be dishonest. From its README, it already provides:

- `RingBase`/`RingStore` two-trait ring framework (a deliberate workaround for Rust's
  borrow/blanket-impl limitations — worth studying before designing Layer 0's traits).
- ℤ (`RustBigintRing`, plus an optional MPIR binding), ℤ/nℤ with **four** implementations
  including a Barrett-reduction `zn_64` for sub-64-bit moduli and an RNS `zn_rns`.
- Dense and sparse univariate `PolyRing`; multivariate `MultivariatePolyRingImpl` "based on
  a sparse representation using ordered vectors."
- `FreeAlgebra` → Galois fields and number fields.
- Cantor–Zassenhaus; factorisation over ℚ/ℤ via Hensel lifting; factorisation over number
  fields; LLL; Finke–Pohst enumeration; **"Buchberger's algorithm (F4-style) to compute
  Gröbner basis."**

**Why it is not the answer, stated plainly:**

1. **It pins nightly.** `rust-toolchain.toml` says `channel = "nightly-2026-03-01"`. A
   library that wants to be a foundation cannot require a pinned nightly of its consumers.
   This alone is disqualifying as a dependency.
2. **Its own README names the exact gap resolvent targets:** *"operations with polynomials
   over infinite rings (integers, rationals, number fields) are currently very slow, since
   efficient implementation require a lot of care to prevent coefficient blowup, which I did
   not have time or need to invest."* That "care" **is** modular-methods-everywhere. It is
   resolvent's entire Layer-2 thesis.
3. **It has no real algebraic numbers and no root isolation.** Layer 3 — the bridge to
   computational geometry, and the whole point for consumer #28 — is absent.
4. **Licence hygiene gap:** `Cargo.toml` says `license = "MIT"` but there is **no `LICENSE`
   file in the repository root** (verified: raw GitHub 404). If we ever wanted to vendor
   anything we would have to get that fixed upstream first. We do not intend to vendor.
5. `MultivariatePolyRingImpl` uses "ordered vectors", not bit-packed exponent vectors. The
   IDEAS spec is explicit that packing *"is most of your Gröbner performance."*

**Verdict: read it closely, cite it, differentially test against it, do not depend on it.**
It is MIT, so unlike the GPL references there is *no* contamination hazard in reading it —
though we still should not copy verbatim, because attribution obligations attach even to
MIT code. It is the single best Rust-idiomatic reference for the ring-trait design problem,
which is genuinely hard in Rust and which `feanor-math` has already hit the walls of.

### 3.3 `alkahest-cas` — the live cautionary tale

`alkahest-cas` 3.7.0 declares `license = "Apache-2.0"` (Apache-only, note — not even dual).
Its crates.io dependency list for 3.7.0 includes, as **normal, non-optional** dependencies:

```
gmp-mpfr-sys ^1   normal          <- LGPL-3.0+
rug          ^1   normal          <- LGPL-3.0+
```

An Apache-2.0 crate that mandatorily links LGPL-3.0+ GMP/MPFR is at minimum a distribution
problem for its users, who cannot honour the Apache-2.0 grant's implications without also
discharging LGPL §4. The project is three months old (repo created 2026-04-21), 9 stars,
302 lifetime downloads, and has shipped 12+ releases in that span. It is not competition. It
is a worked example of the failure mode resolvent's constraint #2 exists to prevent, and it
belongs in the CI gate's regression corpus as a "this is what a bad audit misses" case.

### 3.4 Non-Rust references for algorithm design only

- **`Groebner.jl`** — `sumiya11/Groebner.jl`, **GPL-2.0** (verified 2026-07-30 push, 75 ★).
  A modern, readable F4/Buchberger implementation with the multi-modular machinery visible at
  a high level, and the closest thing to a *legible* reference for the modular-Gröbner
  pipeline. **I initially assumed the usual Julia-ecosystem MIT and was wrong** — it is
  copyleft, so it sits in **Tier B** (§6), same as Singular and FLINT, not in the freely
  readable tier. Recording the mistake because it is exactly the shape of error the §6.4 gate
  and the "verify, do not recall" method exist to catch.
  (Neighbours found in the same search: `ooinaruhugh/GroebnerWalk.jl` GPL-3.0;
  `ederc/GroebnerBasis.jl` no license declared — treat unlicensed as all-rights-reserved,
  i.e. Tier C.)
- **`msolve`** (C) — *"open source, distributed under the license GPLv2"*
  (https://arxiv.org/abs/2104.03572, Berthomieu–Eder–Safey El Din). State of the art for
  0-dimensional solving: DRL Gröbner → FGLM conversion to lex → real solving of the
  univariate. This is precisely resolvent's M2→M4 pipeline. **GPLv2: oracle only, never
  linked, and reading it falls under the §6 discipline.**
- **CoCoA** — note that the crates.io name `cocoa` is macOS FFI bindings, unrelated. CoCoALib
  the CAS is GPLv3. Oracle only, and lower priority than Singular.

### 3.5 Blunt answer: does any of this already solve the problem?

**No.** Ranked by how close they get:

- `symbolica` gets closest on *capability* and is **legally unusable**.
- `feanor-math` gets closest on *license* and is missing the modular-methods engine, real
  algebraic numbers, packed monomials, and stable-Rust compatibility.
- Everything else is a toy, dead, copyleft, or solving a different problem.

The gap resolvent claims — **permissive, stable-Rust, modular-methods-throughout, with
algebraic numbers as a first-class exported type** — is real and unoccupied as of
2026-07-31.

---

## 4. The Layer-4 e-graph question

| | `egg` | `egglog` |
|---|---|---|
| Version / date | 0.11.0, 2025-12-04 | **2.0.0, 2026-02-12** |
| License (verified) | **MIT** | **MIT** |
| Repo activity | pushed 2026-07-19, 1793 ★, 26 open issues | pushed 2026-07-30, 803 ★, 118 open issues |
| Release cadence | 0.9.5 (2023-06) → 0.10.0 (2024-12) → 0.11.0 (2025-12) — roughly annual | 0.1.0 (2023-10) → 1.0.0 (2025-10) → 2.0.0 (2026-02) — accelerating |
| Model | Rust-native `Language` trait, `Rewrite` rules, `Runner` | Datalog-flavoured language; rules are text or built via API |
| Positioning | `egg`'s own README points readers at egglog; egglog's crates.io description says *"It is the successor to the popular rust library egg."* | |

Both are MIT, so **neither poses a license risk**. The question is purely engineering, and
it is not urgent — Layer 4 is the last layer and the IDEAS spec explicitly calls it a thin
layer that is "not the point."

**Recommendation: commit to neither now. Define the L4 seam as a resolvent-owned trait.**

Rationale, in order of weight:

1. `egg` 0.11 has an unresolved successor story. Adopting it means adopting a library whose
   own maintainers point elsewhere. Adopting `egglog` 2.0 means adopting a 2.0-in-February
   API whose 118 open issues suggest it is still moving.
2. `egg`'s `Language` trait requires the expression type to be an enum with `Id` children —
   it wants to *own* the term representation. Resolvent's L4 is specified as a hash-consed
   DAG that L0–L3 already produce. Handing representation ownership to a third-party crate
   at the top of a five-layer stack is exactly the coupling constraint #1 warns about, one
   level up.
3. `alkahest-cas` already depends on `egglog ^0.4` — pinned two majors behind current — which
   is empirical evidence about how fast that API churns under a consumer.
4. The e-graph is *optional to the value proposition*. Geometry, FEM, and SMT consumers call
   Layers 0–3. Shipping L4 behind a feature flag with an `egg` adapter *and* an `egglog`
   adapter, both optional and neither in the default feature set, costs almost nothing and
   defers the bet indefinitely.

**Lane type (constraint #3):** L4 is a *correctness-certifiable* lane in a weak sense —
rewrite soundness can be property-tested by evaluating both sides at random points over
GF(p) — but "did simplification produce a *good* result" is a number to optimise with no
certificate. Sequence it last and do not let it block anything.

---

## 5. Differential-testing oracles

Licenses matter far less here because **nothing links**: every oracle is driven as a
subprocess over a text protocol, so no oracle code enters resolvent's binary or its
dependency graph. (The one exception that would link — FLINT via `flint-sys` — is handled
separately below.)

**Availability on this exact machine** (Arch Linux, verified with `pacman -Si` on
2026-07-31 — **none currently installed**):

| Oracle | Package | Version | License | Install cost | Interface |
|---|---|---|---|---|---|
| **Singular** | `extra/singular` | 4.4.1.p5-11 | GPL-2.0-only OR GPL-3.0-only | **11.4 MiB download / 59.3 MiB installed** | CLI, batch-scriptable (`Singular -q -c '...'`); its own `.lib` language |
| **PARI/GP** | `extra/pari` | 2.17.4-1 | GPL-2.0+ | small | `gp -q -f`, reads stdin, prints results |
| **FLINT** | `extra/flint` | 3.6.0-1 | **LGPL-3.0+** (changed from LGPL-2.1 after 3.1) | small | C library — needs a driver, see below |
| **SageMath** | `extra/sagemath` | 10.9-6 | GPL-2.0+ | **56.4 MiB download / 371.2 MiB installed**, ~60 transitive deps (pulls Singular, PARI, FLINT, GAP, NTL, Maxima, linbox…) | `sage -c '...'`, Python |
| **sympy** | `extra/python-sympy` / already importable via pyenv | **1.14.0 — already present** | BSD-3-Clause | **zero** | `python3 -c` |
| **Maxima** | `extra/maxima` | 5.49.0-13 | GPL-2.0+ | small | CLI |
| **Macaulay2** | **AUR only** (`macaulay2`, 7 votes) | 19030.995c6fd8c | GPL-2 or GPL-3 | **source build, expensive, AUR-fragile** | CLI |
| **Mathematica** | — | — | proprietary, paid | licence + install | `wolframscript` |

**Best oracle per operation** (based on what each system is actually specialised for; these
are architecture claims about the tools, not benchmark claims):

| Operation | Primary oracle | Secondary | Notes |
|---|---|---|---|
| Gröbner bases (DRL, lex, elimination) | **Singular** | Macaulay2, msolve | Singular's `std`/`groebner` is the reference implementation the literature benchmarks against; it is scriptable, cheap to install, and has no Python layer in the way. |
| Multivariate factorisation over ℚ/ℤ | **Singular** (`factorize`) | sympy, PARI | |
| Univariate factorisation over ℤ, ℚ, GF(p), number fields | **PARI/GP** (`factor`, `nffactor`) | FLINT, Singular | PARI is the number-theory specialist. |
| Resultants / subresultant PRS | **PARI/GP** (`polresultant`) | Singular (`resultant`), sympy | sympy's `subresultants` gives the *whole PRS chain*, which is the actual intermediate data a resultant lane must match — more useful than just the final resultant. |
| Real root isolation | **PARI/GP** (`polrootsreal`) | sympy `real_roots`, `CRootOf` | PARI returns certified intervals. |
| Algebraic-number comparison / arithmetic | **sympy** (`CRootOf`, `minimal_polynomial`) | PARI (`nfinit`, `polredabs`), Sage `QQbar` | Sage's `QQbar` is the richest but is the 371 MiB dependency. |
| Bignum integer arithmetic | **`rug`** (dev-dependency, in-process) | PARI | The only oracle worth *linking*, and only as a dev-dep — precedent: `dashu`'s own `fuzz/` uses `rug::Integer` as its oracle. |
| Everything, as a fallback | **SageMath** | — | Install it once on the CI box; do not make it a gate for local dev. |

**Recommended harness shape (an agent lane with an automatic verdict):**

- A single `oracles/` workspace member, `publish = false`, that shells out to whichever
  binaries are present and **skips with a loud, counted `SKIP` rather than a pass** when one
  is absent. Never silently green.
- Tiering by install cost, so a fresh clone still tests something:
  - **Tier 0 (zero install, always runs):** `sympy` via `python3`. Already available here.
  - **Tier 1 (cheap, developer-recommended):** `singular` + `pari`. ~70 MiB combined.
  - **Tier 2 (CI box only):** `sagemath`, `macaulay2` from AUR, `msolve` from source.
- **Text protocol only.** Each oracle gets an adapter that emits a canonical S-expression or
  JSON form of the input and parses a canonical form back. This makes disagreements
  triageable and keeps oracle-specific parsing out of the test bodies.
- **Self-certification runs first and is the *primary* gate; oracles are the secondary
  gate.** Per constraint #4: factorisation multiplies back, Gröbner checks membership via
  stored cofactors, GCD checks divisibility both ways. A self-certifying failure is a bug in
  resolvent with certainty. An oracle disagreement might be a normalisation difference (sign
  of the leading coefficient, monomial-order convention, unit factors) and needs triage.
  Do not let an agent lane treat those as equivalent signals.

**The FLINT exception.** FLINT is LGPL-3.0+ and `flint-sys` (MIT OR Apache-2.0 *bindings*)
would link it in-process, which is far faster than subprocess round-trips for high-volume
property testing. If that speed proves necessary: put it in a `publish = false` crate,
behind a non-default feature, that is **not** in any published crate's dependency graph, and
document the LGPL obligation in that crate's README. Otherwise prefer the uniform
subprocess rule. Note that `flint-sys`'s repo (`alex-ozdemir/flint-rs`, 10 stars, pushed
2026-04-26) reports **no license file on GitHub** despite the crates.io metadata — resolve
that before depending on it even as a dev-dep.

---

## 6. The non-clean-room hazard, and the discipline that contains it

Resolvent's best algorithmic references — Singular (GPL-2/3), PARI (GPL-2+), FLINT
(LGPL-3+), msolve (GPL-2), CoCoALib (GPL-3), Macaulay2 (GPL-2/3), Sage (GPL-3) — are all
copyleft. Resolvent's output must be MIT OR Apache-2.0. This is structurally the same
problem `arrangements` faced with CGAL, and **the correct move is to adopt that project's
existing framing rather than invent a second, differently-worded one.**

`/home/dev/projects/arrangements/DESIGN.md` §1 states it as (quoted verbatim):

> MIT OR Apache-2.0. **Independent reimplementation informed by architectural study of the
> GPL sources** — not "clean-room" (that term means the authors never saw the original; we
> did line-level reading, and the reports in `docs/research/` document it).
> Algorithms/ideas are not copyrightable and the published literature (Bentley–Ottmann;
> Fogel/Halperin/Wein *CGAL Arrangements and Their Applications*; de Berg et al.) covers the
> substance, but CGAL is dual-licensed commercially, so process discipline matters: write
> Rust with the research reports open, **not** the CGAL tree; no copied constants, comments,
> or identifier structure; review diffs against the reports, not the sources.

**Resolvent's version, same shape, substituting its own references:**

> MIT OR Apache-2.0. Independent reimplementation informed by architectural study of the
> GPL/LGPL sources — **not "clean-room"**; that term means the authors never saw the
> original, and we intend to read Singular, FLINT, PARI, and msolve at the level needed to
> understand *what* they do. Algorithms and ideas are not copyrightable, and here the
> published literature covers the substance more completely than it does for CGAL's
> arrangement internals: Faugère (F4, *J. Pure Appl. Algebra* 139, 1999), van Hoeij
> (lattice recombination, *J. Symbolic Comput.* 33, 2002), Zassenhaus, Collins/Brown
> (subresultant PRS), Rouillier–Zimmermann (VCA real root isolation), von zur Gathen &
> Gerhard *Modern Computer Algebra*, Cohen *A Course in Computational Algebraic Number
> Theory*, and Geddes/Czapor/Labahn *Algorithms for Computer Algebra*. Process discipline:
> write Rust with the literature notes open, **not** the reference source tree; no copied
> constants, comments, or identifier structure; review diffs against the notes, not the
> sources.

**Resolvent's position is materially *safer* than `arrangements`'s was**, and the plan should
say so rather than performing anxiety. CGAL's arrangement traits are a large body of design
decisions that exist only in the source. F4, van Hoeij, subresultant PRS, and VCA are all
fully specified in refereed papers and in two standard textbooks. An agent that has read
von zur Gathen & Gerhard chapter 6 does not need to look at `flint/nmod_poly_factor/`.

### The operational rules

**Tier A — freely readable, freely cited.**
- The refereed literature and textbooks above.
- The *user-facing documentation and manuals* of any system (Singular's manual, PARI's
  user guide, Sage's reference). Documentation describes **behaviour**, and matching another
  system's documented behaviour is a compatibility goal, not a derivation.
- Permissively licensed Rust: `feanor-math` (MIT), `dashu` (MIT/Apache), `ark-ff`
  (MIT/Apache). Read freely. Still do not copy verbatim — MIT carries an attribution
  obligation, and a verbatim block would need its notice carried, which defeats the point.

**Tier B — readable for *understanding*, never for *transcription*.**
- Singular, FLINT, PARI, msolve, CoCoALib, Macaulay2, Sage source; `Groebner.jl` (GPL-2.0)
  and `GroebnerWalk.jl` (GPL-3.0).
- Permitted: reading to understand *which* algorithm variant is used, *why* a step exists,
  what edge case a guard is protecting against, what the overall pipeline is.
- **Forbidden, without exception:** copying code, comments, identifier names, file/module
  structure, or **magic constants and tuning thresholds**. Thresholds are the most likely
  accidental transcription and the least defensible — a number like "switch to
  Karatsuba at 32 limbs" is someone's measurement, and it is also *wrong for our machine*.
  **Every threshold in resolvent must be re-derived by measurement against resolvent's own
  benchmark corpus, with the measurement checked in.** This rule is simultaneously a
  licensing rule and a correctness rule, which is why it holds.
- Procedure: read, then **write a note in `docs/research/` in your own words**, then close
  the source, then implement from the note. This is the same "reports open, not the tree"
  discipline as `arrangements`, and it produces the artifact that proves it was followed.

**Tier C — do not read at all.**
- **Symbolica.** Its license grants no copying right and conditions source availability;
  there is no defensible reading posture and no algorithmic content that is not in the
  literature. Blocklist it explicitly, at a stricter tier than the GPL sources, and say why
  — otherwise an agent will reasonably infer that "source-available" means "safe to read"
  because GPL source is.
- Any commercial CAS source (Magma, Maple, Mathematica internals).
- **Any repository with no declared license** — `ederc/GroebnerBasis.jl` is a live example.
  No license means all rights reserved, which is *stricter* than GPL, not looser. Agents
  reliably get this backwards.

### How to document the posture so it is auditable

1. **`docs/decisions/0001-license-posture.md`** — an ADR carrying the Tier A/B/C rules
   verbatim, the derivation from `arrangements/DESIGN.md` §1, and the reasoning. This is the
   document an agent is pointed at, and the one a downstream lawyer reads.
2. **A `Sources:` block in every research note in `docs/research/`**, tagging each reference
   with its tier. An agent implementing from a note can then see at a glance whether the
   note's content came from a paper or from a GPL source tree.
3. **A `Derivation:` line in the module doc-comment of every non-obvious algorithm**, citing
   the *paper*, not the reference implementation. E.g.
   `//! van Hoeij lattice recombination — J. Symbolic Comput. 33(5):425-445, 2002, §3.`
   If a module cannot cite a paper, that is a signal it was written from a source tree, and
   the review should catch it.
4. **A mechanical license gate in CI**: `cargo-deny` (MIT OR Apache-2.0, 0.20.2) with an
   explicit `[licenses] allow = [...]` list, `deny` for every copyleft SPDX id, and — this
   is the part that catches the `alkahest-cas` failure — the check running over the
   **published** dependency graph (`--all-features` minus dev-only features), not just over
   direct dependencies. Add `cargo-about` (MIT OR Apache-2.0, 0.9.1) to generate the
   attribution file. **This lane has a fully automatic verdict** and should be part of the
   very first CI setup, before any algebra exists — it is cheap now and expensive to
   retrofit.
5. **A regression corpus for the gate**, containing at minimum: `malachite` (LGPL hiding
   behind a permissive-looking pure-Rust crate), `polynomen` (GPL-3.0-only with an innocuous
   name), and a synthetic Apache-only crate depending on `rug` (the `alkahest-cas` shape). If
   the gate does not fail on all three, it is not doing its job.

---

## 7. Dependency recommendation table

Every license verified against crates.io on 2026-07-31. "MIT arm?" is the §0.9 test: does
this crate offer an MIT-or-equivalent option, so resolvent's MIT arm stays meaningful and
GPLv2-compatible for downstream consumers?

### Runtime (published) dependencies — keep this list short

| Crate | Version | License | MIT arm? | Role | Verdict |
|---|---|---|---|---|---|
| `dashu` (`-int`, `-ratio`, `-base`) | 0.5.x | MIT OR Apache-2.0 | ✅ | Layer-0 ℤ and ℚ | **ADOPT**, behind a `resolvent-int` newtype wall. No public re-export. |
| `smallvec` | 1.15.2 | MIT OR Apache-2.0 | ✅ | Packed exponent vectors, small coefficient arrays | **ADOPT.** Already proven in the consumer's `lazy-exact`. |
| `thiserror` | 2.0.19 | MIT OR Apache-2.0 | ✅ | Error types | **ADOPT.** |
| `rustc-hash` | 2.1.3 | Apache-2.0 OR MIT | ✅ | Fast non-DoS-resistant hashing for the L4 hash-cons table and monomial maps | **ADOPT** when L4 lands. |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 | ✅ | Raw-entry API for hash-consing, if `std::HashMap` proves insufficient | Adopt only if needed. |
| `rayon` | 1.12.0 | MIT OR Apache-2.0 | ✅ | Parallelism across CRT primes — embarrassingly parallel, the natural first win | **ADOPT**, behind a default-off `parallel` feature. |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | ✅ | Corpus serialisation for benchmarks and oracle protocols | Optional feature only. |
| — | | | | **Everything else is ours to write.** | Layers 1–4 have no dependency. |

### Rejected runtime dependencies, with the reason

| Crate | Reason |
|---|---|
| `malachite*` | **LGPL-3.0-only.** Fastest permissive-looking option; is not permissive. |
| `rug`, `gmp-mpfr-sys` | **LGPL-3.0+.** GMP/MPFR. Dev-oracle only. |
| `num-bigint` | Permitted but **no FFT/NTT** — Toom-3 ceiling. Rejected on capability, not license. |
| `ibig` | Superseded by `dashu`, which is its fork with NTT. |
| `ark-ff`, `ark-poly` | **Compile-time modulus.** Structurally wrong for a CAS. |
| `crypto-bigint` | Constant-time tax, 256-bit+ sizing, wrong batch shape. Useful as an oracle. |
| `num-modular`, `num-prime` | **Apache-2.0 only** — breaks the MIT arm (§0.9). Also small enough to write. |
| `nalgebra` | **Apache-2.0 only**, and F4 needs sparse GF(p) row reduction, not dense LA. |
| `polynomen` | **GPL-3.0-only.** |
| `algebraics` | **LGPL-2.1-or-later**, dead since 2022. |
| `symbolica` | **Proprietary.** See §3.1 and §6 Tier C. |
| `feanor-math` | MIT and excellent, but **pins `nightly-2026-03-01`**. Reference, not dependency. |
| `egg` / `egglog` | Both MIT, no license issue. **Defer** behind a resolvent-owned trait (§4). |
| `flint-sys`, `rug-polynomial` | Permissive bindings to LGPL libraries. Bindings ≠ library. Oracle-only. |

### Dev-dependencies (never in the published graph)

| Crate | Version | License | Role |
|---|---|---|---|
| `proptest` | 1.11.0 | MIT OR Apache-2.0 | Property tests: trichotomy/transitivity for `AlgebraicReal`, ring axioms, `gcd(a·g, b·g)` divisibility |
| `arbitrary` | 1.4.2 | MIT OR Apache-2.0 | Structured fuzzing of polynomial inputs |
| `divan` or `criterion` | 0.1.21 / 0.8.2 | MIT OR Apache-2.0 | Benchmarks. `divan` is lighter; `criterion` has the change-point tooling. |
| `rug` | 1.30.0 | LGPL-3.0+ | **Bignum oracle.** Dev-only, exactly as `dashu`'s own `fuzz/` does. |
| `cargo-deny` | 0.20.2 | MIT OR Apache-2.0 | The license gate (§6.4) |
| `cargo-about` | 0.9.1 | MIT OR Apache-2.0 | Attribution file generation |

**One rule that makes all of this enforceable:** the workspace has exactly two kinds of
crate — `publish = true` crates whose dependency graph `cargo-deny` gates against a
permissive allowlist, and `publish = false` crates (`oracles/`, `fuzz/`, `bench/`) that may
carry LGPL dev-dependencies and shell out to GPL binaries. There is no third category and no
per-crate exception process.

---

## 8. What this report could not settle

These are stated as "what would settle it," not guessed at.

1. **`dashu` 0.5.2's actual large-operand performance post-NTT.** Every published number I
   found used 0.4.2, pre-NTT. **Settle it:** clone `tczajka/bigint-benchmark-rs`, pin
   `dashu` 0.5.2, run locally, commit the numbers to `docs/research/`. Half a day. Do it
   before Layer 0 is written, because if NTT did not close the gap the case for an optional
   GMP backend gets stronger — and that changes the feature-flag design, which is cheap now
   and expensive later.
2. **`dashu`'s GCD behaviour on the operand sizes resolvent actually produces.** Lehmer vs
   GMP's half-GCD is the one identified structural gap (§1.3). **Settle it:** microbenchmark
   `gcd`/`gcd_ext` at 64, 256, 1k, 4k, 16k bits against `rug`. This directly determines how
   aggressive the ℤ-primitive discipline has to be.
3. **Whether any *permissively licensed* F4 implementation exists in any language.** The
   Julia search turned up only GPL ones (§3.4), and `feanor-math`'s is "F4-style Buchberger"
   rather than true F4. If none exists, the F4 lane has no Tier-A reference implementation at
   all and must be built from Faugère's paper plus the Macaulay-matrix literature — which is
   feasible but slower, and worth knowing before the lane is sized. **Settle it:** a focused
   search of the Julia/Python/Go/Haskell ecosystems for `license = MIT|BSD|Apache` + Gröbner.
4. **Whether `feanor-math`'s missing `LICENSE` file matters.** `Cargo.toml` says MIT; the repo
   has no `LICENSE` at root. We do not plan to vendor, so this is probably moot — but if any
   lane wants to lift a *test corpus* or a *test vector* from it, get the file added upstream
   first (a one-line issue).
5. **Whether Singular or msolve is the better Gröbner oracle in practice.** msolve is more
   modern and is the F4-with-FGLM pipeline resolvent is copying architecturally, but it is
   not packaged for Arch and needs a source build. Singular is one `pacman -S` away.
   **Settle it:** build msolve once, compare on a shared corpus, decide whether the install
   cost buys enough. Not blocking — start with Singular.
6. **Whether the `MontyForm`/Barrett choice for word-size GF(p) should be Shoup, Barrett, or
   Montgomery.** I argued for Shoup/Barrett on the grounds that the same `p` is reused across
   many operands so Montgomery conversion overhead is not amortised — but this is an
   architecture argument, not a measurement. **Settle it:** it is the first benchmark of the
   `resolvent-modular` lane, and the answer may differ between the scalar path and the F4
   bulk-row path.
7. **What packing scheme the exponent vectors use.** Out of scope for R1 (this is R-something
   else), but flagged because §0.4 and the one-way-door constraint make it the *other*
   irreversible Layer-1 decision, and `feanor-math`'s "ordered vectors" choice is the
   nearest permissive reference doing it differently.

---

## Sources

Verified 2026-07-31.

- crates.io API — `https://crates.io/api/v1/crates/{num-bigint, dashu, dashu-int, dashu-ratio, dashu-float, dashu-base, malachite, malachite-base, malachite-nz, malachite-q, rug, gmp-mpfr-sys, ibig, crypto-bigint, ramp, fixed-bigint, symbolica, feanor-math, alkahest-cas, groebner, lau-algebraic-geometry, polynomen, algebraics, polynomial, symbolic_polynomials, rust-poly, ark-ff, ark-poly, ark-feanor, flint-sys, rug-polynomial, msolve, egg, egglog, num-modular, num-prime, nalgebra, smallvec, thiserror, rustc-hash, hashbrown, rayon, serde, proptest, arbitrary, criterion, divan, cargo-deny, cargo-about}` and the corresponding `/versions` and `/dependencies` endpoints
- https://github.com/mhogrefe/malachite — README licensing statement, `COPYING` / `COPYING.LESSER`
- https://gmplib.org/ — GMP dual-license statement
- https://flintlib.org/doc/introduction.html — FLINT LGPL-3.0+ (LGPL-2.1 before 3.1)
- https://github.com/symbolica-dev/symbolica/blob/main/License.md and https://symbolica.io/license/
- https://github.com/FeanorTheElf/feanor-math — `Readme.md`, `Cargo.toml`, `rust-toolchain.toml`
- https://github.com/cmpute/dashu — `README.md`, `integer/README.md`, `integer/CHANGELOG.md`, `integer/src/{mul,gcd,div,modular,monty}/`
- https://github.com/rust-num/num-bigint — `RELEASES.md`, `src/biguint/multiplication.rs:93-285`
- https://github.com/tczajka/bigint-benchmark-rs
- https://github.com/DRMacIver/rust-bigint-benchmarks/blob/main/benchmark-report.md
- https://docs.rs/crypto-bigint/latest/crypto_bigint/modular/index.html
- https://github.com/arkworks-rs/algebra/blob/master/ff/README.md
- https://arxiv.org/abs/2104.03572 — Berthomieu, Eder, Safey El Din, *msolve: A Library for Solving Polynomial Systems* (GPLv2)
- https://github.com/sumiya11/Groebner.jl — GPL-2.0, via GitHub repository search API
- https://github.com/egraphs-good/egg — README successor note
- `pacman -Si singular sagemath` on this machine (Arch Linux); AUR RPC v5 for `macaulay2`
- `/home/dev/projects/arrangements/DESIGN.md` §1 — the license-posture framing this report mirrors
- `/home/dev/projects/arrangements/crates/lazy-exact/src/roots.rs:41-45, 317-327` — consumer `QPoly` / `RealRoot` shape
- `/home/dev/projects/arrangements/crates/lazy-exact/src/exact/rational.rs:1-17` — consumer's `dashu::rational::RBig` usage
- `/home/dev/projects/arrangements/crates/arrangements/src/geoms/conics.rs:272-287` — the hand-rolled degree-≤4 conic resultant resolvent eventually replaces
- `/home/dev/projects/IDEAS-crates.md` §4 — the source specification
