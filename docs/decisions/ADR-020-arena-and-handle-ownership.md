# ADR-020 — Arenas are caller-owned values; handles are arena-relative

**Status:** Ratified 2026-07-31
**Reversibility:** one-way — arena ownership is inherited by every term type and every
expression signature above it, and a handle's identity semantics cannot be tightened after
publication without breaking callers
**Amended:** 2026-07-31 — §6 aligned with ADR-008's content-derived `MonomialId`s and
ADR-012's INV-M1.
**Gates lanes:** P2, P3, X1, G2.
**Evidence:** `docs/research/consumer-sinbad.md` §3 (D1, D6), §5.6;
`consumer-cadabra2.md` §11.2; `consumer-solverang.md` §5, §7 R9;
`docs/research/challenge-generality.md` §1.2(a), §1.5, §6;
`docs/research/algorithms-and-representation.md` §1.6; ADR-008, ADR-009, ADR-012, ADR-017.

---

## Context

resolvent has two arenas, at two layers, and they were decided in two documents that did not
reference each other.

**Layer 1, the monomial arena.** ADR-008 specifies "interned arena + packed key + divmask",
marked *one-way (interning)*, because interning is what makes the divisor-query index and
the multiplicative hash `h(u) + h(v) = h(uv)` possible — worth 10–20× in the reduction path,
against bit-packing's measured ~15%. `plans/api-shape.md` L1-4 specifies the opposite in
words: "**No global monomial interner.** Packed exponent keys live inline in the term …
`MPoly` stays a self-contained `Send + Sync` value." `plans/roadmap.md` §2.5 flags this as
live contradiction 2, blocks lanes P1/P2/P3 on a deciding microbenchmark, and observes that
the two may be reconcilable because "no *global* interner" does not forbid an arena owned by
a ring context passed explicitly.

**Layer 4, the expression store.** `api-shape.md` §1.2 adopts a caller-owned `Store`,
rejects a global/thread-local interner (it would break bit-identical output across thread
counts, `sinbad/crates/sinbad-pal/src/repro.rs:20-21`, and content addressing,
`sinbad/crates/rutter/src/lib.rs:11-14`), and rejects a per-call arena (plexus differentiates
the same equation set repeatedly across Pantelides rounds; cadabra2's certificate tether
compares a claim built at mint time against one rebuilt later). It then accepts a residual
hazard: an out-of-range `Expr` returns a typed error, but **an in-range `Expr` from a
different store yields a wrong answer, not an error** — and declines a store tag because it
"taxes every consumer for a bug none of the three would make."

`challenge-generality.md` falsifies that justification against two of five outside
consumers: a Python binding user with two `Store`s in one script or two notebook cells makes
the bug immediately (§1.5), and a search loop that rolls a store back makes it through a
*supported* operation, because after a rollback every outstanding handle is in-range and
points at a different node (§1.2(a)). It also notes (§6) that there is no supported way to
move an expression between two stores at all: `canonical_bytes` is an addressing function,
not a decodable form, so every parallel, multi-process or distributed-cache consumer writes
the same walk-and-rebuild.

The common question underneath both layers is the same one, and it has never been stated as
one decision: **who owns an arena, and what does a handle mean?**

---

## Decision

> **Every arena is a value the caller constructs and owns. Handles are arena-relative,
> never serialized, and never escape into a computed result. There is no global,
> thread-local, or implicit interner at any layer.**

### 1. Layer 1 — the monomial arena is owned by the `Ring` context

The arena, if there is one, lives in the `Ring` context value and is reached through it
explicitly. `Ring` is constructed by the caller (`Ring::new(vars, order) -> Result<Ring>`)
and passed to every operation that needs it. There is no process-global monomial table.

This reconciles ADR-008 with `api-shape.md` L1-4 without conceding either's substance:
interning survives (so the divisor-query index and the multiplicative hash survive), and
ambient state does not (so ADR-012's determinism contract survives).

**What remains open, deliberately:** whether terms are `(MonomialId, C)` into that arena or
`(PackedMon, C)` inline. That is a *term-type* question and it is decided by roadmap §2.5's
microbenchmark — inline packed monomials against ids-plus-arena-lookup on a realistic S-pair
queue workload, plus the divisor-query index's speedup under each — before lanes P1/P2/P3
start. This ADR fixes the ownership rule so that the experiment decides a representation
rather than an architecture.

### 2. `MPoly` carries its ring by an owned handle, never by `&'a Ring`

`plans/architecture.md` §5.2 writes `MPoly { …; borrows &Ring by handle }`. That phrase is
read, and should be restated, as an **owned** handle: `Arc<Ring>` or an index into a
caller-held ring table. Not `&'a Ring`.

Two requirements force it. `MPoly` must be `Send + Sync` and storable in a consumer's own
struct without infecting that struct with a lifetime — a public owned type carries no
lifetime parameter. And `consumer-solverang.md` §7 R9 requires that an adapter build rings
**from data**: per-constraint arity runs 2..14
(`solverang/src/sketch3d/constraints.rs` `Parallel3D`, 12 params;
`assembly/constraints.rs` `Insert`, 14), so the ring is a runtime value and the polynomial
that names it must own its reference to it.

### 3. Layer 4 — `Store` is a caller-owned value, and its growth is monotone

`Store` is a plain owned struct, `Send`, holding the hash-cons table, the symbol interner
and the `FuncTable`. One value, not three. `Expr` handles are `Copy`, store-relative, and
never serialized; canonical bytes are computed structurally, so two stores built by the same
call sequence produce identical bytes and possibly different handles (ADR-012 §9).

**`Store` growth is monotone for the lifetime of the store, and L4 is not designed for a
backtracking search loop.** Stated rather than implied. A consumer whose terms are
polynomials — which is the SMT/MCSAT case — stays on Layer 1, where a term is a
self-contained droppable value with no shared arena state.

### 4. Cross-store movement is supported, once, in resolvent

```rust
impl Store {
    pub fn rebuild_from(&mut self, src: &Store, e: Expr) -> Result<Expr, Error>;
}
```

Roughly 30 lines over `walk_topological`. Without it, every parallel, multi-process or
distributed-cache consumer writes the same walk-and-rebuild in its own glue. Two or more
hypothetical consumers by the placement rule's own arithmetic; it is absent from the earlier
notes only because none of the three surveyed consumers is parallel at Layer 4.

### 5. The wrong-arena hazard: bounds-checked, documented, and closable by the consumers who need it

- Every entry point **bounds-checks** a handle and returns
  `Error::Domain { fault: ForeignNode }` when it is out of range.
- An **in-range** handle from a different arena yields a wrong answer. This is documented on
  `Expr`, on `Store`, and on `MonomialId`, in those words.
- An optional **`store-tags` feature, default off**, closes it for consumers that need it.
  `Store::with_tag(tag: u64)` records a **caller-supplied** tag; `Expr` carries it; every
  entry point checks it and returns `ForeignNode` on a mismatch. Caller-supplied is what
  keeps this compatible with the no-ambient-state rule: there is no hidden counter, no
  address, no clock, and the tag is a pure input, so output remains a pure function of
  input.
- **If a checkpoint API is ever added, the tag is the generation counter.** That is the
  mechanism that would make `mark`/`rollback_to` safe rather than silently wrong, and it is
  why the tag is specified now even though no local consumer turns it on.

### 6. Handles never reach a result

No handle appears in canonical bytes, in a certificate, in an error payload that a consumer
compares across runs, or in any ordering that affects an output.

*Amended 2026-07-31.* This section originally rested on "`MonomialId`s are assigned in
first-encounter order under a deterministic traversal (ADR-012 §4), so they are
reproducible". First-encounter order is reproducible only under single-threaded interning,
and an interner is a shared mutable accumulator that ADR-012 §5 bans — while symbolic
preprocessing, the phase most likely to be parallelized next, is nothing but interning. Two
rules now hold jointly and either alone would suffice:

- **`MonomialId` is a pure function of the packed key** (ADR-008 §1), so parallel interning
  is deterministic by construction.
- **INV-M1: no tie-break anywhere may consult `MonomialId` ordering** (ADR-012 §4).
  Tie-break on `key`, which is content-derived and totally ordered.

---

## Consequences

- **ADR-008 and the API's no-ambient-interner rule stop contradicting each other**, and the
  roadmap §2.5 contradiction-2 experiment is narrowed from "which architecture" to "which
  term type", which is what it is actually able to measure.
- **Determinism and content addressing hold at both layers**, which is sinbad's D1 and D6
  and solverang's cross-platform requirement, from two independent consumers.
- **Two independent consumers in one process never share a table neither can reason about.**
  This is the standard CAS mistake and it is what makes `Ctx`-free embedding fail in most
  symbolic libraries.
- **A consumer pays for interning explicitly.** Passing a `Ring` to every multivariate
  operation is more verbose than a global table. That verbosity is the price of the
  determinism contract and is charged openly.
- **`Arc<Ring>` costs a refcount bump per `MPoly` clone and an indirection per term
  decode.** Whether that indirection defeats the interned comparison key is exactly roadmap
  §2.5's open microbenchmark, and it is the reason the term type is not decided here.
- **The wrong-arena hazard is not eliminated in the default configuration.** It is
  bounds-checked, documented, and closable. A consumer that considers it unacceptable turns
  on one feature. The previous justification — that no consumer would make the mistake — is
  retired, because it was falsified.
- **`rebuild_from` is a small amount of resolvent code that no surveyed consumer calls.**
  Accepted: it is written once here or N times in glue, and the glue version is the one that
  gets it wrong on `FuncId` remapping.

---

## Alternatives considered and why rejected

**A global or thread-local interner at either layer.** Rejected. Output would depend on
interning history and therefore on thread count, breaking `Reproducibility::Bitwise`; and
canonical bytes would depend on process-global insertion order, breaking content addressing.
It also makes two independent consumers in one process share state neither controls.

**A per-call arena.** Rejected. It destroys the only thing hash-consing buys. plexus
differentiates the same equation set repeatedly across Pantelides rounds; cadabra2's
`certifies` tether compares a claim built at mint time against one rebuilt later
(`cadabra2/crates/cadabra-check/src/certificate.rs:41-42`). Both need node identity stable
across calls.

**A lifetime parameter — `MPoly<'a, C>` borrowing `&'a Ring`.** Rejected. A consumer must be
able to put a resolvent value in its own struct without infecting that struct with a
lifetime, and `consumer-sinbad.md` §5.6 and `consumer-solverang.md` §7 R9 both need the ring
to be runtime data the adapter constructs.

**A mandatory store tag, on by default.** Rejected. It costs every consumer 8 bytes per
handle and a comparison per entry point for a hazard that two of five consumer classes face.
Default-off with a documented hazard is the honest trade; the previous position — no tag at
all, justified by "a bug none of the three would make" — was not, because it generalized
from a survey of three.

**An ambient counter to mint store tags automatically.** Rejected. That is
`static`-with-interior-mutability by another name, and it makes the tag — and therefore
whether a call errors — a function of process history.

**`canonical_bytes` as a decodable form, with `Store::import(bytes)`.** Rejected as the
cross-store mechanism. Canonical bytes are an *addressing* function whose stability is a
versioned promise (ADR-012 §9); making them a wire format couples the addressing schema to
the serialization schema and turns every canonical-form change into a data-migration event
rather than only a re-key event. `rebuild_from` moves an expression between two live stores
without that coupling. A `serde` wire form, if one is ever wanted, is a separate,
independently versioned artifact behind the default-off `serde` feature.

**`Store::mark()` / `rollback_to()` now.** Rejected for now, not forever. No surveyed
consumer backtracks at Layer 4, and the SMT case that would has a better answer (stay on
Layer 1). Adding it later is additive **provided** handles can carry a generation, which is
what §5's tag mechanism guarantees. Adding it *without* generation-tagged handles would make
the silent-wrong-answer failure reachable through a supported operation, which is the one
outcome this ADR exists to prevent.

---

## What would reverse this

- **The roadmap §2.5 microbenchmark showing arena lookup dominating the comparison key.**
  Then terms carry inline packed keys and the arena shrinks to a divisor-query index. The
  ownership rule does not change; only what the `Ring` context holds does.
- **A consumer demanding a checkpoint API.** Response: ship `mark`/`rollback_to` **with**
  generation-tagged handles promoted from the optional feature to always-on for `Store`.
  That is a breaking change to `Expr`'s size and should be taken as a major version, which
  is why the tag is designed now.
- **A measured cost for `Arc<Ring>` in `MPoly` clone-heavy workloads.** Response: an index
  into a caller-held ring table, which is the same ownership rule with a cheaper handle.
- **`no_std` becoming a requirement.** `Arc` requires `alloc`; the arena rule does not.
  Layer 1 and Layer 4 are `alloc` consumers by nature, and `resolvent-base` — which holds no
  arena — is the crate for which the `no_std` question is live.
