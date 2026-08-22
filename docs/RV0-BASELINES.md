# RV0 baseline and stress corpus

The repeatable non-gating timing entry point is:

```text
cargo bench --locked --bench ladder
```

It records deterministic input corpora for filtered-versus-exact 2x2
determinants and lazy shared construction versus eager rational evaluation.
Correctness agreement is asserted; elapsed time and speedup are observational.
Coefficient growth is exposed by `Rational::bit_size`, `poly_bits`, and
`PolyMat::max_bits`. Polynomial/resultant and root behavior are covered by
`tests/algebra.rs` and `tests/roots_isolation.rs`; exact matrices by the
`ratmat`/`polymat` unit suites.

The lazy-runtime stress gate is the `real` unit suite:

- 200,000-node exact evaluation without recursive stack use;
- 200,000-node unevaluated teardown;
- shared tuple/formula nodes;
- concurrent forcing of overlapping DAGs;
- monotone interval-cache tightening with exact-value agreement;
- deterministic node-budget exhaustion and resumable forcing.

These locations are frozen for RV0. A performance change is a change-point to
investigate, not by itself a correctness failure.
