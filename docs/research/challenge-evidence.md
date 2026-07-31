# X2 — Adversarial audit of the consumer evidence and the integration story

**Status:** challenge deliverable. Attacks `plans/api-shape.md` (S1) and
`docs/research/consumer-{sinbad,cadabra2,solverang}.md` (E1/E2/E3).
**Method:** every citation reported below was opened in the source repository and read.
Claims I checked and found sound are listed in §4 by name, so this document can be read as
a coverage report and not only as a complaint list.
**Bottom line:** the *citation discipline is real* — I could not find a fabricated file, a
fabricated line range, or an invented benchmark anywhere in the three evaluations. What I
did find is (a) one integration decision that quietly forecloses the deferred
`arrangements` merge and is argued from a misattributed fact, and (b) three places where a
correct observation is written up more strongly than the evidence carries. The API itself
is general; nothing in it is consumer-shaped.

**Verdict on generality: general-with-fixes.** Not three special cases in a trenchcoat.

---

## 0. Severity index

| # | Severity | Target | One line |
|---|---|---|---|
| S1 | **serious** | `api-shape.md` §5.1, §1.4; `consumer-cadabra2.md` §10.4 | `resolvent-seam` collides with an existing zero-dep `scalar-seam` crate that cadabra2 *consumes* (it did not author it); the deferred merge gets more expensive, not less |
| S2 | **serious** | `api-shape.md` §1.4, §5.1 | `Error`/`Certainty`/`Budget` live in the zero-dependency seam crate, making resolvent's evidence vocabulary an ecosystem-wide import — a quiet violation of "additive and non-viral" |
| S3 | **serious** | `consumer-solverang.md` §0.2, §3.3 | The "18×–40×, a conservative lower bound" headline is refuted by the adjacent column of its own table; against a single-pass float baseline the modular echelon is 4.4× faster at n=200 and **2.9× slower** at n=800 |
| S4 | **serious** | `consumer-solverang.md` §4 #10; `api-shape.md` L2-10 | The Nullstellensatz certificate does not decide the failure class it is pitched at: CAD over-constraint is routinely complex-satisfiable and real-unsatisfiable, where `1 ∉ ⟨f⟩` |
| S5 | **serious** | `consumer-cadabra2.md` §4.6 | Eleven capabilities cadabra2 **ships today** are tagged `blocked-now`, contradicting the same document's own verdict header |
| S6 | **serious** | `api-shape.md` §1.3, §5 | `AlgebraicReal` is `!Sync` by decision while both incumbent types are `Sync`; the claimed "rename plus `&mut → &self`" merge is not a rename |
| S7 | **minor** | `api-shape.md` §4.4 | Certificate composition is demonstrated only in the trivial direction; the `Enclosure → Grade::Proven` arm launders a bound into a proof under sinbad's own D5 rule |
| S8 | **minor** | `api-shape.md` L0-11, L1-9, `resolvent-lazy` | Three named capabilities re-implement landed, first-party, same-license code (`lazy_exact::{Real, Bernstein, Interval}`, 1,453 lines) |
| S9 | **minor** | `consumer-sinbad.md` §0.4 | "The plan hardened to 'plexus needs a small CAS'" — the plan says "likely", "leans", lists it as open question #1, and tells Pantelides to reuse anvil's IR-AD three pages later |
| S10 | **minor** | `consumer-solverang.md` §2, §0.1 | Constraint census: `Spline2D` is an entity, not one of the 31 `impl Constraint` blocks; six constraints the document claims to have read are absent from its table |
| S11 | **minor** | `consumer-solverang.md` §0.2 vs §3.1 | "Produces the same rank" contradicts a page of argument that it deliberately answers a different question |
| S12 | **minor** | `consumer-sinbad.md` §0.2, §2 | Two off-by-a-sign / off-by-one citations (`poisson_sine` forcing; `anvil/src/lib.rs:19-20`) |

---

## 1. Job 1 — the evidence audit

### 1.1 The checks the brief specifically asked for

**anvil's deferred e-graph work is NOT resolvent's L4. E1 got this right, and it is the
best-supported analysis in the three documents.** I read the whole crate surface.

- `/home/dev/sinbad/crates/anvil/src/opcodes.rs:14` — `pub struct Reg(pub u16)`, a virtual
  register, not a node handle.
- `:41-58` — `ConstraintOp::LoadVar { dst, var_idx }` / `LoadConst { dst, value: f64 }`.
  The coefficient type is `f64` in the IR itself, not behind a parameter.
- `:278` `uses_register`, `:309` `defines_register` — both are `match` arms over the whole
  opcode enum, i.e. linear scans. There is no interning table, no structural hash, no
  sharing.
- `/home/dev/sinbad/crates/anvil/src/lower.rs:48-53` — `alloc_reg` bumps `next_reg` and
  returns; `load_var` / `const_f64` each mint a fresh register unconditionally. Confirmed:
  no CSE, no hash-consing.
- The transcendental opcode set E1 claims exists, exists: `Sqrt :111`, `Atan2 :135`,
  `Asin :207`, `Sinh :223`, `Tanh :239`.

E1's conclusion — "anvil should call `egg` directly; resolvent should be `egg`-*compatible*
and stop there" — is correct and the api-shape decision that follows from it (L4-8
out-of-scope) is well founded. **This is a genuine, load-bearing negative finding and it
survived the audit intact.**

One citation slip, cosmetic: E1 cites `crates/anvil/src/lib.rs:19-20` for the two deferrals.
`:19` is the sentence "Explicitly **not** attempted here"; the bullets are `:20-21`. (S12)

**Was a sinbad *plan* mistaken for a requirement?** Mostly no — E1 states its method
("read shipped source first, plans second… Where I cite a plan I say so, and I discount
it") and honours it. `/home/dev/sinbad` carries 240 markdown files and 41 plan documents in
`plans/`; E1's five headline findings rest on **code**, and I verified each:

- `crates/sinbad-testkit/src/mms.rs:69-70` — `u` and `laplacian` are two boxed closures;
  `:157-196` — every shipped solution is `sin`/`sinh`/`exp`/`cos`. Finding #2 (L4 must admit
  non-polynomial function symbols) is solid, and it is the finding that most changes
  resolvent's shape.
- `crates/residua/src/lib.rs:358-361` — `SourceField { per_region: BTreeMap<RegionTag,
  f64> }`, verbatim; `:924` — `let load = s * area / 3.0`. Finding #3 (MMS's real blocker is
  a numerical assembly seam) is **correct and is the most valuable finding in E1**, because
  it argues *against* resolvent's own interest.
- `crates/plexus/src/index_reduction.rs:1-6` — "**Not implemented in this slice.** These
  passes need a *symbolic differentiation* (`d/dt`) layer that does not yet exist in the
  federation." Verbatim. `NoStructuralPass` is the identity at `:57-62`. Finding #4's
  code-level claim is exact.
- `crates/meshwright/src/triangulate.rs:500-512` — the snap-to-53-bits comment, verbatim,
  including "blow up the bit-length and make exact predicates progressively, unboundedly
  slower". Finding #5 is exact.

Where E1 overstates is one clause. (S9) E1 §0.4: "The plan hardened to 'plexus needs a small
CAS' (`plans/dae-equation-compiler-architecture.md:283`, restated at `:620`)." The actual
text at `:283`:

> plexus **likely** needs a *small symbolic layer* of its own (Q6 **leans** "yes, a tiny
> CAS").

and the *same document* at `:592` instructs the opposite —

> the cost centers: Pantelides **differentiates equations symbolically** (must reuse
> solverang's IR-AD, not a second engine — Q6)

— and then lists the fork as **open question #1** ("(headline) Symbolic index reduction in
plexus vs numerical index-stabilization in solverang — which is the primary path? … Leaning:
numerical-stabilization *first*"). Nothing hardened. E1 does cite the numerical-first
leaning at `:578-582` and does say plexus "is not on the critical path", so the document as
a whole is honest; the word "hardened" is not.

**Fix:** replace "hardened to" with "leans toward, with the contradictory instruction still
live in the same document and the fork recorded as open question #1". This matters because
api-shape L4-3 (`diff_with` + `LeafRules`) is core status resting on this one consumer, and
the honest urgency is lower than E1's phrasing implies. L4-3 still earns core on clause (b)
— `d/dt` w.r.t. an implicit variable is a textbook CAS operation — so the placement does not
change, only the schedule.

**Is cadabra2 "blocked"?** E2's *header* answers this correctly and explicitly:

> **Why not `blocked-today`.** cadabra2 is not waiting on resolvent — it has a working
> substrate. Resolvent's proposition here is *substitution plus extension*, not unblocking.

I verified the critical-path claim: `ROADMAP.md` item B is topology publication and item C
(quadric-pencil SSI) has not entered; `ROADMAP.md:47-52` makes the whole-carrier enclosure an
entry-gate exit condition for item C, not for item B. So E2:17-19 is right.

Then §4.6's table undoes it. (S5) The table's `Urgency` column — a value never defined
anywhere in the document — tags **eleven** rows `blocked-now`, and at least seven of them are
capabilities cadabra2 *runs in production today*:

| Row | Tagged | Actually |
|---|---|---|
| #1 exact order of two algebraic numbers | `blocked-now` | ships — `lazy-exact/src/roots.rs:549` `cmp_root` |
| #2 sign of `α + β√h` at an algebraic abscissa | `blocked-now` | ships — `sign_radical2`, used at `sine_radical.rs:384` |
| #3 real root isolation with multiplicities | `blocked-now` | ships — `roots.rs:327` `isolate_roots` |
| #4 square-free part / gcd / `is_root_of` | `blocked-now` | ships — `roots.rs:480` |
| #5 Bernstein sup-norm enclosure | `blocked-now` | ships — `lazy-exact/src/bernstein.rs:108,123,135,157` |
| #17 interval arithmetic without global FPU state | `blocked-now` | ships — `lazy-exact/src/interval.rs` (431 lines) |
| #18 lazy filtered exact real | `blocked-now` | ships — `lazy-exact/src/real.rs` (724 lines) |
| #20 `Scalar`-generic seam | `blocked-now` | ships — `arrangements/crates/scalar-seam` |

The rows that *are* genuine lifts — things cadabra2 demonstrably fails closed on — are #9
(number-field linear algebra, `classification.rs:441-445`, verified verbatim), #12 (quadratic
form factorization, `carrier_cylinder_cylinder.rs:216-222`, verified verbatim), #13, #14
(`plane_torus.rs:24-27` and the refusal mint at `:373-378`, verified), #15, #16. That is six,
not seventeen.

**Fix:** retag the column `substitute-now / lift-now / eventual`. A roadmap that reads only
§4.6 will sequence eleven items as unblocking work that are in fact substitution work with
no user-visible outcome — the single most expensive misreading available from these
documents.

**Was the solverang evaluation actually skeptical?** Substantially yes, and in places
admirably so — §4.1 is a list of four falsification attempts that *failed*, §1.1 records an
"anti-finding" that the most vivid evidence for symbolic constraints is not evidence for
resolvent, §4.2 concludes "resolvent should not pitch L4 to solverang", and §4 rejects five
of ten candidates outright. I verified the load-bearing citations: `sketch2d/constraints.rs:3-5`
squared-formulation rule (verbatim), `system.rs:803` `implied_by: vec![]` (verbatim),
`Angle::new` baking `sin_a`/`cos_a` at construction (`sketch2d/constraints.rs:797-798` —
`sin_a: angle.sin(), cos_a: angle.cos()`), `redundancy.rs:257-297` the incremental-SVD loop
(verbatim), `graph/dof.rs:103` `let tolerance = 1e-10` (verbatim),
`tests/differential_oracle.rs:313` the `#[ignore]`d S5 divergence (verbatim).

**The half-angle question is handled honestly and the answer is correct.** The brief worried
that `sin`/`cos` in residuals make the constraint set non-polynomial and that `t = tan(θ/2)`
has real costs. E3's answer — the angle is *constraint data* baked to `f64` at construction,
not a solver variable, so no substitution is needed; and SO(3) is already rationally
parametrized by the quaternion — is verified: `assembly/entities.rs:18-24` documents the
7-parameter quaternion body, `:127-134` the `qw²+qx²+qy²+qz²−1` companion constraint, `:210-226`
`quat_to_rotation_matrix` is quadratic in the components. All confirmed.

One overstatement: §0.1's "**It is already done, and it cost nothing**". The rational
parametrization is not free — it adds one variable and one degree-2 equation per body and
carries a 2:1 double cover, so solution *counts* double and any Bézout/BKK bound inflates.
For **rank**, which is the only demand E3 accepts, the cost really is nil, so the conclusion
holds; "it cost nothing" should read "it costs nothing for the rank computation, and a
doubled solution count for any solving computation E3 rejects anyway".

Where E3 **did** talk itself into something is S3 and S4 below. Those are the two real
failures of skepticism in the document.

### 1.2 S3 — the speedup headline is refuted by its own table

`consumer-solverang.md` §0.2: "**18× faster at 200×200, 40× faster at 800×800**, against
LAPACK", and §3.3: "**Net: the 18×-at-200 / 40×-at-800 figure is a conservative lower bound
on the speedup**."

Proxy A's table has two columns. The 18×/40× is computed against the *incremental loop*
column — k SVDs on a growing row set, which is a bad algorithm regardless of arithmetic. The
right baseline for a **single-pass** modular echelon is the *single-pass float* column, which
is sitting right next to it:

| m = n | two full SVDs (float, single pass) | GF(p) echelon | ratio |
|---|---|---|---|
| 200 | 130 ms | 29.3 ms | **4.4× faster** |
| 400 | 333 ms | 228 ms | 1.5× faster |
| 800 | 639 ms | 1.86 s | **2.9× slower** |

So at the largest size measured the accepted demand is *slower* than the float baseline, and
"a conservative lower bound on the speedup" is false. E3 half-knows this — §3.4 says "A
single pass instead of k passes. Structural, not constant-factor" — but never connects it to
the headline, and never considers the obvious float alternative that also delivers one pass:
a column-pivoted QR gives rank, pivot columns, dependent rows *and* a dependency certificate
in floats, in one pass. That alternative is not mentioned anywhere in E3, and it is the
comparison a skeptic owes this finding.

E3's own mitigations are fair (Montgomery/delayed reduction is worth 5–10×; nalgebra's SVD
is slower than LAPACK; real Jacobians are sparse). They may well restore the win. But they
are *unmeasured*, and the document presents the un-mitigated number as a floor.

**Fix, and it does not change any API decision.** Restate finding 2 as: *"replaces a k-pass
SVD loop with a single pass; against a single-pass float baseline the unoptimized modular
echelon is 4.4× faster at n=200 and 2.9× slower at n=800. The durable wins are exactness at
near-degenerate configurations and the `implied_by` certificate, not wall-clock."* api-shape
L2-5 keeps core status untouched — it is a two-consumer capability and the echelon transform
is the same object as a Gröbner cofactor — but the roadmap should not carry a 40× into a
priority argument.

### 1.3 S4 — Nullstellensatz does not decide the failure it is sold against

`consumer-solverang.md` §4 #10 calls `1 ∈ ⟨f₁…f_k⟩` "**Eventual, and the most attractive new
capability**", "a *proof* of unsatisfiability with a checkable witness — **stronger diagnosis
than the licensed reference oracle produces**". `api-shape.md` L2-10 carries it forward as
"the most attractive *new* capability found in E3".

Two problems, one mathematical and one factual.

**(a) Wrong certificate for the geometry.** Hilbert's Nullstellensatz certifies emptiness of
the **complex** variety. CAD over-constraint is routinely complex-satisfiable and
real-unsatisfiable. Concrete instance in solverang's own vocabulary: fix `A = (0,0)`, and
impose `DistancePtPt(A,B) = 1`, `DistancePtPt(B,C) = 1`, `DistancePtPt(A,C) = 5` — three
squared-distance residuals (`sketch2d/constraints.rs:18`) in four unknowns. No real
configuration exists (triangle inequality), but the complex variety is positive-dimensional
and non-empty, so `1 ∉ ⟨f₁,f₂,f₃⟩` and **no Nullstellensatz certificate exists**. D-Cubed
would report `NOT_SATISFIED`. Real infeasibility needs a Positivstellensatz / SOS witness or
a CAD, which is a different, far more expensive object and appears nowhere in resolvent's
plan (`api-shape.md` L2-13/L2-15 push convex geometry and interval methods out of scope, and
those are the neighbourhoods where a real certificate would live).

E3's own worked example happens to be in the easy subclass: `distance=10` and `distance=7` on
the same pair gives `dx²+dy²−100` and `dx²+dy²−49`, whose difference is the unit `51`, so
`1 ∈ ⟨f⟩` trivially. Generalizing from that instance to "the correct statement of
unsatisfiability" is the error.

**(b) It would not "beat" D-Cubed.** `TODO.md:206-214` and
`tests/differential_oracle.rs:305-313` both show D-Cubed getting this **right**
(`NOT_SATISFIED`) and solverang getting it wrong, for a reason solverang's own TODO diagnoses
as a status-classification bug: "`solverang` *does* detect the conflict via
`analyze_redundancy().conflicts`, so the correct classification is obtainable — but
`solve().status` alone is not a satisfaction certificate." E3 §4.1 says exactly this and then
§4 #10 says the opposite.

**Fix:** in E3 §4 #10, replace "a *proof* of unsatisfiability" with "a proof of *complex*
inconsistency, which is a strict subclass of the CAD-relevant failure"; delete "stronger
diagnosis than the licensed reference oracle produces". In `api-shape.md` L2-10, add one
sentence: *"the Nullstellensatz certificate decides complex inconsistency only; real
infeasibility is not decided by it and resolvent does not ship a Positivstellensatz."*
Costs two lines now; prevents a milestone being justified on a capability that does not
answer the question.

### 1.4 S10, S11, S12 — the small stuff

**S10 — census.** There are exactly 31 `impl Constraint for` blocks, and E3's split (18
sketch2d / 8 sketch3d / 4 assembly-constraints / 1 assembly-entities) is exactly right. But
`Spline2D` is an **entity** (`crates/solverang/src/sketch2d/entities.rs:510`, `pub struct
Spline2D`, no `impl Constraint`), so §0.1's "Exactly three are not [polynomial]" should be
two — `Gear` and `Insert`, both verified at `assembly/constraints.rs:520-522` (`len_sq.sqrt()
.max(1e-15)`) and `:633-635` (`2.0 * sin_half.atan2(cos_half)`). Separately, §2.1 claims "I
read every `impl Constraint` block" but the table omits six: `Collinear` (`:1477`),
`SymmetricAboutLine` (`:1593`), `Coincident3D`, `Fixed3D`, `Coplanar`, `Coaxial`
(`sketch3d/constraints.rs:903`). All six are plausibly polynomial so the ~90% conclusion is
safe; the table should list them.

Also missed, and it *strengthens* E3's best finding: `implied_by: vec![]` has a **second**
unconditional site at `crates/solverang/src/pipeline/analyze.rs:98`. Cite both.

**S11 — internal contradiction.** §0.2: a `GF(p)` echelon "produces **the same rank, the same
dependent-row set**". §3.1: the exact-rank-at-current-floats question is "the wrong question"
and generic rank answers a different one. Both cannot hold. Substituting generic rank for
numerical rank is a **semantics change**: a configuration that is genuinely degenerate right
now but generically full-rank flips verdict, and §3.1 rightly says the disagreement is itself
the diagnostic. Delete "the same rank, the same dependent-row set" from §0.2; the finding is
stronger without it.

**S12 — sign and line-number slips.** `consumer-sinbad.md` §0.2 calls `−2π²·sin(πx)·sin(πy)`
the *forcing* of `poisson_sine`; `crates/sinbad-testkit/src/mms.rs:183-196` shows that value
is the stored **Laplacian** and the doc comment gives the forcing as `f = −∇²u* = +2π²·sin·sin`.
And the anvil deferral bullets are at `lib.rs:20-21`, not `:19-20`. Neither affects an
argument.

---

## 2. Job 2 — attacking the integration story

### 2.1 S1 — the seam crate is the foreclosure, and it is argued from a misattribution

This is the most important finding in the document.

`api-shape.md` §5.1 names `resolvent-seam` "**the single highest-leverage hook**" for
ecosystem integration and defends it as non-speculative:

> it is not speculative: cadabra2 already built its own `scalar-seam` for exactly this and
> renders T0/T1/T2 from one `TierField` program (E2 §10.4).

**cadabra2 did not build `scalar-seam`.** It lives at
`/home/dev/projects/arrangements/crates/scalar-seam` — in the *sibling repository whose
merge with resolvent is the deferred decision* — and cadabra2 consumes it by path:

```toml
# /home/dev/projects/cadabra2/Cargo.toml:39-40
scalar-seam = { path = "../arrangements/crates/scalar-seam" }
lazy-exact  = { path = "../arrangements/crates/lazy-exact" }
```

Its own module header states its purpose and its layering, and it is *not* cadabra2's:

> The SINBAD ecosystem wants numeric kernels … generic over the scalar … The trait that
> expresses that seam has to sit somewhere BOTH the exact backend (`lazy-exact`) and the
> ecosystem (`~/sinbad`) can depend on *downward*, with no repository cycle … the seam lives
> here, in `~/projects`, as a **zero-dependency leaf crate**.
> — `arrangements/crates/scalar-seam/src/lib.rs:5-17`

It is MIT OR Apache-2.0 (`arrangements/Cargo.toml:9`), 257 lines, zero dependencies, and it
already ships `Dual<S>` (`scalar-seam/src/dual.rs`, 412 lines) — which `api-shape.md` L0-10
defers as "one consumer, eventual … ~60 lines outside resolvent once §3's seam ships".

`TierField` is a **third**, separate vocabulary and it is crate-private:
`cadabra2/crates/cadabra-algorithms/src/fastpath/filter.rs:32` — `pub(crate) trait TierField:
Clone`. And there is a **fourth**: `lazy_exact::exact::{RingOps, ExactRing, ExactField}`
(`arrangements/crates/lazy-exact/src/exact/mod.rs:16-29, 58-72`).

Now compare that last one to what api-shape §3.3 proposes:

| `lazy_exact::exact` (shipped) | `resolvent-seam` (proposed) |
|---|---|
| `RingOps: zero, from_i32, add(&), sub(&), mul(&), neg(&)` | `Scalar: zero, one, add(&), sub(&), mul(&), neg(&)` |
| `ExactRing: RingOps + Clone + Ord` + `from_f64 -> Option`, `sign() -> Sign`, `to_interval() -> Interval` | `ScalarOrd: Scalar + sign() -> Sign`; L0-4 `enclosure()`; L0-2 `try_from_f64` |
| `ExactField: div(&)` — **panics on zero** | `TryDiv: try_div(&) -> Option` |

Resolvent's seam is a near-exact re-derivation of the incumbent with the two panics fixed.
That is a *good* design — the fixes are right and `scalar_seam::Scalar::from_f64` panicking
on NaN on the exact rung (`scalar-seam/src/lib.rs:104-109`) plus `to_f64` as a lossy readout
(`:111-115`) genuinely violate resolvent's INV-4 and INV-7, so resolvent cannot simply adopt
`scalar-seam` as-is. But shipping it as a *new, unrelated* vocabulary, in ignorance of the
incumbent, produces the exact collision the brief told this track to avoid:

- **Orphan rule.** A geometry crate cannot `impl resolvent::Scalar for lazy_exact::Real`
  unless it owns one of the two. Neither repo owns both. A glue crate becomes mandatory —
  and §5's "the dependency arrow" says glue crates are owned by whoever wants them, which
  means *nobody*.
- **Every ecosystem crate written against `scalar_seam::Scalar` must be rewritten** to get
  resolvent's exact rung, or must carry both traits.
- **The deferred merge gets harder, not easier.** api-shape §5 claims the opposite: "A future
  merge is a rename plus an `&mut self → &self` fix, not a redesign. Nothing in this document
  makes that merge more expensive." Two competing `Scalar` traits, each with impls in
  downstream crates, is precisely a thing that makes a merge more expensive.

**Is there a hidden dependency inversion?** Yes, and this is it. Resolvent takes no
dependency on the ecosystem — INV-11 holds literally. But the *integration story* only works
if the ecosystem adopts resolvent's trait vocabulary **in its lowest layer**, displacing an
incumbent that is already zero-dependency, already same-license, already consumed by two
repositories, and explicitly designed for that role. "Plug right in" as written means "the
ecosystem re-plugs into us." That is a real cost and the plan records it as a benefit.

**What to do now, at zero cost, preserving every option:**

1. **Fix the attribution** in `api-shape.md` §5.1 and `consumer-cadabra2.md` §10.4:
   `scalar-seam` is first-party to `arrangements`, MIT OR Apache-2.0, consumed by cadabra2,
   designed as the sinbad↔lazy-exact seam. State its path.
2. **Declare `resolvent-seam` a candidate successor to `scalar-seam`, not an independent
   invention**, and record the three deltas that justify a new crate rather than adoption:
   fallible `from_f64`, fallible `div`, no lossy `to_f64` on the trait. Those are exactly
   INV-4/INV-7/INV-8. Written down, a later merge is a diff; unwritten, it is an archaeology
   project.
3. **Make resolvent's traits blanket-implementable from the incumbent.** Keep `Scalar`
   at by-reference `&self` methods (it already is, matching `RingOps` byte-for-byte on five
   of six) and add a one-line note that `impl<T: lazy_exact::exact::RingOps> resolvent::Scalar
   for T` is a legal blanket impl *from a glue crate* — which is only true if resolvent's
   `Scalar` does not require `Send + Sync + 'static`. It currently does not. **Do not add
   those bounds**, even though `scalar_seam::Scalar` has them, or the blanket impl and the
   ecosystem's trait-object usage become mutually exclusive.
4. **Add an item to `api-shape.md` §8 (what this document does not settle):** *"Whether
   `resolvent-seam` should supersede, wrap, or coexist with
   `arrangements/crates/scalar-seam`. Coexistence is the default and is the most expensive
   of the three; it is chosen by inaction."*

### 2.2 S2 — `Error`, `Certainty` and `Budget` do not belong in the seam crate

`api-shape.md` §1.4 puts `Sign, Scalar, ScalarOrd, TryDiv, Hom, Budget, Error, Certainty` in
`resolvent-seam`, and §5.1 wants the whole ecosystem to depend on that crate for the numeric
surface alone. The consequence: a geometry crate that wants `Scalar` also imports resolvent's
**evidence vocabulary**.

Both consumers already own one, and they are not the same shape:

- sinbad: `Grade::{Heuristic, Estimated, Proven, Measured}` with a lattice in which `Proven`
  and `Measured` are deliberately **incomparable** (`sinbad/crates/tiered-core/src/grade.rs:
  6-12, 44-55`, verified verbatim).
- cadabra2: `ProofStrength::{AlgebraicallyExact, IntervalEnclosed}` plus a separate
  `Record → Verified → Ready` readiness ladder.

Making `Certainty` a transitive import of the ecosystem's lowest numeric layer makes
resolvent's two-valued grading the de-facto standard by adoption rather than by argument.
That is the failure mode brief rule 6 names — integration that is not additive. Note that
`scalar-seam`, the incumbent, is scrupulous about this: it carries the numeric surface and
`Dual`, and nothing epistemic.

**Fix, free today:** split the crate.

```
resolvent-scalar   ZERO deps. Sign, Scalar, ScalarOrd, TryDiv, Hom.  ← the ecosystem hook
resolvent-error    Error, DomainFault, Op, Budget, Certainty, ProofKind.  ← resolvent's own
```

`resolvent-error` may depend on `resolvent-scalar`; nothing else changes. INV-9 is unaffected.
A consumer that wants only the seam takes only the seam. Doing this after publication is a
breaking split of a crate the ecosystem depends on.

### 2.3 S6 — `!Sync` is a genuine foreclosure and the merge claim is wrong

`api-shape.md` §1.3 fixes `AlgebraicReal` as `Send + !Sync` with an inline
`cache: RefCell<Isolation>`, calls `!Sync` "the price", and §5 concludes the future merge is
"a rename plus an `&mut self → &self` fix, **not a redesign**".

The incumbents are `Sync`:

- `lazy_exact::RealRoot { poly, lo, hi, multiplicity }` (`roots.rs:317-322`) is a plain
  `Clone` struct with no interior mutability — auto `Send + Sync`. The refinement is
  `&mut self` (`:450`, `:472`, `:549`), which is what forced `Rc<RefCell<_>>` on the *consumer*
  (`arrangements/crates/arrangements/src/geoms/sine_radical.rs:70-88`, verified verbatim
  including the `Rc::ptr_eq` guard, and mirrored in `cadabra-arrange/src/trim.rs:857-862`).
  E2 §4.1's diagnosis of the defect is exactly right.
- `lazy_exact::Real` is deliberately, expensively `Sync`: `Arc`-shared nodes, `AtomicU64`
  interval cache, `Mutex` per node, with a documented five-step lock protocol and a proof
  sketch that no waits-for cycle can form (`real.rs:1-16, 25-45`). This is not incidental
  thread-safety; it is the crate's headline concurrency design.

So a merge is **not** a rename. It is a choice: either the merged algebraic-number type gives
up `Sync` (breaking any parallel arrangement/sweep work that relies on `Real`'s protocol), or
resolvent's `RefCell` becomes a lock or an atomic pair. api-shape does not acknowledge the
fork.

**What costs nothing now.** Keep the *public* decision — `&self` comparison, `impl Ord`,
guard inside resolvent — and make the *cache mechanism* a private implementation detail that
can be swapped in one file:

```rust
pub struct AlgebraicReal {
    poly:  Arc<SqfrPoly<Rational>>,
    cache: IsolationCache,   // private newtype; RefCell today, Mutex/atomics later
    mult:  u32,
}
```

with a one-paragraph note in §1.3: *"`!Sync` is a consequence of today's `IsolationCache`
implementation, not of the API. The public signatures (`cmp(&self)`, `sign_of(&self, …)`)
are already compatible with a `Sync` cache; swapping `RefCell` for a lock or a monotone
atomic pair is a single-file change and no consumer signature moves. The choice is deferred
because the incumbent `lazy_exact::Real` is `Sync` and a merge would have to pick one."*
That is three sentences and it converts a foreclosure into a deferral, which is exactly what
the brief asked for.

Add a matching invariant edit: INV-15 currently reads "`AlgebraicReal` is the single
documented non-`Sync` type" as if that were a design goal. Reword to "…is the single type
whose thread-safety is deliberately unpinned pending the arrangements decision."

### 2.4 S8 — the duplication is bigger than §8.1 admits, and it is all first-party

`api-shape.md` §8.1 flags only `resolvent-lazy` as unsettled. The actual overlap with landed,
MIT-OR-Apache-2.0, same-author code:

| api-shape item | Already exists | Lines |
|---|---|---|
| L0-11 / `resolvent-lazy` — filtered eager-interval / lazy-exact real | `lazy-exact/src/real.rs` — plus the CGAL failure modes it designs out (recursive `update_exact` stack overflow, torn atomic reads) | 724 |
| L0-5 — `Interval<f64>`, directed rounding, **no global FPU mode** | `lazy-exact/src/interval.rs` — same design rule, stated in `lib.rs:5-7` | 431 |
| L1-9 — Bernstein coefficients, exact de Casteljau subdivision, certified range enclosure | `lazy-exact/src/bernstein.rs` — `from_power :43`, `range_bound :108`, `range_interval :123`, `sign_over :135`, `subdivide_at :157`, `subdivide :193` | 298 |
| L0-10 — forward-mode `Dual<S>` over the exact rung | `scalar-seam/src/dual.rs` | 412 |
| L3-1 — `AlgebraicReal { defining_poly, isolating_interval, multiplicity }` | `lazy-exact/src/roots.rs:317-322` `RealRoot { poly, lo, hi, multiplicity }` | — |

That is **1,865 lines** of first-party, same-license, already-tested code that resolvent's
plan proposes to rewrite, of which api-shape acknowledges 724.

I am *not* recommending that resolvent depend on `arrangements` — the brief forbids it and
that constraint is correct. The honest position, and the one the user has explicitly deferred
rather than ruled out, is: **these should eventually merge, and resolvent should be built so
the merge is a move, not a rewrite.** Three things keep that cheap and cost nothing now:

1. **Say it in `api-shape.md` §5.** Replace "Nothing in this document makes that merge more
   expensive" (which S1 and S6 falsify) with an explicit inventory of the five overlapping
   items above and, for each, the one design decision that determines merge cost. That
   sentence as written is the most dangerous line in the plan, because it licenses future
   agents to stop thinking about it.
2. **Keep the shapes identical where they already are.** `RealRoot`'s field set,
   `Bernstein`'s method set, and `Interval`'s no-global-FPU-mode rule should be matched
   deliberately, not coincidentally. Where resolvent deviates (fallible constructors, `&self`
   comparison, `SqfrPoly` newtype, `Budget`) the deviation should be listed as *the fix*, so
   a merge is "take resolvent's version of these five methods".
3. **Do not build `resolvent-lazy` or `Interval` on the critical path.** Root isolation needs
   an eager interval filter internally; that is a private module, not a published tier. Ship
   `Interval` public only when a second consumer needs it, and let §8.1's question stay open
   instead of being answered by having written the code.

### 2.5 S7 — the certificate composition is real in one direction and asserted in the other

`api-shape.md` §4.4 shows two five-line mappings and calls the composition done. The easy
direction is genuinely fine: `Certainty::Probable => None` for cadabra2 is fail-closed and
correct, and a small closed `PartialEq` error enum really does map upward in ~20 lines —
verified against the existing precedent, `cadabra-core/src/exact/algebraic.rs:81-90`, which is
6 lines for `lazy-exact`.

The hard direction is not addressed.

**sinbad.** The sketch maps `Certainty::Proved(_) => Grade::Proven`, including
`ProofKind::Enclosure`. But sinbad's D5 rule — verified at
`sinbad/crates/tiered-core/src/lib.rs:21-22`, "unaccounted error-budget sources cap the
effective grade — a partial bound cannot masquerade as a total one" — means an *enclosure* is
`Proven` only if the enclosure accounts for the caller's entire error budget, which resolvent
cannot know. And `Grade::satisfies` (`grade.rs:44-55`) makes `Proven` a hard gate:
`Proven => matches!(self, Proven)`. A five-line adapter that promotes every resolvent
enclosure to `Proven` launders a bound into a proof at exactly the place sinbad's lattice
exists to prevent it.

**cadabra2.** C3 is a *three*-stage ladder (`Record` diagnostic → `Verified` proof → `Ready`
strict gate, `dual-path-architecture.md:38-41`, verified). §4.4 maps `Certainty` onto
`ProofStrength` only. The `Ready` gate is not a function of `ProofStrength`; it additionally
requires the whole-carrier contract (`ROADMAP.md:47-52`). Nothing composes it, and nothing in
resolvent should — but the plan should say the gate is the consumer's and stop implying two
mappings close the loop.

**Fix, ~6 lines of prose in §4.4:** (a) change the sinbad arm to
`Proved(ProofKind::Enclosure) => Grade::Estimated`, with a comment that promoting it to
`Proven` requires the *caller* to certify its budget is total; (b) add: *"These mappings are
illustrative, not normative. Resolvent's job is to make the distinction visible in the type.
Deciding what a given `ProofKind` is worth inside a consumer's lattice is the consumer's
judgement and resolvent must not ship a table that pre-empts it."* This is also the honest
answer to the brief's question — the composition is **real for the vocabulary and asserted
for the grading**.

### 2.6 What the design gets right, and should not be talked out of

Stated so this document is not read as uniformly negative. Each of these I checked against
source and each holds:

- **INV-11 is satisfied literally.** I grepped the whole of `api-shape.md` for consumer
  names in the proposed public surface. There are none: no type, trait, method, or feature
  flag mentions cadabra2, sinbad, solverang, or arrangements. Features are capability-named
  (`parallel`, `serde`, `lazy`). Rule 1 passes.
- **The two-consumer rule is applied mechanically and the one-consumer cases are labelled as
  such.** L0-8, L0-9, L1-9, L2-3, L2-4, L2-6 and L3-1 are all openly marked one-consumer and
  each carries an explicit clause-(b) argument. L3-1 in particular says "**One consumer**,
  stated plainly … Core status here rests entirely on clause (b)". That is the discipline the
  brief asked for, executed.
- **Excluding `sem1f-biquad-spike` from sinbad's demand set** (E1 §8, `api-shape.md` §2) is
  correct and is the kind of self-denying move that makes the rest credible: it is not a
  workspace member (verified — absent from `sinbad/Cargo.toml:5-45`) and it imports
  `cadabra_geom`. Counting it twice would have manufactured a two-consumer majority for
  algebraic extensions.
- **The L0/L3 merge really is cheap, and that is the good news on the deferred decision.**
  `AlgebraicReal { defining_poly, isolating_interval, multiplicity }` is field-for-field
  `RealRoot { poly, lo, hi, multiplicity }` (`roots.rs:317-322`); both sit on `dashu`; both
  repositories are MIT OR Apache-2.0 (`arrangements/Cargo.toml:9`, `cadabra2/Cargo.toml:17`);
  and `is_root_of` already has the `&self` signature resolvent wants (`roots.rs:480`) — only
  `cmp_root` is `&mut` (`:549`). **The foreclosure risk is entirely in the seam crate (S1),
  the seam crate's contents (S2), and `!Sync` (S6). It is not in the algebra.** Fix those
  three and every option — resolvent adopts a scalar seam / arrangements writes an adapter /
  eventual merge — stays genuinely open.
- **L2-14/L2-15/L2-16 (no numeric root polishing, no Krawczyk, no `eps`) are correctly out of
  scope**, and E2 §6.3's evidence is verbatim in the source
  (`quadric/roots.rs:11-12`, "no numeric root polishing enters the decision path").
- **The `MPoly` runtime-arity decision (L1-2/L1-3, INV-13) is the right one-way door call.**
  E3 R9's argument is verified against the code: per-constraint arity runs 2..14
  (`Parallel3D` 12 params, `assembly::Insert` 14), so a const-generic arity makes an adapter
  that builds rings from constraint data impossible. This is the single highest-stakes
  representation decision in the plan and the evidence for it is sound.

---

## 3. The change list

Ranked. Everything here is a text or naming change to be made **now**, before any code; none
of it costs implementation effort today and each becomes expensive later.

1. **S1** — Fix the `scalar-seam` attribution; declare `resolvent-seam` a candidate successor
   with three named deltas; forbid `Send + Sync + 'static` bounds on `Scalar` so a blanket
   impl from `RingOps` stays legal; add the coexistence question to §8.
2. **S2** — Split `resolvent-seam` into `resolvent-scalar` (numeric surface only) and
   `resolvent-error` (`Error`, `Budget`, `Certainty`, `ProofKind`).
3. **S6** — Put the isolation cache behind a private `IsolationCache` newtype; reword §1.3
   and INV-15 to say thread-safety is deferred, not decided.
4. **S3** — Restate solverang finding 2 with the single-pass float baseline (4.4× at 200,
   2.9× *slower* at 800) and name column-pivoted QR as the unconsidered float alternative.
5. **S4** — Downgrade the Nullstellensatz claim to complex inconsistency; delete "beats the
   licensed reference"; add the caveat to `api-shape.md` L2-10.
6. **S5** — Retag `consumer-cadabra2.md` §4.6's urgency column
   `substitute-now / lift-now / eventual`; only six rows are genuine lifts.
7. **S8** — Replace §5's "Nothing in this document makes that merge more expensive" with the
   five-item overlap inventory; keep `resolvent-lazy` and public `Interval` off the critical
   path.
8. **S7** — Change the sinbad `Enclosure` arm to `Grade::Estimated`; mark both §4.4 mappings
   illustrative, not normative.
9. **S9, S10, S11, S12** — the wording and census corrections above.

---

## 4. Coverage — what I opened

So this can be audited rather than trusted.

**sinbad** (`/home/dev/sinbad` @ `d5726c8`): `Cargo.toml:1-45`; `crates/anvil/src/lib.rs:1-60`,
`opcodes.rs:1-70, 111-333`, `lower.rs:40-110`; `crates/plexus/src/index_reduction.rs:1-70`
and `src/` listing; `crates/sinbad-testkit/src/mms.rs:25-40, 60-80, 150-200`;
`crates/residua/src/lib.rs:285-300, 353-365, 918-932, 1085-1120`;
`crates/meshwright/src/predicates.rs:28-100`, `triangulate.rs:495-540`;
`crates/tiered-core/src/grade.rs:1-60`, `rung.rs:1-30`; `crates/sinbad-pal/src/repro.rs:1-30`;
`plans/dae-equation-compiler-architecture.md:275-290, 575-600`;
`SINBAD-ACCELERATION-SWEEP.md:175-180`.

**cadabra2** (`/home/dev/projects/cadabra2`): `Cargo.toml:1-45`; `ROADMAP.md:33-66`;
`STATUS.md:75-85`; `docs/notes/design/dual-path-architecture.md:10-50`;
`docs/notes/design/ssi-boolean-plan.md:510-525`;
`crates/cadabra-algorithms/src/intersection/quadric/classification.rs:70-95, 430-450`,
`carrier_cylinder_cylinder.rs:210-225`, `rows/plane_torus.rs:1-30, 370-382`,
`fastpath/filter.rs:1-62`; `crates/cadabra-check/src/biquad.rs:25-45`;
`crates/cadabra-arrange/src/trim.rs:840-865`; grep for Gröbner/Buchberger/F4/ideal-membership
across `crates/` — zero hits, confirming E2 §6.1.

**solverang** (`/home/dev/projects/solverang`): `crates/solverang/src/sketch2d/constraints.rs:1-25,
765-815`, `entities.rs:500-520`; `sketch3d/constraints.rs` (impl census);
`assembly/entities.rs:18-30, 125-136, 205-230`, `assembly/constraints.rs:515-525, 540-565, 628-640`;
`graph/redundancy.rs:199-300`, `graph/dof.rs:100-110, 205-222`; `system.rs:795-810`;
`pipeline/analyze.rs:98`; `tests/differential_oracle.rs:305-320`; `TODO.md:185-245`; full
`impl Constraint for` census (31 blocks, enumerated).

**arrangements** (`/home/dev/projects/arrangements`): `Cargo.toml:6-14`;
`crates/scalar-seam/src/lib.rs:1-125` and `src/` listing;
`crates/lazy-exact/Cargo.toml`, `src/lib.rs:1-60`, `roots.rs:25-33, 310-345, 450-500, 549`,
`real.rs:1-45`, `ladder.rs:1-50`, `uncertain.rs:1-30`, `sqrt_ext.rs:30-50`,
`exact/mod.rs:16-29, 58-72`, `bernstein.rs` (public method census), line counts for
`bernstein.rs`/`real.rs`/`interval.rs`/`scalar-seam`;
`crates/arrangements/src/geoms/sine_radical.rs:70-95`, `conics.rs:265-290`.

**resolvent**: `plans/api-shape.md` (all 951 lines), `docs/research/consumer-sinbad.md`,
`consumer-cadabra2.md`, `consumer-solverang.md`; `/home/dev/projects/IDEAS-crates.md:100-175`.

**Not checked**, and therefore not attacked: `plans/{architecture,verification,roadmap}.md`
and `docs/research/{algorithms-and-representation,prior-art-and-licensing,consumer-requirements}.md`
— the R1/R3 references api-shape leans on (§1.3 F6, §2.2 L1-3, §3.4, §4.2 tier X) are cited
but I did not verify that R3 says what api-shape says it says. That is the obvious next
audit, and the one place a claim could be wrong without this document catching it.
