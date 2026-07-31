# ADR-028 — A `ProofKind` variant names the certificate that is actually run

**Status:** Proposed (2026-07-31)
**Reversibility:** one-way in practice — `ProofKind` is public, consumer-read, and matched on;
renaming a variant after publication is a breaking change
**Amends:** ADR-010 §2 (the certificate type and its `ProofKind` union).
**Gates lanes:** Z0, U3, and every certificate-graded lane that mints a `Certainty`.
**Evidence:** `docs/decisions/RECONCILIATION.md` §2.4; `docs/research/critique-plan.md` C3;
`ADR-023` §2 (rule C); `VERIFICATION.md` §2.2, §14 row 2; `DESIGN.md` §5.4, §8.3;
`ROADMAP.md` M2 exit gate; `API.md` §5.1.

---

## Context

`critique-plan.md` C3 found both gcd certificates circular. The Layer-0 form
(`g|a`, `g|b`, `gcd(a/g, b/g) == 1`) and the Layer-2 form (`H|A`, `H|B`,
`deg H == deg gcd(A mod p, B mod p)`) are each passed by `fn gcd(_,_) -> 1`, because the
clause that is supposed to do the work is computed by the routine under test. The Layer-2 row
was marked **Complete** with an empty "does not prove" cell.

The fix landed everywhere. `VERIFICATION.md` §2.2 is now rule C — "a certificate may not
invoke what it certifies, nor any routine on that operation's call graph" — and §14 row 2
records the rewrite around **Bézout witnesses**. `ADR-023` §2 carries the rule. `DESIGN.md`
§8.3 states it inside the API sketch. `ROADMAP.md` M2's exit gate requires the gcd
certificate "in its **non-circular** form … `H|A`, `H|B`, **and** a Bézout pair `(u,v)` with
`u·A + v·B == H`", with the identity mutant rejected. `CLAUDE.md` §1 and `NEXT.md` day 6 both
spell out that `gcd(a/g, b/g) == 1` is the wrong test and why.

**The certificate changed everywhere. The name of the public enum variant that reports it did
not.** `DESIGN.md` §5.4 and `API.md` §5.1 both carried:

```rust
DivisibilityAndDegree,     // the gcd certificate
```

That names the retired, demonstrably circular argument. It is not a comment: `ProofKind` is
a public type a consumer reads off `Certainty::Proved(_)` and matches on, and the enum is the
library's only self-description of what it proved. A variant named for the degree half tells
every consumer that resolvent's gcd is certified by a degree comparison — which is the claim
C3 falsified — and it is a standing invitation to the next implementor to reintroduce it,
since the enum says that is what the certificate is.

This is a small defect with a general cause, and the general cause is what this ADR is for: a
certificate's *implementation* was corrected in five documents by a process that had no rule
tying the correction to the public name that reports it.

---

## Decision

### 1. The variant is renamed, and the union is fixed

```rust
/// The union of the variant sets the founding documents each declared
/// (ADR-021 register item 9). Adding a variant is additive; renaming one is breaking.
#[non_exhaustive]
pub enum ProofKind {
    Identity,                                           // a·b/b == a and friends
    DivisibilityAndBezout,                              // H|A, H|B, and u·A + v·B == H
    BoundDriven { bound_bits: u64, primes_used: u32 },  // Landau–Mignotte / Hadamard
    CofactorRepresentation,                             // Gröbner: f = Σ hᵢgᵢ
    ProductAndModularIrreducibility { primes: SmallVec<[u32; 4]> },
    RootCount,                                          // the sign-variation witness
    Enclosure,                                          // Bernstein / de Casteljau
    ExhaustiveSmallCase,
}
```

`DivisibilityAndDegree` does not survive under any spelling. There is no deprecated alias,
because the crate is unpublished and an alias would preserve exactly the wrong reading.

### 2. The rule, which is the point of this ADR

> **A `ProofKind` variant names the argument the shipped certificate actually runs. When a
> certificate's argument changes, the variant is renamed in the same commit, and the
> certificate catalogue row, the mutant set and the variant name are checked against each
> other in review.**

Two corollaries:

- **`ProofKind` is `#[non_exhaustive]`.** Adding a proof kind is additive and must not be a
  breaking change, or the pressure will be to reuse an ill-fitting existing variant — which
  is the same failure in a different direction.
- **A variant is not a category label.** `DivisibilityAndBezout` names three checks that are
  all run. If a future gcd path can only produce two of them, it returns a *different*
  variant or `Certainty::Probable`, never this one with a weaker payload.

### 3. The grep gate covers it

`ADR-021` §4's code-block divergence gate already lists `ProofKind`, so two documents
defining the enum differently fails CI. This ADR adds nothing mechanical; it adds the reason
the gate must be pointed at the variant *names* and not only at the type name, since the
divergence that produced this ADR was two documents agreeing on a name that was wrong in
both.

---

## Consequences

- **A consumer that matches on `ProofKind` learns what was actually proved.** For the
  surveyed consumers this is thin — `API.md` §5.5 records that all three use a certificate's
  *presence* as an admission ticket and none reads the evidence — but `API.md` §8.1's
  proof-assistant class reads exactly this, and it is the class the design rule was
  previously over-generalized against.
- **`gcd_ext` is not a convenience, and the enum now says so.** The Bézout cofactors are free
  in the extended Euclid that already computes them, they make the certificate non-circular,
  and they are the same data an SMT consumer needs for external proof production
  (`DESIGN.md` §9.4). Naming the variant after them makes that visible at the type.
- **One more thing to keep in sync.** Accepted: the rule in §2 is the mechanism, and it is
  the same discipline `CLAUDE.md` §8 already applies to plan-and-code corrections landing in
  one commit.
- **`DESIGN.md` §5.4 must be corrected**, and this ADR is the record of why. `API.md` §5.1 is
  already patched.

---

## Alternatives considered and why rejected

**Leave the name and fix the comment.** Rejected. A doc comment is not what a consumer
matches on, and the defect is precisely that the machine-readable name and the shipped
behaviour disagreed for a week without either track noticing.

**Name it `Gcd`.** Rejected. It names the *operation*, not the argument, and the whole point
of `ProofKind` is that a consumer can tell which argument was run — two operations can share
an argument (`Identity` covers several) and one operation can produce different arguments at
different sizes.

**Keep `DivisibilityAndDegree` as a deprecated alias.** Rejected. The crate is unpublished,
so there is nothing to be compatible with, and an alias would keep the retired argument
discoverable and citable.

**Collapse `DivisibilityAndBezout` into `Identity`, since `u·A + v·B == H` is an identity
check.** Rejected. `Identity` names a round-trip against the operation's *own* inverse
(`a·b/b == a`), which rule C permits only because an independent naive reference exists; the
Bézout witness is a check by data the operation emitted against arithmetic that shares no
control flow with it. Merging them would erase the distinction rule C exists to draw.

---

## What would reverse this

- **A published `1.0` with this enum in it**, after which a rename is a major version and the
  response to a wrong name is a new variant plus a documented deprecation, not a rename.
- **`ProofKind` growing past roughly a dozen variants**, which would mean the enum is being
  used as a per-operation label rather than a per-argument one. Response: split it by layer
  (`ProofKind::Arith`, `::Algebra`, `::Real`) rather than widen the rule.
- **A consumer needing to match exhaustively.** `#[non_exhaustive]` forbids it deliberately;
  if a proof-assistant consumer demonstrates a real need, the answer is a
  `ProofKind::category() -> ProofCategory` accessor over a small closed enum, not removing
  `#[non_exhaustive]`.
