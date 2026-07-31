# resolvent — architecture

Status: **proposed, for ratification before any code exists** (2026-07-31).
Inputs: `docs/research/prior-art-and-licensing.md` (R1),
`docs/research/consumer-requirements.md` (R2),
`docs/research/algorithms-and-representation.md` (R3),
`/home/dev/projects/IDEAS-crates.md` §4.

This document decides the crate-level shape. Every one-way door it touches has a
corresponding ADR in `docs/decisions/`; this document states the decision and points at
the ADR for the argument. Where the two disagree, the ADR wins.

---

## 0. What resolvent is, in one paragraph, so scope creep has something to hit

resolvent is the algebra engine — polynomials, ideals, algebraic numbers, resultants —
that exact computational geometry, FEM form compilers, and SMT NRA theories call. Its
first useful release is not a CAS. It is roughly twenty-five functions
(R2 §0.1: the entire algebraic surface a shipping 17k-LOC exact geometry engine consumes
is re-exported under six names at
`/home/dev/projects/arrangements/crates/lazy-exact/src/lib.rs:82-83`) generalized from
degree ≤ 4 to arbitrary degree, with the coefficient-growth control that makes arbitrary
degree computable. Symbolic calculus is a thin optional layer at the top and is not the
point.

Two framings from the research that this architecture is built around, both of which
correct the source spec:

- **The Gröbner one-way doors do not gate the first consumer.** `arrangements` touches no
  multivariate machinery: its polynomial type is a dense `Vec<Rational>`
  (`crates/lazy-exact/src/roots.rs:43-45`), its resultants are hand-rolled 2×2 conic
  determinants (`crates/arrangements/src/geoms/conics.rs:276-287`), and its `RealRoot`
  (`roots.rs:317-322`) is exactly `AlgebraicReal`. The multivariate/F4 program is a
  **parallel track**, and it stays parallel only if the univariate type is defined first
  and standalone (ADR-007).
- **Packed monomials are not "most of your Gröbner performance."** Measured, packing is
  worth ~15%; sparse GF(p) linear algebra is 73–91% of an F4 run and the divisor-query
  index is worth 10–20× (R3 §1.6). The genuine one-way door in Layer 1 is the
  *interning/id/key structure*, not the field width (ADR-008). A lane brief that says
  "optimize monomial comparison" buys 15% and misses a 20×.

---

## 1. Crate split

### 1.1 The decision

**A workspace of seven published crates and three unpublished ones, versioned in
lockstep, with `resolvent` the only crate a consumer is expected to name.**

```
resolvent-base      no algebra, no bignum. Trait vocabulary + verdict types + errors.
   ↑
resolvent-int       Integer / Natural / Rational newtypes over dashu. The bignum wall.
   ↑
resolvent-modular   Fp (word primes), Zn, GF(p^k), CRT, rational reconstruction,
                    deterministic prime registry, bulk GF(p) vector kernels.
   ↑
resolvent-poly      UPoly<C> (dense univariate) · Ring context + monomial arena +
                    MPoly (sparse distributed) · RecursiveView · Kronecker bridge.
   ↑
resolvent-algebra   gcd, squarefree, resultants + subresultant PRS, factorization
                    (Zassenhaus → van Hoeij), Buchberger, F4, FGLM, ideal ops.
   ↑                                    ↖
resolvent-real      root isolation (Sturm oracle, Descartes/VCA, ANewDsc),
                    separation bounds, Bernstein enclosure, SqrtExt, AlgebraicReal,
                    radical-tower signs, rational_between, curve analysis, RUR.
   ↑
resolvent           facade. Re-exports, feature plumbing, docs, prelude. No algorithms.

resolvent-expr      L4 hash-consed DAG, differentiation, transcendental symbols, CSE,
                    straight-line lowering, e-graph seam trait.
                    depends on: base, int, poly, algebra.  NOT on real.

publish = false:
resolvent-oracles   subprocess drivers for Singular/PARI/sympy/Sage/msolve; rug dev-oracle.
resolvent-bench     benchmark corpus + harness + change-point tracking.
resolvent-fuzz      structured fuzz targets.
```

### 1.2 Why not the shape the founding brief sketched

The brief's plausible shape was `resolvent-core` (rings + polys), `resolvent-algebra`,
`resolvent-real`, `resolvent-expr`, `resolvent`. Two objections, both load-bearing:

**`resolvent-core` would be the crate every lane touches.** Rings, the bignum wall, the
GF(p) kernels, and both polynomial representations in one compilation unit makes that
crate the serial bottleneck for compilation, for merge conflicts, and for lane
independence — which is exactly what constraint #3 cannot afford. Splitting it four ways
gives four lanes with disjoint files, disjoint test suites, and `cargo test -p` verdicts
that do not interfere.

**A consumer implementing a ring must not have to pull `dashu`.** `resolvent-base` has no
third-party dependency except `thiserror`. A consumer that wants `UPoly<TheirScalar>`
depends on `resolvent-base` alone and never sees a version-pinned bignum in its tree. That
is constraint #1's "thin adapter the consumer writes" made mechanical rather than
aspirational.

The counter-argument — more crates means more release coordination — is dissolved by
**lockstep versioning**: one `version` in `[workspace.package]`, all crates released
together, inner crates documented as "the supported surface is `resolvent`; these are
published so they can be depended on directly, not so they can be mixed and matched across
versions." That is a documentation promise plus a `=x.y.z` inter-crate dependency pin, and
it costs one CI step.

### 1.3 What the layering forbids

Mechanically enforced, not by habit. Each rule has a named CI gate.

| # | Rule | Gate |
|---|---|---|
| L1 | No crate depends on a crate above it in §1.1. | Checked-in expected dependency graph; CI diffs `cargo tree --edges normal` against it. |
| L2 | `dashu` appears in exactly one `Cargo.toml`: `resolvent-int`. | `cargo tree -i dashu` must list exactly one direct dependent. |
| L3 | No published crate re-exports a third-party type, and no third-party type appears in any public signature outside `resolvent-int`'s private modules. | `cargo public-api` snapshot, reviewed on diff. |
| L4 | No geometric vocabulary anywhere in a published crate: no `Point`, `Curve`, `Segment`, `Arc` (the shape, not `std::sync::Arc`), `Vertex`, `Face`, `tolerance`, `epsilon`, `snap`. | grep gate. |
| L5 | No published crate names `arrangements`, `lazy-exact`, `cadabra2`, or `sinbad` in source, `Cargo.toml`, feature name, or doc example. | grep gate. |
| L6 | `publish = false` crates may depend on `publish = true` crates; never the reverse, including dev-dependencies. | `cargo metadata` assertion. |
| L7 | `rayon` is behind a default-off `parallel` feature, appears only in `-algebra` and `-real`, and appears in no public signature. | grep + feature-matrix build. |
| L8 | The facade crate `resolvent` contains no `fn` with a body longer than a re-export or a feature `cfg`. | line-count lint on `resolvent/src`. |
| L9 | Every published crate denies `clippy::unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `arithmetic_side_effects` outside `#[cfg(test)]`. | `cargo clippy -- -D warnings`. |
| L10 | `cargo-deny` runs over the **published** graph (`--all-features` minus dev-only features) against a permissive allowlist, with the R1 §6.4 regression corpus (`malachite`, `polynomen`, a synthetic Apache-only crate depending on `rug`) asserted to fail. | `cargo deny check`. |

L1 and L10 are the two that must exist on day zero, before any algebra. They are cheap now
and expensive to retrofit (R1 §6.4).

### 1.4 Crate-to-lane map, with verdict types

Constraint #3 requires every lane to have an automatic verdict, and requires
number-to-optimize lanes to be sequenced differently from certificate lanes. The crate
boundary is the lane boundary; the verdict type is per-lane, not per-crate.

| Crate | Lane | Verdict | Notes |
|---|---|---|---|
| `resolvent-base` | trait vocabulary | **certificate** (trait-law property tests) | Must land first; everything inherits it. |
| `resolvent-int` | bignum wall | **certificate** (differential vs `rug` dev-oracle, per `dashu`'s own `fuzz/` precedent) | Plus one **number** sub-lane: re-run `tczajka/bigint-benchmark-rs` on `dashu` 0.5.2 (R1 §8.1 — every published figure is pre-NTT 0.4.2 and is not evidence). |
| `resolvent-modular` | `Fp`/`Zn`/`GF(p^k)` correctness | **certificate** (exhaustive small-`p` vs `i128`, field axioms) | Separate from the bulk-kernel lane below. |
| `resolvent-modular` | bulk GF(p) row kernels | **number** | Months, not days. Needs the tracked corpus and change-point detection, not a pass/fail gate. |
| `resolvent-poly` | `UPoly<C>` arithmetic | **certificate** (`(a·b)/b == a`, degree additivity, naive O(n²) reference) | Gates the whole consumer track. Build first. |
| `resolvent-poly` | monomial arena / keys / overflow | **certificate** (order axioms, round-trip encode/decode, overflow always detected never wrapped) | ADR-008, ADR-009. |
| `resolvent-algebra` | gcd | **certificate**, fully self-certifying (R3 §3.2: `H\|A`, `H\|B`, `deg H = deg gcd mod p`) | The degree half is the one people forget; divisibility alone accepts any common divisor. |
| `resolvent-algebra` | resultants / subresultant PRS | **certificate** — the strongest and cheapest in Layer 2 (two independent implementations + three structural invariants, R3 §6.3) | Also the first consumer's unlock. Build early. |
| `resolvent-algebra` | factorization | **certificate for the product, partial for irreducibility** (R3 §3.3) | An oracle that only multiplies back silently accepts a coarse factorization. Write that into the lane brief. |
| `resolvent-algebra` | Gröbner certified | **certificate** (cofactors) | The oracle for the fast mode. |
| `resolvent-algebra` | Gröbner fast (F4) | **number** | Graded against certified mode + R3 §9.3 thresholds. |
| `resolvent-real` | root isolation correctness | **certificate** (Sturm counts, disjointness, sign changes, round-trip) | Build Sturm *as the oracle*; it is never the production isolator. |
| `resolvent-real` | root isolation cliffs (ANewDsc) | **number** | Mignotte family. Do not start before plain Descartes passes. |
| `resolvent-real` | `AlgebraicReal` | **certificate, the strongest in the library** (trichotomy / transitivity / sort-stability / **step budget**) | R3 §8.3 point 9 is the primary detector: hangs, not wrong answers, are the expected failure mode. "Did not finish" must be graded as "wrong". |
| `resolvent-real` | curve analysis | **certificate** (Bézout/Euler consistency) + **number** | Largest component with no counterpart in the prior art. |
| `resolvent-expr` | L4 | weak certificate (rewrite soundness by random GF(p) evaluation); "is the simplification *good*" is a **number** with no certificate | Sequence last. Must block nothing. |
| `resolvent-oracles` | harness | **certificate** (skips are counted and loud, never silently green) | Tier 0 sympy (zero install) / Tier 1 Singular+PARI / Tier 2 Sage+msolve+Macaulay2. |

Ordering constraints that fall out and are not negotiable:

- `resolvent-poly`'s `UPoly<C>` before anything in `-algebra` or `-real`.
- Modular gcd/squarefree (R2's R2) **before** `Res_y` (R11) and curve analysis (R13).
  Retrofitting modular arithmetic under a working ℚ implementation is a rewrite, not an
  optimization (R2 §7 D9).
- Buchberger before F4. Plain Descartes before ANewDsc. Zassenhaus before van Hoeij. A
  number lane may not start before its own reference/oracle passes (R3 §10).

---

## 2. Coefficient rings, and where generics end

Full argument in **ADR-006**. The summary the plan needs:

### 2.1 Three tiers, and the rule that separates them

> **Generics may cross a crate boundary. They may not cross into an inner loop.**
> Every generic algorithm has a `where C: CoeffRing` entry point and delegates to a
> monomorphic kernel selected by at most one runtime `match` per *call*, never per
> element.

**Tier G — generic, monomorphized, source-level.** `UPoly<C>` and `MPoly` arithmetic and
the reference implementations of every algorithm. Generic over `C: CoeffRing`, but
resolvent instantiates it over a **closed set it controls**: `Fp`, `Fp4` (the batched
tuple ring), `Integer`, `Rational`, `Zn`, `GFpk`, and — behind a `number-fields` feature —
`NumberFieldElem`. Consumers may instantiate over a foreign `C`; they get correctness, not
speed, and the docs say so in those words.

**Tier M — monomorphic, concrete, no trait bounds.** The kernels. The rule of thumb: *any
loop whose body is a single coefficient operation and whose trip count is data-dependent
and unbounded is written over a concrete type.* Named, exhaustively:

| Kernel | Concrete over |
|---|---|
| F4 Macaulay row reduction | `u32` payloads + `FpParams` by value; sparse row format |
| GF(p) bulk vector ops (axpy, scale, normalize, dot) | `u32`/`u64`, `FpParams` by value |
| Descartes/VCA Taylor shift `x → x+1` and dyadic scaling `x → 2^k x` | `UPoly<Integer>` and a dyadic `i64` filter path |
| Sign-variation counting | `&[Integer]`, with a dyadic-approximation fast path (R3 §7.3) |
| CRT accumulation and rational reconstruction | `Integer` |
| Monomial SWAR add/sub/compare/divisibility | `[u64; W]`, `W` a **const generic** over `{1,2,4,8}` — const generics, not trait generics |
| Horner evaluation in the `AlgebraicReal` refinement loop | `UPoly<Integer>` at a dyadic rational |

**Tier D — dynamic, runtime data.** The `Ring` context object carries variable count,
monomial order, exponent field width, characteristic, and the coefficient-ring tag. It is
consulted **once per phase**, never per element (ADR-009).

### 2.2 What the trait must not be

Rejected shapes, with the reason, because an agent will otherwise reinvent them:

- **`Box<dyn Ring>` or `&dyn Ring` in a hot path.** An indirect call per coefficient
  operation. Never.
- **Ring-object arithmetic — `ring.add(&a, &b)`.** `feanor-math`'s `RingBase`/`RingStore`
  two-trait split exists precisely to work around Rust's borrow and blanket-impl
  limitations under this style (R1 §3.2), and it is a warning, not a model. resolvent's
  `Fp` is `Copy` and carries `p` plus its Barrett reciprocal by value; arithmetic is
  `#[inline]` inherent methods.
- **A trait tower deep enough to need associated-type projections in bounds.** Compile
  time and error messages both explode. Depth is capped at three:
  `Ring` → `CommutativeRing` → {`Field`, `EuclideanDomain`, `UniqueFactorizationDomain`},
  plus the orthogonal marker traits below.
- **Requiring `Ord` on the coefficient ring.** `Fp4` (four residues at once) has no
  meaningful order, and R3 §3.5 measures up to ~2.7× amortized from batching four primes.
  Requiring `Ord` closes that door permanently (ADR-006).

### 2.3 The trait vocabulary (`resolvent-base`)

```rust
pub trait Ring: Clone + PartialEq + Send + Sync + 'static {
    const LANES: usize;              // 1 for scalar rings; 4 for Fp4. Batching stays open.
    type Scalar: Ring;               // Self for LANES == 1
    fn zero() -> Self;  fn one() -> Self;
    fn add(&self, r: &Self) -> Self; fn sub(&self, r: &Self) -> Self;
    fn mul(&self, r: &Self) -> Self; fn neg(&self) -> Self;
    fn is_zero(&self) -> bool;
}
pub trait CommutativeRing: Ring {}
pub trait Field: CommutativeRing { fn inv(&self) -> Option<Self>; }
pub trait EuclideanDomain: CommutativeRing { fn div_rem(&self, d: &Self) -> Option<(Self, Self)>; }

/// Orthogonal capability markers. Absence is a *capability* statement, not a defect.
pub trait Ordered: Ring { fn sign(&self) -> Sign; }            // Integer, Rational. NOT Fp.
pub trait Reducible: Ring { type Image: Field;                  // Integer -> Fp
    fn reduce(&self, m: &Modulus) -> Option<Self::Image>; }
pub trait Liftable: Ring { fn crt_lift(images: &[..], ..) -> Result<Self>; }
pub trait BulkOps: Ring { fn axpy(dst: &mut [Self], a: &Self, src: &[Self]); /* … */ }
```

The modular pipeline is bounded by `C: Reducible + Liftable`, not by `C: Ring`. That is
what makes "modular methods everywhere" a *type-level* statement rather than a slogan: a
coefficient ring that cannot be reduced mod p simply cannot reach the fast path, and the
generic reference path is what it gets instead. Honest, and mechanically enforced.

### 2.4 Compile-time budget

Monomorphization count is `|generic algorithms| × |instantiations|`. Controls:

- The instantiation set is closed (§2.1) and `number-fields` is feature-gated.
- Kernels are Tier M, so the expensive code is compiled once, not once per `C`.
- Generic functions that are large and cold use the *inner-function* trick: a thin generic
  wrapper that converts to a concrete representation and calls a non-generic body.
- CI tracks `cargo build --timings` on the workspace and fails on a >20% regression in
  total front-end time. This is a **number** lane and is graded as one.

---

## 3. Error model

Full argument in **ADR-011**. The centerpiece:

> **Fail at construction, not at query.**
> Every invariant is checked when a value is built — squarefree-ness, isolation, nonzero
> at interval endpoints, ring compatibility, degree and variable bounds, exponent range.
> Construction returns `Result`. Every method on a well-formed value that is
> *mathematically* total is total *in the type system* too.

This is what lets `AlgebraicReal::cmp` be a real `Ord` (§5.3) and `sign_of` return a bare
`Sign`, while keeping the "no panics" rule absolute. It also matches how the first consumer
already works: its exact families declare `type Error = Infallible` and push all
fallibility into construction, and `spherical_circle.rs:47-60` fails closed with a
structured `SphError::Unsupported` at exactly that point.

### 3.1 What is a `Result`

- All constructors and parsers.
- Every operation whose *input domain* is narrower than its input type: `div_rem` by zero,
  `inv` of a non-unit, Gröbner of a non-zero-dimensional ideal where the caller asked for
  a zero-dimensional routine.
- Every operation that can hit a documented capability limit: exponent overflow (ADR-008),
  variable count over the arena's width, degree over a packed bound.
- Every operation whose termination argument is a *budget* rather than a theorem (§3.4).

### 3.2 What is a panic

**Nothing, in any published crate, outside `#[cfg(test)]`.** Enforced by L9. Specifically:

- `debug_assert!` is permitted and encouraged; it compiles out.
- A violated internal invariant returns `Error::Internal { invariant: &'static str }`, it
  does not panic. Rationale: a geometry kernel embedding resolvent may be behind a
  `extern "C"` boundary where unwinding is UB, and callers running under `panic = "abort"`
  cannot recover. More importantly, a panic and a hang are the same thing to a user of an
  exact kernel: an operation that did not return an answer.
- Allocation failure keeps Rust's default behaviour (abort). resolvent does not pretend to
  handle OOM; it says so.

### 3.3 "Unsupported" is a structured value

```rust
#[non_exhaustive]
pub enum Unsupported {
    CoefficientRing  { got: RingTag, required: &'static [RingTag] },
    Characteristic   { got: u64, required: CharacteristicClass },
    VariableCount    { got: u32, max: u32 },
    TotalDegree      { got: u64, max: u64 },
    MonomialOrder    { got: OrderTag, required: &'static [OrderTag] },
    NotSquarefree,
    NotZeroDimensional { dimension: u32 },
    PositiveDimensionalRealSolve,
    TranscendentalSymbol { name: SymbolId },
}
```

Never a string. The consumer's own fail-closed path matches on variants
(`spherical_circle.rs:47-60` is the shape); a string forces string-matching and silently
breaks on rewording.

### 3.4 Budgets, and why they are not timeouts

R3 §8.2 F5 is the load-bearing observation: `sign_of(h)` where `h(α) = 0` **never
terminates** unless zero-ness is settled algebraically first, and refinement stalls
forever on a non-squarefree defining polynomial. The expected failure mode of a wrong
implementation is a silent hang, which is undebuggable in production and invisible to a
test suite that grades on assertions.

So:

- Every unbounded loop takes a `Budget`, counted in **steps** — never wall-clock time,
  because wall-clock is nondeterministic and §4 forbids that.
- Where a proven bound exists (Mignotte–Davenport separation for comparison; Landau–
  Mignotte for factor coefficient size; Hadamard for resultant coefficient size), the
  default budget is *derived from the bound*, so exhaustion is **proven impossible** and
  the budget is a bug detector, not a control-flow exit. Exceeding it is `debug_assert!`
  in debug and a counter on the diagnostics hook in release; the loop continues, because
  it is still mathematically correct.
- Where no proven bound exists (van Hoeij lattice iteration; stabilization-driven modular
  reconstruction), the budget *is* a control-flow exit and exhaustion returns
  `Err(Error::BudgetExhausted { consumed, partial })` carrying enough state to resume.
- The `AlgebraicReal` property-test harness runs every case under a step budget and grades
  "did not finish" as **wrong**, not as "timeout" (R3 §8.3 point 9).

### 3.5 Partial results are values, not exceptions

- **`AlgebraicReal` refinement is monotone.** Any observation of its bounds is a *valid*
  enclosure, including a partially-completed one; there is no torn intermediate state that
  is unsound (the protocol is lifted from `crates/lazy-exact/src/real.rs:1-15`, which
  documents exactly this discipline for a different problem).
- **Probabilistic results are typed, not annotated.** Every routine whose correctness
  depends on a probabilistic step returns `Certified<T>`:

  ```rust
  pub struct Certified<T> { pub value: T, pub certificate: Certificate }
  pub enum Certificate { Proved(ProofKind), Probable(Evidence) }
  ```

  `Probable` carries its evidence (prime indices used, stabilization rounds, whatever
  bound is derivable). Default API paths return `Proved`; `Probable` requires opting in
  (ADR-010). Every competing system defaults to uncertified over ℚ — Groebner.jl says so
  in its own documentation — so resolvent will lose those benchmarks by construction, and
  the harness must compare like with like rather than hide the difference (R3 §3.1).

### 3.6 One verdict vocabulary, and the rule that keeps it one

R2 §8 open question 8 asked where the fail-closed enclosure verdict lives, and warned that
returning `Uncertain<Sign>` from range-bounding while returning `Sign` from comparison
gives consumers two vocabularies. Settled:

> A function returns a **bare `Sign`** if and only if it is total and exact.
> A function that can be indeterminate returns **`Verdict<Sign>`** and never `Sign`.
> `Verdict` is produced only by *enclosure and filter* APIs — Bernstein `sign_over`,
> the f64 enclosure comparison — and never by an algebraic-decision API.

```rust
pub enum Verdict<T> { Certain(T), Unknown }
```

`Unknown` means "this cheap rung declined to decide", and the caller's response is to
climb to the exact rung, never to guess. This maps 1:1 onto the consumer's existing
`Uncertain<T>` (`crates/lazy-exact/src/uncertain.rs`), which keeps the adapter trivial
without resolvent importing anything.

**No API anywhere takes a tolerance, epsilon, or "close enough" parameter.** L4 greps for
it. The first consumer refuses tolerance parameters by construction and would be unable to
use one (R2 §6).

---

## 4. Determinism posture

Full argument in **ADR-012**. The requirement is bit-for-bit reproducibility of output for
identical input, *including under parallelism*, and reproducibility of the *path* taken,
so that a Las Vegas failure is debuggable.

### 4.1 The four sources of nondeterminism, each closed

**(a) Ambient randomness — closed by banning it.** `rand` is not a dependency of any
published crate. `SystemTime`, `std::process::id`, address-derived values, and
`HashMap`'s default `RandomState` are denied by lint. There is exactly one RNG type in the
workspace.

**(b) Randomized algorithms — closed by seeding and by counter-based substreams.**
resolvent's RNG is **counter-based** (Philox/ChaCha-style: `output = F(key, counter)`),
not sequential. A `Session` carries a `Seed`; a worker at index `k` uses
`rng.substream(k)`, so the value drawn at a given logical position depends on the *index*
and not on scheduling, thread count, or chunk size. This is the mechanism that makes
determinism survive `rayon`. The default seed is a **fixed checked-in constant**, not
entropy, so the default path is reproducible without the caller doing anything.

**(c) Prime and evaluation-point selection — closed by index-addressing.** Primes are
never "random". `resolvent-modular` owns a deterministic, checked-in generator of word
primes; `prime(i)` is a pure function of `i`. A modular run consumes primes in index
order; a prime rejected as bad (degree drop, unlucky specialization) is recorded **by
index with its rejection reason**, so a replay follows an identical path. Evaluation
points for Brown/Zippel recursion and for modular bivariate subresultants come from the
seeded counter RNG at index-derived positions and are recorded the same way.

**(d) Hash iteration order — closed by never letting it reach an output.** Interning uses
a fixed-seed hasher (`rustc-hash` is seedless by construction). `MonomialId`s are assigned
in first-encounter order under a deterministic traversal, which means the ids themselves
are reproducible — this matters because tie-breaks that consult id order would otherwise
smuggle hash order into the result. Any table that is iterated to produce output is sorted
by a declared total order first. No `HashMap` iteration order is observable in any return
value or in any decision.

### 4.2 Parallelism

Determinism under parallelism is a *combining-order* property, not a locking property.

- Results are combined in **index order**, never completion order. The permitted shape is
  `par_iter().map(..).collect::<Vec<_>>()` and reductions over an ordered `Vec`; shared
  mutable accumulators updated from `for_each` are banned.
- Work-splitting granularity (chunk size, batch size, thread count) may change timing and
  must not change values. CI asserts this: the same corpus is run at
  `RAYON_NUM_THREADS ∈ {1, 2, 8}` and the serialized outputs must be byte-identical.
- No floating point appears in any decision path. The only f64 in the library is the
  outward-correct enclosure (§5.2), computed by a fixed operation sequence with no FMA
  contraction and no reassociation.

### 4.3 Traces: recorded and replayable

```rust
pub struct Trace { seed: Seed, tuning: Tuning, events: Vec<TraceEvent> }
pub enum TraceEvent {
    PrimeAccepted { index: u32 }, PrimeRejected { index: u32, reason: BadPrime },
    EvalPoint { index: u32, value: i64 }, Stabilized { rounds: u32 },
    TracerDecision { .. }, WidenRestart { from: u8, to: u8 }, /* … */
}
```

`op_with_trace(input) -> (Certified<T>, Trace)` and
`op_replay(input, &Trace) -> Certified<T>` are paired, and CI asserts the replay is
byte-identical. This is what makes every Las Vegas lane debuggable and every bug report
reducible to `(input, trace)`.

### 4.4 Tuning thresholds are inputs, not constants

Every crossover threshold — Karatsuba/Toom/NTT handoffs consumed from the bignum layer,
the fast-Taylor-shift crossover (~degree 512 per msolve), the Zassenhaus→van Hoeij `r`
threshold (~10), the F4 batch size, the modular batch width `N` (Groebner.jl ships 4) —
lives in a single `Tuning` struct with documented defaults.

Two consequences, both load-bearing:

1. **Same input + same `Tuning` ⇒ same output.** Different `Tuning` may change timing and
   must not change values. CI asserts value-equality across a small `Tuning` matrix, which
   doubles as a free implementation-agreement oracle: the naive path and the fast path are
   forced to agree on every corpus instance.
2. **Every threshold is re-derived by measurement on resolvent's own corpus, and the
   measurement is checked in.** This is simultaneously a licensing rule and a correctness
   rule (R1 §6 Tier B): a threshold lifted from a GPL source tree is both a transcription
   hazard and *wrong for our machine*.

### 4.5 Canonical serialization, fixed now

R3's open question about cross-implementation certificates is settled before any oracle is
written, because a SHA-256 certificate only works if normalization is byte-identical.
The canonical form:

- Polynomials: content removed; leading coefficient positive; terms in **descending**
  order of the ring's declared monomial order; coefficients as decimal integers with an
  explicit `-` and no `+`; exponent vectors as full-length comma-separated non-negative
  integers.
- Gröbner bases: elements each canonicalized as above, then the *list* sorted by leading
  monomial descending.
- Algebraic numbers: canonical form is the **minimal polynomial plus a root index**
  (0-based, ascending) — which requires factorization, which is exactly why `Hash` is not
  implemented on the un-canonicalized type (ADR-014).
- The certificate is `SHA-256` of that byte string, and the serializer is in
  `resolvent-base` so every crate and every oracle adapter shares one implementation.

---

## 5. Public API sketch

This is the surface a consumer touches. It is deliberately small; R2 §0.1 found the entire
algebraic surface of a shipping 17k-LOC geometry engine to be about twenty-five functions.

### 5.1 Numbers, and how a consumer gets its values in and out

```rust
// resolvent (facade) -> resolvent-int
pub struct Integer(/* private */);
pub struct Rational(/* private */);
```

`dashu` appears in no signature and is not re-exported (ADR-002). The **conversion surface
is over primitives and slices, never over a third-party bignum type**, so an adapter can
be written without naming resolvent's dependency:

```rust
impl From<i8|i16|i32|i64|i128|u8|..|u128|isize|usize> for Integer { .. }
impl TryFrom<&Integer> for i64 / i128 / u64 / u128 { .. }
impl FromStr for Integer / Rational { .. }              // decimal; 0x/0b prefixes
impl Integer {
    pub fn from_le_limbs64(sign: Sign, limbs: &[u64]) -> Integer;
    pub fn to_le_limbs64(&self) -> (Sign, Vec<u64>);     // and a borrowing variant
    pub fn from_signed_bytes_be(b: &[u8]) -> Integer;
    pub fn to_signed_bytes_be(&self) -> Vec<u8>;
}
impl Rational { pub fn new(num: Integer, den: Integer) -> Result<Rational>; // den != 0
                pub fn numer(&self) -> &Integer; pub fn denom(&self) -> &Integer; }
```

A consumer whose own bignum is also `dashu` converts through `to_le_limbs64` at a
memcpy-ish cost and never couples versions. A consumer with a different bignum uses the
same path. This is what keeps the deferred integration decision cheap (§6).

### 5.2 Polynomials

```rust
// resolvent-poly. Dense, low-to-high, trailing zeros trimmed. No monomial type, no order.
pub struct UPoly<C> { /* Vec<C> */ }

impl<C: CoeffRing> UPoly<C> {
    pub fn from_coeffs_low_to_high(c: Vec<C>) -> UPoly<C>;
    pub fn degree(&self) -> Option<usize>;          // None for the zero polynomial
    pub fn lc(&self) -> Option<&C>;
    pub fn eval_horner(&self, at: &C) -> C;
    pub fn derivative(&self) -> UPoly<C>;
    pub fn scale(&self, k: &C) -> UPoly<C>;
    pub fn add(&self, o: &Self) -> Self;  pub fn sub(&self, o: &Self) -> Self;
    pub fn mul(&self, o: &Self) -> Self;
    pub fn div_rem(&self, d: &Self) -> Result<(Self, Self)>;     // C: Field
    pub fn pseudo_div_rem(&self, d: &Self) -> Result<(Self, Self, u32)>; // C: CommutativeRing
    pub fn compose_affine(&self, a: &C, b: &C) -> Self;          // p(a·x + b)
    pub fn reverse(&self) -> Self;                  // x^n·p(1/x) — PUBLIC. See note.
    pub fn taylor_shift_1(&self) -> Self;           // p(x+1)
    pub fn scale_pow2(&self, k: i32) -> Self;       // p(2^k·x), dyadic
}
impl UPoly<Integer> {
    pub fn content(&self) -> Integer;
    pub fn primitive_part(&self) -> UPoly<Integer>;
    pub fn canonical_associate(&self) -> UPoly<Integer>;   // content-free, lc > 0
}
impl UPoly<Rational> { pub fn clear_denominators(&self) -> (UPoly<Integer>, Integer); }
```

`reverse` is public deliberately. It is private in the prior art
(`crates/lazy-exact/src/roots.rs:242-246`) and consumers need it: the Weierstrass-
rationalized families move the point at infinity to zero with exactly this transform
(R2 §4).

`canonical_associate` is what makes the associate test an `==`. The consumer currently
hand-rolls it as all 2×2 minors over six coefficients (`conics.rs:259-270`, O(k²)).

```rust
// Multivariate. The order, variable count, field width, and arena live in `Ring`.
pub struct Ring { /* vars, order, width, monomial arena, coefficient tag */ }
pub struct MonomialId(u32);
pub struct MPoly { /* Vec<(MonomialId, C)>, sorted descending; borrows &Ring by handle */ }

impl Ring {
    pub fn new(vars: &[&str], order: Order) -> Result<Ring>;
    pub fn var(&self, name: &str) -> Option<VarId>;
    pub fn order(&self) -> Order;                     // runtime data, not a type param
}
```

**Two polynomial types, one algebra, explicit conversions, no unification** (ADR-007).
`MPoly → UPoly` conversion exists (extract a univariate); `UPoly → MPoly` exists (embed);
nothing is implicit. The bivariate resultant bridges them:
`Res_y: MPoly × MPoly → UPoly<Integer>`.

### 5.3 `AlgebraicReal`

```rust
// resolvent-real. Arc-backed. Send + Sync. &self refinement. Monotone.
#[derive(Clone)]
pub struct AlgebraicReal(/* Arc<Inner> */);

impl AlgebraicReal {
    // ---- construction is where fallibility lives ----
    pub fn from_rational(q: Rational) -> AlgebraicReal;
    pub fn new(poly: UPoly<Integer>, lo: Rational, hi: Rational) -> Result<AlgebraicReal>;
    //   Err if poly is not squarefree, or (lo,hi) does not isolate exactly one root,
    //   or poly(lo) == 0 or poly(hi) == 0.  (R3 §8.2 F1, F4.)

    // ---- the query surface is total ----
    pub fn defining_poly(&self) -> &UPoly<Integer>;      // squarefree by invariant
    pub fn bounds(&self) -> (Rational, Rational);         // current enclosure, monotone
    pub fn enclosure_f64(&self) -> (f64, f64);            // outward-correct. Not an Interval.
    pub fn refine_to(&self, width: &Rational);            // &self, idempotent, monotone
    pub fn as_rational(&self) -> Option<Rational>;        // Some iff collapsed to a point
    pub fn sign_of(&self, h: &UPoly<Integer>) -> Sign;    // zero-ness settled by gcd FIRST
    pub fn is_root_of(&self, h: &UPoly<Integer>) -> bool;
    pub fn cmp_rational(&self, q: &Rational) -> Ordering;
    pub fn canonicalize(&self) -> Result<CanonicalAlgebraicReal>; // costs a factorization
}

impl PartialEq for AlgebraicReal {}  impl Eq for AlgebraicReal {}
impl PartialOrd for AlgebraicReal {} impl Ord  for AlgebraicReal {}   // total; see ADR-013
// NO impl Hash for AlgebraicReal.  CanonicalAlgebraicReal has one.  See ADR-014.

pub fn isolate_roots(p: &UPoly<Integer>) -> Result<Vec<(AlgebraicReal, u32)>>;
//   multiplicity is a PAIR ELEMENT, never a field of the number (ADR-014). √2 has no
//   multiplicity; the prior art conflates them at roots.rs:321 and reads it off a
//   resultant root at conics.rs:569.

pub fn rational_between(a: &AlgebraicReal, uppers: &[AlgebraicReal]) -> Rational;
//   certified strictly-between witness. The prior art hand-rolls it at conics.rs:357-380
//   and it is the mechanism by which cmp_y_right_of avoids derivative towers.
```

`Ord` is a real, total, infallible `Ord`, and the separation bound (R2's R5) is what makes
that honest: with a Mignotte–Davenport bound, comparison terminates in a computable number
of steps, so there is no failure to report. The prior art's own module header concedes it
has no such machinery — "comparisons terminate because distinct algebraic numbers are
eventually separated by bisection" (`roots.rs:8-11`) — which is true and unbounded.
Converting "terminates eventually" to "terminates in a computable number of steps" is what
buys the `Ord` impl.

### 5.4 Degree-2 radicals and radical towers — the fast paths that must not be subsumed

```rust
pub struct SqrtExt<T> { /* a + b·√r */ }
impl<T: Ordered + Field> SqrtExt<T> {
    pub fn sign(&self) -> Sign;                          // by squaring, exact
    pub fn cmp_cross(&self, o: &SqrtExt<T>) -> Ordering; // total across different r
}

/// Exact sign of Σ cᵢ(α)·√hᵢ(α) at arbitrary depth. Generalizes the prior art's
/// depth-1 and depth-2 ladders (roots.rs:622-683); depth ≥ 3 exists nowhere today.
pub fn sign_radical_tower(
    coeffs: &[UPoly<Integer>], radicands: &[UPoly<Integer>], at: &AlgebraicReal,
) -> Sign;
```

`SqrtExt` is first-class and is **not** routed through defining-poly + interval machinery
(ADR-014 §4, where this is stated as an explicit decision rather than left implicit). The
evidence is direct: `crates/arrangements/src/geoms/circle_segments.rs` is 931 LOC that uses
`SqrtExt` exclusively and never imports `RealRoot` or `QPoly` (R2 §4), and
`SqrtExt::cmp_cross` has 31 call sites. Subsuming it would be a large silent regression on
the cheapest and most common case.

`sign_radical_tower` is the documented **fast path** for y-coordinates; materializing an
algebraic number over ℚ is the general fallback, costs a resultant of degree
`deg(ξ)·deg(h)` per point per predicate, and the docs say which is which so consumers do
not silently pick the slow one (R2 §7 D3).

### 5.5 Certified enclosure, and curve analysis

```rust
pub fn sign_over(p: &UPoly<Rational>, lo: &Rational, hi: &Rational) -> Verdict<Sign>;
//   Bernstein coefficients + de Casteljau. Returns Unknown, never a guess.

pub struct CurveAnalysis { /* … */ }
impl CurveAnalysis {
    pub fn of(f: &MPoly, x: VarId, y: VarId) -> Result<CurveAnalysis>;
    pub fn critical_abscissas(&self) -> &[AlgebraicReal];
    pub fn branch_count_over(&self, interval_index: usize) -> u32;
    pub fn branch_map_across(&self, critical_index: usize) -> &BranchMap;
}
```

Curve analysis ships in resolvent, and its signature mentions **no geometric type**:
inputs are `MPoly`, outputs are algebraic numbers, counts, and index maps. That is what
keeps constraint #1's "thin adapter the consumer writes" true while still not making every
consumer rebuild the hardest component (R2 §7 D7).

Its construction is the expensive part, and it is an explicit **handle** for exactly that
reason: the prior art recomputes the resultant *and* re-isolates its roots on every
`cmp_y_right_of` and every `intersect` call with no cache
(`conics.rs:459-460`, `conics.rs:565-567`).

### 5.6 How a consumer supplies its own number type

Two directions, both supported, with an honest statement of what each costs.

**(a) Consumer brings its own coefficient type.** Implement
`resolvent_base::{Ring, CommutativeRing, …}` for it and use `UPoly<TheirType>`. This
requires depending on `resolvent-base` only — no bignum, no algorithms. What it gets:
every generic (Tier G) algorithm. What it does not get: the modular pipeline, which is
bounded by `Reducible + Liftable` (§2.3). A type that cannot be reduced mod p gets the
reference implementation, and the reference implementation is not fast. **This is stated
in the trait's own doc comment**, not buried in a guide.

**(b) Consumer adapts to resolvent's types.** Convert at the boundary through §5.1's
primitive/slice surface. This is the expected path and the one the deferred integration
decision assumes (§6).

**Not supported: resolvent becoming generic over a consumer-shaped scalar seam.** No trait
in resolvent's public API mirrors `lazy-exact`'s `RingOps`/`ExactRing`/`ExactField`
(`crates/lazy-exact/src/exact/mod.rs:16-29`) — which is explicitly an *ops surface* and
"not an algebraic claim", since `Interval` implements it too. resolvent's traits are
algebraic claims. Two similarly-named traits with different contracts across an adapter
boundary is a bug generator. Adding such a seam later is additive; removing one is
breaking, so the door stays closed and openable (§6).

---

## 6. Deferred: consumer integration

**Status: explicitly deferred. Not deferred by omission — deferred by a decision recorded
in ADR-018, with a named list of things not to do so that all three options stay open.**

### 6.1 The question

`/home/dev/projects/arrangements` ships 3,602 LOC of `lazy-exact` whose `roots.rs` (927
LOC) is resolvent's L1+L2+L3 in miniature, built over dense ℚ with monic-normalized Euclid
gcd and no separation bounds. resolvent will do the same work over ℤ with modular methods.
The two will coexist. Whether they should eventually be one thing is the deferred
decision.

### 6.2 The three options, none currently foreclosed

| | Option | Shape | Cost if chosen | Cost if it turns out wrong |
|---|---|---|---|---|
| **A** | resolvent adopts a scalar-seam trait | resolvent's polynomial and algebraic-number types become generic over a consumer-supplied scalar | A public generic parameter on the headline types; monomorphization across an *open* instantiation set (§2.4); the modular fast path becomes conditional on a consumer trait impl | Very high. A public generic parameter cannot be removed without a major version and a rewrite of every consumer. |
| **B** | arrangements writes an adapter | a small `arrangements`-side crate maps `resolvent::{Integer, Rational, UPoly, AlgebraicReal, SqrtExt}` onto `lazy-exact`'s vocabulary | conversion at the boundary; two enclosure implementations; two `Sign` types trivially mapped | Low. Delete the adapter. |
| **C** | eventual merge | `lazy-exact`'s `roots.rs` and `sqrt_ext.rs` are deleted; `arrangements` depends on `resolvent` directly | one number vocabulary; but resolvent inherits geometry's latency requirements and `lazy-exact`'s `Real`/`Interval` filtering layer stays behind, so the seam moves rather than disappearing | Medium. Reverting means reinstating deleted code. |

**B is the default and the only one being built toward.** A and C are kept reachable.

### 6.3 What would settle it

Each of these is a measurement or an event, not an opinion.

1. **The degree/coefficient profile of the real workload** (R2 §8 Q1, R3 open questions).
   Generate degree 3–8 curve pairs with realistic coefficient bit-size; record `Res_y`
   degree and coefficient length, and the wall time of `isolate_roots` + a `sign_of` sweep,
   against the existing `QPoly`. If resolvent's ℤ + modular pipeline wins by a large factor
   on that corpus, C becomes attractive. If the crossover sits above the workload,
   B is correct indefinitely and A never pays for itself.
2. **Whether resolvent's `SqrtExt` matches `sqrt_ext.rs`'s cross-root comparison on the
   `circle_segments.rs` path.** If it does not, C is off the table — 931 LOC of the shipping
   consumer never touches an algebraic number and must not start.
3. **Whether a second consumer with a different number type materializes** (#12 SMT, #27
   medial axis, #34 FEM). Two consumers with different scalars argue for A or B and against
   C, because C is a merge with *one* consumer.
4. **Whether the f64 enclosure semantics can be made to agree exactly** with
   `lazy-exact`'s outward-widening `Interval` (431 LOC, no global rounding-mode state).
   Two enclosure semantics that silently disagree at an adapter boundary is the specific
   failure mode ADR-015 exists to prevent.
5. **Whether `AlgebraicReal`'s `Arc` + `&self`-refinement model actually removes the
   `Rc<RefCell<_>>` tax.** The consumer has four independent copies of
   `type SharedRoot = Rc<RefCell<RealRoot>>` plus four copies of the `Rc::ptr_eq`
   self-deadlock guard (`conics.rs:32-46`, `sine_waves.rs:31-42`,
   `sine_radical.rs:71-84`, `spherical_circle.rs:70-88`). If the adapter still needs a
   wrapper, the model is wrong and should be revisited before, not after, C.

### 6.4 What to avoid doing now, so all three stay open

- **Do not put a scalar-seam trait in resolvent's public API.** Adding one later is
  additive; removing one is breaking. (Closes A cheaply either way.)
- **Do not add a generic parameter to `AlgebraicReal`.** It is `AlgebraicReal`, not
  `AlgebraicReal<S>`. If A is ever chosen, the generic type is a *new* type and the
  monomorphic one stays.
- **Do not expose a float interval type** (ADR-015). One of the two enclosure semantics
  must be the adapter's, and it must be the consumer's.
- **Do not name `arrangements` or `lazy-exact` anywhere** — not in a feature flag, not in a
  doc example, not in a comment (gate L5). A `lazy-exact` feature would be option B
  smuggled into resolvent, which is the one place it must not live.
- **Do not copy `lazy-exact`'s trait names** (`RingOps`, `ExactRing`, `ExactField`,
  `Scalar`, `Uncertain`). Same names with different contracts across an adapter boundary
  is a bug generator; deliberately different names force the adapter to be explicit.
- **Do not subsume `SqrtExt` into `AlgebraicReal`.** Keeping it first-class is what keeps
  C from being a regression.
- **Do not accept a tolerance parameter anywhere**, at any layer, under any name. The
  consumer's exact families declare `type Error = Infallible` and its design permanently
  excludes snap rounding; a tolerance argument would make resolvent unusable by the
  consumer it was designed for.
- **Do write the `resolvent-oracles` differential harness so that `lazy-exact` can be
  *added* as an oracle** (subprocess or a `publish = false` dev-only path). That is how
  option C's evidence gets collected without option C's coupling.

---

## 7. Open questions this architecture does not close

Stated as "what would settle it", per house style. None of these blocks ratification;
each blocks a specific lane.

1. **`dashu` 0.5.2 large-operand performance post-NTT.** Every published figure is from
   0.4.2, one release before NTT landed (R1 §1.3). *Settle:* pin 0.5.2 in
   `tczajka/bigint-benchmark-rs`, run locally, commit the numbers. Half a day, and it must
   happen **before `resolvent-int` is written**, because a negative result strengthens the
   case for an optional non-default GMP backend, which is cheap to design now and
   expensive later (ADR-002).
2. **`dashu`'s Lehmer GCD vs GMP's half-GCD at 64 / 256 / 1k / 4k / 16k bits.** The one
   identified structural pure-Rust deficit. *Settle:* microbenchmark against the `rug`
   dev-oracle. It sets how aggressive the ℤ-primitive discipline has to be (ADR-004).
3. **Barrett/Shoup vs Montgomery for word-size GF(p).** R1 argued Barrett because the same
   `p` is reused against many operands, but that is an architecture argument, not a
   measurement, and the answer may differ between the scalar path and the F4 bulk-row path.
   *Settle:* first benchmark of the `resolvent-modular` lane (ADR-003).
4. **Does the interned-monomial design defeat its own comparison key?** Comparing by id
   requires an arena load, and that cache miss may dominate a `u64` compare — which would
   deflate the packing benefit further. *Settle:* microbenchmark inline packed monomials in
   terms versus ids-plus-arena-lookup on a realistic S-pair queue workload (ADR-008).
5. **The real cost of cofactor tracking through F4's linear algebra.** The certified
   Gröbner mode depends on it and no cited source measures it. *Settle:* prototype cofactor
   tracking on Katsura-8 / Cyclic-7 and measure the memory and time multiplier **before**
   committing `groebner_certified` to the plan (ADR-010).
6. **The exact conditions of the Idrees–Pfister–Steidel non-homogeneous verification
   theorem, given Noro–Yokoyama's correction.** Load-bearing for whether the fast Gröbner
   path can ever return `Proved` without cofactors. *Settle:* obtain Noro & Yokoyama,
   ICMS 2014 and Math. Comp. Sci. 11(3) 2017 (both paywalled at the time of research).
7. **Is `lll-rs` (MIT) usable at van Hoeij precision?** The lattice has dimension ~`r` with
   entries of size `p^k` exceeding twice the Landau–Mignotte bound — potentially thousands
   of bits. *Settle:* run it on a Swinnerton-Dyer degree-64 lattice. If it fails, LLL
   becomes its own lane and van Hoeij's schedule doubles.
8. **Does any permissively licensed F4 implementation exist in any language?** The Julia
   search found only GPL (Groebner.jl is GPL-2.0), and `feanor-math`'s is "F4-style
   Buchberger" rather than true F4. If none exists, the F4 lane has no Tier-A reference and
   must be built from Faugère's paper plus the Macaulay-matrix literature — feasible but
   slower, and worth knowing before the lane is sized.
9. **Does ANewDsc's Newton acceleration interact safely with the `AlgebraicReal`
   refinement API?** ANewDsc replaces bisection with Newton steps that *jump*, so the
   isolating interval does not shrink monotonically by halving. The invariants in
   R3 §8.2 (endpoints never roots; refine collapses on an exact hit) were derived for
   bisection. *Settle:* re-derive them for the Newton path before implementing, not after.
10. **Should intersection multiplicity come from resultant-root multiplicity or from
    factoring the resultant?** And does the consumer's `CrossingKind::Unknown` escape hatch
    survive at arbitrary degree? If `Unknown` becomes the common case, factorization moves
    from Tier B to must-have. *Settle:* the corpus in (§6.3.1), counting the `Unknown` rate
    and constructing the documented ambiguous case (two common points over one abscissa
    with parallel gradients).

---

## 8. Index of decisions

| ADR | Decision | Reversibility |
|---|---|---|
| [ADR-001](../docs/decisions/ADR-001-license-posture.md) | MIT OR Apache-2.0; Tier A/B/C reading discipline; mechanical `cargo-deny` gate | one-way |
| [ADR-002](../docs/decisions/ADR-002-bignum-backend.md) | `dashu` behind the `resolvent-int` newtype wall; no re-export | costly |
| [ADR-003](../docs/decisions/ADR-003-modular-arithmetic-in-house.md) | Hand-roll `resolvent-modular`; reject `ark-ff`, `crypto-bigint`, `num-modular` | cheap |
| [ADR-004](../docs/decisions/ADR-004-z-primitive-coefficients.md) | ℤ-primitive, not ℚ-primitive; ℚ is a boundary façade | one-way |
| [ADR-005](../docs/decisions/ADR-005-workspace-crate-split.md) | Seven published crates + three unpublished, lockstep versioned | costly |
| [ADR-006](../docs/decisions/ADR-006-generics-boundary.md) | Generics cross crate boundaries, never inner loops; closed instantiation set; `LANES` kept open | one-way |
| [ADR-007](../docs/decisions/ADR-007-polynomial-representations.md) | Three representations; `UPoly<C>` defined first and standalone | one-way |
| [ADR-008](../docs/decisions/ADR-008-monomial-representation-and-overflow.md) | Interned arena + packed key + divmask; guard-bit overflow detection; widen-and-restart | one-way (interning), cheap (field width) |
| [ADR-009](../docs/decisions/ADR-009-monomial-order-runtime.md) | Order is runtime ring data normalized into the key at intern time | one-way |
| [ADR-010](../docs/decisions/ADR-010-modular-methods-and-certificates.md) | Modular everywhere; `Certified<T>` with `Proved`/`Probable`; two Gröbner modes | one-way |
| [ADR-011](../docs/decisions/ADR-011-error-model.md) | Fail at construction, not at query; no panics; structured `Unsupported`; step budgets | one-way |
| [ADR-012](../docs/decisions/ADR-012-determinism.md) | Counter-based seeded RNG; index-addressed primes; ordered combination; replayable traces | one-way |
| [ADR-013](../docs/decisions/ADR-013-algebraic-real-mutability.md) | `Arc<Inner>`, `&self` monotone refinement, `Send + Sync`, total `Ord` via separation bound | one-way |
| [ADR-014](../docs/decisions/ADR-014-algebraic-real-no-hash-no-arithmetic.md) | No `Hash`, no general arithmetic; `canonicalize()` opt-in; multiplicity is not a field; **`SqrtExt` stays first-class** | one-way |
| [ADR-015](../docs/decisions/ADR-015-no-float-interval-type.md) | No float interval in the public API; rational bounds + outward `(f64, f64)` | cheap |
| [ADR-016](../docs/decisions/ADR-016-oracles-are-subprocesses.md) | Subprocess-only oracles; two-category workspace; no exception process | cheap |
| [ADR-017](../docs/decisions/ADR-017-layer-4-egraph-seam.md) | Resolvent-owned L4 seam; no `egg`/`egglog` dependency now | cheap |
| [ADR-018](../docs/decisions/ADR-018-deferred-consumer-integration.md) | Defer the arrangements question; adapter-by-consumer is the default; keep A and C open | cheap (by design) |
