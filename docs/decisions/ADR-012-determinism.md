# ADR-012 — Determinism: seeded, index-addressed, recorded, replayable

**Status:** Ratified 2026-07-31
**Reversibility:** one-way (retrofitting determinism onto a library with ambient randomness
is a rewrite of every randomized path)
**Amended:** 2026-07-31 — §4 replaces first-encounter id assignment with content-derived
ids; INV-M1 (no tie-break consults id order) is added; §9 excludes certificates and
telemetry from canonical bytes (critique-engineering §5, §6, §16).
**Gates lanes:** H2, H3, Z0, and every lane thereafter.
**Evidence:** `docs/research/algorithms-and-representation.md` §3.1, §3.5, §4.4, §9.2;
`docs/research/prior-art-and-licensing.md` §6 Tier B;
`docs/research/critique-engineering.md` §5, §6, §16.

---

## Context

resolvent is full of legitimately randomized machinery: modular methods pick primes,
Brown/Zippel picks evaluation points, Cantor–Zassenhaus picks splitting elements, Steel's
probabilistic linear algebra reduces random combinations of rows, F4's tracer makes
data-dependent decisions across primes. Every one of those is a place where an ambient
`thread_rng()` would make a bug irreproducible.

Two consequences make this worse than usual:

- **The failure mode is a Las Vegas hang or a rare wrong answer.** A bug that manifests once
  in 10⁴ prime choices, with no record of which primes were chosen, is not debuggable at
  all. Constraint #3 requires every lane to have an automatic verdict; a nondeterministic
  lane has no stable verdict.
- **Parallelism is the natural first win** — CRT over primes is embarrassingly parallel —
  and naive parallelism is exactly where determinism is lost, because results get combined
  in completion order.

There is also a licensing dimension. ADR-001 Tier B forbids transcribing tuning thresholds
from GPL sources, and requires every threshold to be re-derived by measurement on our own
corpus **with the measurement checked in**. That is only meaningful if changing a threshold
provably cannot change an *answer*.

---

## Decision

**Same input ⇒ same output, bit-for-bit, on any machine, at any thread count, in any build
profile. And the *path* taken is recorded and replayable.**

### 1. No ambient randomness

- `rand` is **not a dependency of any published crate**.
- `SystemTime`, `Instant` (in any decision path), `std::process::id`, address-derived
  values, and `std::collections::HashMap`'s default `RandomState` are denied by lint.
- There is exactly one RNG type in the workspace.

### 2. The RNG is counter-based and index-addressed

resolvent's RNG is **counter-based** (`output = F(key, counter)`, Philox/ChaCha-shaped),
not sequential. A `Session` carries a `Seed`; a worker at logical index `k` uses
`rng.substream(k)`.

This is the mechanism that makes determinism survive parallelism: **the value drawn at a
given logical position is a function of that position, not of scheduling, thread count, or
chunk size.** A sequential RNG cannot give this without a lock that serializes the whole
computation.

The default seed is a **fixed checked-in constant**, not entropy. A caller who does nothing
gets a reproducible run. A caller who wants independent runs supplies a seed explicitly.

### 3. Primes are a pure function of an index

`resolvent-modular` owns a deterministic generator: `prime(i)` is the `i`-th word prime
under a checked-in ordering. A modular run consumes primes in index order. A prime rejected
as bad (degree drop, unlucky specialization, disc divisibility) is recorded **by index with
its rejection reason**.

**Never "pick a random prime".** Evaluation points for Brown/Zippel recursion and for
modular bivariate subresultants come from the seeded counter RNG at index-derived positions
and are recorded identically.

### 4. Hash iteration order never reaches an output

- Interning uses a fixed-seed hasher (`rustc-hash` is seedless by construction).
- **`MonomialId` is a pure function of the monomial's packed key** — a deterministic content
  hash with a fixed collision-resolution order (ADR-008 §1). Ids are therefore reproducible
  *and* independent of encounter order, which is what makes parallel interning safe.

  *Amended 2026-07-31.* This bullet previously read "`MonomialId`s are assigned in
  first-encounter order under a deterministic traversal". That is reproducible only if
  interning is single-threaded, and an interner is a shared mutable accumulator, which §5
  bans — while symbolic preprocessing, the natural second parallel target, is nothing but
  interning. The failure is quiet: `terms.par_iter().map(|t| ring.intern(t)).collect()`
  matches §5's permitted shape (the *collection* is ordered) while assigning ids in
  thread-arrival order, and the thread matrix catches it only on instances whose tie-breaks
  happen to consult id order — data-dependent, so it passes for months and then fails once,
  on a schedule bug the minimizer cannot shrink.
- **INV-M1 — no tie-break anywhere may consult `MonomialId` ordering.** Tie-break on `key`,
  which is content-derived and totally ordered by construction. This holds belt-and-braces
  alongside content-derived ids: it is what makes the fallback (serialized or two-phase
  interning) still correct if content-derived ids are ever abandoned.
- Any table iterated to produce output is sorted by a declared total order first.
- No `HashMap` iteration order is observable in any return value or in any decision.

### 5. Parallelism is deterministic by combining order, not by locking

- Results are combined in **index order**, never completion order. The permitted shape is
  `par_iter().map(..).collect::<Vec<_>>()` plus reductions over the resulting ordered `Vec`.
  Shared mutable accumulators updated from `for_each` are **banned**.
- Work-splitting granularity may change timing and must not change values. **CI asserts
  this**: the corpus runs at `RAYON_NUM_THREADS ∈ {1, 2, 8}` and the serialized outputs must
  be byte-identical.
- `rayon` is behind a default-off `parallel` feature and appears in no public signature
  (`plans/architecture.md` §1.3, L7).

### 6. No floating point in any decision path

The only `f64` in the library is the outward-correct enclosure returned by
`AlgebraicReal::enclosure_f64` and the dyadic-approximation *filter* inside sign-variation
counting — and the filter is a filter: when it declines, the exact path runs, and the
verdict never depends on which happened. Enclosures are computed by a fixed operation
sequence with no FMA contraction and no reassociation.

### 7. Traces are recorded and replayable

```rust
pub struct Trace { seed: Seed, tuning: Tuning, events: Vec<TraceEvent> }
pub enum TraceEvent {
    PrimeAccepted { index: u32 },
    PrimeRejected { index: u32, reason: BadPrime },
    EvalPoint     { index: u32, value: i64 },
    Stabilized    { rounds: u32 },
    TracerDecision{ matrix: u32, kept: u32, dropped: u32 },
    WidenRestart  { from: u8, to: u8 },
    BatchSplit    { batch: u32, faulting_lane: u8, prime_index: u32 },  // ADR-010 §7
    BudgetTick    { site: &'static str, consumed: u64 },   // TELEMETRY: not replay-compared
}
```

`op_with_trace(input) -> (Certified<T>, Trace)` is paired with
`op_replay(input, &Trace) -> Certified<T>`, and CI asserts the replay is byte-identical.
A bug report is `(input, trace)` and nothing else.

### 8. Tuning thresholds are inputs, not scattered constants

One `Tuning` struct with documented defaults holds every crossover: the fast-Taylor-shift
crossover, the Zassenhaus→van Hoeij `r` threshold, the F4 batch size, the modular batch
width `N`, the delayed-reduction cutoff, the packed/unpacked variable-count switch, the
Barrett/Montgomery selection.

Two rules, both load-bearing:

1. **Same input + same `Tuning` ⇒ same output. Different `Tuning` ⇒ same *values*,
   different timing.** CI asserts value-equality across a `Tuning` matrix, which doubles as
   a free implementation-agreement oracle: the naive path and the fast path are forced to
   agree on every corpus instance.
2. **Every threshold is re-derived by measurement on resolvent's own corpus and the
   measurement is checked in** (ADR-001 Tier B). A threshold lifted from a GPL source tree
   is both a transcription hazard and wrong for our machine.

### 9. Canonical serialization, fixed before any oracle is written

A SHA-256 certificate only works if normalization is byte-identical across implementations.
Fixed now:

- **Polynomials**: content removed; leading coefficient positive; terms **descending** in
  the ring's declared order; coefficients as decimal integers with explicit `-` and no `+`;
  exponent vectors as full-length comma-separated non-negative integers.
- **Gröbner bases**: each element canonicalized as above, then the list sorted by leading
  monomial descending.
- **Algebraic numbers**: minimal polynomial plus a 0-based ascending root index — which
  requires factorization, which is exactly why `Hash` is not implemented on the
  un-canonicalized type (ADR-014).
- The certificate is SHA-256 of that byte string. The serializer lives in `resolvent-base`
  so every crate and every oracle adapter shares one implementation.

**What is *not* in canonical bytes** — *added 2026-07-31, and this closes a gate that would
otherwise have failed on its first run.* **Only the mathematical value is serialized.**
Certificates, `Certainty`, `ProbableReason` and `Telemetry` are excluded. §8's rule 1
asserts value-equality across a `Tuning` matrix; the modular batch width `N` is a tuning
knob and it changes `primes_used`, so if evidence were inside the bytes the tuning-matrix
gate could never be green. The same exclusion covers `TraceEvent::BudgetTick`, whose
`consumed` count is a function of how much cached refinement progress existed
(ADR-011 §4, INV-AR1) and is therefore not a pure function of the input.

Consequence for the replay gate: `op_replay` asserts byte-identity of the **value** and of
the *decision* events (`PrimeAccepted`, `PrimeRejected`, `EvalPoint`, `Stabilized`,
`TracerDecision`, `WidenRestart`, `BatchSplit`), **not** of `BudgetTick`. That is stated in
the type: `Trace::decisions()` is what replay compares, and `Trace::telemetry()` is what it
does not.

---

## Consequences

- **Every Las Vegas lane becomes debuggable and gradeable.** This is what makes constraint
  #3 achievable for the randomized half of the library.
- **Benchmarks become comparable across machines** for *values*, which lets the number lanes
  regress against a checked-in baseline rather than against a machine.
- **The counter-based RNG costs a little throughput** versus a sequential one (a block
  cipher round instead of a multiply-xorshift). Irrelevant: the RNG is called once per
  prime and once per evaluation point, not per coefficient.
- **`rayon`'s `for_each` idiom is unavailable**, and some parallel algorithms are more
  awkward as ordered map-reduce. Accepted.
- **The `Trace` has a memory cost on long runs** (Cyclic-10 would record >2000 prime
  events). It is opt-in (`op_with_trace`) and bounded by an event cap that, when hit,
  records a `TraceTruncated` marker rather than silently dropping events.
- **Determinism must be *tested*, not assumed.** Three CI gates: thread-count matrix,
  tuning matrix, replay equality. Without them this ADR is a comment.

---

## Alternatives considered and why rejected

**Ambient `thread_rng()` with a "set the seed in tests" escape hatch.** Rejected. The bugs
that matter appear in long randomized runs, not in tests, and by the time one is observed
the seed is gone. It also makes the `Tuning`-matrix oracle impossible.

**A sequential seeded RNG (e.g. `StdRng` with a seed) threaded through.** Rejected for
parallelism: draw order then depends on chunking and scheduling, so a run at 8 threads
differs from a run at 1. Counter-based substreams are the standard fix and cost nothing
here.

**Random prime selection from a large pool, "for independence".** Rejected. Independence
across runs is not a property resolvent wants; reproducibility is. Where genuine
independence is needed (a fuzz campaign exploring the prime space), it is obtained by
varying the *seed* explicitly and recording it.

**Determinism only in the default single-threaded path, with parallel runs documented as
nondeterministic.** Rejected. It makes the parallel path untestable against the serial one,
which removes the cheapest available oracle for the parallel implementation.

**Wall-clock-based adaptive strategies** ("switch algorithms if this is taking too long").
Rejected entirely — nondeterministic by construction. Adaptive switching is permitted only
on *step counts*, which are deterministic.

**Letting `HashMap` iteration order affect only "internal" choices.** Rejected. There is no
such thing as an internal choice in an algorithm whose output depends on the search order
— and F4's pair selection, symbolic preprocessing, and tracer all do.

---

## What would reverse this

Nothing reverses the requirement. Two things could change the *mechanism*:

- **A measured need for nondeterministic work-stealing at a granularity where index-order
  combination is too coarse.** Response: keep the combination ordered but make the *unit* of
  work smaller; do not make the combination unordered.
- **Trace memory becoming prohibitive on the largest instances.** Response: a sampled or
  hierarchical trace (record prime indices and rejections always, matrix-level tracer
  decisions only under a flag). The replay guarantee is then conditional on the flag, and
  that must be stated in the type — e.g. `Trace::Full` vs `Trace::Sampled`, with `op_replay`
  accepting only `Full`.
