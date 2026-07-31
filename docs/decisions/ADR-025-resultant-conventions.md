# ADR-025 — Resultant, subresultant and degenerate-input conventions

**Status:** Ratified 2026-07-31
**Reversibility:** costly — three independent implementations and every external oracle
adapter are written against it, and a convention change re-triages every recorded
disagreement
**Gates lanes:** T1, T2, T3, T4, T5, T6, U4, M8-S.
**Evidence:** `docs/research/critique-plan.md` C20c and its closing note;
`plans/verification.md` §4.3 (normalization), §5.2 (degree-drop specializations);
`docs/research/algorithms-and-representation.md` §6.3.

---

## Context

M4 builds **three independent resultant implementations** — Ducos subresultant PRS,
modular evaluation–interpolation, and a Bareiss/Bézout determinant — specifically so that
their agreement is a strong verdict. That agreement is worth exactly as much as the
convention they share, and resultant conventions genuinely differ between sources on three
axes:

1. **Degenerate inputs.** `Res(f, g)` when a leading coefficient vanishes, when a degree
   drops under specialization, when an argument is constant, or when an argument is zero.
   Published conventions include `0`, `1`, `lc(f)^{deg g}`, and "undefined".
2. **Sign under argument swap.** `Res(f,g) = (−1)^{mn} Res(g,f)`, and "up to sign" is the
   comparison a naive harness writes — which hides genuine sign bugs permanently.
3. **Subresultant scalar normalization.** The `S_i` differ by a scalar factor between
   sources (Collins, Brown, Ducos, Loos, and the computer-algebra systems all pin it
   differently), and the *degree sequence* — which is the structural content — is common to
   all of them.

The generator fleet contains **degree-drop specializations as an adversarial family**, so
all three routes will meet degenerate instances on purpose, from the first corpus run. If
the convention is unpinned when T1/T2/T3 start, they will disagree there permanently, every
disagreement will be triaged as a bug, and the triage queue will never empty. Pinning it is
five lines and it is only cheap before the three lanes exist.

Two further facts constrain the choice. First, the consumer-facing requirement is that an
identically-vanishing resultant — the shared-component case `f = h·f'`, `g = h·g'` — is a
**distinguishable result, never a silently-empty root list**; the nearest prior art fails
closed here and resolvent must too. Second, ADR-004 makes coefficients ℤ-primitive, so
"equal up to a positive rational factor" is not an acceptable answer: the canonical associate
is defined, and a resultant is a specific integer, not an equivalence class.

---

## Decision

### 1. The value

`Res_x(f, g)` is the determinant of the Sylverster matrix of `f` and `g` **with respect to
`x`, at their nominal degrees `m = deg_x f` and `n = deg_x g` as stored** — i.e. after
resolvent's canonical normalization (content removed, leading coefficient positive,
ADR-004), so the nominal degree is the true degree and there is no "declared degree" concept
in the public API.

`Res_x(f, g) ∈ ℤ` for `f, g ∈ ℤ[x]`; `Res_y(f, g) ∈ ℤ[x]` for bivariate `f, g ∈ ℤ[x,y]`,
returned as `UPoly<Integer>` in canonical form.

### 2. Degenerate inputs, exhaustively

| Case | Returns |
|---|---|
| `f = 0` or `g = 0` | `Res = 0` |
| `f`, `g` both nonzero constants | `Res = 1` (the empty determinant) |
| `f` constant `c ≠ 0`, `deg g = n ≥ 1` | `Res = c^n` |
| `deg f = m ≥ 1`, `g` constant `c ≠ 0` | `Res = c^m` |
| `deg f = deg g = 0`, one of them `0` | covered by row 1 |
| **Common non-constant factor** (`deg gcd(f,g) ≥ 1`) | `Res = 0`, **and** the API returns `Degenerate::CommonComponent { gcd }` alongside — never a bare zero that a caller mistakes for "no common roots" |

The last row is the load-bearing one and it is why the return type is not a bare integer:

```rust
pub enum ResultantOutcome {
    Value(UPoly<Integer>),                  // bivariate; UPoly of degree 0 in the univariate case
    CommonComponent { gcd: MPoly },         // Res == 0 because f and g share a factor
}
pub fn res_y(f: &MPoly, g: &MPoly, y: VarId, b: Budget)
    -> Result<Certified<ResultantOutcome>>;
```

An identically-zero resultant and a resultant that happens to evaluate to zero are different
events with different consumer responses, and a `0` that means both is the shape of a silent
wrong answer in an arrangement engine.

**Degree drop under specialization is not a convention question — it is a bad
specialization.** In the modular evaluation–interpolation route, an evaluation point at which
`lc_y(f)` or `lc_y(g)` vanishes is **rejected and recorded by index** with reason
`BadPrime::DegreeDrop` (ADR-010 §4, ADR-012 §7). The route never computes a "resultant at a
reduced degree" and then corrects it, because the correction factor is exactly where the
conventions differ.

### 3. Sign under argument swap

`Res(f, g) = (−1)^{mn} · Res(g, f)`, with `m = deg f`, `n = deg g`. This is applied
**explicitly**, in the oracle adapters and in the cross-route comparison, and:

> **No comparison anywhere accepts "up to sign".** A harness that accepts `±Res`
> unconditionally cannot distinguish the swap convention from a sign bug in variation
> counting or in a pseudo-division ladder, and sign bugs in exactly those two places are a
> named failure class (ADR-023 §1, the sign-flip mutant).

Argument order in every emitted oracle input is pinned and the oracle is asserted to echo it
back.

### 4. Subresultant chain

- The **degree sequence** is compared first and must match exactly; it is the structural
  content and it is convention-free.
- Each `S_i` is compared **after** applying the pinned scalar convention, and the conversion
  lives in the adapter, never in a test body.
- **resolvent's convention is Ducos's**, as stated in the paper the module cites
  (`Derivation:` per ADR-001 gate 4), because the Ducos PRS is the reference implementation
  and pinning the convention to the reference removes one conversion from the internal
  cross-check.
- The **principal subresultant coefficients** are returned as first-class data (M8 needs
  them for CAD), in the same convention.

### 5. Two structural invariants that every route asserts on every output

Free, and they are the difference between "three implementations agree" and "three
implementations are right":

- **Degree bound.** `deg_x Res_y(f,g) ≤ deg_y(f)·deg_x(g) + deg_y(g)·deg_x(f)`.
- **Vanishing criterion.** `Res == 0 ⇔ deg gcd(f,g) ≥ 1`, cross-checked against the gcd lane
  — which is an independent implementation, so this is a genuine cross-check and not a
  restatement.

The cofactor identity `u·f + v·g == Res` is a certificate (ADR-010 §2, `ProofKind::Identity`)
and is produced by the PRS route directly; the other two routes carry it by cross-check
against the PRS route rather than by producing it.

---

## Consequences

- **The three-route agreement becomes a real verdict**, because the routes are comparing the
  same object on the degenerate instances the fleet generates on purpose.
- **The triage queue does not fill with convention disagreements**, which is the concrete
  saving and it is large: degree-drop specializations are an adversarial *family*, so the
  disagreements would have arrived in bulk.
- **`ResultantOutcome` is more verbose than an integer**, and every caller matches on it.
  Accepted: the alternative is a `0` with two meanings in the API's most consumer-visible
  elimination call.
- **Pinning to Ducos's scalar convention means the oracle adapters carry conversions**, one
  per oracle, written once in the adapter. That is the correct place for them — a conversion
  in a test body is a conversion nobody reviews.
- **Cost of being wrong about the convention later is three implementations plus every
  recorded disagreement.** That is why this is `costly` rather than `cheap`, and why it is
  decided before T1 rather than during it.

---

## Alternatives considered and why rejected

**Leave the conventions to the implementations and reconcile in the comparison.** Rejected —
this is the status quo the ADR exists to prevent. It converts a structural agreement test
into a per-instance judgement call, and judgement calls are what founding constraint #3
cannot afford.

**Accept `±Res` in cross-route comparison "because the sign convention is a detail".**
Rejected. The sign is where two of the plan's named bug classes live, and accepting both
signs makes the strongest available check blind to exactly them.

**Return a bare `UPoly<Integer>` and document that `0` may mean a common component.**
Rejected. The nearest prior art fails closed on this case explicitly, an arrangement engine
that reads `0` as "no intersections" produces a topologically wrong arrangement, and
documentation is not a mechanism.

**Define `Res` at a caller-declared degree, so degenerate specializations can be handled by
the caller.** Rejected: it introduces a "declared degree" into the public API, it makes the
canonical-associate invariant conditional, and the case it exists for — degree drop under
specialization — is correctly handled by *rejecting the specialization*, which the modular
machinery already does for other reasons.

**Adopt a system's convention (PARI's, or sympy's) so the oracle comparison needs no
conversion.** Rejected on two grounds. The systems disagree with each other, so at most one
adapter is simplified; and pinning resolvent's internals to an external system's convention
is the shape of decision ADR-001 Tier B warns about — it makes an external source normative
for something resolvent must own.

---

## What would reverse this

- **A consumer needing a resultant at a declared degree** — e.g. a CAD implementation that
  wants the specialized resultant rather than a rejected specialization. Response: an
  explicitly named `res_at_degree(f, g, m, n)` alongside, with its own convention stated;
  additive, and it does not change the default.
- **The Ducos scalar convention proving awkward for the principal subresultant coefficients
  M8 needs.** Response: change the convention *once*, in this ADR, before M8's lanes start,
  and re-triage — which is the cost this ADR prices as "costly" and is exactly why the
  convention is written down rather than discovered.
- **A fourth route** (e.g. a Bézout-matrix variant with a different degenerate behaviour).
  Response: it conforms to §2 or it is not a route; the conventions are the interface between
  routes, not a property of any one of them.
