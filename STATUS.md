# Resolvent status

**Updated:** 2026-08-21
**Landed milestone:** R1 shared CAS cutover

## Ownership

Resolvent owns consumer-neutral exact algebra. The `.res` scientific compiler
has moved to Scientia; no compatibility facade remains here.

## Implemented

- exact rational symbolic expressions;
- bounded deterministic canonicalization;
- exact symbolic differentiation for arithmetic and common scalar functions;
- exact evaluation and sign queries for decidable rational expressions;
- dense univariate polynomial arithmetic, division, gcd, and derivatives;
- exact Sylvester resultants with a matrix-dimension budget;
- Sturm real-root isolation with an explicit bisection budget;
- deterministic algebra-operation receipts.

## Validation

Passed locally on 2026-08-21:

```text
cargo fmt --all -- --check
cargo check --locked --workspace --all-targets
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets             # 4 tests
RUSTDOCFLAGS='-D warnings' cargo doc --locked --workspace --no-deps
cargo test --locked --workspace --doc
git diff --check
```
