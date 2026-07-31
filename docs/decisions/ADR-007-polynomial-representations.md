# ADR-007 — Three polynomial representations; `UPoly<C>` is defined first and standalone

**Status:** Ratified 2026-07-31
**Reversibility:** one-way
**Amended:** 2026-07-31 — `MPoly` carries its ring by an **owned** handle, never `&'a Ring`
(ADR-020 §2; contradiction register item 8).
**Gates lanes:** U1, U2, P3, T1, T2.
**Evidence:** `docs/research/algorithms-and-representation.md` §2.1–§2.5;
`docs/research/consumer-requirements.md` §7 D6, §0.1;
`docs/research/critique-engineering.md` §2 item 8.

---

## Context

The source spec names one Layer-1 representation: "sparse distributed multivariate with
packed exponent vectors." That is the right representation for Gröbner bases and the wrong
one for everything the first consumer does.

Measured differences in access pattern (R3 §2.3):

- **Univariate root isolation** touches *every coefficient of every intermediate* at every
  subdivision node. Its cost model is bandwidth on a contiguous array plus bignum
  arithmetic. Its primitives — Taylor shift `x → x+1`, dyadic scaling `x → 2^k x`, Horner
  evaluation, Descartes sign-variation counting — all want O(1) indexed access into a dense
  array. A Taylor shift is a binomial transform; it is inherently dense.
- **Gröbner** touches the *lead term* of many polynomials and does random-access
  divisibility queries against a large index. Its cost model is cache misses on a hash
  table plus GF(p) arithmetic on sparse rows.
- **Subresultant PRS** is neither: it is univariate pseudo-division *parameterized over a
  coefficient domain* that is itself polynomial, and the specialization property that makes
  the modular scheme work is a statement about ring homomorphisms `D → D'`. It wants a
  recursive view.

There is no representation good at all three. Attempting one produces a library slow at
everything.

And a sequencing observation that is worth more than the performance argument: **the first
consumer touches none of the multivariate machinery.** Its polynomial type is
`Vec<Rational>` (`crates/lazy-exact/src/roots.rs:43-45`); its whole univariate toolkit
(`divrem` :143, `gcd` :169, `square_free_part` :186, Yun :199, `compose_affine` :232,
`reverse` :242, `variations` :249, `descartes_in` :270, `cauchy_bound` :291) is dense
univariate; its resultants are hand-rolled and specialized to degree 2 in the eliminated
variable (`conics.rs:272-287`). 100% of its usage is dense univariate of degree ≤ 8, and
≤ ~40 at arbitrary degree.

Storing a dense degree-8 univariate as `Vec<(packed_exp, coeff)>` costs ~2× memory and
loses the O(1) indexed access that Horner, Taylor shift, and Descartes all require.

---

## Decision

**Three concrete types with cheap, explicit conversions. `UPoly<C>` is defined first, is
standalone, and does not depend on the monomial machinery.**

```rust
UPoly<C>            // Vec<C>, low-to-high, trailing zeros trimmed.
                    // No monomial type. No order. No Ring context.
                    // Layer 2-univariate and all of Layer 3.

MPoly               // Vec<(MonomialId, C)>, sorted descending in the ring's order,
                    // + an OWNED handle to the Ring context (arena, order, width):
                    // Arc<Ring>, or an index into a caller-held ring table. Never &'a Ring.
                    // Layer 1-multivariate, F4, elimination.

RecursiveView<'a>   // a BORROWED view of an MPoly as D[x_main].
                    // Built on demand for subresultant PRS. Never owned.
```

**Conversions are explicit and one-directional in the dependency graph:** `MPoly` knows how
to produce a `UPoly` (extract a univariate; embed a `UPoly`). `UPoly` knows nothing about
`MPoly`, `MonomialId`, or `Ring`. The bridge that matters is
`Res_y: MPoly × MPoly → UPoly<Integer>`.

**`RecursiveView` is a view, not an owned tree.** Subresultant PRS needs the recursive shape
for its *control flow*, but the coefficients stay in the distributed arena. Building an
owned recursive tree is what makes classical PRS implementations allocate themselves to
death.

**Kronecker substitution (`x_i → y^(d^(i-1))`) is a utility, not a representation.** It
turns dense-support multivariate multiplication into one large univariate multiplication,
which is how multivariate products get FFT speed without a multivariate FFT. It lives
alongside the conversions.

---

## Consequences

- **The consumer track never depends on the multivariate one-way doors.** `resolvent-poly`
  can ship `UPoly<C>` and its whole toolkit while the monomial arena, packing, and order
  machinery are still being designed. The entire Layer-1-multivariate/F4 program becomes a
  genuinely parallel lane. This is the single largest sequencing win available and it is
  free — but only if the direction of the dependency is fixed now.
- **`MPoly` cannot be "the" polynomial type**, which means some code is written twice: gcd
  over `UPoly<Integer>` and multivariate gcd over `MPoly` are different functions. Accepted.
  They are genuinely different algorithms (Euclidean/modular vs Brown/Zippel recursion),
  not two spellings of one.
- **Consumers see two types and must pick.** Mitigated by the fact that the choice is
  obvious from the arity — a univariate problem uses `UPoly`, a multivariate one uses
  `MPoly` — and by conversions that are cheap and named.
- **`MPoly` carries an owned ring handle**, so it is `Send + Sync`, `'static`, and storable
  in a consumer's own struct without infecting that struct with a lifetime. *Amended
  2026-07-31:* this ADR originally said "`MPoly` carries a `&Ring` handle, which makes it
  not `'static`-free". That is superseded by ADR-020 §2 and by `API.md` INV-10 — a public
  owned type carries no lifetime parameter, and `/home/dev/projects/solverang`'s adapter
  must build rings **from data** (per-constraint arity runs 2..14), so the ring is a runtime
  value the polynomial must own its reference to. The *semantic* half of the original
  statement stands and is what the type should say: two `MPoly`s over different rings are
  not comparable, and `MonomialId` is meaningful only relative to the `Ring` that issued it.
  The cost of the owned handle — a refcount bump per clone and an indirection per term
  decode — is exactly what ADR-008's term-type microbenchmark measures.
- **`RecursiveView`'s borrow means PRS cannot mutate the underlying arena mid-chain.** That
  constrains the implementation and is the right constraint: PRS produces *new* polynomials
  in the arena, it does not rewrite its inputs.
- **Dense univariate over a *sparse* high-degree input is wasteful** — e.g. `x^10000 + 1`.
  Accepted and documented. Root isolation of such a polynomial is dominated by the
  subdivision tree anyway, and the workloads that produce it (cyclotomics, Kronecker
  substitution outputs) are handled by dedicated paths.

---

## Alternatives considered and why rejected

**One unified sparse-distributed type, univariate as the 1-variable case.** This is the
source spec's implied shape. Rejected on measurement and on sequencing: ~2× memory and loss
of O(1) indexed access on 100% of the first consumer's usage, plus it would make the
multivariate one-way doors (ADR-008, ADR-009) block the consumer track, throwing away the
sequencing gift for nothing.

**One unified dense type, multivariate as a dense tensor.** Rejected immediately. Gröbner
inputs are sparse in high dimension; Groebner.jl's largest Goodwin matrix is
403 677 × 374 837 at 0.0276% density. Anything that materializes that densely is dead.

**Recursive as the primary multivariate representation (`D[x_n]` nested).** Rejected as
primary: it degenerates into deep nesting of mostly-empty levels for genuinely sparse
high-variable input, and every Gröbner primitive ("the lead term with respect to a *global*
order") is awkward in it. Kept as the borrowed `RecursiveView` for exactly the algorithms
that want it.

**Two types (`UPoly` + `MPoly`), with PRS working directly on `MPoly`.** The closest
alternative. Rejected because subresultant PRS's control flow is genuinely univariate over a
coefficient domain, and expressing that against a distributed representation means
recomputing "the coefficient of `x_main^k`" at every step. The view costs one small struct
and makes the algorithm transcribable from the literature.

**Defining `UPoly<C>` as a type alias for a 1-variable `MPoly`.** Rejected — this is the
specific mistake the decision exists to prevent, because it inverts the dependency and
makes `UPoly` require a `Ring` context, an order, and a monomial arena to exist.

---

## What would reverse this

- **The dense-univariate representation proving wasteful on real inputs** — i.e. the
  resultants a geometry consumer generates turn out to be sparse in a way that matters.
  R2 §8 Q1's corpus measurement settles this. Response: add a *third* univariate storage
  behind the same `UPoly` API (dense/sparse as an internal enum), not a change to the
  three-type split.
- **`RecursiveView`'s borrow proving too restrictive** for a modular bivariate scheme that
  wants to evaluate-and-interpolate in place. Response: an owned `Recursive<C>` for that
  path specifically. It is additive.

Nothing reverses the "`UPoly` first and standalone" half. That is the one-way door, and its
reversal would mean re-plumbing every Layer-2-univariate and Layer-3 algorithm through a
`Ring` context they have no use for.
