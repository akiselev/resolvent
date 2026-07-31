# Fact-check: the load-bearing claims behind the one-way doors

**Checked 2026-07-31, independently of the agents that made the claims.** These are the facts
that, if wrong, would poison a decision that is expensive or impossible to reverse. Every check
below was run directly against crates.io or the local filesystem, not recalled.

---

## 1. ADR-002 — the bignum license landscape

**Claim:** the fastest permissive-*looking* option is not actually permissive, forcing the
choice to `dashu`.

**Verified against the crates.io API:**

| Crate | Latest | License | Verdict |
|---|---|---|---|
| `dashu` | 0.6.0-rc.1 | **MIT OR Apache-2.0** | usable |
| `num-bigint` | 0.4.8 | MIT OR Apache-2.0 | usable |
| `ibig` | 0.3.6 | MIT OR Apache-2.0 | usable |
| `crypto-bigint` | 0.7.5 | Apache-2.0 OR MIT | usable |
| `malachite` | 0.10.0 | **LGPL-3.0-only** | **barred** |
| `rug` | 1.30.0 | **LGPL-3.0+** | **barred** (dev-oracle only) |

**Confirmed.** `malachite` is LGPL-3.0-only, consistent with the ADR's reasoning that it is
derived from GMP/FLINT/MPFR source. `rug` is LGPL-3.0+. Both are unavailable to an
MIT-OR-Apache-2.0 product, which is exactly the constraint that makes this project's dependency
choice narrow rather than free.

## 2. ADR-001 — the `alkahest-cas` example

**Claim:** `alkahest-cas` 3.7.0 ships today as Apache-2.0 with mandatory `rug` dependencies,
which is why the license gate must be mechanical rather than habitual.

**Partially verified.** The crate exists, `max_version` is 3.7.0, and versions 3.5.1 / 3.6.0 /
3.7.0 are all **Apache-2.0** (single-arm, no MIT). That is enough to carry ADR-001's argument:
Apache-2.0-only is GPLv2-incompatible, so an Apache-only crate is barred under the stated
policy regardless of what it depends on. The *specific* "mandatory `rug` deps" half of the claim
was not independently checked and is not load-bearing for the decision.

## 3. ADR-002 — is `dashu` a safe bet?

**Not previously checked by any agent.** Findings:

- `max_stable_version` is **0.5.2**, released **2026-07-31** — i.e. today. The `0.6.0-rc.1` that
  the API returns as "latest" is a release candidate; resolvent should pin the stable line.
- Sub-crates resolvent would actually depend on are current and healthy:
  `dashu-int` 0.5.1 (876k recent downloads), `dashu-ratio` 0.5.1 (574k), `dashu-base` 0.5.1.
- **Release cadence has a gap worth knowing about:** 0.4.1/0.4.2 landed in January 2024, then
  nothing until 0.4.3 in May 2026, followed by a burst — 0.4.4, 0.5.0, 0.5.1, 0.5.2 and an RC
  all within the last ten weeks.

**Assessment:** the dependency is alive and heavily used, but the entire 0.5.x line is three
weeks old, so API churn is a live risk and the ~16-month dormancy in 2024–2025 shows the project
can go quiet. This is an argument *for* ADR-002's `resolvent-int` newtype wall, not against the
choice — the wall is what makes a later swap or an in-house half-GCD survivable. Pin to the
stable 0.5.x line, not the RC.

## 4. Challenge finding S1 — the `scalar-seam` collision

**Claim:** a proposed `resolvent-seam` crate would collide with an existing zero-dependency
`scalar-seam` crate that cadabra2 *consumes but did not author*.

**Confirmed, and the real situation is stronger than the finding states.**

`/home/dev/projects/arrangements/crates/scalar-seam` exists — 669 LOC, and its own package
description reads:

> The Scalar seam: the minimal arithmetic/comparison trait that lets numeric code be written
> once and run on either fast f64 or a certified-exact real. A zero-dependency leaf crate (no
> bignum), so float-only builds never pull exact-arithmetic weight.

Its `[dependencies]` section is **empty** — the zero-dependency claim is literally true, not
aspirational. Consumers, verified from `Cargo.toml` files:

- `arrangements` workspace member, and `lazy-exact` depends on it
- `cadabra2` depends on it **cross-repo by path** (`cadabra2/Cargo.toml:37` →
  `../arrangements/crates/scalar-seam`) and uses it in three crates: `cadabra-core`,
  `cadabra-geom`, `cadabra-algorithms`

So a `Scalar` seam serving exactly the intended role already exists, is already shared across two
repositories, and is already the common vocabulary between `arrangements`, `lazy-exact`, and
`cadabra2`. Shipping a second one from resolvent would mean two competing seams in the same
dependency graph.

**This needs the repository owner's decision, because every clean resolution has a cost:**

1. **Resolvent ships no seam at all.** Consumers implement their own trait for resolvent's
   concrete types. Keeps the dependency arrow pointing at resolvent and honors constraint #1
   exactly. Cost: no shared vocabulary, so each consumer writes its own adapter — which is what
   the 200-line adapter test already assumes, so this may be free in practice.
2. **Resolvent implements `scalar-seam::Scalar` for its types.** Best ecosystem fit and zero
   duplication. Cost: **violates constraint #1** — resolvent would depend on a crate inside the
   `arrangements` repository. Could be made optional behind a feature, but an optional dependency
   on a local, unpublished, path-only crate is not something a published crate can express.
3. **`scalar-seam` is extracted to its own published, neutral crate** that both `arrangements`
   and `resolvent` depend on. Cleanest long-term. Cost: it is a change to `arrangements`, i.e.
   it partly un-defers the deferred integration decision.

Option 1 is the only one that requires no change outside resolvent and forecloses nothing.
Recommend it as the default, with option 3 recorded as the eventual target if and when the
deferred integration decision is taken up.
