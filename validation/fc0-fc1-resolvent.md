# Resolvent FC0-FC1 validation

Semantic implementation commit: `09a73c977bdd858ec33d5bebb98d5e6e5988d6a9`
Validation workflow source: `e715d059185a0d9f42d35f47de534efbea2a4b60`
GitHub Actions run: `32413315825`

- `cargo fmt --all -- --check`: PASS
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS
- `cargo test --all-features`: PASS

The gate includes deterministic artifacts, scalar-V1 compatibility, truthful claims,
complete receipt edges, declaration-order invariance, explicit facet sides, complex
inner-product convention, mixed blocks, and typed frame/variance contractions.

This metadata-only commit retriggers the ordinary repository CI against the finalized semantic
tree; downstream repositories pin its resulting green head.
