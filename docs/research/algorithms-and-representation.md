# Algorithms and representation — technical reference (research lane R3)

Status: research input. Nothing here is ratified. This document exists so that the
architecture decisions in `docs/decisions/` can cite evidence instead of intuition.

Scope: the one-way doors (Layer 0/1 representation, modular-methods structure) in depth,
plus enough of Layer 2/3 to size the lanes and to say which lanes have certificates and
which only have numbers.

Two framing corrections up front, both evidence-backed below:

- **The spec's claim that packed monomials are "most of your Gröbner performance" is
  false as stated.** Measured, packing is worth ~15%. Sparse linear algebra over GF(p)
  is 73–91% of an F4 run, and the divisor-query index is worth 10–20× in Buchberger-style
  reduction. See §1.6.
- **The consumer that unblocks first (`arrangements`) touches none of the multivariate
  machinery.** It needs dense univariate over ℚ, resultants, root isolation, and exact
  comparison — Layers 0, 1-univariate, 2-univariate, 3. The Gröbner one-way doors do not
  gate it. See §2.5. This is a sequencing gift; do not throw it away by making the
  multivariate monomial type a prerequisite of the univariate type.

---

## 0. Licensing constraint on *reading*, not just on linking

MIT OR Apache-2.0 is load-bearing (project constraint #2). The consequence for this lane
is sharper than "pick permissive dependencies": **almost every mature implementation of
the Layer-2 algorithms is copyleft**, so the algorithms must be written from papers.

| Implementation | License | Has |
| --- | --- | --- |
| Singular | GPL | Buchberger/std, monomial packing (Bachmann–Schönemann) |
| msolve | GPLv2 (stated in its own paper, §1) | F4, FGLM, real root isolation |
| Groebner.jl | GPL (stated in its own paper, §1) | F4, packed monomials, multi-modular + tracing |
| FLINT | LGPL | van Hoeij factorization, modular GCD, fast Taylor shift |
| PARI/GP, Macaulay2, Giac | GPL | everything |
| SymPy | **BSD-3** | Zassenhaus, modular GCD, subresultant PRS, Gröbner (Buchberger) |
| SymEngine | **MIT** | expression DAG, some polynomial arithmetic |

SymPy is the only substantial permissively-licensed body of prior art, and it **does not
implement van Hoeij** — its factorization is plain Zassenhaus, and its own docs note that
"LLL-based techniques are not implemented (so worst case is not polynomial time)".

Direct consequence for the plan: **van Hoeij lattice recombination has no permissive
reference implementation anywhere.** It must be built from van Hoeij (2002), Klüners'
exposition, and Hart–van Hoeij–Novocin (ISSAC 2011). Budget it as the single hardest
correctness lane in Layer 2, and note that it needs an LLL — `lll-rs` (MIT) exists but is
unproven at the precision van Hoeij demands.

Bignum, for completeness (this is R2's lane, but it constrains algorithm choice): `dashu`
and `num-bigint` are MIT OR Apache-2.0; `malachite` is LGPL-3.0-only; `rug` is LGPL-3.0+.
GMP/FLINT-class speed is not available permissively. **This is an argument for pushing
work into machine-word GF(p) arithmetic and touching bignums only at CRT and
reconstruction boundaries** — which is exactly the modular-methods thesis, now motivated
by licensing as well as by coefficient growth.

The existing consumer already made this call: `arrangements` uses `dashu::rational::RBig`
(`/home/dev/projects/arrangements/crates/lazy-exact/src/exact/rational.rs:1-10`).

---

## 1. Monomial and exponent-vector representation

### 1.1 What actually has to be fast

A monomial type in a Gröbner engine is asked for five operations, in roughly this order of
call frequency:

1. **Compare** under the term order (sorting matrix columns, sorting polynomial terms,
   finding lead terms, priority queues of S-pairs).
2. **Divisibility test** `m | n` (symbolic preprocessing: "which basis element can reduce
   this monomial?" — asked myriad times per matrix).
3. **Multiply** `m·n` and **divide** `n/m` (S-pair construction, reducer multipliers).
4. **Hash / intern** (every monomial encountered must be canonicalized to one copy).
5. **lcm / gcd of monomials** (S-pair construction, Gebauer–Möller criteria).

The spec optimizes for (1). Measurements say (2) and (4) matter more, and that (1) is
mostly eliminated by a different trick entirely — see §1.6.

### 1.2 Capacity: what fits in 64 bits

Let the packing use fields of `w` bits with one bit per field reserved as an overflow
guard, so each field holds values in `[0, 2^(w-1) - 1]`. A 64-bit word holds `64/w` fields.
An `n`-variable monomial needs `n` fields for lex, `n+1` for grlex, `n` for grevlex (see
§1.4 — grevlex drops one exponent as redundant).

| field width `w` | payload max (= total-degree bound `D`) | fields / 64-bit word | vars in one word (grevlex layout) |
| --- | --- | --- | --- |
| 4 | 7 | 16 | 16 |
| 8 | 127 | 8 | 8 |
| 16 | 32 767 | 4 | 4 |
| 32 | 2 147 483 647 | 2 | 2 |

These numbers are corroborated by two independent implementations:

- Groebner.jl reports its packed representation "handles up to 31 variables and total
  degrees up to 127" — exactly 8-bit fields, 32 slots, 4 words.
- Singular's documented limits are "the maximal allowed exponent of a ring variable
  depends on the ordering of the ring and is at least 32767" (16-bit signed fields, 15-bit
  payload) and "monomial degree ≤ 2147483647" (32-bit fields for the degree word).

Note that Singular's limit is *order-dependent*. That is not an accident of implementation:
the number of fields the key needs depends on the order (§1.4), so the width that fits in
a given number of words does too.

**Practical reading**: for the resultant/elimination workloads a computational-geometry
consumer generates, `n ≤ 8` and `D ≤ 127` in a single word is comfortable. For Katsura-14
(15 variables) or Cyclic-9 you are at 2–4 words. For the 48–66 variable systems in the
literature (`yang1`, `mayr42`) packing buys little and unpacked `Vec<u32>` is the honest
representation — Groebner.jl says exactly this: "when the number of variables is large, we
use exponent vectors."

### 1.3 Overflow must be fail-closed, and it is cheap to make so

Wraparound in an exponent field silently corrupts both the comparison and the arithmetic,
and the corruption is *not* detectable downstream: the Gröbner basis you get back is a
correct basis of a different ideal. Every certificate in the verification thesis (§3) would
pass. This is the single most dangerous silent-failure mode in the whole library.

Three facts make fail-closed cheap:

1. **If every field has the same width, an overflow in any exponent field implies an
   overflow in the total-degree field**, because `a_i ≤ Σ a_j = deg` always. So one
   comparison of the degree field against `D` after each multiply suffices — no per-field
   check needed.
2. **SWAR guard bits catch it without even that.** Reserve the top bit of each field as
   zero. After a field-wise add of two packed words, `(sum & GUARD_MASK) != 0` iff some
   field carried into its guard bit. One AND and one compare per word.
3. **Overflow is recoverable by widening and restarting.** Exponents only grow during a
   run. On overflow: abort the computation, re-encode all monomials at `w' = 2w` (or fall
   back to unpacked), restart. The work lost is bounded by the run so far, and the
   *concept* (interned monomials, precomputed keys, id-based terms) survives unchanged.

**Therefore the field width is not a one-way door — only the interning/key/id structure
is.** The API must return `Result` (or have a `checked_mul` that the hot loop uses and a
`debug_assert`-free `unchecked` variant used only where a bound was proved), and the
top-level driver must own the widen-and-restart loop.

Rejected: `u8` exponents with wrapping arithmetic and "degrees never get that big in
practice." Rejected: panicking on overflow — a library that panics on a legitimate input is
not usable inside a geometry kernel that has its own error discipline.

### 1.4 Term orders, and why grevlex

Three orders matter. `a`, `b` are exponent vectors over `x_1 > x_2 > … > x_n`;
`|a| = Σ a_i`.

- **lex**: `a > b` iff the *first* nonzero entry of `a − b` is positive. Elimination-
  friendly (a lex GB solves triangularly), catastrophically expensive to compute directly.
- **grlex**: `|a| > |b|`, ties broken by lex.
- **grevlex** (degree reverse lexicographic): `|a| > |b|`, ties broken by: the *last*
  nonzero entry of `a − b` is **negative**.

**F4 wants grevlex, and the whole industry agrees.** msolve's pipeline is "Gröbner basis
computation w.r.t. the degree reverse lexicographical order, Gröbner conversion to a
lexicographical Gröbner basis" — i.e. compute in drl, then change order with FGLM, because
computing lex directly is fatal. Groebner.jl "specializes in computation in the degree
reverse lexicographical monomial ordering." Every benchmark table in §9 is drl.

The reason grevlex is cheap is structural, not implementational: for a homogeneous ideal
the grevlex GB is degree-by-degree minimal, the intermediate bases stay small, and the
last variable behaves like a generic linear form, which keeps the Macaulay matrices near
their minimum size. Computing a lex GB directly on a system where drl+FGLM takes seconds
routinely does not terminate in useful time.

**Consequence for the plan: a lex Gröbner path is not a Gröbner lane. It is
`drl-GB → FGLM change-of-order`, and FGLM (plus sparse-FGLM) is its own lane with its own
certificate (the resulting lex basis must reduce the drl basis to zero and vice versa).**

### 1.5 The comparison key: make compare order-independent

Here is the design that dissolves the "type parameter vs runtime data" question. Derivation,
because it is the load-bearing trick and it should be checkable by the reader:

Every order in the family above is a **matrix order**: fix an integer matrix `M` whose rows
are weight vectors, and compare `M·a` against `M·b` lexicographically. If every entry of
`M` is non-negative and the results are bounded, then `M·a` can be *packed big-endian into
machine words, most significant weight first*, and comparison of two monomials is exactly
`unsigned` comparison of the packed words — no order-specific branch at all.

- **lex**: `M = I`. Key `= (a_1, …, a_n)`, `n` fields.
- **grlex**: `M = [1…1; I]`. Key `= (|a|, a_1, …, a_n)`, `n+1` fields.
- **grevlex**: the naive matrix has a block of `−1`s, which breaks non-negativity. Replace
  it with complements. Claim: for `c ≥ D`, grevlex equals lex on

  `K(a) = ( |a|, c − a_n, c − a_{n-1}, …, c − a_2 )` — `n` fields, and `a_1` is dropped.

  *Proof.* If `|a| ≠ |b|` the first field decides, matching grevlex. If `|a| = |b|` and
  `a ≠ b`, let `k` be the largest index with `a_k ≠ b_k`. `k = 1` is impossible: it would
  force `a_j = b_j` for all `j ≥ 2`, and then `|a| = |b|` gives `a_1 = b_1`. So `k ≥ 2`,
  and scanning `K` from field 2 (which holds index `n`) the first difference is at index
  `k`. There `K(a) = c − a_k` and `K(b) = c − b_k`, so `K(a) > K(b)` iff `a_k < b_k` — the
  grevlex condition. ∎

  So grevlex needs no more fields than lex, and `a_1` is genuinely redundant given the
  degree.

**Multiplication in key space still works.** For lex and grlex the key is linear, so
`K(ab) = K(a) + K(b)` field-wise. For grevlex, `(c − a_i) + (c − b_i) = c + (c − (a_i+b_i))`,
so `K(ab) = K(a) + K(b) − C` where `C` is the constant packed word holding `c` in every
complement field and `0` in the degree field. **Two SWAR ops per word.** Underflow (i.e.
`a_i + b_i > c`) is caught by the same guard-bit test as overflow.

Divisibility `m | n` needs raw exponents (per-field `a_i ≤ b_i`), which in key space is a
per-field comparison with the direction flipped in complement fields. It is SWAR-able but
it is order-dependent, unlike compare.

**Recommended layout for an interned monomial** (evidence for interning in §1.6):

```
struct MonomialEntry {
    key:     [u64; W],   // order-normalized comparison key; compare = memcmp/word cmp
    raw:     [u64; W],   // raw packed exponents; divisibility, lcm, gcd, degree queries
    divmask: u64,        // Bloom-style filter for fast negative divisibility answers
}
// terms in a polynomial are (MonomialId: u32, Coeff), not (Monomial, Coeff)
```

Three words per *distinct* monomial, paid once, not per term. Groebner.jl and msolve both
do exactly the intern-and-divmask thing; msolve uses "hashing tables with linear probing"
for exponent vectors and "a divisor mask of 32-bits, if there are more than 32 variables we
just recognize the first 32."

**Now the type-parameter question answers itself.** The three candidates:

| Option | Compare cost | Cost you actually pay |
| --- | --- | --- |
| (a) Order as a type parameter `Poly<C, O: Order>` | inlined, static | Monomorphization across (orders × coefficient rings × widths). Heterogeneous storage impossible. Every consumer that reads an order from data needs a hand-written dispatch shim. FGLM becomes a type conversion (fine) but so does every debugging detour. |
| (b) Order as runtime data, branch/fn-ptr per compare | one predicted branch or an indirect call | Loses inlining inside sorts; the indirect call variant is genuinely slow in a comparison sort. |
| (c) **Order normalized into the key at intern time** | `u64` compare, no branch, no order | An order-specific encode at intern time, an order-specific constant subtract on multiply, an order-specific divisibility direction. All O(1), all outside the sort inner loop. |

**Recommendation: (c), with (a) available as a zero-cost newtype wrapper for callers who
want the order in the type system.** The order lives in the *ring/context* object (runtime
data), which is also where the field width, variable count, and coefficient ring live —
this is the same place Singular and msolve put it, and it is what makes widen-and-restart
(§1.3) possible without a recompile.

The seam this preserves matters for constraint #1: a consumer writing an adapter deals with
`&Ring` + `MonomialId`, not with a generic parameter that infects its own types.

### 1.6 Testing the spec's claim: what actually drives Gröbner performance

The spec asserts the monomial representation is "most of your Gröbner performance." Here is
the measured decomposition, from two independent sources.

**Where an F4 run's time goes** (Groebner.jl, Table 1, mod `2^30+3`, drl, seconds; the
table captures >99% of runtime):

| phase | Cyclic-9 | Eco-14 | Goodwin (w.) | Yang1 |
| --- | --- | --- | --- | --- |
| Pair selection | 4.07 | 1.95 | 20.57 | 1.68 |
| Symbolic preprocessing | 8.34 | 8.64 | 5.90 | 5.62 |
| **Linear algebra** | **242.03** | **168.83** | **284.01** | **7.73** |
| Pair update | 6.71 | 2.03 | 79.50 | 10.80 |
| Auto-reduction | 4.67 | 44.60 | 0.00 | 0.19 |
| Other | 0.47 | 1.40 | 0.09 | 0.98 |
| **LA share** | **91%** | **75%** | **73%** | **28%** |

**What packing itself is worth**: Groebner.jl, comparing packed vs. plain exponent vectors,
reports "the packed representation provides a total speed-up of **15%** for some problems
and slightly reduces memory consumption."

**What the divisor-query index is worth** (Roune & Stillman, Table 3, seconds, signature
Buchberger with all other optimizations held fixed):

| reducer-lookup structure | joswig101 | hcyclic8 | yang1 | mayr42 |
| --- | --- | --- | --- | --- |
| divmask + kd-tree (baseline) | 93 | 112 | 1333 | 273 |
| divmask + monomial list | 84 | 139 | 3917 | 835 |
| plain monomial list | 179 | 1270 | **> 8 hours** | > 30 min |

That is **11× on hcyclic8 and >20× on yang1**, from the divisibility index alone. Roune &
Stillman's own diagnosis of why their naive Buchberger beats mature systems on `yang1` and
`mayr42` is "the other systems do not use kd-trees for divisor queries and those two ideals
stress the divisor query infrastructure."

**And what the S-pair criteria are worth** (Roune & Stillman, Table 2, `yang1`): 1 998 099 720
S-pairs generated; 148 812 survive to need reduction; 63 150 produce basis elements. The
criteria eliminate 99.99% of the work.

**The actual ranking of what drives Gröbner performance:**

1. **Not doing the work**: Gebauer–Möller (and signature) criteria eliminating S-pairs — 4
   orders of magnitude.
2. **Sparse GF(p) linear algebra** — 73–91% of what remains in F4. AVX2 in msolve "can
   lower the time spent for linear algebra in F4 by more than the half."
3. **Divisor query index** (divmask + kd-tree or equivalent) — 10–20× in the reduction path.
4. **Monomial interning + a multiplicative hash** (`h(u) + h(v) = h(uv)`, so a product's
   hash is a sum) — makes the whole matrix-construction phase possible.
5. **Bit-packing the exponents** — 15%.

**And the decisive structural point the spec misses: in F4, monomial comparison mostly
disappears.** F4 sorts the monomials of a Macaulay matrix once to assign column indices;
after that, every inner-loop comparison is a comparison of small integers (column indices),
not of monomials. The packed-monomial compare is used in matrix *construction* and in the
S-pair queue, not in the 73–91% of the run that is elimination.

**Verdict on the claim:** the representation choice is a genuine one-way door — but
because everything above inherits the *interning and id* structure, not because compare
speed dominates. Restate the constraint that way in the architecture doc. A lane brief that
tells an agent "optimize monomial comparison" will get a 15% improvement and miss a 20×
one.

---

## 2. Sparse distributed vs recursive vs dense

### 2.1 The three shapes

- **Sparse distributed**: `p = Σ (m_i, c_i)`, a flat array of (monomial, coefficient)
  pairs kept sorted descending in the term order. Lead term is `p[0]`. This is what
  Groebner.jl uses ("a polynomial is represented with a pair of arrays: the array of
  monomials sorted with respect to the current monomial ordering and the array of
  coefficients"), and the coefficient array doubles as a sparse Macaulay-matrix row.
- **Recursive**: `p ∈ D[x_n]` where `D = R[x_1,…,x_{n-1}]`, itself recursive. A univariate
  polynomial over a coefficient *domain* that happens to be polynomial.
- **Dense univariate**: `Vec<C>`, coefficients low-to-high, no monomial type at all.

### 2.2 Where each wins

**Sparse distributed wins** whenever the algorithm's primitive is "the lead term with
respect to a global order": Gröbner bases, normal-form reduction, ideal membership, Macaulay
matrix rows. Also for genuinely sparse high-variable input, where the recursive form
degenerates into deep nesting of mostly-empty levels. Multiplication and division are done
with an auxiliary **heap of pointers** to bound monomial comparisons while keeping storage
linear (Monagan & Pearce, CASC 2007; this is the standard and it is what makes sparse
distributed division not quadratic).

**Recursive wins** whenever the algorithm is univariate but *parameterized over the
coefficient domain*:

- **Subresultant PRS** — the entire algorithm is univariate pseudo-division with
  coefficients in `D`, and the "specialization property" that makes modular/evaluation
  schemes work (§6) is a statement about ring homomorphisms `D → D'`.
- **Hensel lifting**, multivariate GCD by content/primitive-part recursion, evaluation
  and interpolation in one variable at a time (Brown, Zippel).
- **Anything with a distinguished main variable**, which is most of elimination theory.

**Dense univariate wins** for the whole of Layer 2-univariate and Layer 3:

- **Root isolation**: the Descartes/VCA subdivision step is a Taylor shift `p(x) → p(x+1)`
  plus a scaling. A Taylor shift is inherently a dense operation (a binomial transform), and
  the asymptotically fast version is a middle-product/FFT convolution. msolve's real solver
  is built on exactly this — "we only need the two basic operations: (i) shifting `x → x+1`
  … and (ii) scaling the coefficients by `x → 2^k x`" with FLINT's FFT multiplication, and
  the crossover for the fast Taylor shift "is around degree 512."
- **Factorization over ℤ[x]**: Hensel lifting and the van Hoeij lattice are dense.
- **Sylvester and Bézout matrices**.

### 2.3 Access patterns really do differ

Univariate root isolation touches *every coefficient of every intermediate polynomial* at
every subdivision node, `O(nτ)` times (§7). Its cost model is bandwidth on a contiguous
array plus bignum arithmetic. Gröbner touches the *lead term* of many polynomials and does
random-access divisibility queries against a large index. Its cost model is cache misses on
a hash table plus GF(p) arithmetic on sparse rows.

There is no representation that is good at both. Attempting one is the classic mistake that
produces a library slow at everything.

### 2.4 Recommendation

**Three concrete types with cheap, explicit conversions — not one generic type.**

```
DenseUni<C>       // Vec<C>. Layer 2-univariate, Layer 3. No monomial type. No order.
SparseDist<C>     // Vec<(MonomialId, C)> + &Ring. Layer 1-multivariate, F4.
RecursiveView<'a> // a borrowed view of SparseDist as D[x_main]; built on demand for PRS.
```

`RecursiveView` as a *view* rather than an owned type is the key economy: subresultant PRS
needs the recursive shape for its control flow, but the coefficients can stay in the
distributed arena. Building an owned recursive tree is what makes classical PRS
implementations allocate themselves to death.

Bridge: **Kronecker substitution** (`x_i → y^(d^(i-1))`) turns dense-support multivariate
multiplication into one large univariate multiplication, which is how you get FFT speed for
multivariate products without writing a multivariate FFT. Monagan & Pearce's GCD work uses
it; FLINT uses it in GCD. Add it as a utility, not as a representation.

### 2.5 What the first consumer actually calls — and why this matters for sequencing

Reading `arrangements` (context only; resolvent does not depend on it):

- `/home/dev/projects/arrangements/crates/lazy-exact/src/roots.rs:41-45` — `QPoly` is
  `Vec<Rational>`, dense univariate, low-to-high, trailing zeros trimmed. That is
  `DenseUni<Rational>` and nothing else.
- `.../roots.rs:143` `divrem`, `:169` `gcd`, `:186` `square_free_part`, `:199`
  `square_free_decomposition` (Yun), `:232` `compose_affine`, `:242` `reverse`, `:249`
  `variations`, `:270` `descartes_in`, `:291` `cauchy_bound` — the entire univariate
  toolkit, dense, over ℚ.
- `.../roots.rs:317-322` — `RealRoot { poly, lo, hi, multiplicity }`, i.e. exactly the
  spec's `AlgebraicReal { defining_poly, isolating_interval }` plus a multiplicity.
- Resultants are **hand-rolled inline and specialized to degree 2 in the eliminated
  variable**:
  `/home/dev/projects/arrangements/crates/arrangements/src/geoms/conics.rs:272-287`
  computes `Res = (p₂q₀ − q₂p₀)² − (p₂q₁ − q₂p₁)(p₁q₀ − q₁p₀)` for two conics, giving a
  degree-≤4 polynomial in `x`;
  `.../geoms/spherical_circle.rs:589` does the same for two latitude quadratics;
  `.../geoms/sine_radical.rs:614` gets to degree ≤8 by *double squaring* rather than by a
  resultant, because a general resultant was not available.

**So the consumer-unblocking API surface is: dense univariate over ℚ; `resultant(f, g, var)`
for bivariate input; `isolate_roots`; and an `AlgebraicReal` with `cmp`, `sign_of(h)`, and
`is_root_of(h)`.** No monomial order, no Gröbner basis, no ideal.

That means the fan-out can put the entire Layer-1-multivariate/F4 program on a *parallel*
track that never blocks the consumer track, provided the univariate type is not defined in
terms of the multivariate one. **Define `DenseUni<C>` first and standalone; let
`SparseDist` convert to it, not the other way round.** This is a cheap decision now and an
expensive one later.

---

## 3. Modular methods: the reduce → compute → reconstruct → verify loop

### 3.1 The shape, and what "Las Vegas" costs

The loop, for a computation over ℚ or ℤ:

1. Choose primes `p_1, p_2, …` (word-size; msolve and Groebner.jl both use 31-bit primes
   with 32-bit storage and 64-bit accumulators).
2. Reduce the input mod `p_i`; **detect and discard bad primes** (algorithm-specific,
   §3.2–3.4).
3. Compute in `GF(p_i)` — fast, no coefficient growth.
4. Combine images by CRT; lift to ℚ by rational reconstruction.
5. **Verify.** Without this the algorithm is Monte Carlo (an answer that is probably right,
   with no certificate). With it, it is Las Vegas (always right, running time random).

The verification step is what the spec calls "cheap." **That is true for GCD and
factorization and false for Gröbner.** The three cases must be planned separately.

Stopping rule matters too. Two families:

- **Bound-driven**: compute an a-priori bound on the size of the answer's coefficients
  (Landau–Mignotte for factors/GCDs, Hadamard for determinants/resultants), use enough
  primes to exceed `2·bound`, and the CRT result is provably the integer answer. Fully
  deterministic, sometimes wildly pessimistic.
- **Stabilization-driven**: keep adding primes until the reconstruction stops changing, then
  verify. The subresultant work does exactly this ("continuing to add modular images … until
  the reconstruction stabilizes"). Cheaper, but **stabilization alone is not a proof** — it
  is a heuristic that must be closed by a verification step.

**Design rule: every modular routine in resolvent returns a value plus a `Certificate`
enum — `Proved(reason)` or `Probable(reason)`. `Probable` is allowed to exist (Gröbner over
ℚ needs it), but it must be visible in the type, and the default path must be `Proved`.**
This is the "fail closed" version of the modular thesis. Note that msolve, Maple/FGb and
Groebner.jl all default to *uncertified* over ℚ — Groebner.jl says so explicitly: "The
multi-modular approach is Monte-Carlo probabilistic: there is no out of the box guarantee
that the reconstructed basis is correct over the rational numbers." If resolvent defaults
to certified, it will be slower than them on the same benchmark and that is the correct
trade; the benchmark harness must compare like with like.

### 3.2 GCD — verification is genuinely cheap

**Bad primes.** `p` is bad if `p | lc(A)` or `p | lc(B)` (degree drop — directly
detectable), or *unlucky*: `deg gcd(A mod p, B mod p) > deg gcd(A,B)`. The mod-`p` GCD can
only be *bigger*, never smaller, which gives Brown's detection rule: **compute images,
keep only those of minimal degree seen, discard the rest.** Unlucky primes are exactly
those dividing `res(A/G, B/G)`, a fixed nonzero integer, so they are finite in number.

The same argument one level down governs **unlucky evaluation points** in the multivariate
(Brown/Zippel) recursion: `α` is unlucky iff `R(α) = 0` where `R` is the Sylvester resultant
of the cofactors, and `deg R ≤ deg A · deg B` by Bézout — so random points from a large
field hit them with low probability, and the failure is caught downstream.

**The certificate.** Let `H` be the reconstructed candidate. Check:

- `H | A` and `H | B` — two exact divisions. This proves `H | G`.
- `deg H = deg gcd(A mod p, B mod p)` for one good prime `p`. Since `deg gcd(A mod p, B mod p) ≥ deg G`
  always, this gives `deg H ≥ deg G`.

Together: `H | G` and `deg H ≥ deg G` ⇒ `H = G` up to a unit. **Two polynomial divisions and
one modular GCD.** Cost is a small constant fraction of the GCD itself. Monagan & Pearce
rely on exactly this: "All cases of undetected failure are caught in Algorithm GCD by the
trial divisions `H|A` and `H|B`."

The spec's "GCD: check divisibility both ways" is right, and the degree half is the part
people forget — without it, divisibility alone would accept any common divisor.

**Lane grade: certificate. Fully self-certifying, cheaply.**

### 3.3 Factorization — self-certifying for the product, *not* for irreducibility

**Bad primes.** `p | lc(f)` or `p | disc(f)` (the mod-`p` factorization is not squarefree),
both directly detectable. Beyond that, `p` is merely *unhelpful* rather than wrong: a prime
where `f` splits into many small factors makes recombination harder but does not give a
wrong answer. Standard practice is to try several small primes and keep the one minimizing
the number of modular factors `r`.

**The certificate, in two halves.**

- *Half 1 — the factorization is a factorization.* Multiply the factors back and compare to
  the input. One polynomial multiplication. Trivially cheap. This is the spec's check.
- *Half 2 — each factor is irreducible.* **The product check does not test this at all.** A
  buggy recombination that merges two true factors produces `f = g·h` with `g` reducible,
  and half 1 passes. **An oracle that only multiplies back will silently accept a coarse
  factorization.** This must be written into the lane brief.

Cheap certificate for half 2, when it exists: exhibit a prime `p` with `p ∤ lc(f_i)`,
`p ∤ disc(f_i)`, such that `f_i mod p` is irreducible of degree `deg f_i`. Then `f_i` is
irreducible over ℚ. One modular factorization per candidate factor. **This certificate does
not always exist** — a polynomial whose Galois group contains no `n`-cycle (Swinnerton-Dyer
polynomials being the canonical example) factors nontrivially modulo *every* prime. For
those, fall back to: (a) degree-pattern consistency across many primes (a necessary
condition, not sufficient), (b) the algorithm's own termination argument (van Hoeij's
lattice, at sufficient Hensel precision, proves the recombination is complete), (c)
differential testing against another system.

**Lane grade: certificate for the product; partial certificate for irreducibility. Say so
explicitly in the lane brief, and make the oracle check both halves.**

### 3.4 Gröbner — this is where the modular thesis gets expensive

**Bad primes.** Arnold's characterization: `p` is good iff the set of *lead monomials* of
the GB mod `p` agrees with that over ℚ. You cannot check this directly (you do not have the
answer over ℚ), so the practical algorithm is Idrees–Pfister–Steidel's: compute GB mod
several primes, take a **majority vote over the lead-monomial sets**, discard the minority,
and run a **stabilization test** on the reconstruction. msolve does the equivalent with its
tracer: "If the first prime number for which we generate the tracer is a good prime number
we can be sure that only a finite number of other prime numbers exist such that the Gröbner
basis computed modulo these primes via applying the tracer is not correct."

**The certificate, and why it is not cheap.** Let `G` be the reconstructed candidate and `F`
the input generators, `I = ⟨F⟩`.

- *`I ⊆ ⟨G⟩`*: reduce every `f ∈ F` to zero modulo `G`. This is `|F|` normal forms **over
  ℚ**, with full coefficient blowup. Moderate cost, sometimes not moderate.
- *`G` is a Gröbner basis*: all S-pairs of `G` reduce to zero, over ℚ. This is
  approximately the cost of recomputing the basis. Expensive.
- *`⟨G⟩ ⊆ I`*: **this is the hard half and there is no cheap general certificate.** Arnold
  showed a Hilbert-function argument removes it for *homogeneous* ideals. Idrees–Pfister–
  Steidel proved a corresponding theorem for the non-homogeneous case with a global
  ordering — and Noro & Yokoyama subsequently showed that theorem needs an additional
  assumption. Any lane brief that says "verify the Gröbner basis" without saying *which of
  these three* it means is underspecified.

The spec's proposal — "check ideal membership via the stored cofactor representation
`f = Σ h_i g_i`" — is the direct route to `⟨G⟩ ⊆ I`: track, for each basis element `g`, its
expression in terms of `F`. It is a genuine certificate and it is checkable by an oracle
with no algebra beyond multiplication and addition. **Its cost is cofactor swell**: the
`h_i` are typically far larger than the `g_i`, in both degree and coefficient size, and
tracking them through F4's linear algebra means carrying an extra (dense) block of columns
through every elimination. Expect a large constant factor and a real memory risk.

**Recommended split**: two Gröbner modes.

- `groebner_certified` — tracks cofactors, returns `Proved`, is the differential-test and
  regression-suite workhorse, is not expected to be competitive.
- `groebner` — modular + tracing + majority vote + stabilization + a randomized check,
  returns `Probable`, is the performance lane.

And a cross-check that costs nothing: **the certified mode's output must equal the fast
mode's output on every regression instance.** That is a free oracle for the fast path, and
it is exactly the structure the AI-agent build model wants.

**Lane grade: `groebner_certified` is a certificate lane. `groebner` is a
number-to-optimize lane graded against `groebner_certified` plus external systems.**

### 3.5 Prime and evaluation-point selection, concretely

Both Groebner.jl and msolve use primes just under `2^31`, coefficients in `u32`, arithmetic
accumulated in `u64`. Three GF(p) arithmetic strategies are worth knowing, all from
Groebner.jl §4.2:

- **Generic**: Barrett-style reduction — precompute `m, s` with `⌊x/p⌋ = ⌊m·x / 2^s⌋`,
  replacing a division with a multiply and a shift.
- **Signed**: keep coefficients signed; after `x -= c*y`, do `x += p²` if `x < 0`.
- **Delayed**: exploit leading zero bits and reduce only when overflow is imminent.
  Groebner.jl's cut-off: "if the prime fits in 27 bits, we opt to use delayed arithmetic."

And one structural trick worth stealing at the design level: **batch the modular runs.**
Groebner.jl computes over `Z/p₁ × … × Z/p_N` as tuples, sharing all of the non-arithmetic
work (matrix construction, pair handling) across `N` primes and exposing SIMD. Measured
speedup vs. one-prime-at-a-time: "the ratio of the runtime of application in batches of
size 4 to the runtime of non-batched application is 1.43 < ratio < 2.44," i.e. up to ~2.7×
amortized; `N = 4` is their production choice, and `N = 8, 16, 32` gave no further gain.

This is a design constraint on Layer 0: **the coefficient-ring trait must admit a
`Zp4 = [u32; 4]`-style tuple ring**, or the batching trick is unavailable later. That is a
one-way door in the trait signature, cheap to keep open now.

---

## 4. F4 and Buchberger

### 4.1 What F4 actually is

Buchberger: maintain a basis `G` and a set of critical pairs. Repeatedly pick a pair
`(f, g)`, form the S-polynomial `S(f,g) = (lcm/lt f)·f − (lcm/lt g)·g`, reduce it to normal
form modulo `G`, and if nonzero add it to `G` and generate new pairs.

F4 changes exactly one thing: **instead of reducing one S-polynomial at a time by repeated
division, select a whole batch of pairs, build the matrix of all polynomials involved, and
row-reduce it.** Concretely:

1. **Select** a batch `L` of critical pairs (§4.2).
2. **Symbolic preprocessing**: collect every monomial appearing in the S-polynomial
   components; for each, search `G` for a divisor of it (this is the divisor query, §1.6);
   if found, add the corresponding shifted basis element as a *reducer* row, and recurse on
   its monomials. Terminates because monomials only decrease.
3. **Build the Macaulay matrix**: rows = the S-poly halves and the reducers, columns = the
   monomials, sorted descending in the order. Assign integer column indices — after this,
   the monomial order plays no further role in the step.
4. **Row-reduce over GF(p)**. Rows that reduce to zero are useless pairs. Rows with new lead
   columns are new basis elements.

The matrix is structured, and exploiting the structure is most of the implementation.
Groebner.jl's four-block picture: the reducer rows `A|B` can be permuted so `A` is upper
triangular (symbolic preprocessing produces at most one reducer per column); the S-poly rows
`C|D` are reduced by `A|B`, `C` vanishes, `D'` is then inter-reduced to `D*`, whose rows are
the candidate new basis elements.

### 4.2 Selection strategy

- **Normal strategy**: select all pairs whose lcm has minimal total degree. This is what
  Groebner.jl uses ("we use the normal selection strategy") and it is the right default for
  F4, because it makes the batch homogeneous in degree, which is what makes the matrix
  well-structured. For homogeneous input it is provably degree-by-degree optimal.
- **Sugar**: for inhomogeneous input, track a "sugar degree" (the degree the pair would have
  had if the input were homogenized) and select on that instead. Without sugar, the normal
  strategy on inhomogeneous input can select pairs in an order that produces large
  intermediate coefficients. Giovini et al., 1991.
- **Batch size** is a tuning knob with a real trade: bigger batches amortize symbolic
  preprocessing and expose more parallelism, but the matrix gets larger and the peak memory
  grows.

### 4.3 Gebauer–Möller criteria

The efficient installation of Buchberger's two criteria, applied at *pair-update* time
rather than at pair-selection time. Given a new basis element `g_t` and the existing pairs:

- **Criterion B (chain)**: drop the pair `(i,j)` if there is a `t` with `lm(g_t) | lcm(i,j)`
  and `lcm(i,t) ≠ lcm(i,j) ≠ lcm(j,t)`.
- **Criterion M**: among new pairs `(i,t)`, drop any whose lcm is a proper multiple of
  another's.
- **Criterion F**: among new pairs with equal lcm, keep one.
- **Coprime (Buchberger's first) criterion**: drop `(i,j)` if `lm(g_i)` and `lm(g_j)` are
  coprime.

Both msolve ("we use the Gebauer–Möller installation … in order to discard useless critical
pairs") and Groebner.jl use exactly this. The measured effect is in §1.6: 4 orders of
magnitude on hard instances.

Note from Roune & Stillman's Table 2 that the **coprime criterion is nearly worthless when
the chain criterion is already installed** (0–31 762 pairs out of ~2×10⁹ on `yang1`) —
worth knowing before an agent spends a week on it.

### 4.4 Where implementations die

In rough order of how often it is the actual cause:

1. **Zero reductions.** Rows that reduce to nothing are pure waste. Mitigations: GM criteria;
   the **tracer** (learn the useful rows on the first prime, then skip the useless ones on
   every subsequent prime — msolve §4.3, Groebner.jl §4.4); and Steel's **probabilistic
   linear algebra** (reduce random linear combinations of a block of `ℓ` rows; the chance of
   a spurious zero is ~`1/p`). Note msolve's Remark 4: **tracing and probabilistic linear
   algebra are mutually exclusive in the learn phase** — you cannot learn which rows reduce
   to zero if you never reduced them individually.
2. **Linear algebra throughput.** 73–91% of the run. Sparse row format, sparsest-row-first
   pivoting, delayed modular reduction, SIMD. msolve: AVX2 "can lower the time spent for
   linear algebra in F4 by more than the half."
3. **Divisor queries in symbolic preprocessing.** §1.6: 10–20×.
4. **Memory.** Groebner.jl: "while symbolic preprocessing corresponds to a smaller fraction
   of total runtime, it causes the most memory allocations." Their largest matrix on
   Goodwin (w.) is **403 677 × 374 837 with 41 698 725 nonzeros (0.0276% dense)**. Any
   design that materializes that densely is dead.
5. **Auto-reduction of the final basis.** Not free: 44.6 s of Eco-14's ~227 s (20%).
6. **Coefficient growth over ℚ.** The reason modular methods are structural and not an
   optimization. §9.
7. **Trying to compute a lex basis directly.** §1.4.

### 4.5 F5 and signature-based methods: trap or not?

**Verdict: a trap for a first implementation. Do not put it in the critical path.**

The evidence:

- **The two fastest open implementations both chose non-signature F4.** msolve: F4 +
  Gebauer–Möller. Groebner.jl: F4 + normal strategy + Gebauer–Möller. Neither is
  signature-based.
- **Roune & Stillman, who built a serious signature implementation, could not show it
  beating F4-based systems**, and diagnosed why: "we suspect that FGb and Magma are faster
  because they use F4 reduction, so we suspect that the comparison is not useful for
  determining if SB is faster than F5." The two paradigms are not directly comparable
  because signature-based reduction is (in the classical formulation) one-polynomial-at-a-
  time, and F4's win is batching.
- **The signature basis can be far larger than the reduced Gröbner basis.** Roune &
  Stillman on `yang1`: the SB has 63 216 elements; that is why SB is slow there.
- **Correctness is subtle.** F5's original formulation had termination issues that took
  years and multiple papers to settle (F5C, "modifying Faugère's F5 to ensure termination",
  GVW, Arri–Perry). For an agent-built codebase graded by oracles, an algorithm whose
  *termination* is a research question is the wrong shape.

When signatures *are* worth it, and worth recording as a future lane:

- Computing **syzygy modules** — SB gets the initial syzygy module essentially for free
  ("SB should be the best algorithm for computing Gröbner bases of syzygy modules").
- **Regular sequences**, where F5's criterion provably eliminates *all* zero reductions.
- Signature-based F4 hybrids exist (Eder), but they compound both implementations'
  complexity.

Recommended sequencing: Buchberger (reference implementation, the oracle) → F4 (the
performance path) → signatures only if a consumer demands syzygies.

---

## 5. Factorization: Zassenhaus and van Hoeij

### 5.1 Zassenhaus

For squarefree primitive `f ∈ ℤ[x]` of degree `d`:

1. Choose a prime `p` with `p ∤ lc(f)` and `f mod p` squarefree.
2. **Factor mod `p`** into `r` irreducible factors — distinct-degree + equal-degree
   splitting (Cantor–Zassenhaus), or Berlekamp. Polynomial time.
3. **Hensel-lift** the factorization from `p` to `p^k`, where `k` is chosen so that `p^k`
   exceeds twice the **Landau–Mignotte bound** on the coefficients of any factor of `f`.
   (Landau–Mignotte: any factor `g | f` of degree `m` has `‖g‖_∞ ≤ 2^m · ‖f‖_2`.)
4. **Recombine**: the true irreducible factors over ℤ correspond to *subsets* of the `r`
   lifted factors. Try subsets of size 1, then 2, …, testing each candidate product by trial
   division.

**The exponential blowup lives entirely in step 4**: `2^r` subsets in the worst case. Steps
1–3 are polynomial.

**The worst case is not artificial.** Swinnerton-Dyer polynomials — e.g. the minimal
polynomial of `√2 + √3 + √5 + …` — are irreducible over ℚ but split into factors of degree
≤ 2 modulo *every* prime. So `r ≈ d/2`, and Zassenhaus must exhaust essentially all `2^r`
subsets before concluding "irreducible". van Hoeij's motivating example is a degree-924
resolvent with `r = 84` modular factors; `2^84` is not a number.

Note the asymmetry that makes this worse than it looks: the exponential cost is paid
*exactly when the answer is "irreducible"*, i.e. on the most common input.

### 5.2 What van Hoeij buys

van Hoeij (2002) reformulates recombination as a **knapsack problem solved by lattice
reduction**. The idea:

- Each true factor `g` of `f` corresponds to a 0/1 vector `v ∈ {0,1}^r` selecting which
  lifted factors multiply to `g`.
- Certain **linear functionals of the roots** — traces (power sums), equivalently the
  low-order coefficients of the logarithmic derivative `g'/g` — are (a) computable from the
  lifted factors modulo `p^k`, and (b) *small integers* when `v` is a true factor vector,
  because they are actual coefficients/traces of a genuine integer factor bounded by
  Landau–Mignotte.
- So build a lattice containing the identity block (forcing 0/1-ish coordinates) alongside
  the trace data scaled by `p^k`. **Short vectors of this lattice are exactly the true
  factor vectors.** LLL finds them in polynomial time.
- Iterate: add more traces, re-reduce, until the lattice's reduced basis consists of `s`
  vectors that are 0/1 and partition `{1..r}`. Then the factorization is complete and each
  block is irreducible.

**What it buys**: the `2^r` search becomes polynomial. The 84-factor example becomes
routine. What it costs: LLL on a lattice of dimension ~`r` with entries of size ~`p^k`, and
`p^k` is large (the Hensel precision). Hart–van Hoeij–Novocin (ISSAC 2011) is the
practically-tuned version — "the first algorithm for factoring polynomials in ℤ[x] which is
both comparable to the best algorithms in practice and has a proven polynomial complexity
bound" — and it is what FLINT implements.

### 5.3 Plan implications

- **Zassenhaus first, van Hoeij second**, with an explicit `r` threshold (e.g. brute-force
  recombination while `r ≤ 10`, i.e. ≤1024 subsets; lattice above that). Zassenhaus is also
  the *oracle* for van Hoeij on small inputs: for `r ≤ 20` both must agree.
- **The Swinnerton-Dyer family is the generator for the hard-instance suite**, and it is
  parameterizable: `Π (x ± √p_1 ± √p_2 ± … ± √p_m)` has degree `2^m` and `r ≈ 2^(m-1)`.
  Degree 16 (`m=4`) already puts Zassenhaus at ~2^8; degree 32 at ~2^16; degree 64 at ~2^32
  — that is where the lane's grading threshold sits.
- **No permissive reference implementation exists** (§0). This lane must be built from
  papers and graded by (a) multiply-back, (b) irreducibility certificates where they exist,
  (c) differential testing against Pari/Sage as an external oracle (running the reference,
  not reading its code, is not a licensing problem).

---

## 6. Resultants and subresultant PRS

### 6.1 The classical algorithm

`Res(f, g)` = determinant of the Sylvester matrix; it vanishes iff `f` and `g` have a common
root (over the algebraic closure, with the caveat that leading coefficients must not both
vanish). Computing it as a determinant directly is a bad idea over ℤ (entry growth in
Gaussian elimination) and a worse idea over `D = R[x_1..x_k]`.

The classical route is a **polynomial remainder sequence**. Naive Euclidean PRS over ℚ has
exponential coefficient growth. The **subresultant PRS** (Collins; Brown–Traub) fixes the
growth to polynomial by dividing out an exactly-predicted factor at each step — the point
being that the intermediate remainders are, up to sign and a known scalar, the *subresultants*
of `f` and `g`, which are determinants of submatrices of the Sylvester matrix and therefore
have Hadamard-bounded size. The last nonzero element of the chain is (a multiple of) the
GCD; the degree-0 element is the resultant.

**Ducos (2000)** optimizes the inner pseudo-division to reduce both operation count and
memory traffic; it is the standard non-modular choice. A cache-friendly restructuring of
Ducos' optimization uses "11× and 3× less memory than the original Ducos algorithm
implemented in Maple" for degree up to 2000.

Also worth having: the **Bézout matrix**, which is `n×n` rather than `2n×2n` and is often the
better determinant route for small degree, and **Bareiss fraction-free elimination** as a
third, completely independent implementation.

### 6.2 The modular / evaluation–interpolation alternative

The **specialization property** is what makes this work: for a ring homomorphism `Ψ: A → B`
with `Ψ(lc(a)) ≠ 0 ≠ Ψ(lc(b))`, the subresultant chain of `Ψ(a), Ψ(b)` is the image of the
subresultant chain of `a, b`. So:

- For `a, b ∈ ℤ[y]`: compute the chain mod several primes, CRT.
- For `a, b ∈ ℤ[x,y]` (the bivariate case a geometry consumer actually needs): mod `p`,
  **evaluate `x` at `N` points**, compute `N` univariate subresultant chains over `GF(p)[y]`,
  **interpolate** in `x`, then CRT over the primes.
- The number of evaluation points is bounded a priori:
  `N ≥ deg(b,y)·deg(a,x) + deg(a,y)·deg(b,x) + 1`.

**Measured payoff** (BPAS, CASC 2021): modular subresultant chains are "up to 10× and 400×
faster than non-modular counterparts (mainly Ducos' subresultant chain algorithm) in ℤ[y]
and ℤ[x,y] respectively," with a further 7× / 2× from using Half-GCD to compute only the low
subresultants speculatively.

400× on the bivariate case is exactly the case `arrangements`-like consumers generate. This
is not a nice-to-have.

**Bad specializations**: `Ψ` is bad when it drops a leading coefficient. Detectable directly
(evaluate `lc(a)` and `lc(b)` at the point; reject if either vanishes). Also, evaluation
points where the *degree of the gcd jumps* corrupt the low subresultants — the same
"unlucky point" phenomenon as GCD (§3.2), detected by the same minimal-degree-wins rule.

**Termination**: the BPAS scheme adds primes "until the reconstruction stabilizes," which is
heuristic. Closing it to Las Vegas needs a Hadamard-type bound on the resultant's
coefficients: for `f, g` of degrees `m, n` and coefficient bitsize `τ`, the Sylvester
determinant is bounded by `‖f‖^n · ‖g‖^m` up to binomial factors, giving a bitsize bound of
`O((m+n)τ + (m+n)log(m+n))`. Compute the bound, use enough primes, done — deterministically.

### 6.3 The free differential oracle

Having both implementations gives resolvent an internal oracle at zero marginal cost, and it
is a *strong* one because the algorithms share almost no code:

| route | shares with the others |
| --- | --- |
| subresultant PRS (Ducos) over ℤ | pseudo-division, exact division |
| modular evaluation–interpolation | GF(p) arithmetic, CRT, interpolation |
| Bareiss / Bézout determinant | dense linear algebra |

Plus three *structural* invariants that any single implementation can be checked against
without a second implementation:

1. **Degree bound**: `deg_x Res_y(f,g) ≤ deg_y(f)·deg_x(g) + deg_y(g)·deg_x(f)`. An output
   violating this is a detected bug, no oracle needed.
2. **Vanishing ⇔ common root**: `Res(f,g) = 0` iff `deg gcd(f,g) > 0`. Cross-check against
   the GCD lane. Over the reals, cross-check the *real* roots of the resultant against
   root isolation of the gcd.
3. **Poisson product formula**: `Res(f,g) = lc(f)^{deg g} · Π_i g(α_i)` over the roots `α_i`
   of `f`. Checkable numerically at high precision as a smoke test, and exactly for small
   degree over a splitting field.

**Lane grade: certificate lane. Two independent implementations plus three structural
invariants. This is the easiest lane in Layer 2 to grade automatically and it should be
built early, because it is also the one the first consumer needs.**

---

## 7. Real root isolation

### 7.1 The three families

Given squarefree `f ∈ ℤ[x]` of degree `d` and coefficient bitsize `τ`, produce disjoint
rational intervals each containing exactly one real root.

**Sturm sequences.** Build the Sturm chain `f, f', −rem(f,f'), …`; the number of sign
variations at `a` minus that at `b` is *exactly* the number of distinct real roots in
`(a,b]`. Bisect on that count.
Worst case `Õ_B(d⁴τ²)`, and — importantly — that bound is **tight**. The bottleneck is
computing the chain itself, which is a PRS with all the coefficient growth that implies.

**Descartes / Vincent–Collins–Akritas (VCA).** Descartes' rule: the number of sign
variations `σ(f)` in the coefficient list exceeds the number of positive real roots by a
non-negative even number. So `σ = 0` proves no roots and `σ = 1` proves exactly one. Map an
interval to `(0,∞)` via `f̃(x) = (x+1)^d f(1/(x+1))`, count variations, and bisect when the
count is >1. Termination is Vincent's theorem.
Worst case `Õ_B(d⁴τ²)`, same as Sturm — but the practical behaviour is radically better.

**Continued fractions (Vincent–Akritas–Strzeboński, VAS).** Instead of bisecting, use the
continued-fraction expansion: compute a lower bound on the positive roots, shift by it, and
recurse. Adapts beautifully to roots with small partial quotients, and the subdivision tree
is often near-optimal.

**Bernstein-basis / de Casteljau subdivision.** Algebraically the same as Descartes (the
Bernstein coefficients' sign variations bound the roots in the interval), but numerically
much better behaved, which is why it is the basis of the "bitstream Descartes" method that
CGAL and Sage use. `arrangements` already has `bernstein.rs`.

### 7.2 Which to build, with evidence

The worst-case bounds do not separate Descartes from Sturm. **Average-case analysis does,
and matches practice.** For uniform random bit polynomials of degree `d` and bitsize `τ`,
Ergür–Tonelli-Cueto–Tsigaridas (2025) prove *expected* times:

| solver | expected | worst case |
| --- | --- | --- |
| `descartes` | `Õ_B(d² + dτ)` | `Õ_B(d⁴τ²)` |
| `sturm` | `Õ_B(d²τ)` | `Õ_B(d⁴τ²)` |
| `aNewDsc` | `Õ_B(d² + dτ)` | `Õ_B(d³ + d²τ)` |

They state the conclusion plainly: their Sturm bounds "are worse than the one of descartes
by an order of magnitude. This provides the first theoretical explanation of the superiority
of descartes over sturm that is commonly seen in practice."

**And the practical cliff is root *clusters*, not degree.** ANewDsc benchmarks (seconds,
timeout 600 s):

Mignotte polynomials `x^n − ((2^(τ/2) − 1)x − 1)²`, `τ = 14`:

| n | MPSolve | CF (Mathematica) | Sage | RS | ANewDsc |
| --- | --- | --- | --- | --- | --- |
| 257 | 0.7 | 0.1 | 1.6 | 7.6 | 0.1 |
| 1025 | 13.8 | 1.1 | 4.8 | **> 600** | 0.7 |
| 8193 | > 600 | 224.3 | > 600 | — | 43.2 |

Dense random coefficients in `(−2^τ, 2^τ)`, `n = 1024..16384`, `τ = n`:

| n | MPSolve | CF | Sage | RS | ANewDsc |
| --- | --- | --- | --- | --- | --- |
| 1024 | 2.5 | 0.3 | 4.1 | 0.6 | 0.5 |
| 8192 | 159.4 | 22.1 | 183.5 | 36.0 | 36.5 |

Read together: **on random inputs plain Descartes is fine and ANewDsc adds nothing; on
clustered roots plain Descartes falls off a cliff and ANewDsc is 1000× faster.** The
mechanism is stated directly: the subdivision tree "shrinks from approximately 33 500 nodes
for RS and ADsc to a mere 47 for ANewDsc" on `(n,τ) = (129,512)`.

Clustered roots are *exactly what a geometry consumer produces*: near-tangential contact
between two curves gives a near-double root of the resultant. This is not a synthetic
hazard.

### 7.3 Recommendation

- **Build plain Descartes/VCA first**, exactly as `arrangements` did, as the reference and
  the oracle. Correctness first, and it is genuinely adequate for degree ≤ 8.
- **Get the representation right from the start, because it is where the constant factor
  lives.** Two rules:
  1. **Work in ℤ[x] on dyadic intervals, not in ℚ[x] on arbitrary rational intervals.** With
     dyadic endpoints, the interval transforms are `x → x+1` (a Taylor shift, integer
     arithmetic) and `x → 2^k x` (a shift of the coefficients — "which can be handled by
     specific GMP `mpz_shift` operators"). With arbitrary rational endpoints, every
     subdivision multiplies denominators. The current consumer implementation takes the
     expensive route:
     `/home/dev/projects/arrangements/crates/lazy-exact/src/roots.rs:270-288` composes
     affine maps with `Rational` coefficients on arbitrary `(lo,hi)`. Correct, but it is the
     slow form, and it is the single highest-leverage thing to change.
  2. **Dyadic approximation of coefficients is enough to count sign variations**, so the
     sign-variation count does not need exact bignum arithmetic except when a cancellation
     is genuinely close. msolve: "taking appropriate dyadic approximations of these
     coefficients is sufficient to decide the sign (unless some unexpected cancellations
     occur)."
- **Add Newton acceleration (ANewDsc) as a second phase.** It is a strictly-larger algorithm
  with a fallback to plain Descartes, so the plain path stays as its oracle.
- **Asymptotically fast Taylor shift crosses over "around degree 512"** (msolve). Below that,
  don't bother.
- **Add Abbott's quadratic interval refinement (QIR)** for the *refinement* loop, which is
  what `AlgebraicReal::cmp` hammers (§8). msolve implements it explicitly. Plain bisection
  refinement of an isolating interval to `2^-60` is ~60 exact polynomial evaluations at
  growing rational precision — the consumer does this at
  `.../roots.rs:600-618` and it is the hot path of every geometric predicate.

### 7.4 Two algorithms grading each other — concretely

- **Sturm is the counting oracle for Descartes.** Sturm gives the *exact* number of distinct
  real roots in an interval; Descartes gives an upper bound congruent mod 2. So
  `count_sturm(f, a, b) == len(isolate_descartes(f, a, b))` is a genuine, cheap, fully
  automatic verdict for every test instance up to moderate degree. Build Sturm even though
  it will never be the production isolator — build it *as the oracle*.
- **Structural invariants that need no second implementation**: intervals are pairwise
  disjoint and ordered; every interval has a sign change at its endpoints; `f(lo) ≠ 0 ≠
  f(hi)`; the count is ≤ `deg f` and has the same parity as `deg f` when `lc(f) · f(0) < 0`
  (etc.); every root lies within the Cauchy bound.
- **Round-trip**: build `f = Π (x − r_i)` from known rationals/small algebraic numbers, then
  isolate and check the roots come back.
- **Cross-check against the resultant lane** (§6.3, invariant 2).

**Lane grade: certificate lane for correctness; number-to-optimize for the cliff instances.**

---

## 8. `AlgebraicReal` exact comparison — where inconsistency hides

### 8.1 The representation and the two comparison mechanisms

`AlgebraicReal { poly: squarefree DenseUni<ℤ or ℚ>, lo, hi }`, where `poly` has exactly one
real root in `(lo, hi)` and `poly(lo) ≠ 0 ≠ poly(hi)`. The consumer's version adds a
multiplicity field
(`/home/dev/projects/arrangements/crates/lazy-exact/src/roots.rs:317-322`).

Comparison of `α` and `β` uses **two mechanisms that must be combined, never one alone**:

1. **Refine until disjoint.** Bisect both intervals; as soon as `α.hi ≤ β.lo` or
   `β.hi ≤ α.lo`, the order is decided. **This terminates only if `α ≠ β`.** On equal
   values it loops forever.
2. **Prove equality algebraically.** `g = gcd(α.poly, β.poly)`. If `deg g = 0`, they cannot
   be equal — mechanism 1 is guaranteed to terminate. If `deg g > 0`, then a root of `g` in
   the overlap of the two isolating intervals *is* both roots, so a **sign change of `g`
   across the overlap** certifies equality.

The consumer implements exactly this at `.../roots.rs:549-598`, and the two-mechanism
structure is why it is correct.

### 8.2 The failure modes, enumerated

**F1 — non-squarefree defining polynomial.** If `poly` has a double root at `α`, the sign
never changes across the interval, bisection cannot decide which half to keep, and
refinement either stalls or picks the wrong half. Every downstream guarantee collapses.
*Mitigation: squarefree-ness is a construction-time invariant, enforced by Yun decomposition
before isolation, and the type must make it impossible to construct otherwise.* Fail closed
— a constructor that takes an arbitrary polynomial must return `Result`.

**F2 — equality by tolerance.** "The intervals overlap and are narrower than ε, so call them
equal." This is *the* canary failure. It is not merely wrong on some inputs; it is
**intransitive**: with `α < β < γ` and `|α − γ| > ε > |α − β|, |β − γ|`, you get
`α = β`, `β = γ`, `α ≠ γ`. A sort then produces garbage and a geometry consumer produces a
topologically inconsistent arrangement. *Mitigation: equality is decided by gcd or not at
all. No epsilon exists anywhere in the type.*

**F3 — treating a failed equality certificate as evidence of inequality.** The gcd sign-
change test can fail *spuriously*: if an overlap endpoint happens to be a root of `g`, the
"sign change" test sees a zero and cannot conclude. The correct response is **refine and
retry**, not "return Less/Greater." A implementation that returns an ordering on a failed
certificate is intransitive in exactly the same way as F2. The consumer gets this right —
`if slo != Sign::Zero && shi != Sign::Zero && slo != shi { return Equal }` and otherwise
falls through to another refinement round (`.../roots.rs:578-592`). *This is subtle enough
that it must be an explicit property test, not a code review item.*

**F4 — root exactly at an interval endpoint.** The invariant `poly(lo) ≠ 0 ≠ poly(hi)` must
survive every bisection. If the midpoint is exactly the root, the interval must **collapse to
a point** and the number becomes exactly rational; if instead you keep a half-open interval
with a root at the boundary, all subsequent reasoning is unsound. The consumer's `refine`
collapses (`.../roots.rs:450-470`).

**F5 — `sign_of(h)` at a root, when `h(α) = 0`.** The natural loop — "refine until `h` has no
root in the interval, then evaluate at the midpoint" — never terminates when `h(α) = 0`.
*Mitigation: decide `h(α) = 0` algebraically first, via `gcd(poly, h)` plus a sign-change
certificate, and only then enter the refinement loop.* The consumer does this
(`.../roots.rs:480-524`). **Any code path that can produce a "sign at an algebraic number"
without first settling zero-ness algebraically is a hang, not a wrong answer** — which in a
library is worse, because it is undebuggable in production.

**F6 — `cmp` needs `&mut self`, but `Ord::cmp` takes `&self`.** This is a real API fork with
three exits, each with a real cost:

| exit | cost |
| --- | --- |
| Interior mutability (`Cell`/`RefCell`) | `!Sync`. Self-comparison re-borrows and panics — the consumer must guard, and does: `if Rc::ptr_eq(p, q) { return Equal }` at `.../geoms/conics.rs:41-46`. That guard is load-bearing and easy to forget. |
| Interior mutability with a lock | `Sync`, but a self-comparison deadlocks instead of panicking, and every comparison pays an atomic. |
| No cached refinement — recompute each `cmp` | `Ord` works, is `Send + Sync`, is pure. But sorting `n` algebraic numbers redoes `O(n log n)` refinements from scratch, each `O(precision)` bignum work. Quadratic-ish blowup in practice. |
| Explicit context: `ctx.cmp(&a, &b)` with refinement state in a side table | Correct, `Send`, no aliasing hazard, no `Ord`. Consumers must thread the context. |

**CGAL chose the pure route and says so**: "there is no way to directly ask for the
refinement of the current isolating interval since this would impose a state to every object
of an Algebraic kernel." `arrangements` chose shared interior mutability
(`type SharedRoot = Rc<RefCell<RealRoot>>`, `.../geoms/conics.rs:33-35`) and pays with
`!Send`.
**This is a decision the architecture must make explicitly and cannot defer**, because it
determines whether resolvent's headline type is `Send + Sync`.

**F7 — `Eq`/`Hash` inconsistency.** Two `AlgebraicReal`s can be *equal* while having
different defining polynomials (`x² − 2` and `x⁴ − 4`, both with root `√2`). If `Eq` is the
gcd test but `Hash` hashes the polynomial, they hash differently and a `HashMap` silently
holds two entries for one number. **The only consistent fix is a canonical form: the minimal
polynomial plus a root index — and computing the minimal polynomial requires factorization
over ℚ.** So: either do not implement `Hash` (and document why), or implement it behind an
explicit, lazy `canonicalize()` that costs a factorization. Do not implement a "cheap"
`Hash`. This is a trap that will not show up in any unit test and will show up as
nondeterministic behaviour in a consumer.

**F8 — degree blowup under arithmetic.** `α + β` has a defining polynomial of degree ≤
`deg α · deg β` (computed as `Res_y(f(y), g(x − y))`); similarly for products. Without
reducing to the minimal polynomial at each step, degree 4 + degree 4 → 16 → 256 → 65536
after three operations. *Mitigation: either (a) factor after each arithmetic operation
(expensive but bounded), or (b) do not expose general arithmetic on `AlgebraicReal` at all
and instead expose the operations a geometry consumer actually needs — `sign_of(h)` at a
root, and sign of `a(ξ) + b(ξ)√h(ξ)` ladders.* The consumer took route (b)
(`sign_radical1`/`sign_radical2`, `.../roots.rs:622-687`) and it is why its predicates stay
in degree 4. **Route (b) is the right default and route (a) should be opt-in and loudly
documented.**

**F9 — multiplicity vs value.** Two roots with the same value but different multiplicities in
their respective source polynomials must compare `Equal`. Any comparison that includes
multiplicity in the tie-break is wrong.

**F10 — the enclosure must never contradict the exact verdict.** `to_interval()` returns a
float enclosure; if `a.cmp(&b) == Less` then `a.to_interval().sup()` must not exceed
`b.to_interval().inf()` by more than the enclosure's own width, and in particular a
*disjoint* pair of enclosures must agree with the exact ordering. This is a free property
test and it catches enclosure-direction bugs (outward vs. nearest rounding) that are
otherwise invisible.

### 8.3 The property-test suite (this is the lane's verdict function)

Over randomly generated algebraic numbers — including deliberately-equal pairs constructed
from different polynomials, and deliberately-near pairs from Mignotte-style clusters:

1. **Trichotomy**: exactly one of `a < b`, `a == b`, `a > b`.
2. **Transitivity**: `a ≤ b ∧ b ≤ c ⇒ a ≤ c`. *The named canary.* Generate triples where two
   pairs are within `2^-1000` of each other.
3. **Antisymmetry**: `a ≤ b ∧ b ≤ a ⇒ a == b`.
4. **Equality is an equivalence relation**, and equal elements have equal `sign_of(h)` for
   every `h`.
5. **Sort stability**: sorting a shuffled list gives the same sequence of equality classes
   every time, regardless of shuffle — this catches state-dependence in the refinement cache.
6. **Consistency with enclosures** (F10).
7. **Consistency with the isolator**: `isolate_roots(f)` returns roots in strictly ascending
   order under `cmp`, and their count matches Sturm's (§7.4).
8. **Idempotence under refinement**: refining either operand any number of times before
   comparing never changes the verdict.
9. **No hangs**: every property test runs under a step budget; exceeding it is a failure, not
   a timeout. F5 makes hangs the *expected* failure mode of a wrong implementation, so the
   harness must treat "did not finish" as "wrong."

**Lane grade: certificate lane, and the strongest one in the library. Point 9 is not
boilerplate — it is the primary detector.**

---

## 9. Known performance cliffs and benchmark calibration

### 9.1 The four cliffs

**Coefficient explosion.** The reason modular methods are structural. Numbers:

- Cyclic-10 over ℚ (Giac): the basis has **5690 elements, ~20 million monomials**, most
  coefficients need **more than 2000 primes of 29 bits** (≈58 000 bits) to reconstruct,
  the basis needs **more than 50 GB of RAM**, and the run takes **14 hours**.
- Katsura-`n` over ℚ, primes needed by msolve: `n=9` → 83, `n=10` → 188, `n=11` → 388,
  `n=12` → 835, `n=13` → 1772, `n=14` → 3847. Roughly doubling per step.
- The same effect on a smaller scale in the geometry path: a resultant of two degree-`d`
  bivariate polynomials with `τ`-bit coefficients has degree up to `2d²` in the remaining
  variable and coefficients of `O(dτ + d log d)` bits. Two degree-10 curves with 32-bit
  coefficients already give a degree-200 polynomial with ~500-bit coefficients — well past
  where naive `Rational` arithmetic is acceptable.

**Intermediate expression swell.** Distinct from output size: the *intermediates* are bigger
than the answer. Classic instances: naive Euclidean PRS (exponential growth, fixed by
subresultants); Buchberger over ℚ without modular methods; cofactor tracking in a certified
Gröbner basis (§3.4); Hensel lifting to the Landau–Mignotte bound when the true factors are
small.

**Degenerate and near-degenerate inputs.** These are the geometry-specific cliffs and they
are *the normal case* for a consumer, not the exception:

- **Tangential contact** between two curves ⇒ a double (or near-double) root of the
  resultant ⇒ the Mignotte cliff of §7.2 (RS: >600 s vs ANewDsc: 0.7 s).
- **Coincident / overlapping components** ⇒ an identically-zero resultant. The consumer
  handles this by fail-closed detection:
  `/home/dev/projects/arrangements/crates/arrangements/src/geoms/conics.rs:565-566` — "A
  zero resultant needs a shared component — excluded above";
  `.../geoms/sine_radical.rs:1158-1161` — "Distinct coefficients yet an identically-vanishing
  resultant". Resolvent must return a distinguishable result here, never a silently-empty
  root list.
- **Degree drop under specialization** (leading coefficient vanishing at an evaluation
  point) ⇒ the bad-specialization case of §6.2.
- **Exactly-rational algebraic numbers** appearing where the code expects an interval —
  handled by collapsing to a point (F4).

**Memory.** The Goodwin (w.) Macaulay matrix at 403 677 × 374 837 with 41.7M nonzeros; the
50 GB Cyclic-10 basis; the "symbolic preprocessing causes the most memory allocations"
finding. Memory, not time, is what makes the big instances impossible.

### 9.2 Benchmark families

The standard suite, as used by every source cited here: **Katsura-`n`, Cyclic-`n`, Eco-`n`,
Noon-`n`, Reimer-`n`, Henrion-`n`, Chandra-`n`**, plus random dense and random sparse
systems and application systems (SIAN/StructuralIdentifiability, BioModels).

**Definitions must be pinned, because conventions differ by an index shift and that changes
which instance you are benchmarking.** Two guards:

- **Cyclic-`n`**: variables `x_0..x_{n-1}`; `f_k = Σ_{i=0}^{n-1} Π_{j=i}^{i+k-1} x_{j mod n}`
  for `k = 1..n-1`, and `f_n = x_0 x_1 ⋯ x_{n-1} − 1`.
- **Katsura-`n`**: the ideal degree is a checkable invariant — msolve's table gives
  Katsura-9 → 256, 10 → 512, 11 → 1024, 12 → 2048, 13 → 4096, 14 → 8192, i.e. `2^(n-1)`
  under that naming. **A generator that does not reproduce the published degree is generating
  a different system**; make that an assertion in the harness rather than a comment.

Beyond system-solving, the univariate suite:

- **Mignotte** `x^n − ((2^(τ/2) − 1)x − 1)²` and nested Mignotte — the clustered-root cliff.
- **Random dense integer coefficients** — the easy case, `Θ(log n)` real roots.
- **Gaussian-coefficient squares** `f² − 1` — many clusters of multiplicity two.
- **Swinnerton-Dyer** — the factorization cliff (§5.3).
- **Wilkinson**, **Chebyshev/Legendre** — well-separated, tests that the accelerated path
  does not regress the easy case.

### 9.3 Calibration: "working" vs "competitive"

Concrete published numbers, single-threaded, drl. Use these as the lane's grading thresholds
rather than inventing any.

**Gröbner over GF(p ≈ 2^30), seconds:**

| system | Groebner.jl | Maple/FGb | msolve | OpenF4 | Singular (Buchberger) |
| --- | --- | --- | --- | --- | --- |
| Cyclic-8 | 1.46 | 1.23 | 1.44 | 9.43 | — |
| Cyclic-9 | 259 | 341 | 271 | **5 552** | — |
| Katsura-11 | 9.20 | 7.72 | 8.90 | 61.2 | **1 388** |
| Katsura-13 | 381 | 692 | 268 | 2 860 | — |
| Eco-13 | 11.0 | 13.2 | 19.4 | 74.5 | — |
| Noon-9 | 19.5 | 18.7 | 22.3 | 198 | 47.9 |

Reading: **the three state-of-the-art implementations are within ~1.5× of each other.**
OpenF4 — a real F4 implementation, not a toy — is 4–21× off. Singular's Buchberger is 150×
off on Katsura-11.

| milestone | threshold |
| --- | --- |
| **Correct** | Cyclic-7, Katsura-8, Eco-10 complete and agree with an external system. |
| **Working** | Cyclic-8 < 60 s, Katsura-11 < 500 s, Eco-13 < 500 s. (≈ Singular-Buchberger class: it is a real implementation, just not an F4 one.) |
| **Competitive** | Cyclic-9 < 600 s, Katsura-13 < 900 s, Eco-14 < 600 s. (≈ 2× the SOTA column.) |
| **State of the art** | within 1.5× of the msolve/Maple/Groebner.jl column. Do not plan for this. |

**Gröbner over ℚ, seconds** (multi-modular; the `# primes` column is the coefficient-growth
signal):

| system | # primes (31-bit) | Groebner.jl | Maple | msolve | Singular |
| --- | --- | --- | --- | --- | --- |
| Katsura-10 | 54 | 18.7 | 84.8 | 17.5 | 2 864 |
| Katsura-11 | 78 | 188 | 1 318 | 168 | — |
| Cyclic-8 | 54 | 19.7 | 23.8 | 26.1 | — |
| Chandra-13 | 166 | 528 | 4 409 | 555 | — |
| Reimer-8 | 78 | 550 | 2 472 | 257 | — |
| Hexapod | 1 102 | 4.88 | 63.7 | 3.42 | 300 |

Note **Hexapod**: 1102 primes for a computation whose single modular run is 0.00 s. That is a
pure reconstruction-bound instance and it is the one that will find bugs in CRT / rational
reconstruction. Include it early.

**Real root isolation**: §7.2's tables. Thresholds:

| milestone | threshold |
| --- | --- |
| **Correct** | degree ≤ 20 random and Mignotte instances, verified against Sturm counts. |
| **Working** | random dense `n = 1024, τ = 1024` in < 30 s; Mignotte `n = 257, τ = 14` in < 60 s. |
| **Competitive** | random dense `n = 8192, τ = 8192` in < 200 s; Mignotte `n = 1025, τ = 14` in < 5 s (i.e. Newton acceleration is present and working). |

**Resultants**: modular bivariate must beat a Ducos implementation by ≥ 100× on
`ℤ[x,y]` inputs of degree ~20 to be considered done (the literature figure is 400×).

**Factorization**: Swinnerton-Dyer of degree 32 (`r ≈ 16`) must complete — Zassenhaus can do
this. Degree 64 (`r ≈ 32`) separates van Hoeij from Zassenhaus. Degree 256 is the "van Hoeij
is really working" mark.

---

## 10. Lane classification: certificates vs numbers

Constraint #3 asks for this explicitly. A lane whose verdict is a *number* converges slowly,
needs a stable benchmark machine, needs change-point detection rather than pass/fail, and
cannot be fanned out to many agents in parallel without a shared baseline.

| Lane | Verdict type | Automatic verdict |
| --- | --- | --- |
| Coefficient rings (ℤ, ℚ, GF(p), ℤ/n) | **Certificate** | Ring-axiom property tests; agreement with a reference bignum on random inputs. |
| Monomial representation | **Certificate** | Order-axiom property tests (total order, multiplicative compatibility, well-ordering on ℕⁿ); overflow always detected, never wrapped; round-trip encode/decode. |
| Dense univariate arithmetic | **Certificate** | `(a·b)/b == a`; degree additivity; agreement with a naive `O(n²)` reference. |
| GCD | **Certificate** | `H\|A`, `H\|B`, `deg H = deg gcd mod p` (§3.2). Fully self-certifying. |
| Resultants | **Certificate** | Two independent algorithms + three structural invariants (§6.3). |
| Root isolation (correctness) | **Certificate** | Sturm counts, disjointness, sign changes, round-trip (§7.4). |
| `AlgebraicReal` | **Certificate** | Trichotomy / transitivity / sort-stability / step budget (§8.3). |
| Factorization (product) | **Certificate** | Multiply back. |
| Factorization (irreducibility) | **Partial** | Modular irreducibility certificate when one exists; differential testing otherwise (§3.3). |
| Gröbner, certified mode | **Certificate** | Cofactor representation `f = Σ h_i g_i` + Buchberger test. Expensive by construction. |
| Gröbner, fast mode | **Number** | Graded against certified mode on regressions; against §9.3 thresholds for speed. |
| F4 linear algebra | **Number** | Throughput. No certificate beyond "the basis is right." |
| Modular / tracing / batching | **Number** | Speedup vs. independent modular runs. |
| Root isolation (cliff performance) | **Number** | §9.3 thresholds on Mignotte families. |
| van Hoeij recombination | **Number + partial certificate** | Correct: product check + agreement with Zassenhaus for small `r`. Fast: Swinnerton-Dyer degree ladder. |

**Sequencing consequence**: the certificate lanes can be fanned out immediately and in
parallel, each with a self-contained verdict. The number lanes need (a) a pinned benchmark
harness with generated-and-degree-checked instances (§9.2), (b) a stable machine, (c) a
baseline implementation to regress against — which means **the reference/certified
implementation of each number lane must be built and frozen first**. Do not start F4 before
Buchberger passes; do not start ANewDsc before plain Descartes passes; do not start van
Hoeij before Zassenhaus passes.

---

## 11. Summary of the decisions this document forces

1. **Monomial key normalization** (§1.5): comparison key packed big-endian, order-normalized
   at intern time, comparison is an order-free unsigned integer compare; order lives in the
   runtime ring context, not in the type parameter. Multiply = SWAR add + per-order constant
   correction.
2. **Overflow is fail-closed and recoverable** (§1.3): guard bits, `Result`-returning
   multiply, widen-and-restart owned by the driver. Field width is *not* a one-way door;
   the intern/id/key structure is.
3. **Three polynomial types, not one** (§2.4): `DenseUni<C>` standalone and first,
   `SparseDist<C>` for multivariate, `RecursiveView` as a borrowed view.
4. **grevlex + FGLM, never direct lex** (§1.4).
5. **Every modular routine returns a `Certificate`** (§3.1); `Probable` is legal but typed.
6. **Two Gröbner modes** (§3.4): certified-with-cofactors as the oracle, modular-with-tracing
   as the performance path, cross-checked against each other.
7. **`AlgebraicReal` mutability model** (§8.2 F6) must be chosen before the type ships:
   pure-and-recompute, interior-mutable-and-`!Sync`, or explicit-context. This determines
   whether resolvent's headline type is `Send + Sync`.
8. **No `Hash` on `AlgebraicReal` without an explicit factorization-backed canonical form**
   (§8.2 F7).
9. **No general arithmetic on `AlgebraicReal` by default** (§8.2 F8); expose the sign-ladder
   operations a geometry consumer actually calls.
10. **F5/signature is not in the critical path** (§4.5).
11. **Sturm and Zassenhaus are built as oracles, not as production algorithms** (§7.4, §5.3).

---

## Sources

- Bachmann, Schönemann. *Monomial representations for Gröbner bases computations.* ISSAC 1998, 309–316. — [record](https://kluedo.ub.rptu.de/frontdoor/index/index/docId/469)
- Monagan, Pearce. *Polynomial Division using Dynamic Arrays, Heaps, and Packed Exponent Vectors.* CASC 2007, LNCS 4770, 295–315. — [PDF](http://www.cecm.sfu.ca/~mmonagan/teaching/TopicsinCA09/sdmp16.pdf)
- Demin, Gowda. *Groebner.jl: A package for Gröbner bases computations in Julia.* — [arXiv:2304.06935](https://arxiv.org/pdf/2304.06935) (packing capacity, 15% figure, runtime breakdown Table 1, benchmark Tables 4–5, batched tracing). GPL-licensed software.
- Roune, Stillman. *Practical Gröbner Basis Computation.* ISSAC 2012. — [arXiv:1206.6940](https://arxiv.org/pdf/1206.6940) (divisor-query Table 3, S-pair criteria Table 2, signature assessment).
- Berthomieu, Eder, Safey El Din. *msolve: A Library for Solving Polynomial Systems.* ISSAC 2021. — [arXiv:2104.03572](https://arxiv.org/pdf/2104.03572) (hashing + divmasks, AVX2, tracer, real root isolation, Katsura-`n` primes table). GPLv2 software.
- Singular manual, *Limitations.* — [page](https://www.singular.uni-kl.de/Manual/4-0-3/sing_455.htm)
- Arnold. *Modular algorithms for computing Gröbner bases.* J. Symbolic Computation, 2003. — [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0747717102001402)
- Idrees, Pfister, Steidel. *Parallelization of modular algorithms.* — [arXiv:1005.5663](https://arxiv.org/abs/1005.5663)
- Noro, Yokoyama. *Verification of Gröbner Basis Candidates.* ICMS 2014. — [Springer](https://link.springer.com/chapter/10.1007/978-3-662-44199-2_64)
- Monagan, Pearce et al. *Speeding up polynomial GCD, a crucial operation in Maple.* — [PDF](http://www.cecm.sfu.ca/CAG/papers/MonGCD21.pdf) (unlucky primes/points, trial-division verification)
- van Hoeij. *Factoring polynomials and the knapsack problem.* J. Number Theory 95 (2002). — [PDF](https://www.math.fsu.edu/~hoeij/knapsack/paper/May16_2001/knapsack.pdf)
- Klüners. *The van Hoeij Algorithm for Factoring Polynomials.* — [PDF](https://math.uni-paderborn.de/fileadmin-eim/mathematik/AG-Computeralgebra/Publications-klueners/factor_lll.pdf)
- Hart, van Hoeij, Novocin. *Practical polynomial factoring in polynomial time.* ISSAC 2011. — [PDF](https://www.math.fsu.edu/~hoeij/papers/issac11/A.pdf)
- Ducos. *Optimizations of the subresultant algorithm.* J. Pure Appl. Algebra 145 (2000) 149–163.
- Asadi, Brandt, Moir, Monagan, Maza. *Computational schemes for subresultant chains.* CASC 2021. — [PDF](https://www.bpaslib.org/media/ComputationalSchemesForSubresultants-CASC2021.pdf) (10×/400× modular figures, evaluation-point bound)
- Kobel, Rouillier, Sagraloff. *Computing Real Roots of Real Polynomials … and now For Real!* ISSAC 2016. — [arXiv:1605.00410](https://arxiv.org/pdf/1605.00410) (Mignotte and random-dense benchmark tables, complexity Table 1)
- Sagraloff, Mehlhorn. *Computing Real Roots of Real Polynomials.* — [arXiv:1308.4088](https://arxiv.org/pdf/1308.4088)
- Ergür, Tonelli-Cueto, Tsigaridas. *Beyond Worst-Case Analysis for Symbolic Computation: Root Isolation Algorithms.* 2025. — [arXiv:2506.04436](https://arxiv.org/pdf/2506.04436) (expected-time separation of Descartes from Sturm)
- Parisse. *Computing huge Groebner basis like cyclic10 over ℚ with Giac.* 2019. — [arXiv:1903.12427](https://arxiv.org/abs/1903.12427) (Cyclic-10: 5690 elements, >2000 primes, >50 GB, 14 h)
- Eder, Faugère. *A survey on signature-based algorithms for computing Gröbner bases.* J. Symbolic Computation 80 (2017) 719–784. — [ScienceDirect](https://www.sciencedirect.com/science/article/pii/S0747717116300785)
- CGAL. *Algebraic Kernel user manual.* — [docs](https://doc.cgal.org/latest/Algebraic_kernel_d/index.html) (no-refinement-state design decision)
- SymPy polynomials documentation and `sympy/polys/factortools.py` — BSD-3; Zassenhaus only, no LLL.

Consumer code read for grounding (context only; resolvent does not depend on it):
`/home/dev/projects/arrangements/crates/lazy-exact/src/roots.rs`,
`/home/dev/projects/arrangements/crates/lazy-exact/src/exact/rational.rs`,
`/home/dev/projects/arrangements/crates/arrangements/src/geoms/conics.rs`,
`/home/dev/projects/arrangements/crates/arrangements/src/geoms/sine_radical.rs`,
`/home/dev/projects/arrangements/crates/arrangements/src/geoms/spherical_circle.rs`.
