# S1 — The general-purpose API shape

**Status:** synthesis deliverable of the consumer-analysis track. Binding on the founding
architecture unless explicitly overturned.
**Inputs:** `docs/research/consumer-sinbad.md` (E1), `docs/research/consumer-cadabra2.md`
(E2), `docs/research/consumer-solverang.md` (E3), plus R1 (`prior-art-and-licensing.md`)
and R3 (`algorithms-and-representation.md`) where they force a decision this track must
respect.
**Scope:** what resolvent's public surface *is*, what is core, what is adapter, what is out
of scope, and the proof that each of the three consumers can be adapted in under 200 lines
without resolvent changing.

Every claim about an existing repository is cited to the evaluation that measured it, which
in turn cites file and line. Nothing here invents a benchmark.

---

## 0. The five decisions this document makes

1. **Embedding is by owned values, not by a session.** No global state, no ambient context,
   no thread-local caches, no interpreter. The L4 hash-cons arena is a **caller-owned
   `Store` value**; node ids are store-relative and never escape (§1).
2. **The numeric seam is split in two.** Coefficient rings are resolvent-owned concrete
   types reached by `TryFrom`/`From` at the boundary; *evaluation scalars* are an open,
   deliberately minimal `Scalar` trait a consumer may implement for its own type. Never a
   consumer-implemented coefficient ring (§3).
3. **Certificates are tiered by cost, never by a boolean flag.** Free evidence always ships;
   cheap verification is on by default with an explicitly-named unchecked escape; expensive
   tracing is a separate entry point (§4).
4. **L4 is a term-algebra container with a caller-owned function table, and it never
   rewrites anything implicitly.** That is what lets one mechanism serve sinbad's
   transcendental differentiation and cadabra2's deliberately-un-rewritten `Cos2` atom
   (§2, §6.4).
5. **All three adapters pass the 200-line test**, with one honest exception that is not
   resolvent's to fix: solverang's per-constraint transcription (~300 lines) scales with
   solverang's constraint vocabulary and is invariant to resolvent's API shape (§6).

---

## 1. The embedding model

### 1.1 What "directly embedded" means, operationally

A consumer holds resolvent values in its own structs, calls resolvent synchronously on its
own thread, and links it as a library. That is the only supported mode. It forbids, as hard
rules enforced in CI (§7, INV-1..INV-4):

| Forbidden | Why, with the consumer that would break |
|---|---|
| `static mut`, `static` with interior mutability, `thread_local!` anywhere in a `publish = true` crate | sinbad D1 requires bit-identical output across thread counts (E1 §3, `sinbad-pal/src/repro.rs:20-21`). Any thread-local cache makes output a function of scheduling. |
| A `Ctx`/session/capability handle as first parameter | sinbad's own conventions *want* this (`SINBAD-API-CONVENTIONS.md:86-90`) and resolvent must still refuse: a session object owning every value is the thing that makes a library un-embeddable. The adapter accepts `&Ctx` and drops it — ~5 lines (E1 §5.7). |
| Any I/O: `std::fs`, `std::net`, `std::env`, `std::time`, `std::process` | A CAS that reads the clock cannot be deterministic; a CAS that reads the environment cannot be content-addressed (sinbad D6, artifacts BLAKE3-keyed in `rutter/src/lib.rs:11-14`). |
| Unseeded randomness — `thread_rng()`, OS entropy | E1 D2 and E3 R2 independently demand it. Prime sequences are deterministic; evaluation points take a `u64` seed **as a parameter**. |
| `HashMap` iteration in any position that affects an output value | Same source. Ordering-visible positions use `BTreeMap`/sorted `Vec`. `rustc-hash` maps are permitted *only* where iteration order cannot reach a result (the L4 hash-cons lookup table). |
| A custom global allocator, or an allocator parameter | Embedding must not fight the host's allocator choice. |
| A lifetime parameter on any public owned type | A consumer must be able to put a resolvent value in its own struct without infecting that struct with a lifetime. Borrowed *views* (`RecursiveView<'a>`, iterators) are the only exception and are never returned by value from a constructor. |
| Panics on any input-dependent path | E1 D3 (rule E1) and E2 §6.8 both make this non-negotiable, and E2 shows the cost of violating it: `lazy_exact::SqrtExt::new` panics on a negative radicand and cadabra2 had to hand-write a guard (`cadabra-core/src/exact/radical.rs:77-88`). |

### 1.2 The arena fork — decided: caller-owned, one `Store` per consumer scope

Three candidates, and this is a real fork with different thread-safety and determinism
consequences.

| Option | Verdict | Consequence |
|---|---|---|
| **Global/thread-local interner** | **Rejected.** | Breaks D1 (output depends on interning history and therefore on thread count), breaks D6 (canonical bytes would depend on process-global insertion order), and makes two independent consumers in one process share a table they cannot reason about. This is the standard CAS mistake and it is why `Ctx`-free embedding fails in most symbolic libraries. |
| **Per-call arena** | **Rejected.** | Destroys the only thing hash-consing buys. plexus differentiates the same equation set repeatedly across Pantelides rounds (E1 §1.2); cadabra2's certificate tether compares a claim built at mint time against one rebuilt later (E2 §11.2). Both need node identity stable across calls. |
| **Caller-owned `Store` value** | **Adopted.** | `Store` is a plain owned struct. `Store: Send`. Node ids (`Expr`) are `Copy`, store-relative, and never serialized — canonical bytes are computed structurally (§2, L4-6). Two stores built by the same call sequence produce identical canonical bytes and possibly different node ids; that is fine because ids never escape. |

**Consequence that must be stated, not hidden:** an `Expr` used against the wrong `Store` is
a caller logic error. Resolvent makes it *safe and deterministic* — every entry point
bounds-checks the id and returns `Error::Domain { fault: ForeignNode }` when it is out of
range — but an in-range id from a different store yields a wrong answer, not an error. The
alternative (a store tag) requires either an ambient counter, which §1.1 forbids, or a
caller-supplied tag, which taxes every consumer for a bug none of the three would make.
Cost recorded, decision taken.

**The `Store` also owns the symbol interner and the function table** (§2, L4-2). One owned
value, not three, and no ambient anything.

### 1.3 The one place interior mutability is admitted, and why it is not a violation

`AlgebraicReal` caches its refinement state. R3 §8.2 F6 records this as an unavoidable fork;
E2 §4.1 records what the wrong choice costs — `lazy-exact`'s `&mut self` comparison forced
cadabra2 into `Rc<RefCell<RealRoot>>` plus an `Rc::ptr_eq` self-comparison guard in its hot
path (`cadabra-arrange/src/trim.rs:857-862`).

**Decision: interior mutability, `Send + !Sync`, with the self-comparison guard inside
resolvent.**

```rust
pub struct AlgebraicReal {
    poly:  Arc<SqfrPoly<Rational>>,   // immutable, shared
    cache: RefCell<Isolation>,        // monotonically narrowing interval
    mult:  u32,
}
```

- Comparison takes `&self`; `impl Ord` exists; the type is `BTreeMap`-able and sortable.
- The re-borrow hazard is eliminated *inside* resolvent by `if std::ptr::eq(self, other) {
  return Ordering::Equal }` before any borrow. Because the cache is inline (not `Rc`-shared),
  distinct values have distinct addresses, so this guard is exactly correct and the consumer
  never writes one.
- `!Sync` is the price. It costs nothing for determinism, because **the refinement cache can
  never change a verdict** — only how much work the verdict took. That is pinned by property
  tests 5 and 8 of R3 §8.3 (sort stability under shuffling; idempotence under pre-refinement).
- `Clone` is cheap (one `Arc` bump plus two rationals), so a consumer that wants a value on
  another thread clones it. Documented; no `Sync` shim, no lock, no atomic per comparison.

`Store`, `UPoly`, `MPoly`, `Rational`, `Fp`, `SqrtExt`, `NumberField`, `Certificate` are all
`Send + Sync`. `AlgebraicReal` is the single documented exception.

### 1.4 Crate layout

```
resolvent            facade; re-exports, no logic
├─ resolvent-seam    ZERO dependencies. Sign, Scalar, ScalarOrd, TryDiv, Hom, Budget,
│                    Error, Certainty. Publishable and dependable on its own (§5).
├─ resolvent-int     Integer, Rational — newtype wall over dashu (R1 §1.4). dashu appears
│                    in no public signature and is never re-exported.
├─ resolvent-modular Fp (runtime modulus), FpElem, prime sequences, seeded points
├─ resolvent-poly    UPoly<C>, MPoly<C>, Ring, monomial packing, Bernstein/de Casteljau
├─ resolvent-linalg  Matrix<S>, row_echelon (field), bareiss_det (domain)
├─ resolvent-engine  gcd, square_free, factor, resultant, isolate_roots, groebner (later)
├─ resolvent-alg     AlgebraicReal, SqrtExt, NumberField
├─ resolvent-expr    Store, Expr, FuncTable, diff_with, walk_topological, canonical_bytes
└─ resolvent-lazy    OPTIONAL, non-default: filtered eager-interval/lazy-exact Real<C>
```

`resolvent-seam` having zero dependencies is load-bearing for §5 and is the pattern cadabra2
already validated with its own `scalar-seam` crate (E2 §10.4).

---

## 2. Core vs adapter, capability by capability

The rule, mechanically: a capability is core iff **(a)** two or more independent consumers
need it, **or** **(b)** it is a general algebraic primitive any CAS would be expected to
have. One consumer plus no general-primitive argument ⇒ adapter or out of scope.

"Independent" matters: E1 §8 correctly excludes `sem1f-biquad-spike` from sinbad's demand set
because it imports `cadabra_geom` and belongs to the cadabra2 line. Counting it twice would
manufacture a two-consumer majority for algebraic extensions. It is counted once, under
cadabra2.

Legend for consumers: **S** = sinbad, **C** = cadabra2, **V** = solverang.

### 2.1 Layer 0 — coefficient rings and scalars

| # | Capability | Wanted by | Placement | Justification |
|---|---|---|---|---|
| L0-1 | `Integer`, `Rational` (ℤ, ℚ) with `sign()`, by-reference arithmetic | S, C, V | **core** | Three consumers; the floor of the library. By-reference arithmetic is E1 §1.4's explicit ask (`predicates.rs:33-70`). |
| L0-2 | `Rational::try_from_f64` — exact dyadic, typed failure on non-finite | S, C, V | **core** | Three consumers (E1 §5.5, E2 §10.8, E3 R8). **No "nice rational" heuristic ships, ever** — E3 §5 shows why: silently turning `sin(30°)`'s f64 into `1/2` analyses a different system. |
| L0-3 | `num_bits()`/`den_bits()` and explicit `round_to_f64_grid()` on ℚ | S, C | **core** | meshwright snaps circumcenters to 53 bits by hand because unbounded growth is a *measured* perf failure (E1 §0.5, `triangulate.rs:500-512`); cadabra2 needs the same knob under its three-exits discipline. Policy stays with the caller; resolvent exposes size and rounding and applies neither. |
| L0-4 | The three exits: `demote_exact` / `enclosure` / `approx_lossy` | C (explicit), S (D5 by another name) | **core** | E2 §10.2 calls this "the single most transplantable design decision in the consumer" and it is already clippy-enforced there. sinbad's D5 ("unaccounted error caps the grade") is the same discipline stated as a grading rule. Adopt the shape verbatim; the names are domain-neutral already. |
| L0-5 | `Interval<f64>` with directed rounding and **no global FPU mode** | C, plus resolvent's own root isolation | **core** | Blocked-now for cadabra2 (E2 #17). Independently, it is the codomain of `enclosure()` (L0-4) and the filter inside Descartes/VCA, so resolvent needs it regardless. Implemented by ulp-widening, never by changing the rounding mode — a library that mutates process FPU state is not embeddable. |
| L0-6 | `Fp` — prime field with a **runtime** modulus, `Copy` word-sized elements | V (explicit); C explicitly does **not** want it user-facing | **core, public** | Clause (b): modular methods are the spec's structural decision (`IDEAS-crates.md:126-127`) and every one of them needs `Fp` internally. Making it public costs zero marginal implementation. E3 §5 shows the ask is that it be *callable without a forced CRT/rational-reconstruction lift* — solverang never wants the ℚ answer. cadabra2 pays nothing: it simply never imports the module. |
| L0-7 | Seeded uniform random points over `GF(p)` | V (explicit), S (D2, as a prohibition on the alternative) | **core** | Schwartz–Zippel, sparse interpolation and modular GCD all need it internally. The seed is a **parameter**, never ambient. |
| L0-8 | `SqrtExt` — `a + b√r` with a total order **across distinct radicands** | C | **core** | One consumer. Clause (b) holds but weakly on its own; it earns core as the degree-2 specialization of L0-9 with identical semantics and an inner-loop budget. Recorded honestly: if L0-9 did not exist, this would be an adapter. |
| L0-9 | `NumberField` — ℚ(α) with a known minimal polynomial, arithmetic, degenerate-tower detection | C (three fail-closed sites) | **core** | One consumer, but named in the spec's L0 ("algebraic extensions"), and it is the thing that unblocks *number-field linear algebra* without resolvent writing any linear algebra (§2.3, L2-6). Detecting "is `q` a rational square / does the tower collapse" is L0-9's job (E2 §3.3). |
| L0-10 | Forward-mode `Dual<S>` over the exact rung | C (eventual) | **consumer-adapter** | One consumer, eventual, and it is a *generic construction over the `Scalar` seam* — ~60 lines outside resolvent once §3's seam ships. Resolvent shipping it would be scope creep with no second consumer. |
| L0-11 | Lazy filtered real (eager interval + lazy exact DAG) | C (blocked-now, but already owned by the consumer) | **resolvent-optional-crate** (`resolvent-lazy`, non-default) | One consumer, which already has a working implementation. It is a *strategy*, not an algebraic object, and it is expressible outside resolvent given the seam. Shipping it optionally is justified only because resolvent's own root isolation wants an eager filter internally; it must never be on the default path or in a core signature. |
| L0-12 | ℤ/n for composite n | nobody | **out-of-scope** | Named in the spec, wanted by no consumer, and it is not needed by any modular method (all of which use prime moduli). Build it when something asks. |
| L0-13 | A bignum `Q` offered as a units/exponent type | nobody — actively wrong | **out-of-scope** | `league::Exp` is `{num: i16, den: i16}`, `Copy`, `const fn` gcd, frozen wire form (E1 §1.5). Resolvent's `Rational` is the wrong type and must not be advertised as a substitute. |

### 2.2 Layer 1 — polynomials

| # | Capability | Wanted by | Placement | Justification |
|---|---|---|---|---|
| L1-1 | `UPoly<C>` dense univariate, standalone, defined **before and independently of** the multivariate type | C (every path), S (dense-output event polynomial) | **core, first** | Two consumers, and R3 §2.5 shows the entire consumer-unblocking surface is dense univariate. Defining it standalone is what lets the L1-multivariate/F4 program run on a track that never blocks the consumer track. |
| L1-2 | `MPoly<C>` sparse distributed, packed exponents, **runtime arity** | V (blocked-now on runtime arity), C (weakly: torus lane, Steinmetz) | **core** | The spec's one-way door. E3 R9 is the binding constraint: `MPoly<Q, 4>` with a const-generic arity makes an adapter that builds rings from constraint data *impossible*; solverang's per-constraint arity varies 2..14. Arity lives in a runtime `Ring` value. |
| L1-3 | `Ring` carried **by value**, not borrowed | V (implied by L1-2), embedding (§1.1) | **core** | R3 §2.4 sketches `SparseDist<C> = Vec<(MonomialId, C)> + &Ring`. This track overrides the borrow: a lifetime parameter on `MPoly` would infect every consumer struct that stores one (§1.1). `Ring { nvars: u32, order: Order, packing: Packing }` is `Copy` and ~12 bytes; carry it inline. |
| L1-4 | **No global monomial interner.** Packed exponent keys live inline in the term | embedding | **core** | An interner is ambient state by another name (§1.1). Terms are `(PackedMon, C)` with the key order-normalized at *construction*, so comparison stays a single unsigned integer compare (R3 §1.5) and `MPoly` stays a self-contained `Send + Sync` value. |
| L1-5 | `MPoly::derivative(var)` | V | **core** | Textbook primitive. It also halves solverang's transcription: the adapter transcribes residuals only and resolvent produces the Jacobian (E3 R4). |
| L1-6 | `evaluate` at a point in a **different** ring, via an explicit hom | V | **core** | E3 R5. Modular evaluation, interpolation-based GCD, and sign-at-an-algebraic-point are all this one signature internally. |
| L1-7 | Coefficient hom `MPoly<Q> → MPoly<Fp>`, fallible when `p ∣ den` | V | **core** | The core of every modular algorithm. Fail-closed, not silent (E3 R3). |
| L1-8 | `total_degree()` | V | **core** | Trivial, and it is *all* Bézout counting needs — which is precisely why Bézout gets no dedicated API (E3 §4 #2). |
| L1-9 | Bernstein coefficients + exact de Casteljau subdivision + certified range enclosure over a rational box | C (blocked-now) | **core** | One consumer, but clause (b) with an unusually strong internal argument: the Descartes/VCA test *is* a Bernstein coefficient sign count, so resolvent computes these anyway. Exposing them is near-zero marginal cost. E2 §4.2 is emphatic that naive interval evaluation provably rejects true identities (`whole_carrier_enclosure.md:29-46`), so a CAS without this cannot serve a certification path at all. Univariate/bivariate only; the general tensor-product box version is adapter work. |
| L1-10 | `RecursiveView<'a>` — borrowed recursive view for subresultant PRS | resolvent internals | **core** (internal-facing, public read-only) | R3 §2.4. A view, not an owned tree: the coefficients stay in the distributed arena. |
| L1-11 | Kronecker substitution | resolvent internals | **core** utility | Not a representation. |
| L1-12 | `RatFunc` (rational function type) | nobody | **out-of-scope** | E3 §5: the two rational residuals are cleared by the adapter, which records the extraneous `len_sq = 0` factor itself. No second consumer. |

### 2.3 Layer 2 — the engine

| # | Capability | Wanted by | Placement | Justification |
|---|---|---|---|---|
| L2-1 | Real root isolation over ℚ with exact **multiplicities**, over an optional window, under a budget | C (blocked-now), S (event detection) | **core** | Two consumers. Multiplicity must be on the returned root, not recomputed: for cadabra2 a double radicand root *is* the sheet-junction signature (E2 §10.6). The window matters: sinbad isolates only within `[t_n, t_n+h]` (E1 §1.3). |
| L2-2 | Yun square-free decomposition, `gcd`, `divrem`, `square_free_part` | C, plus every internal path | **core** | Two-consumer via C's direct use and S's transitive use through L2-1. |
| L2-3 | `resultant` / subresultant PRS eliminating one variable | C (next-milestone) | **core** | One consumer today. Clause (b): spec-named, and resolvent needs it internally for `AlgebraicReal` arithmetic (F8 degree bookkeeping) and for curve topology. E2 §3.5 is the largest net-new demand: the unbuilt torus lane is pure resultant work. Note `arrangements` currently reaches degree ≤8 by *double squaring* because no general resultant was available (R3 §2.5) — that is the shape of the hole. |
| L2-4 | Univariate factorization over ℚ (Zassenhaus, then van Hoeij) | C (degree-4 plane curve) | **core** | One consumer. Clause (b): spec-named, and it is a hard prerequisite for L0-9 (minimal polynomials) and for any `Hash`/canonical form on `AlgebraicReal` (R3 §8.2 F7). E2 §3.4 is the cleanest single lift in the whole evaluation: one general capability replaces three hand-coded circle strata *and* covers the generic case they were carved out of. |
| L2-5 | `row_echelon` over a field returning **rank, pivot rows, dependent rows, and the transform** | V (explicit), C (ℚ linear algebra) | **core, public** | Two consumers. The transform is not a bonus: it is solverang's `implied_by` certificate, shipped unconditionally empty today at `system.rs:803` (E3 §0.3), *and* it is the same object as a Gröbner cofactor representation. Same discipline, one layer down. |
| L2-6 | Fraction-free (Bareiss) determinant over an integral domain, incl. ℚ[λ] | C (2.448 ms recursive Laplace today) | **core** | One consumer directly, but it is the same routine L2-5 needs over non-fields, and modular determinants are internal to L2-3. **Primes must not appear in the signature** (E2 §4.3): cadabra2 asks for a fast exact determinant, and modular is *how* you give it one, not *what* it asked for. |
| L2-7 | Sylvester inertia / congruence diagonalization of a symmetric matrix | C only | **consumer-adapter** | This is the rule working correctly. cadabra2 already has 49 lines of it (`classification.rs:292-338`). Once resolvent ships the `Scalar`/`ScalarOrd` seam (§3) and `NumberField` (L0-9), that *existing* routine becomes generic and instantiates at ℚ(α) **for free** — which closes cadabra2's largest fail-closed site (E2 §3.1) with **zero new resolvent API**. Putting inertia in core would add a public routine one consumer calls and would not close the site any faster. |
| L2-8 | Rank of a polynomial matrix at an algebraic root, by minor vanishing | C only | **consumer-adapter** | ~20 lines over `AlgebraicReal::is_root_of` (L3-3) and L2-5. cadabra2 already wrote it (`classification.rs:242-260`). |
| L2-9 | Factorization of a quadratic form into linear factors over ℚ / ℚ(√d) | C only | **consumer-adapter** | Diagonalize (L2-7, adapter) + square-root detection (L0-9) + split. No second consumer, and it is a two-line composition once L0-9 exists. Replaces cadabra2's guessed factor pair plus ten-coefficient identity check (E2 §3.2). |
| L2-10 | Gröbner / F4, ideal membership, Nullstellensatz certificate | V (eventual, gated); C zero; S zero | **core, explicitly not in the first fan-out** | Zero current consumers. Clause (b) alone. E3 §0.6 is unambiguous: solverang's algebra demand is gated behind a Laman/DR-planner decomposition it has not begun, and a whole-sketch cluster is ~250 quadratics — intractable for anyone's engine. **Do not build F4 for solverang.** Build it because a CAS has one, after L0–L3 close real consumer sites. The Nullstellensatz certificate `1 ∈ ⟨f₁…f_k⟩` falls out of certified-mode cofactors and is the most attractive *new* capability found in E3 (§4 #10). |
| L2-11 | Topology of a real bivariate curve `G(a,b)=0` | C (next-milestone) | **core, later** | One consumer; clause (b) (the spec's M4 "resultants, CAD"). E2 §13 flags it as genuinely unsettled whether the torus lane needs this or whether L2-3 suffices; do not build it before that is settled by working one plane×torus case by hand. |
| L2-12 | Multivariate factorization at scale (van Hoeij recombination, Zassenhaus in n variables) | nobody — both C and V reject it | **core, post-v1** | Clause (b) only. E2 §6.2: the only factorizations wanted anywhere are degree ≤4 in ≤3 variables. Sequence it last. |
| L2-13 | BKK / mixed volume root counting | V, rejected by V | **out-of-scope** | Convex geometry over Newton polytopes, not algebra. E3 §4 #3. It belongs in a polytope crate. |
| L2-14 | Numeric root polishing, Newton/corrector, homotopy/continuation | nobody — C calls it an "attractive nuisance" | **out-of-scope** | E2 §6.3-6.4: `quadric/roots.rs:11-12` exists precisely so "no numeric root polishing enters the decision path". E3 §4 #5: continuation is a Davidenko-ODE predictor-corrector needing only the residual and Jacobian solverang already has. An f64 root-finder in resolvent's API actively damages one consumer and helps none. |
| L2-15 | Interval-Newton / Krawczyk existence-and-uniqueness | V names it as the *right* tool for a job resolvent should not do | **out-of-scope** | E3 §4 #6 and §7: it is a numerics library, not a CAS. Resolvent ships `Interval<f64>` (L0-5) because root isolation needs it internally, not because a consumer asked for a solver. |
| L2-16 | Any API taking `eps: f64` for an equality or sign decision | nobody — actively forbidden | **out-of-scope** | E2 §6.5: cadabra2 has a role-typed tolerance context and forbids global epsilons. R3 §8.2 F2: equality-by-tolerance is intransitive and is *the* canary failure of exact arithmetic. No epsilon exists anywhere in resolvent's decision surface. |

### 2.4 Layer 3 — algebraic numbers

| # | Capability | Wanted by | Placement | Justification |
|---|---|---|---|---|
| L3-1 | `AlgebraicReal { defining_poly, isolating_interval, multiplicity }` with `&self` comparison and `impl Ord` | C (inner loop); S no; V no | **core** | **One consumer**, stated plainly. Clause (b), and the spec calls this type "the whole bridge to computational geometry" — E2 §1 empirically confirms it: cadabra2's arc endpoints, seam events, sheet junctions and p-curve split parameters are all this type. Core status here rests entirely on clause (b) plus the fact that it is resolvent's headline differentiator, not on consumer count. |
| L3-2 | Construction is fail-closed on non-square-free input | R3 §8.2 F1 | **core** | The constructor takes a `SqfrPoly`, a newtype whose only fabrication path is `square_free`/`SqfrPoly::new -> Result`. This makes `isolate_roots` total on its domain except for budget — which also fixes E2 §9's "single-variant error that the caller always pre-checks is a signature smell". |
| L3-3 | `is_root_of(h)`, `sign_of(h)` at an algebraic number | C | **core** | The sign query must settle zero-ness *algebraically first* (gcd) before entering any refinement loop, or it hangs rather than answering (R3 §8.2 F5). A hang in a library is worse than a wrong answer because it is undebuggable in production. |
| L3-4 | `rational_strictly_between(a, b)` and `rational_sample_in_gap(&[roots], i)` | C — hand-rolled **twice in two crates of one consumer** | **core** | E2 §4.4. Two independent implementations of one primitive inside a single consumer is the clearest possible signal it belongs in the library. Both carry a hard 256-step budget; resolvent's takes a `Budget` and returns a typed decline. |
| L3-5 | Sign of an element of a real tower — `α + β√h + γ√h'` at an algebraic abscissa | C (inner loop) | **core, in general form** | Expose it as *sign of an element of a real extension tower over ℚ(ξ)*, not as `sign_radical2`. cadabra2's two-radical ladder is then a two-line instantiation. This is R3 §8.2 F8's route (b) made general: it is what keeps predicates in degree 4 instead of 65536. |
| L3-6 | General field arithmetic (`+`, `×`) on `AlgebraicReal` | nobody asked | **core, opt-in and loudly documented** | R3 §8.2 F8: `α + β` has degree ≤ `deg α · deg β`; three operations take degree 4 to 65536 without a factorization after each step. Default is route (b) — the sign ladder (L3-5). Route (a) is available, costs a factorization per operation, and says so in its name (`arith_reduced`). |
| L3-7 | `Hash` on `AlgebraicReal` | nobody | **out-of-scope until L2-4 lands** | R3 §8.2 F7: `x²−2` and `x⁴−4` are equal numbers with different polynomials. A "cheap" `Hash` silently puts two entries in a `HashMap` for one number and shows up as nondeterminism in a consumer, never in a unit test. `Hash` exists only behind an explicit `canonicalize()` that costs a factorization. |

### 2.5 Layer 4 — expressions

| # | Capability | Wanted by | Placement | Justification |
|---|---|---|---|---|
| L4-1 | Hash-consed `Store` with `Expr` handles and **structural** equality | S, C | **core** | Two consumers. But they want it for opposite reasons — sinbad for a canonical content address, cadabra2 for a certificate tether that a canonicalizer would *break* (E2 §11.2). See §6.4 conflict 1. |
| L4-2 | Open, **caller-owned** `FuncTable`: `Apply(FuncId, args)` with a per-function derivative rule | S (needs sin/sinh/exp/cos), C (needs opaque `Cos2`, `Radical` with *no* rule), V (must not see `Atan2` at all) | **core** | This is the synthesis. One mechanism serves all three because resolvent ships **no transcendental semantics in core** — only a table the caller constructs. `FuncTable::standard_elementary()` is a constructor resolvent offers; `FuncTable::empty()` plus `register(name, arity, deriv)` is what cadabra2 uses. solverang never builds a table, so `Atan2` structurally cannot appear in its world (E3 §5). |
| L4-3 | `diff` with a caller-supplied leaf rule | S (plexus is a stub blocked on exactly this) | **core** | One consumer. Clause (b): differentiation with respect to an implicit variable, where the derivative of an unknown is a *new* unknown the caller mints, is textbook CAS (Pantelides "grows new variables", E1 §5.2). Plain `diff` cannot express it. Signature changed from sinbad's ask — see §6.4 conflict 6. |
| L4-4 | `walk_topological(expr) -> impl Iterator<Item = (NodeId, NodeRef<'_>)>` with stable ids | S | **core** | Shared-subexpression let-binding falls out of hash-consing for free and is the main value the DAG adds over a tree. |
| L4-5 | `is_polynomial_in(&syms) -> Option<MPoly<Rational>>` — the L4→L1 bridge as a **predicate** | S (speculative), C (morally, via its Weierstrass rationalization) | **core** | Small, and it is the thing that keeps an open term algebra honest about where the exact engine applies (E1 §5.1). A coercion would lie; a predicate cannot. |
| L4-6 | `canonical_bytes(expr)` — pure function of mathematical content, plus `SCHEMA_VERSION` | S (D6: BLAKE3 content addressing), C (certificate tether) | **core** | Independent of interning order, node ids, arena addresses, insertion history, build configuration. A canonical-form change is a re-key event for every downstream artifact, so it is versioned explicitly. |
| L4-7 | A code emitter (Rust/C/WASM printer) | S wants one and says resolvent must **not** ship it | **out-of-scope** | E1 §5.3. sinbad needs Rust closures; the next consumer needs its own opcode tape. Resolvent exposes L4-4 and stops. |
| L4-8 | e-graph / equality-saturation simplifier, `egg`/`egglog` integration | nobody — C actively hostile, S says "anvil should call egg directly" | **out-of-scope for core; external glue** | The spec's L4 assumes simplification-by-e-graph is the point. Both evaluations say no. cadabra2 keeps `Cos2` as a first-class atom *deliberately* rather than rewriting it to `2cos²t−1` (E2 §11.1); a canonicalizing e-graph rewrites exactly the thing that must be left alone. anvil's want is Herbie-style FP-accuracy rewriting whose rewrites **change the computed value** (E1 §2) — resolvent's must not. What resolvent ships is L4-4, a stable structural encoding, which is what makes an external `egg` adapter possible without resolvent depending on `egg` (R1 §4). |
| L4-9 | A general `simplify()` | nobody | **out-of-scope** | The spec's own risk section says to refuse it. Both L4 consumers independently confirm. `canonicalize()` exists, is explicit, is opt-in, and is defined as *value-preserving normalization*, not cleverness. |

### 2.6 Cross-cutting

| # | Capability | Wanted by | Placement | Justification |
|---|---|---|---|---|
| X-1 | `Budget` on every entry point that can loop, with `BudgetExhausted` distinct from `MalformedInput` | S (D4, tiered-core rungs), C (C8, two 256-step witness loops) | **core** | Two consumers, independently derived. sinbad's `Decline` "always means the next rung may succeed" (`tiered-core/src/rung.rs:11`); cadabra2's budgets "turn a surprise into a typed refusal instead of a hang". A resolvent that can only run to completion or panic cannot be a rung. |
| X-2 | Small closed error enum: `PartialEq + Clone`, no `Box<dyn>`, no `String`, offending data on the variant | C (explicit), S (maps to `DiagCode`) | **core** | E2 §9: consumers `matches!` on it in tests, which `String` payloads defeat. It must **not** try to be the consumer's kernel error type — the consumer maps it upward in ~20 lines, which is exactly what cadabra2's existing 6-line `From<lazy_exact::…>` proves works. |
| X-3 | Typed `Unsupported { what }` refusal naming the missing capability | C (CORE RULE), S (D4 "not-implemented" refusal) | **core** | Fail closed. "Even if a fallback is implemented, it should still fail if the main algorithm is not." |
| X-4 | `Certainty::{Proved, Probable}` on every modular result | R3 §3.1 design rule; S's D5 grading; C's `ProofStrength` | **core** | `Probable` is allowed to exist (Gröbner over ℚ needs it) but must be visible in the type, and the default path must be `Proved`. |
| X-5 | Certificates: private fields, crate-private mint, carry the claim, `certifies(claim) -> bool` | C, S | **core** | §4. |
| X-6 | Determinism: no ambient RNG, deterministic prime sequence, seeded points, `BTreeMap` in ordering-visible positions | S (D1/D2, hard), V (R2, hard cross-platform requirement) | **core** | Two consumers with independent hard requirements. |
| X-7 | `#![forbid(unsafe_code)]` above one auditable leaf | S (D7), C (§10.10) | **core** | 11 of 28 sinbad lib crates already forbid it; both `lazy-exact` and `cadabra-check` do. |
| X-8 | Warm-start metadata (tier reached, bisections spent, precision attained) as **plain data with no proof type attached** | C | **core** | E2 §6.6 is a firm no to certificates feeding `cadabra-hints`, whose whole simplification is "a hint is never evidence … hint values need no unforgeability". Resolvent must expose the non-decision metadata separately from the certificate so a consumer can cache one without laundering the other. |
| X-9 | An adapter crate for any consumer | nobody | **out-of-scope** | Rule 3. Resolvent ships no adapter crates and has no optional dependency on any ecosystem crate. |

---

## 3. The numeric-type seam

This is the hardest question in the brief, and the three candidate answers are all partly
right. The resolution is that **"how do a consumer's numbers get in" is two questions
wearing one coat**, and they have different answers.

### 3.1 The two questions

1. **What are a polynomial's coefficients?** ℤ, ℚ, GF(p), ℚ(α). Algorithms over them need
   *ring theory*: content and primitive part, exact division, gcd domains, Landau–Mignotte
   bounds, characteristic, rational reconstruction, bad-prime detection.
2. **What do you evaluate a polynomial *at*, and what does a generic algorithm text
   instantiate over?** f64, an interval, a filtered lazy real, an exact rational, a dual
   number. These need only *ring operations plus a sign*.

### 3.2 The three options, evaluated

**(a) Resolvent-owned concrete types plus `From`/`TryFrom` at the boundary.**

- *Performance:* zero conversion cost when the consumer newtypes resolvent's type, which is
  exactly what cadabra2's adapter already does — `pub struct ExactScalar(lazy_exact::Rational)`
  and three siblings (E2 §12). A newtype forward is a no-op. Cost is O(n) only when the
  consumer keeps a genuinely different representation, and none of the three do for
  *coefficients*: `league::Exp` is not a coefficient, `reckon` is f64 summation, and
  lazy-exact's `Rational` is dashu-backed exactly as resolvent's would be.
- *Compile time:* minimal. Algorithms monomorphize over a closed set of ~4 types.
- *Failure mode:* forces wholesale conversion at every call **iff** the consumer's numbers are
  structurally different. For coefficients they are not.

**(b) A resolvent-defined coefficient trait the consumer implements for its own type.**

- *Performance:* looks free, is not. To be useful the trait must expose everything §3.1(1)
  lists. That is the trap the brief names: implementing `CoefficientRing` for an inner-loop
  number type pushes bignum-shaped obligations — exact division, content, bit-length,
  reconstruction — into a type whose entire purpose was to be word-sized. A consumer that
  implements it honestly has written a bignum; a consumer that implements it dishonestly has
  silently broken every modular algorithm's bad-prime detection.
- *Compile time:* worst of the three. Every L2 algorithm monomorphizes per consumer type.
- **Rejected.** No consumer asked for it, and E3's `Fp` demand is explicitly for
  *resolvent's* prime field, not for solverang's own.

**(c) Generic over a ring trait with resolvent supplying default impls.**

- *Performance:* what cadabra2 actually asks for, in a bounded form. It writes de Boor once
  and runs it at three tiers; `fastpath/filter.rs` renders T0/T1/T2 from one `TierField`
  program (E2 §10.4). Applied to *straight-line evaluation* — Horner, de Casteljau, Bareiss
  determinant, sign evaluation, interval filtering — the generic text is small and the
  instantiation count is bounded by how many scalars a consumer uses.
- *Compile time:* the cost scales with how much of the library is generic. Applied to the
  whole engine it is severe; applied to a handful of straight-line texts it is negligible.

### 3.3 Decision

**(a) for coefficients. (c) for evaluation scalars, restricted to straight-line algorithm
texts. (b) never.**

```rust
// resolvent-seam — zero dependencies, no bignum obligations anywhere in this file.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Sign { Neg, Zero, Pos }

pub trait Scalar: Clone + PartialEq + Sized {
    fn zero() -> Self;
    fn one()  -> Self;
    fn add(&self, rhs: &Self) -> Self;      // by reference, never consuming (E1 §1.4)
    fn sub(&self, rhs: &Self) -> Self;
    fn mul(&self, rhs: &Self) -> Self;
    fn neg(&self) -> Self;
}

pub trait ScalarOrd: Scalar { fn sign(&self) -> Sign; }
pub trait TryDiv:    Scalar { fn try_div(&self, rhs: &Self) -> Option<Self>; }

/// A ring homomorphism carried as a *value*, so a runtime modulus is expressible
/// and reduction can fail closed.
pub trait Hom<A, B> { fn apply(&self, a: &A) -> Result<B, Error>; }
```

Six methods, one sign, one fallible division. **Nothing in `Scalar` obliges an implementor
to be a bignum**, which is precisely the trap avoided. `f64` implements `Scalar + ScalarOrd`.
`Interval<f64>` implements `Scalar`. cadabra2's filtered `Real<Rational>` implements it in
~30 lines. solverang's f64 configuration point implements it trivially.

Coefficients are a **sealed** set — `Rational`, `Integer`, `FpElem`, `NfElem` — because every
L2 algorithm needs facts about them that `Scalar` cannot carry. A consumer cannot add a
coefficient ring, and this is stated as a limitation, not hidden.

The evaluation signature that ties them together, and which satisfies E3 R5 exactly:

```rust
impl<C> MPoly<C> {
    /// Evaluate a polynomial with coefficients in `C` at a point in `S`,
    /// under an explicit hom `C -> S`. Fails closed if the hom fails
    /// (e.g. `p` divides a denominator).
    pub fn evaluate_with<S: Scalar>(
        &self, hom: &impl Hom<C, S>, point: &[S],
    ) -> Result<S, Error>;
}
```

`Fp` implements `Hom<Rational, FpElem>` and returns `Err(Domain { fault: BadPrime })` when
`p ∣ den`. The consumer's own scalar implements `Scalar`, supplies its own
`Hom<Rational, TheirType>`, and never converts a polynomial.

### 3.4 The boundary rules

- **The only f64 ingress is `Rational::try_from_f64`** — exact dyadic, `Err` on NaN/±∞. No
  heuristic sibling exists.
- **The only f64 egress is `approx_lossy()`**, documented as diagnostic. No `impl From<X> for
  f64`, no `as`, no `Display` that rounds.
- **Lift-then-operate is the only expressible order.** Resolvent ships no
  `Rational::from_f64(a - b)`-shaped convenience. cadabra2 codified this as SEM-0 and enforced
  it structurally (E2 §10.8); resolvent should not hand back the footgun.
- **Which algorithm texts are generic** is a closed list, fixed at design time and not grown
  casually: Horner evaluation, de Casteljau/Bernstein, Bareiss determinant, matrix
  multiplication, sign-of-expression ladders, interval filtering. Everything else is
  concrete. This is the compile-time budget, written down.

---

## 4. The certificate API

Resolvent's differentiator is emitting checkable certificates. The design question is not
*whether* but *what it costs when unwanted* and *how a consumer verifies one*.

### 4.1 Shape

```rust
pub struct Certificate<C: Claim> {
    claim:     C,              // private
    evidence:  C::Evidence,    // private
    certainty: Certainty,      // private
}

impl<C: Claim> Certificate<C> {
    pub fn claim(&self)     -> &C;                    // read
    pub fn evidence(&self)  -> &C::Evidence;          // read
    pub fn certainty(&self) -> Certainty;             // read
    pub fn certifies(&self, claim: &C) -> bool;       // structural equality against the tether
    pub fn verify(&self, budget: Budget) -> Result<(), Error>;   // re-checks, using only public ops
}

pub enum Certainty { Proved(ProofKind), Probable(ProbableReason) }
pub enum ProofKind { Identity, Divisibility, Cofactor, Enclosure, DegreeBound }
```

**No public constructor exists on any certificate type.** Mints are `pub(crate)`. A
certificate exists iff resolvent proved the claim (E2 C4). But **the accessors expose the
mathematical content**, which is what lets a paranoid consumer re-verify with its own
arithmetic instead of trusting `verify()`. That is the resolution of the apparent tension in
E2 §8: cadabra2 wants resolvent's certificates *and* expects to cross-check them
independently, with a from-scratch `BigInt` in its own TCB. Unforgeable means *no public
mint*; checkable means *public read*. Both, not either.

**Tether.** Every certificate carries the claim it attests, and `certifies` is structural
equality against it, so "a transplanted certificate fails the comparison instead of riding
along" (E2 C5). Claims hold `Arc`-shared operands so carrying them is cheap.

### 4.2 Cost tiering — the answer to "always produced, or opt-in?"

Neither. **Opt-in by choosing an entry point, tiered by what the evidence actually costs.**
A boolean flag would be wrong because the three tiers have different orders of magnitude.

| Tier | Definition | Behaviour | Examples |
|---|---|---|---|
| **F — free** | The evidence *is* the answer's shape; producing it costs nothing extra | Always returned, part of the return type. No opt-out because there is nothing to opt out of. | Isolating intervals from `isolate_roots`; the echelon transform from `row_echelon` (E3 R7 — the `implied_by` certificate falls out of the same pass); the factor list from `factor`; multiplicities from Yun. |
| **C — cheap** | Verification is `O(one multiplication)` or less | Verification runs **by default**; the escape is a separately-named `*_unchecked` entry point that returns `Certainty::Probable` | `gcd` (check `g ∣ a`, `g ∣ b`, degree match — R3 §3.2 says this is genuinely cheap); factorization *product* (multiply back); resultant (two independent algorithms plus three structural invariants). |
| **X — expensive** | Evidence requires tracing the computation | **Separate entry point.** The uncertified path does not pay | `groebner` vs `groebner_certified` — cofactor tracing costs memory and time, and R3 §3.4 records that the modular thesis' "verification is cheap" claim is *false* for Gröbner specifically. |

Written as a rule: **no certificate may add more than a documented constant factor to the
answer path; where it would, it lives behind a separate entry point.** That is the whole cost
model, and it is auditable.

### 4.3 Verification without re-implementation

`verify(&self, budget)` is implemented by resolvent using only its own public operations. A
consumer that trusts resolvent calls it and gets `Result<(), Error>`. A consumer that does
not trust resolvent reads `evidence()` and checks it with whatever it likes — for a gcd
certificate that is two polynomial multiplications, for an echelon certificate a
matrix-vector product, for a Gröbner certificate the cofactor sum. Nothing in the check
requires resolvent's internals.

### 4.4 Composition with the two consumer models — kept general

Resolvent's vocabulary is `Certainty` and `ProofKind`. It names neither consumer's ladder.
Mapping is trivial and lives in the adapter:

```rust
// cadabra2 adapter, 6 lines
fn strength(c: Certainty) -> Option<ProofStrength> {
    match c {
        Certainty::Proved(ProofKind::Enclosure) => Some(ProofStrength::IntervalEnclosed),
        Certainty::Proved(_)                    => Some(ProofStrength::AlgebraicallyExact),
        Certainty::Probable(_)                  => None,   // fail closed: refuse to certify
    }
}

// sinbad adapter, 5 lines
fn grade(c: Certainty) -> Grade {
    match c { Certainty::Proved(_) => Grade::Proven, Certainty::Probable(_) => Grade::Estimated }
}
```

Two constraints this must respect, both taken from the evaluations and neither obvious:

1. **`Probable` must map to a refusal, not to a weaker yes, in a fail-closed consumer.** The
   mapping above does. Resolvent's job is to make the distinction visible in the type; the
   consumer decides what to do with it.
2. **Certificates must be separable from warm-start metadata** (X-8). cadabra2's hint store is
   defined by "a hint is never evidence"; feeding a certificate into it would destroy the
   simplification that makes the hint cache untrusted. So resolvent returns
   `(answer, Certificate<_>, Telemetry)` where `Telemetry { tier_reached, bisections,
   precision_bits, primes_used }` is plain `Copy` data carrying no proof type and no
   `Certainty`. One goes to the trust boundary, the other to the cache, and the type system
   keeps them apart.

### 4.5 What a certificate is *not*

E2 §8's second twist is load-bearing: cadabra2 consumes a certificate **as an admission
ticket, not as a proof to read** — its *presence* classifies a Parasolid disagreement as a
certified divergence rather than a bug, and nobody re-verifies it. So the value is (a) it
cannot be forged, (b) it names what it attests, (c) it is cheap to carry. **Elaborate proof
objects a consumer must interpret are not wanted by anyone.** Keep `Evidence` payloads to the
minimum that makes `verify` possible.

---

## 5. Upstream integration, additively

The requirement: a consumer needing geometry or multiphysics should be able to plug into an
existing ecosystem *through* resolvent, while resolvent depends on nothing and knows nothing
about that ecosystem. Concretely, resolvent exposes five things that make an ecosystem
possible, and takes on zero obligations for it.

1. **`resolvent-seam` — a zero-dependency trait crate.** `Sign`, `Scalar`, `ScalarOrd`,
   `TryDiv`, `Hom`, `Budget`, `Error`, `Certainty`. A geometry crate can depend on
   `resolvent-seam` *without* depending on resolvent's engine, write its algorithm text once,
   and instantiate it at its own f64 tier today and at resolvent's exact tier when someone
   links it. This is the single highest-leverage hook, and it is not speculative: cadabra2
   already built its own `scalar-seam` for exactly this and renders T0/T1/T2 from one
   `TierField` program (E2 §10.4). Resolvent supplying the vocabulary means two ecosystem
   crates can interoperate at the scalar level without either depending on the other.
2. **Canonical bytes plus a schema version** (L4-6). Content-addressed caches, cross-process
   oracles, and reproducible artifact pipelines become possible without resolvent knowing what
   an artifact is. sinbad's `rutter` keys by BLAKE3 of bytes; that works against any producer
   whose bytes are a pure function of content.
3. **Public certificate accessors** (§4.3). An independent verification crate — owned by
   whoever wants it — can check resolvent's outputs without depending on resolvent's
   internals. That is how a consumer with a trusted computing base admits resolvent at all
   (E2 §8: `lazy-exact` got into cadabra2's TCB on `dashu + smallvec + thiserror` plus a
   zero-dep seam crate; that is the admission budget, and resolvent should stay inside it).
4. **`walk_topological` + `NodeRef`** (L4-4). A stable structural encoding of the DAG is what
   makes an external `egg`/`egglog` adapter, an external code emitter, and an external
   pretty-printer all writable by third parties. Resolvent ships none of them and depends on
   none of them (R1 §4).
5. **A small, closed, `PartialEq` error enum** (X-2). Every upward mapping is ~20 lines and
   every consumer's test suite can `matches!` on it.

**The dependency arrow.** Consumers depend on resolvent. Glue crates (`resolvent-egg`,
`geom-resolvent`, whatever) are owned by whoever wants them, live outside this repository,
and depend on both sides. Resolvent has **no optional dependency on any ecosystem crate** and
**no feature flag named after a consumer**. Optional features are capability-named:
`parallel`, `serde`, `lazy`.

**Keeping the deferred decision cheap.** Whether resolvent and `arrangements`/`lazy-exact`
later refactor into each other is explicitly deferred. Two choices keep deferral cheap:
resolvent's L0 lives behind a newtype wall over `dashu` (R1 §1.4), which is the same substrate
`lazy-exact` uses; and resolvent's `AlgebraicReal` is shape-identical to `lazy-exact`'s
`RealRoot { poly, lo, hi, multiplicity }` (E2 §1). A future merge is a rename plus an
`&mut self → &self` fix, not a redesign. Nothing in this document makes that merge more
expensive, and nothing assumes it happens.

---

## 6. The adapter sketches

The acceptance test: for each consumer, an adapter in roughly under 200 lines with **zero
changes to resolvent**. Sketched in real Rust against the API above; line counts are honest
estimates of the real thing, not of the sketch.

### 6.1 sinbad — three adapters, three call sites

sinbad's demands land in three unrelated crates, so it gets three adapters, not one.

**(a) `sinbad-testkit` MMS forcing generation — ~105 lines, build-time.**

```rust
// xtask/src/mms_gen.rs — runs offline, emits a committed .rs file.
use resolvent::expr::{Store, Expr, Sym, FuncTable, LeafRules};

struct Gen { st: Store, x: Sym, y: Sym, pi: Sym }

impl Gen {
    fn new() -> Self {
        let mut st = Store::with_functions(FuncTable::standard_elementary());
        let (x, y, pi) = (st.sym("x"), st.sym("y"), st.sym("pi"));
        Gen { st, x, y, pi }                                   // pi is a SYMBOL, not a value
    }

    // u* = sin(pi*x) * sinh(pi*y)
    fn harmonic_trig(&mut self) -> Expr {
        let (x, y, pi) = (self.st.var(self.x), self.st.var(self.y), self.st.var(self.pi));
        let a = self.st.sin(self.st.mul(pi, x));
        let b = self.st.sinh(self.st.mul(pi, y));
        self.st.mul(a, b)
    }

    // f = -div(kappa grad u) = -(kx*ux + ky*uy) - kappa*(uxx + uyy)
    fn poisson_forcing(&mut self, u: Expr, kappa: Expr) -> Result<Expr, resolvent::Error> {
        let d = |st: &mut Store, e, s| st.diff(e, s);            // implicit-zero leaf rule
        let ux  = d(&mut self.st, u, self.x)?;
        let uy  = d(&mut self.st, u, self.y)?;
        let uxx = d(&mut self.st, ux, self.x)?;
        let uyy = d(&mut self.st, uy, self.y)?;
        let kx  = d(&mut self.st, kappa, self.x)?;
        let ky  = d(&mut self.st, kappa, self.y)?;
        let lap = self.st.add(uxx, uyy);
        let adv = self.st.add(self.st.mul(kx, ux), self.st.mul(ky, uy));
        Ok(self.st.neg(self.st.add(adv, self.st.mul(kappa, lap))))
    }
}

/// Printer. resolvent hands back a topological walk; sinbad chooses the target language.
/// ~55 lines: match on NodeRef, emit f64 Rust, bind Sym("pi") -> std::f64::consts::PI,
/// bind Sym("x")/Sym("y") -> closure params, `let t{k} = ...;` for every shared node.
fn emit_rust(st: &Store, e: Expr, out: &mut String) {
    for (id, node) in st.walk_topological(e) { /* ... */ }
}
```

Uses: L4-1, L4-2 (`standard_elementary`), L4-3, L4-4. Resolvent changes: none. **~105 lines.**

**(b) `plexus` symbolic `d/dt` for Pantelides — ~140 lines, build-time.**

```rust
// crates/plexus/src/symdiff.rs
use resolvent::expr::{Store, Expr, Sym, LeafRules, LeafDefault};
use std::collections::BTreeMap;

/// plexus's variable convention: state variable `v` at differentiation order `n`.
/// The adapter owns this mapping; resolvent never learns what a "state variable" is.
struct DerVars {
    by_key: BTreeMap<(VarId, u32), Sym>,
    by_sym: BTreeMap<Sym, (VarId, u32)>,
}

impl DerVars {
    fn sym(&mut self, st: &mut Store, v: VarId, n: u32) -> Sym { /* intern "v#n" ~10 lines */ }
}

/// d/dt of an equation, with d/dt(der(v,n)) = der(v,n+1).
/// Two-phase: ask which symbols occur, mint their derivative symbols, then differentiate.
fn ddt(st: &mut Store, dv: &mut DerVars, e: Expr, t: Sym) -> Result<Expr, resolvent::Error> {
    let mut rules = LeafRules::new(LeafDefault::Zero);          // parameters differentiate to 0
    for s in st.symbols_in(e) {                                 // BTreeSet -> deterministic
        if let Some(&(v, n)) = dv.by_sym.get(&s) {
            let next = dv.sym(st, v, n + 1);
            let node = st.var(next);
            rules.set(s, node);
        }
    }
    st.diff_with(e, t, &rules)
}

/// Pantelides: the matching says WHICH equations to differentiate; we differentiate.
fn pantelides_step(st: &mut Store, dv: &mut DerVars, sys: &mut FlatSystem, unmatched: &[EqId])
    -> Result<(), resolvent::Error> { /* ~55 lines */ }

/// Alias elimination a = b / a = -b, on the canonical form so the Schedule content-addresses.
fn is_alias(st: &mut Store, e: Expr) -> Option<(Sym, Sym, bool)> {
    let c = st.canonicalize(e).ok()?;                            // explicit, opt-in
    /* ~25 lines of structural match on the canonical node */
}
```

Uses: L4-1, L4-3 (`diff_with` with a `LeafRules` table), L4-6 (`canonicalize` +
`canonical_bytes` for the `Schedule` content address). Resolvent changes: none. **~140 lines.**

**(c) `solverang` DAE event root isolation — ~65 lines, per-operation.**

```rust
// crates/solverang/src/events/exact_roots.rs
use resolvent::{Rational as Q, UPoly, SqfrPoly, Interval, Budget, isolate_roots, Error as RErr};

fn crossings_in_step(coeffs: &[f64], h: f64, budget: Budget)
    -> Result<Vec<Interval<Q>>, Decline>
{
    let mut c = Vec::with_capacity(coeffs.len());
    for &a in coeffs {
        c.push(Q::try_from_f64(a).map_err(|_| Decline::CannotCertify)?);   // fails closed
    }
    let p  = UPoly::from_coeffs(c);
    let sf = SqfrPoly::new(&p).map_err(|_| Decline::CannotCertify)?;       // fail-closed F1
    let hi = Q::try_from_f64(h).map_err(|_| Decline::CannotCertify)?;
    let window = Interval::new(Q::ZERO, hi);
    isolate_roots(&sf, Some(&window), budget)
        .map(|rs| rs.into_iter().map(|r| r.enclosure_q()).collect())
        .map_err(|e| if e.is_decline() { Decline::Budget } else { Decline::CannotCertify })
}
```

Uses: L0-2, L1-1, L2-1, L3-2, X-1. Resolvent changes: none. **~65 lines.**

**Verdict: all three pass, comfortably.**

### 6.2 cadabra2 — one delegation core, ~175 lines

E2 §12 measured the existing `lazy-exact` delegation at ~250 lines and attributed the ~50-line
overage to error mapping and hand-written panic guards. Against the API above those disappear.
Sketch and count:

```rust
// cadabra-core/src/exact/  — the resolvent delegation core
use resolvent::{Sign, Budget, Interval, Rational, SqrtExt, AlgebraicReal, UPoly,
                Error as RErr, Op, DomainFault};
use crate::error::{KernelError, KernelResult, Capability, Subject};

// ---- error mapping ---------------------------------------------------------- 22 lines
impl From<RErr> for KernelError {
    fn from(e: RErr) -> Self {
        match e {
            RErr::Unsupported { what }   => KernelError::not_implemented(cap(what), Subject::None),
            RErr::Budget { what, .. }    => KernelError::not_implemented(cap(what), Subject::None),
            RErr::Overflow { .. }        => KernelError::resource_exhausted(),
            RErr::Domain { fault, .. }   => KernelError::invalid_geometry(reason(fault)),
        }
    }
}
fn cap(op: Op) -> Capability { /* 8 arms */ }
fn reason(f: DomainFault) -> InvalidReason { /* 5 arms */ }

// ---- ExactScalar ------------------------------------------------------------ 34 lines
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ExactScalar(Rational);
impl ExactScalar {
    pub fn lift(x: f64) -> KernelResult<Self> { Ok(Self(Rational::try_from_f64(x)?)) }
    pub fn sign(&self) -> Sign                { self.0.sign() }
    pub fn add(&self, o: &Self) -> Self       { Self(self.0.add(&o.0)) }
    pub fn sub(&self, o: &Self) -> Self       { Self(self.0.sub(&o.0)) }
    pub fn mul(&self, o: &Self) -> Self       { Self(self.0.mul(&o.0)) }
    pub fn try_div(&self, o: &Self) -> KernelResult<Self> {
        self.0.try_div(&o.0).map(Self).ok_or_else(|| KernelError::invalid_geometry(DivByZero))
    }
    // the three exits, forwarded verbatim
    pub fn demote_exact(&self) -> KernelResult<i64>  { Ok(self.0.demote_exact()?) }
    pub fn enclosure(&self)    -> Interval<f64>      { self.0.enclosure() }
    pub fn approx_lossy(&self) -> f64                { self.0.approx_lossy() }
    pub fn num_bits(&self) -> u32 { self.0.num_bits() }
    pub fn den_bits(&self) -> u32 { self.0.den_bits() }
}

// ---- ExactRadical ----------------------------------------------------------- 26 lines
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ExactRadical(SqrtExt<Rational>);
impl ExactRadical {
    /// Total, not panicking: negative radicand is an Err, not an abort.
    pub fn new(a: ExactScalar, b: ExactScalar, r: ExactScalar) -> KernelResult<Self> {
        Ok(Self(SqrtExt::new(a.0, b.0, r.0)?))       // was 12 lines of hand-written guard
    }
    pub fn sign(&self) -> Sign { self.0.sign() }
    /* add/sub/mul/try_div/enclosure/approx_lossy forwards, one line each */
}

// ---- AlgebraicNumber -------------------------------------------------------- 33 lines
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct AlgebraicNumber(AlgebraicReal);          // Ord is resolvent's; no Rc<RefCell<_>>
impl AlgebraicNumber {
    pub fn multiplicity(&self) -> u32 { self.0.multiplicity() }
    pub fn is_root_of(&self, h: &UPoly<Rational>) -> KernelResult<bool> { Ok(self.0.is_root_of(h)?) }
    pub fn sign_of(&self, h: &UPoly<Rational>, b: Budget) -> KernelResult<Sign> {
        Ok(self.0.sign_of(h, b)?)
    }
    pub fn between(a: &Self, b: &Self, bud: Budget) -> KernelResult<ExactScalar> {
        Ok(ExactScalar(resolvent::rational_strictly_between(&a.0, &b.0, bud)?))
    }                                                // replaces TWO hand-rolled 256-step loops
    pub fn enclosure(&self)    -> Interval<f64> { self.0.enclosure() }
    pub fn approx_lossy(&self) -> f64           { self.0.approx_lossy() }
    pub fn demote_exact(&self) -> KernelResult<ExactScalar> { Ok(ExactScalar(self.0.demote_exact()?)) }
}
// The self-comparison guard is gone: resolvent's `cmp` handles ptr-equal operands.

// ---- IntervalScalar + Scalar impls ------------------------------------------ 34 lines
pub struct IntervalScalar(Interval<f64>);
impl resolvent::Scalar    for ExactScalar    { /* 6 fns, one line each */ }
impl resolvent::ScalarOrd for ExactScalar    { fn sign(&self) -> Sign { self.sign() } }
impl resolvent::Scalar    for ExactRadical   { /* 6 fns */ }
impl resolvent::ScalarOrd for ExactRadical   { /* 1 fn */ }

// ---- mint guard (decide exactly, demote once, validate) --------------------- 26 lines
```

| Block | Lines |
|---|---|
| Error mapping (`From<RErr>`, `cap`, `reason`) | 22 |
| `ExactScalar` | 34 |
| `ExactRadical` | 26 |
| `AlgebraicNumber` | 33 |
| `IntervalScalar` + four `Scalar`/`ScalarOrd` impls | 34 |
| `mint` guard | 26 |
| **Total** | **175** |

Against E2's measured ~250, the savings are: ~40 lines of hand-rolled
`rational_between`/`sample_between_roots` (now L3-4), ~15 lines of `Rc<RefCell<_>>` plumbing
and the `Rc::ptr_eq` self-comparison guard (now inside resolvent, §1.3), ~12 lines of the
`SqrtExt::new` negative-radicand guard (now total, X-2), and ~10 lines of error-information
laundering around a single-variant `RootError` (now a closed enum plus `SqfrPoly`).

Everything E2 measured *above* 250 — `WeierstrassParam`/`WeierstrassSample`/`WeierstrassSpan`,
`Radicand`, `CarrierExpr`, `BiquadTower`, the componentwise proof rule — is geometry, not
adapter. It would exist regardless of which CAS sat underneath, and it stays in cadabra2.
**Passes, at ~175 lines.**

**Note on what this adapter unlocks with no further resolvent API.** cadabra2's existing
49-line `inertia` (`classification.rs:292-338`) becomes generic over `ScalarOrd` and
instantiates at `NumberField` — closing its largest fail-closed site (`classification.rs:78-87`,
`:441-445`) without resolvent shipping an `inertia` function (L2-7). That is the seam paying
for itself.

### 6.3 solverang — a 40-line seam over an afternoon of transcription

```rust
// crates/solverang/src/exact/mod.rs
use resolvent::{Fp, FpElem, MPoly, Rational as Q, Ring, Order, linalg};
use crate::{constraint::Constraint, id::ParamId, param::SolverMapping};

/// Consumer-side trait. Lives in solverang. Resolvent knows nothing about it.
pub trait AlgebraicConstraint: Constraint {
    fn poly_vars(&self) -> &[ParamId];
    fn residual_polys(&self, ring: Ring) -> Option<Vec<MPoly<Q>>>;   // None = transcendental
}

pub fn generic_rank(
    cs: &[(usize, &dyn AlgebraicConstraint)],
    mapping: &SolverMapping,
    seed: u64,
) -> Option<GenericRank> {
    let fp    = Fp::new(2_147_483_647).ok()?;                 // runtime modulus        L0-6
    let n     = mapping.len();
    let point = fp.random_point(n, seed);                     // seeded, never ambient  L0-7

    let (mut rows, mut owner) = (Vec::new(), Vec::new());
    for (idx, c) in cs {
        let vars = c.poly_vars();
        let ring = Ring::new(vars.len() as u32, Order::GrevLex).ok()?;   // runtime arity L1-2/3
        let polys = c.residual_polys(ring)?;                  // bail on Gear/Insert/Spline2D
        let cols: Vec<usize> = vars.iter().map(|p| mapping.col(*p)).collect();
        let local: Vec<FpElem> = cols.iter().map(|&j| point[j]).collect();
        for f in &polys {
            let mut row = vec![fp.zero(); n];
            for (k, &j) in cols.iter().enumerate() {
                row[j] = f.derivative(k as u32)               //                        L1-5
                          .evaluate_with(&fp, &local).ok()?;  // hom Q -> GF(p)   L1-6/L1-7
            }
            rows.push(row);
            owner.push(*idx);
        }
    }
    let ech = linalg::row_echelon(&fp, rows).ok()?;           //                        L2-5
    Some(GenericRank {
        rank:       ech.rank(),
        dependent:  ech.dependent_rows().iter().map(|&r| owner[r]).collect(),
        implied_by: ech.transform_rows(&owner),               // the implied_by certificate
    })
}
```

**40 lines, zero resolvent changes.** Note `derivative` + `evaluate_with` means the adapter
transcribes only residuals; resolvent produces the Jacobian, which halves the transcription
below and removes a class of hand-differentiation bugs.

The second half is transcription:

```rust
impl AlgebraicConstraint for DistancePtPt {
    fn poly_vars(&self) -> &[ParamId] { &self.params }               // [x1, y1, x2, y2]
    fn residual_polys(&self, r: Ring) -> Option<Vec<MPoly<Q>>> {
        let (x1, y1, x2, y2) = (r.var(0), r.var(1), r.var(2), r.var(3));
        let dx = x2.sub(&x1);
        let dy = y2.sub(&y1);
        let t  = MPoly::constant(r, Q::try_from_f64(self.target_sq).ok()?);
        Some(vec![dx.mul(&dx).add(&dy.mul(&dy)).sub(&t)])
    }
}
```

× 28 algebraic constraint types at 5–12 lines each ≈ **250–320 lines**, using nothing but
`Ring::var`, `MPoly::constant`, and `+ − ×`.

**Honest verdict, both numbers reported.** The resolvent-facing seam is **40 lines and
passes**. The total adapter is **~300 lines and does not**, and that is not resolvent's to
fix: the transcription scales with solverang's constraint vocabulary and is byte-for-byte
identical under any polynomial API with a runtime-arity ring. The test as intended — "does
this consumer force resolvent to expose something bespoke?" — passes: every item the seam
touches (L0-6, L0-7, L1-2, L1-5, L1-6, L1-7, L2-5) is justified in §2 as a general primitive
independent of solverang.

The one thing that would fail it is a const-generic arity (`MPoly<Q, 4>`): per-constraint
arity varies 2..14, so the adapter could not instantiate rings from data at all. That is why
L1-2/L1-3 is a one-way door to settle before fan-out.

### 6.4 Conflicts, and who ate the cost

| # | Conflict | Resolution | Who absorbs it |
|---|---|---|---|
| 1 | cadabra2 needs **structural, non-canonical** L4 equality (its `certifies` tether, `Cos2` deliberately un-rewritten); sinbad needs a **canonical** form so its `Schedule` content-addresses | Resolvent never rewrites implicitly. Construction hash-conses and constant-folds and stops. `canonicalize(expr) -> Expr` is an explicit, value-preserving function returning a *new* node; `canonical_bytes` hashes whatever you hand it | **Resolvent** takes the general shape (no implicit rewriting, ever); **sinbad's adapter** pays 1 extra line per call site |
| 2 | sinbad needs transcendental function nodes; solverang forbids `Atan2`/`Sqrt`/piecewise nodes; cadabra2 needs opaque domain atoms with no derivative rule | Resolvent ships **no transcendental semantics** — only an open `FuncTable` the caller owns. `standard_elementary()` is a constructor, not a default. solverang never builds one, so `Atan2` cannot appear in its world | **Resolvent** — a genuine scope addition over the spec's polynomial-only L4, and the only way any of the three gets served |
| 3 | solverang needs `GF(p)` **public and callable with no CRT lift**; cadabra2 explicitly does not want `GF(p)` user-facing | `Fp` is public in `resolvent-modular` and appears in no signature cadabra2 calls. Modular methods are an *exposed layer*, not only an internal strategy | **Resolvent** (public surface it would have built anyway); cadabra2 pays zero by not importing it |
| 4 | cadabra2 needs `&self` comparison and `impl Ord` on `AlgebraicReal`; a refinement cache makes the type non-`Sync` | Interior mutability, `Send + !Sync`, self-comparison guard inside resolvent, cheap `Clone` for per-thread copies. The cache provably cannot change a verdict | **Resolvent** gives up `Sync` on its headline type; consumers wanting parallelism clone |
| 5 | cadabra2 wants `impl Ord` (infallible); sinbad's rung protocol wants a budget on everything | Comparison of two algebraic reals over ℚ is **decidable in finitely many steps**, so `Ord` is total and ships; `cmp_exact(other, budget) -> Result<Ordering, Error>` ships alongside for latency-bounded callers. Documented: `Ord` may allocate unboundedly | **Both get what they asked for**; resolvent pays two entry points |
| 6 | sinbad asked for `diff_with(e, sym, impl FnMut(Sym) -> Option<Expr>)`; a closure that mints nodes needs `&mut Store` while `diff_with` holds it | Signature changed to `symbols_in(e) -> BTreeSet<Sym>` + `diff_with(e, sym, &LeafRules)` where `LeafRules` is a `BTreeMap<Sym, Expr>` with a `Zero`/`Refuse` default. Borrow-clean, reentrancy-free, and deterministic by construction | **Resolvent** chose a different shape than the consumer asked for; **sinbad's adapter** pays ~6 lines for the two-phase loop and gains determinism |
| 7 | sinbad wants `Ctx` first-param and a `DiagCode` on every error; resolvent must be ambient-free and registry-free | Resolvent takes no capability handle and no diagnostics registry; its error enum is small, closed and stable | **Adapter** (~5 lines to drop `Ctx`, ~20 to map variants to codes) |
| 8 | sinbad demands bitwise reproducibility across thread counts; modular methods want randomness | Deterministic prime sequence; evaluation points from an explicit `u64` seed parameter; `rayon` parallelism only over a prime set fixed *before* the parallel region | **Resolvent** — non-negotiable, and solverang independently requires the same |
| 9 | solverang's coefficients are f64 bakes of transcendentals (`sin(30°)`); exact analysis of the dyadic rational answers a subtly different question | `try_from_f64` is exact-dyadic and no "nice rational" sibling exists. The adapter documents that it is analysing the system as authored in f64 | **Adapter** (a doc comment); **resolvent** absorbs the cost of refusing a convenience users will ask for |
| 10 | cadabra2's arrangement sweep cannot afford a bignum allocation per call; resolvent's ℚ is a bignum | Resolvent ships the `Scalar` seam and `Interval<f64>` so the consumer keeps its own filter and only descends to ℚ when the filter fails. `resolvent-lazy` exists but is non-default and never in a core signature | **Shared**: resolvent ships the seam, the consumer owns the filtering policy |
| 11 | cadabra2's TCB admission budget is roughly `dashu + smallvec + thiserror` + a zero-dep seam crate; resolvent wants `rayon` and `serde` too | `rayon` and `serde` are default-off features. Core runtime deps stay inside the measured budget | **Resolvent** |
| 12 | cadabra2 wants certificates unforgeable (private fields, crate-private mint); its TCB also wants to re-verify them with its own from-scratch arithmetic | Private fields + no public mint + **public read accessors** on the evidence. Unforgeable means no public constructor; checkable means public read | **Resolvent** |

---

## 7. API invariants

The short list. Any future change must preserve every one of these, or be argued as an
explicit override with the consumer cost named.

**INV-1 — No ambient state.** No `static mut`, no `static` with interior mutability, no
`thread_local!`, no global interner, no session object, no capability handle parameter, in any
`publish = true` crate. CI greps for these.

**INV-2 — No I/O and no clock.** No `std::fs`, `std::net`, `std::env`, `std::time`,
`std::process` in any published crate. A CAS that reads the environment cannot be
content-addressed.

**INV-3 — No unseeded randomness.** Prime sequences are deterministic; every random choice
that can reach an output takes a `u64` seed as an explicit parameter. No `thread_rng`, no OS
entropy. No `HashMap` iteration order may reach a result value.

**INV-4 — Total functions. No panics on any input-dependent path.** Overflow, coefficient
blowup, division by zero, square root of a negative, non-finite f64 ingress, exponent-packing
overflow: all `Result`. An adapter cannot absorb a panic.

**INV-5 — Errors are a small, closed, `Clone + PartialEq`, `String`-free enum**, with the
offending data on the variant, and with declines (`Budget`, `Unsupported`) distinguishable
from faults (`Domain`, `Overflow`) via `is_decline()`. Resolvent's error type never tries to
be a consumer's error type.

**INV-6 — Every loop that can run long takes a `Budget` and returns a typed decline.**
Exhaustion is a value, never a hang and never an abort.

**INV-7 — Exactly three exits from every exact type**: `demote_exact` (lossless or typed
error), `enclosure` (certified outward interval), `approx_lossy` (nearest double, diagnostic).
No `as f64`, no `impl From<_> for f64`, no `Display` that rounds.

**INV-8 — No `eps` parameter on any equality, sign, or ordering decision, anywhere.**
Equality is decided algebraically or not at all.

**INV-9 — Certainty is visible in the type.** Every modular or heuristic result carries
`Certainty::{Proved, Probable}`; the default path is `Proved`; certificates have private
fields, no public mint, public read accessors, and a `certifies(claim)` structural tether.
Warm-start telemetry is a separate, proof-free value.

**INV-10 — No public owned type carries a lifetime parameter.** Consumers store resolvent
values in their own structs. Borrowed views and iterators are the only exception.

**INV-11 — Nothing consumer-specific in the public API.** No type, trait, method, name, or
feature flag that mentions or presumes a consumer's domain. Features are capability-named.
Resolvent ships no adapter crates and has no optional dependency on any ecosystem crate.

**INV-12 — Canonical bytes are a pure function of mathematical content**, independent of
interning order, node ids, arena addresses, insertion history and build configuration, and
carry an explicit `SCHEMA_VERSION`. Changing canonical form is a breaking, versioned event.

**INV-13 — Polynomial ring arity is a runtime value.** No const-generic arity, no borrowed
ring, no global monomial interner. `MPoly` is a self-contained `Send + Sync` value.

**INV-14 — Coefficient rings are a sealed set; evaluation scalars are open.** A consumer may
implement `Scalar`/`ScalarOrd` for its own type and must never be asked to implement a
coefficient ring. `Scalar` stays at six methods plus a sign plus a fallible division, so
implementing it never obliges a consumer's inner-loop type to be a bignum.

**INV-15 — `AlgebraicReal` is the single documented non-`Sync` type**, and its refinement
cache can never change a verdict. Pinned by the trichotomy/transitivity/sort-stability/
idempotence property suite; "did not finish" counts as "wrong".

---

## 8. What this document does not settle

Stated so it is not mistaken for settled.

1. **Whether `resolvent-lazy` should exist at all.** It has one consumer, which already owns
   a working implementation. The argument for shipping it is that root isolation wants an
   eager filter internally. If that turns out to be false in practice, delete the crate.
2. **Whether L2-11 (bivariate curve topology) is real demand or whether L2-3 suffices.** E2
   §13 says working one generic plane×torus case by hand would settle it. Do not build it
   first.
3. **The `Ord` allocation bound.** Comparison is decidable, so `Ord` is total, but memory use
   on a Mignotte-style near-equal pair is unbounded in principle. Whether that ever bites a
   real consumer is unmeasured; the budgeted `cmp_exact` exists because it might.
4. **Whether the sealed coefficient set is too tight.** If a fourth consumer arrives with a
   genuine need for a coefficient ring resolvent does not have (p-adics, a specific tower),
   the answer is to add it to the sealed set, not to open the trait. That is a prediction, not
   a proof.
5. **The compile-time cost of the generic straight-line texts** (§3.4) is asserted to be
   negligible on a closed list of six algorithms. Measure it once L1 and L2 exist; if it is
   not negligible, the list shrinks rather than the seam disappearing.
