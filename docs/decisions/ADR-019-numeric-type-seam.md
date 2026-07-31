# ADR-019 — The numeric-type seam: one open trait tower, no ops-surface scalar trait, no seam crate

**Status:** Ratified 2026-07-31
**Reversibility:** one-way — the trait signature is inherited by every polynomial and every
algorithm above it, and a public scalar trait cannot be removed without a major version
**Amended:** 2026-07-31 — §1 adopts ADR-006's *corrected* tower (`Ring::Ctx`,
`Liftable: Reducible`, `Reducible::Image: CommutativeRing`, `BatchField`, no `BulkOps`).
The shape decision in this ADR — one open tower, no ops-surface scalar trait, no seam
crate — is unchanged; only the tower's signature moved.
**Gates lanes:** Z0, and every lane thereafter.
**Evidence:** `docs/research/consumer-cadabra2.md` §5, §10.4; `consumer-solverang.md` §7
R1–R5; `consumer-sinbad.md` §1.4; `docs/research/challenge-generality.md` §2, §3, §5.1;
`docs/research/challenge-evidence.md` §2.1, §2.2; `plans/architecture.md` §2.3, §5.6;
ADR-006, ADR-015, ADR-018.
**Supersedes:** `plans/api-shape.md` §3 and its INV-14.
**Note on `docs/decisions/RECONCILIATION.md`:** referenced in §Consequences below and in
`API.md`; that file was never written and its role is filled by **ADR-021**, which carries
the precedence rule and the contradiction register.

---

## Context

Two founding documents were written in parallel and gave incompatible answers to the same
question, on a decision both label one-way. `challenge-generality.md` §2 found the
contradiction, and the contradiction census in `plans/roadmap.md` §2.5 — which exists
specifically to catch this — missed it while fan-out was being scheduled against it.

| Source | The coefficient seam |
|---|---|
| `plans/api-shape.md` §3.2(b), §3.3, INV-14 | A consumer-implementable coefficient trait is **rejected**. Coefficients are a **sealed** set `{Rational, Integer, FpElem, NfElem}`. "A consumer cannot add a coefficient ring." Separately, an **open** six-method `Scalar` trait is introduced for *evaluation scalars*, in a zero-dependency `resolvent-seam` crate pitched as the ecosystem's numeric vocabulary. |
| `plans/architecture.md` §2.3 / ADR-006 *(as written then; the tower's signature was corrected on 2026-07-31 — see §1)* | A public **open** tower: `Ring` → `CommutativeRing` → {`Field`, `EuclideanDomain`, `UniqueFactorizationDomain`}, plus orthogonal markers `Ordered`, `Reducible`, `Liftable`, `BulkOps`. "The modular pipeline is bounded by `C: Reducible + Liftable`, not by `C: Ring`." No scalar seam; ADR-018 §6.4 explicitly says not to add one. |

These are not two phrasings of one decision. A closed enum of four types and an open trait
tower imply different signatures for every Layer-1 and Layer-2 function.

Three further facts bear on the answer.

**The argument against the open coefficient seam was an argument against a badly factored
trait, and it was used six paragraphs later to justify the open scalar seam.**
`api-shape.md` §3.2(b) rejects a coefficient trait because it "pushes bignum-shaped
obligations — exact division, content, bit-length, reconstruction — into a type whose
entire purpose was to be word-sized". §3.3 then justifies `Scalar` with "Six methods, one
sign, one fallible division. Nothing in `Scalar` obliges an implementor to be a bignum."
The second sentence is exactly what a well-factored coefficient tower achieves, and
`architecture.md` §2.3 is that tower.

**The sealed set failed the founding acceptance criterion.** Rule 4 requires an adapter in
under 200 lines **with zero changes to resolvent**. A cryptography or coding-theory
consumer needs GF(p^k) with its own basis; `api-shape.md` §8.4 answers "add it to the
sealed set", i.e. resolvent must change and the consumer waits on an upstream release. For
that consumer class this is usually terminal, because the point is *their* tower chosen for
speed.

**A `resolvent-seam` scalar crate would have been the fourth scalar vocabulary in this
workspace, and its stated justification was misattributed.** `api-shape.md` §5.1 defends it
as non-speculative because "cadabra2 already built its own `scalar-seam` for exactly this".
cadabra2 did not build it. It lives at `/home/dev/projects/arrangements/crates/scalar-seam`
— in the repository whose merge with resolvent is ADR-018's deferred decision — is MIT OR
Apache-2.0 (`arrangements/Cargo.toml:9`), is 257 lines with zero dependencies, already ships
`Dual<S>` (`scalar-seam/src/dual.rs`, 412 lines), and cadabra2 consumes it by path
(`cadabra2/Cargo.toml:39`). Its own header says it exists so that `lazy-exact` and
`~/sinbad` can both depend *down* on it with no repository cycle
(`arrangements/crates/scalar-seam/src/lib.rs:5-17`). The other two vocabularies are
`lazy_exact::exact::{RingOps, ExactRing, ExactField}`
(`arrangements/crates/lazy-exact/src/exact/mod.rs:16-29, 58-72`) and cadabra2's
crate-private `TierField` (`cadabra2/crates/cadabra-algorithms/src/fastpath/filter.rs:32`).
`challenge-evidence.md` §2.1 tabulates the correspondence: the proposed `Scalar` is a
near-exact re-derivation of `RingOps`/`ExactRing` with two panics fixed.

---

## Decision

> **One open trait tower, in `resolvent-base`, covering both coefficients and evaluation.
> No second, ops-surface scalar trait. No `resolvent-seam` crate. `Ring` gains defaulted
> in-place forms.**

### 1. The tower is ADR-006's, adopted unchanged in shape

`Ring` → `CommutativeRing` → {`Field`, `EuclideanDomain`, `UniqueFactorizationDomain`},
plus the orthogonal markers `Ordered`, `Reducible`, `Liftable`, and — for `LANES > 1` only
— `BatchField`. Depth capped at three. `Ord` is **not** required (the batched tuple ring has
no meaningful order and requiring it would close that door permanently).

**ADR-006 is the authority on the signature and it was corrected on 2026-07-31.** Four
changes, none of which touches this ADR's *shape* decision but all of which change what a
consumer writes: `Ring` gained an associated `Ctx` so that `zero`/`one` can name a runtime
modulus (the receiverless form was unimplementable for five of the seven rings in the
instantiation set); `Liftable`'s supertrait is `Reducible` (the original did not compile);
`Reducible::Image` is a `CommutativeRing` with a fallible `reduce`, because reduction of an
algebraic-extension element mod `p` is not a field for a split prime and some towers have no
inert prime at all; and `BulkOps` is **deleted**, because re-exposing a Tier-M kernel as a
trait method either duplicates it per instantiation or forwards to it for nothing. The
in-place defaults this ADR §3 adds are unchanged and are now part of ADR-006's block.

**What a consumer implements is still small**, which was this ADR's load-bearing claim: for
a word-sized type, `Ctx = ()` costs one line and `ctx()` returns `&()`; `reduce` and
`crt_lift` are still absent; content and bit-length are still absent.

A consumer may implement it for its own type and instantiate every Tier-G algorithm over it.
It gets correctness, not speed, **and the trait's own doc comment says so in those words**.
The modular fast path is bounded by `Reducible + Liftable`, so a ring that cannot be reduced
mod `p` *cannot compile* into it. That is honest and mechanically enforced.

### 2. The sealed set is abandoned

resolvent still *instantiates* over a closed set it controls (`Fp`, `Fp4`, `Integer`,
`Rational`, `Zn`, `GFpk`, and `NumberFieldElem` behind `number-fields`) — that is ADR-006's
compile-time budget and it is unchanged. What is abandoned is the claim that the set is
**closed to consumers**. `GF(p^k)` and `Zn` are core and public
(`plans/architecture.md:57`), and factorization over `GF(p)` becomes a public capability on
the same zero-marginal-implementation argument that made `Fp` public.

### 3. `Ring` gains defaulted in-place forms

```rust
fn add_assign(&mut self, r: &Self) { *self = self.add(r); }
fn sub_assign(&mut self, r: &Self) { *self = self.sub(r); }
fn mul_assign(&mut self, r: &Self) { *self = self.mul(r); }
```

`challenge-generality.md` §5.1 traced the cost of not having them: Bareiss over ℚ at *n* = 4
is Θ(n³) ≈ 64 multiply–subtract–divide steps, each allocating two or three fresh bignum
rationals that are dead one line later — and Bareiss exists to replace a measured 2.448 ms
recursive Laplace determinant, so the win would come entirely from the algorithm while the
seam forecloses recovering the rest. A **defaulted** body obliges no implementor to do
anything: a word-sized type ignores it, a bignum overrides it.

The invariant that replaces `api-shape.md`'s INV-14 is therefore a property, not a method
count: **`Ring` imposes no obligation a word-sized type cannot discharge.**

### 4. There is no evaluation-scalar trait, and no seam crate

`Interval<f64>` is not a `Ring` and resolvent ships no interval type (ADR-015). A consumer
that writes one algorithm text and runs it at f64 / interval / exact tiers keeps its
existing seam for the inexact tiers and implements `resolvent::Ring` for the exact one.
resolvent's generic texts — Horner, Bernstein/de Casteljau, Bareiss, dense row echelon,
matrix multiply, sign ladders — instantiate at the exact tier.

The distinction that makes this the right cut: `RingOps` is explicitly an *ops surface* and
"not an algebraic claim" — `Interval` implements it. resolvent's traits are algebraic
claims: `Field::inv` means a multiplicative inverse, not a best-effort division. Two
similarly named traits with different contracts across an adapter boundary is a bug
generator (`plans/architecture.md` §5.6, ADR-018 §6.4).

### 5. `Send + Sync + 'static` on `Ring` stays, deliberately

`challenge-evidence.md` §2.1 recommends omitting these bounds so that
`impl<T: RingOps> resolvent::Ring for T` stays a legal blanket impl from a glue crate. That
recommendation is **declined**, and the reason is recorded because it is a genuine cost:
`MPoly<C>` and `UPoly<C>` must be `Send + Sync` (ADR-012's determinism contract and
`plans/api-shape.md`'s successor INV-13), and that is not negotiable for the convenience of
a blanket impl. A blanket impl from an ops-surface trait remains legal for thread-safe
types, which is what `lazy_exact::Real` already is
(`arrangements/crates/lazy-exact/src/real.rs:1-16, 25-45`: `Arc`-shared nodes, `AtomicU64`
interval cache, per-node `Mutex` with a documented five-step protocol).

### 6. Homomorphisms are applied to polynomials, never inside evaluation loops

```rust
impl<C: Ring> UPoly<C> {
    pub fn map_coefficients<D: Ring, E>(&self, f: impl Fn(&C) -> Result<D, E>) -> Result<UPoly<D>, E>;
    pub fn eval_horner(&self, at: &C) -> C;     // same ring; no hom parameter
}
```

There is **no** `evaluate_with(hom, point)`. `challenge-generality.md` §5.2 caught the
earlier acceptance sketch folding a ℚ→GF(p) reduction — a bignum reduction on numerator and
denominator plus a modular inverse of the denominator — into the innermost column loop, for
the one consumer whose entire case is "no bignums appear anywhere; all arithmetic is
single-word modular" (`consumer-solverang.md` §3.2), inside a per-edit loop a MUS extraction
calls O(k) times. Removing the signature removes the idiom. This is a type-level rule, not
a documentation rule.

---

## Consequences

- **A consumer with its own coefficient ring is served without resolvent changing.** That
  restores the founding acceptance criterion for the crypto/coding-theory class, which the
  sealed set failed outright.
- **The two founding documents now agree**, and the fan-out that was scheduled against a
  contradicted one-way door is unblocked. See `docs/decisions/RECONCILIATION.md` §2.1.
- **resolvent is not the ecosystem's scalar vocabulary, and does not try to be.** This is
  the real cost of the decision and it is accepted rather than argued away. A geometry crate
  that wants "one text, three tiers" gets it from
  `arrangements/crates/scalar-seam`, which already exists, is zero-dependency, is the same
  license, and is already consumed by two repositories. resolvent contributes algebraic
  traits and algorithms.
- **ADR-018's deferral gets cheaper, not more expensive.** With no competing `Scalar`, the
  orphan-rule collision does not arise, no mandatory glue crate appears, and the merge
  question stays about `roots.rs` and `sqrt_ext.rs` — which is what ADR-018 already says it
  is. `api-shape.md` §5's sentence "Nothing in this document makes that merge more
  expensive" was false as written; under this ADR it becomes true.
- **`resolvent-base` carries resolvent's evidence vocabulary** (`Certified`, `Certainty`,
  `ProofKind`) alongside the trait tower, and a consumer implementing `Ring` imports both.
  `challenge-evidence.md` §2.2 flagged this as a quiet violation of "additive and
  non-viral" *on the assumption that the crate would be pitched as the ecosystem's numeric
  hook*. It is not, so the vocabulary does not spread by import. Splitting the crate is
  therefore not done — `gcd` returns `Certified<T>`, so the "numeric" and "evidence" halves
  would depend on each other and the split would buy nothing. **If `resolvent-base` is ever
  re-pitched as an ecosystem dependency, split it first**; doing so after publication is a
  breaking split of a crate the ecosystem depends on.
- **A word-sized consumer type is genuinely cheap to implement.** Seven methods, three
  defaulted, no `reduce`, no `crt_lift`, no content, no bit-length.
- **Compile time is unchanged.** The instantiation set resolvent itself compiles is closed
  and unchanged (ADR-006 §2.4). A consumer's foreign instantiation is compiled in the
  consumer's crate, at the consumer's cost.

---

## Alternatives considered and why rejected

**The sealed coefficient set (`api-shape.md` §3.3).** Rejected. Its argument does not
survive contact with a well-factored tower, it contradicts `architecture.md` on a declared
one-way door, and it fails the founding 200-line acceptance criterion for a whole consumer
class. Its one genuine virtue — a bounded instantiation set — is preserved by ADR-006's Tier
G, which bounds what *resolvent* instantiates without bounding what a consumer may.

**Two seams: the sealed set for coefficients plus the open `Scalar` for evaluation.** The
shape the earlier notes actually proposed. Rejected on three counts: a consumer's type then
needs two impls and the docs must say which text is written against which; `Scalar` would be
a fourth scalar vocabulary in a workspace with three, colliding with an incumbent designed
for exactly the role it claimed; and the split is not real — Horner, Bareiss and de
Casteljau need ring operations, which is what `Ring` is.

**Adopting `arrangements/crates/scalar-seam` directly.** Rejected, and the reason is
specific rather than territorial: `scalar_seam::Scalar::from_f64` panics on NaN at the exact
rung (`scalar-seam/src/lib.rs:104-109`) and `to_f64` is a lossy readout on the trait
(`:111-115`), which violate INV-4 and INV-7. `lazy_exact::exact::ExactField::div` panics on
zero. resolvent cannot adopt a trait whose contract it must break. The three deltas that
justify a distinct vocabulary are exactly: fallible `from_f64`, fallible division, no lossy
`to_f64` on the trait. They are written down here so that a later merge is a diff rather
than an archaeology project.

**`Box<dyn Ring>` or ring-object arithmetic (`ring.add(&a, &b)`).** Rejected in ADR-006 and
unchanged: an indirect call per coefficient operation, and `feanor-math`'s
`RingBase`/`RingStore` split exists to work around Rust limitations under that style — a
warning, not a model.

**Omitting `Send + Sync + 'static` from `Ring` to keep a blanket impl legal.** Declined,
§5. The cost is recorded.

**Keeping `evaluate_with(hom, point)` as a convenience alongside `map_coefficients`.**
Rejected. An acceptance sketch is a specification of intended use, and the convenience is
the slow idiom. If both exist, the wrong one gets written.

---

## What would reverse this

- **A measured compile-time blowup traced to consumers' foreign instantiations.** Response,
  in order: the inner-function trick on large cold generic bodies; feature-gate more of
  resolvent's own instantiations; last, reduce Tier G. The openness of the tower is not the
  variable.
- **A consumer implementing `Ring` dishonestly and silently breaking a modular algorithm.**
  This is the sealed set's strongest argument and it is answered structurally rather than by
  trust: the modular pipeline is bounded by `Reducible + Liftable`, which a consumer's ring
  does not implement unless it genuinely can reduce and lift. If a *bad* `Reducible` impl is
  ever observed in the wild, the response is a documented contract test in `resolvent-base`
  that a consumer runs against its own impl, not a re-sealing.
- **`arrangements` and resolvent merging under ADR-018 option C.** Then `scalar-seam` and
  `resolvent-base` must be reconciled, and this ADR is the record of which three contract
  deltas resolvent will not give up.
