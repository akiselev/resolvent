# FC0-FC1 Resolvent CI diagnostic

- `fmt`: PASS
- `clippy`: PASS
- `test`: FAIL (101)

## fmt

```text

```

## clippy

```text
[1m[92m    Updating[0m crates.io index
[1m[92m     Locking[0m 22 packages to latest compatible versions
[1m[92m Downloading[0m crates ...
[1m[92m  Downloaded[0m arrayvec v0.7.8
[1m[92m  Downloaded[0m cfg-if v1.0.4
[1m[92m  Downloaded[0m cpufeatures v0.3.0
[1m[92m  Downloaded[0m itoa v1.0.18
[1m[92m  Downloaded[0m find-msvc-tools v0.1.11
[1m[92m  Downloaded[0m constant_time_eq v0.4.2
[1m[92m  Downloaded[0m zmij v1.0.23
[1m[92m  Downloaded[0m thiserror v2.0.20
[1m[92m  Downloaded[0m thiserror-impl v2.0.20
[1m[92m  Downloaded[0m quote v1.0.47
[1m[92m  Downloaded[0m shlex v2.0.1
[1m[92m  Downloaded[0m proc-macro2 v1.0.107
[1m[92m  Downloaded[0m serde_derive v1.0.229
[1m[92m  Downloaded[0m unicode-ident v1.0.24
[1m[92m  Downloaded[0m memchr v2.8.3
[1m[92m  Downloaded[0m serde_core v1.0.229
[1m[92m  Downloaded[0m serde v1.0.229
[1m[92m  Downloaded[0m cc v1.4.3
[1m[92m  Downloaded[0m blake3 v1.8.7
[1m[92m  Downloaded[0m serde_json v1.0.151
[1m[92m  Downloaded[0m syn v3.0.3
[1m[92m   Compiling[0m proc-macro2 v1.0.107
[1m[92m   Compiling[0m quote v1.0.47
[1m[92m   Compiling[0m unicode-ident v1.0.24
[1m[92m   Compiling[0m serde_core v1.0.229
[1m[92m   Compiling[0m shlex v2.0.1
[1m[92m   Compiling[0m find-msvc-tools v0.1.11
[1m[92m   Compiling[0m cc v1.4.3
[1m[92m   Compiling[0m zmij v1.0.23
[1m[92m   Compiling[0m thiserror v2.0.20
[1m[92m   Compiling[0m serde v1.0.229
[1m[92m   Compiling[0m syn v3.0.3
[1m[92m   Compiling[0m serde_json v1.0.151
[1m[92m    Checking[0m constant_time_eq v0.4.2
[1m[92m    Checking[0m memchr v2.8.3
[1m[92m   Compiling[0m blake3 v1.8.7
[1m[92m    Checking[0m arrayvec v0.7.8
[1m[92m    Checking[0m cpufeatures v0.3.0
[1m[92m    Checking[0m cfg-if v1.0.4
[1m[92m    Checking[0m itoa v1.0.18
[1m[92m   Compiling[0m thiserror-impl v2.0.20
[1m[92m   Compiling[0m serde_derive v1.0.229
[1m[92m    Checking[0m resolvent-quantities v0.1.0 (/home/runner/work/resolvent/resolvent/crates/resolvent-quantities)
[1m[92m    Checking[0m resolvent v0.1.0 (/home/runner/work/resolvent/resolvent)
[1m[92m    Finished[0m `dev` profile [unoptimized + debuginfo] target(s) in 12.39s
```

## test

```text
[1m[92m   Compiling[0m serde_core v1.0.229
[1m[92m   Compiling[0m zmij v1.0.23
[1m[92m   Compiling[0m thiserror v2.0.20
[1m[92m   Compiling[0m memchr v2.8.3
[1m[92m   Compiling[0m arrayvec v0.7.8
[1m[92m   Compiling[0m constant_time_eq v0.4.2
[1m[92m   Compiling[0m cfg-if v1.0.4
[1m[92m   Compiling[0m cpufeatures v0.3.0
[1m[92m   Compiling[0m itoa v1.0.18
[1m[92m   Compiling[0m blake3 v1.8.7
[1m[92m   Compiling[0m serde v1.0.229
[1m[92m   Compiling[0m serde_json v1.0.151
[1m[92m   Compiling[0m resolvent-quantities v0.1.0 (/home/runner/work/resolvent/resolvent/crates/resolvent-quantities)
[1m[92m   Compiling[0m resolvent v0.1.0 (/home/runner/work/resolvent/resolvent)
[1m[92m    Finished[0m `test` profile [unoptimized + debuginfo] target(s) in 10.19s
[1m[92m     Running[0m unittests src/lib.rs (target/debug/deps/resolvent-27420dd34b98a32e)

running 46 tests
test calculus::tests::derivative_matches_quadratic ... ok
test calculus::tests::sin_chain_rule ... ok
test form_v2::tests::explicit_contract_rejects_equal_spatial_variance ... ok
test form_compile::tests::diffusion_becomes_explicit_discrete_stages ... ok
test form_v2::tests::interior_facet_requires_explicit_sides ... ok
test form_v2::tests::invalid_contraction_reports_frame_mismatch ... ok
test form_v2::tests::complex_inner_and_hermitian_adjoint_are_distinct_and_typed ... ok
test generated_verify::tests::dot_gate_detects_transpose_identity ... ok
test generated_verify::tests::order_is_computed ... ok
test generated_verify::tests::manufactured_derivative_is_exact ... ok
test latex::tests::parses_nonlinear_heat_fragment ... ok
test latex::tests::rejects_unknown_tex ... ok
test migration::tests::frozen_case_detects_tampering ... ok
test migration::tests::tolerance_is_local_and_explicit ... ok
test physics::tests::macro_uses_same_parser_and_elaborator ... ok
test property_tensor::tests::isotropic_tensor_is_rotation_invariant ... ok
test form_v2::tests::mixed_jacobian_blocks_have_stable_digests ... ok
test physics::tests::lock_rejects_source_drift ... ok
test property_tensor::tests::orthotropic_tensor_rotates_axes_and_creates_cross_term ... ok
test reference::tests::shifted_contains_mass_and_stiffness ... ok
test reference::tests::p1_diffusion_mass_source_and_boundary_compile ... ok
test reference_hdiv::tests::constant_divergence_gives_rank_one_local_div_div ... ok
test reference_mixed::tests::elasticity_has_rigid_translation_null_modes ... ok
test reference_hdiv::tests::rt0_is_invariant_to_triangle_vertex_order ... ok
test reference_mixed::tests::nedelec_orientation_is_invariant_to_triangle_vertex_order ... ok
test scientific::tests::canonical_heat_source_is_smooth_and_finite ... ok
test reference_mixed::tests::stokes_has_an_exact_zero_pressure_block_and_symmetric_coupling ... ok
test scientific::tests::production_catalog_covers_agent_gate_spaces ... ok
test scientific::tests::parses_structured_heat_source ... ok
test rsl::tests::parses_and_elaborates_heat_model ... ok
test scientific::tests::property_expression_symbolic_derivative_matches_finite_difference ... ok
test scientific_bridge::tests::constrained_latex_lowers_to_scientific_expression_nodes ... ok
test structural::dae::tests::detects_alias_class ... ok
test scientific_weak::tests::nonlinear_heat_lowers_without_named_physics_special_case ... ok
test structural::scc::tests::components_are_reverse_topological ... ok
test structural::dae::tests::hidden_constraint_requests_differentiation ... ok
test scientific_weak::tests::electrothermal_aliases_lower_into_two_generic_blocks ... FAILED
test structural::schedule::tests::coupled_solver_can_keep_raw_algebraic_loop ... ok
test structural::schedule::tests::dense_loop_is_torn_deterministically ... ok
test form_v2::tests::scalar_adapter_is_lossless_serializable_and_claim_free ... ok
test structural::schedule::tests::lower_triangular_chain_is_explicit_in_order ... ok
test structural::schedule::tests::rlc_shape_has_explicit_loop_explicit_blocks ... ok
test units::tests::parses_density ... ok
test structural::schedule::tests::structural_singularity_reports_unmatched_rows_and_columns ... ok
test units::tests::parses_thermal_conductivity ... ok
test structural::scc::tests::deep_chain_is_iterative_and_deterministic ... ok

failures:

---- scientific_weak::tests::electrothermal_aliases_lower_into_two_generic_blocks stdout ----

thread 'scientific_weak::tests::electrothermal_aliases_lower_into_two_generic_blocks' (3157) panicked at src/scientific_weak.rs:405:64:
called `Result::unwrap()` on an `Err` value: UnsupportedDifferential { equation: "thermal", expression: Binary { op: Mul, lhs: Call { function: "electrical_conductivity", args: [Name("T")] }, rhs: Call { function: "dot", args: [Call { function: "grad", args: [Name("V")] }, Call { function: "grad", args: [Name("V")] }] } } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    scientific_weak::tests::electrothermal_aliases_lower_into_two_generic_blocks

test result: FAILED. 45 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

[1m[91merror[0m: test failed, to rerun pass `--lib`
```
