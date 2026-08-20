# Resolvent FC0-FC1 validation

Validated source commit: `9f22d2edbb62feee81db988cde93cf3d0149b1af`

- `fmt`: FAIL (1)
- `clippy`: PASS
- `test`: PASS

## fmt

```text
Diff in /home/runner/work/resolvent/resolvent/src/scientific_weak.rs:334:
         }
         // These operators require a recognized weak-form lowering. Leaving one nested in
         // a pointwise expression would defer unavailable differential semantics to runtime.
-        Expr::Call { function, .. }
-            if matches!(function.as_str(), "div" | "curl" | "sym_grad") =>
-        {
+        Expr::Call { function, .. } if matches!(function.as_str(), "div" | "curl" | "sym_grad") => {
             true
         }
         Expr::Call { args, .. } => args.iter().any(contains_unlowered_differential_operator),
Diff in /home/runner/work/resolvent/resolvent/src/scientific_weak.rs:347:
         }
         Expr::Index { value, indices } => {
             contains_unlowered_differential_operator(value)
-                || indices
-                    .iter()
-                    .any(contains_unlowered_differential_operator)
+                || indices.iter().any(contains_unlowered_differential_operator)
         }
-        Expr::Vector(values) => values
-            .iter()
-            .any(contains_unlowered_differential_operator),
+        Expr::Vector(values) => values.iter().any(contains_unlowered_differential_operator),
         Expr::Number { .. } | Expr::String(_) | Expr::Name(_) => false,
     }
 }
```

## clippy

```text
[1m[92m    Updating[0m crates.io index
[1m[92m     Locking[0m 22 packages to latest compatible versions
[1m[92m Downloading[0m crates ...
[1m[92m  Downloaded[0m quote v1.0.47
[1m[92m  Downloaded[0m cpufeatures v0.3.0
[1m[92m  Downloaded[0m itoa v1.0.18
[1m[92m  Downloaded[0m proc-macro2 v1.0.107
[1m[92m  Downloaded[0m thiserror v2.0.20
[1m[92m  Downloaded[0m cc v1.4.3
[1m[92m  Downloaded[0m unicode-ident v1.0.24
[1m[92m  Downloaded[0m serde v1.0.229
[1m[92m  Downloaded[0m serde_derive v1.0.229
[1m[92m  Downloaded[0m serde_core v1.0.229
[1m[92m  Downloaded[0m memchr v2.8.3
[1m[92m  Downloaded[0m constant_time_eq v0.4.2
[1m[92m  Downloaded[0m syn v3.0.3
[1m[92m  Downloaded[0m blake3 v1.8.7
[1m[92m  Downloaded[0m zmij v1.0.23
[1m[92m  Downloaded[0m serde_json v1.0.151
[1m[92m  Downloaded[0m thiserror-impl v2.0.20
[1m[92m  Downloaded[0m find-msvc-tools v0.1.11
[1m[92m  Downloaded[0m arrayvec v0.7.8
[1m[92m  Downloaded[0m cfg-if v1.0.4
[1m[92m  Downloaded[0m shlex v2.0.1
[1m[92m   Compiling[0m proc-macro2 v1.0.107
[1m[92m   Compiling[0m quote v1.0.47
[1m[92m   Compiling[0m unicode-ident v1.0.24
[1m[92m   Compiling[0m serde_core v1.0.229
[1m[92m   Compiling[0m shlex v2.0.1
[1m[92m   Compiling[0m find-msvc-tools v0.1.11
[1m[92m   Compiling[0m thiserror v2.0.20
[1m[92m   Compiling[0m cc v1.4.3
[1m[92m   Compiling[0m zmij v1.0.23
[1m[92m   Compiling[0m serde v1.0.229
[1m[92m   Compiling[0m syn v3.0.3
[1m[92m   Compiling[0m serde_json v1.0.151
[1m[92m    Checking[0m cpufeatures v0.3.0
[1m[92m    Checking[0m memchr v2.8.3
[1m[92m    Checking[0m cfg-if v1.0.4
[1m[92m    Checking[0m arrayvec v0.7.8
[1m[92m   Compiling[0m blake3 v1.8.7
[1m[92m    Checking[0m constant_time_eq v0.4.2
[1m[92m    Checking[0m itoa v1.0.18
[1m[92m   Compiling[0m serde_derive v1.0.229
[1m[92m   Compiling[0m thiserror-impl v2.0.20
[1m[92m    Checking[0m resolvent-quantities v0.1.0 (/home/runner/work/resolvent/resolvent/crates/resolvent-quantities)
[1m[92m    Checking[0m resolvent v0.1.0 (/home/runner/work/resolvent/resolvent)
[1m[92m    Finished[0m `dev` profile [unoptimized + debuginfo] target(s) in 12.32s
```

## test

```text
[1m[92m   Compiling[0m serde_core v1.0.229
[1m[92m   Compiling[0m zmij v1.0.23
[1m[92m   Compiling[0m thiserror v2.0.20
[1m[92m   Compiling[0m constant_time_eq v0.4.2
[1m[92m   Compiling[0m arrayvec v0.7.8
[1m[92m   Compiling[0m itoa v1.0.18
[1m[92m   Compiling[0m memchr v2.8.3
[1m[92m   Compiling[0m cfg-if v1.0.4
[1m[92m   Compiling[0m cpufeatures v0.3.0
[1m[92m   Compiling[0m blake3 v1.8.7
[1m[92m   Compiling[0m serde v1.0.229
[1m[92m   Compiling[0m serde_json v1.0.151
[1m[92m   Compiling[0m resolvent-quantities v0.1.0 (/home/runner/work/resolvent/resolvent/crates/resolvent-quantities)
[1m[92m   Compiling[0m resolvent v0.1.0 (/home/runner/work/resolvent/resolvent)
[1m[92m    Finished[0m `test` profile [unoptimized + debuginfo] target(s) in 12.83s
[1m[92m     Running[0m unittests src/lib.rs (target/debug/deps/resolvent-27420dd34b98a32e)

running 47 tests
test calculus::tests::derivative_matches_quadratic ... ok
test form_v2::tests::explicit_contract_rejects_equal_spatial_variance ... ok
test form_compile::tests::diffusion_becomes_explicit_discrete_stages ... ok
test calculus::tests::sin_chain_rule ... ok
test form_v2::tests::complex_inner_and_hermitian_adjoint_are_distinct_and_typed ... ok
test form_v2::tests::invalid_contraction_reports_frame_mismatch ... ok
test form_v2::tests::interior_facet_requires_explicit_sides ... ok
test generated_verify::tests::dot_gate_detects_transpose_identity ... ok
test generated_verify::tests::manufactured_derivative_is_exact ... ok
test generated_verify::tests::order_is_computed ... ok
test latex::tests::parses_nonlinear_heat_fragment ... ok
test latex::tests::rejects_unknown_tex ... ok
test migration::tests::frozen_case_detects_tampering ... ok
test migration::tests::tolerance_is_local_and_explicit ... ok
test form_v2::tests::mixed_jacobian_blocks_have_stable_digests ... ok
test physics::tests::lock_rejects_source_drift ... ok
test property_tensor::tests::isotropic_tensor_is_rotation_invariant ... ok
test property_tensor::tests::orthotropic_tensor_rotates_axes_and_creates_cross_term ... ok
test physics::tests::macro_uses_same_parser_and_elaborator ... ok
test reference::tests::p1_diffusion_mass_source_and_boundary_compile ... ok
test reference::tests::shifted_contains_mass_and_stiffness ... ok
test reference_hdiv::tests::constant_divergence_gives_rank_one_local_div_div ... ok
test reference_hdiv::tests::rt0_is_invariant_to_triangle_vertex_order ... ok
test reference_mixed::tests::elasticity_has_rigid_translation_null_modes ... ok
test reference_mixed::tests::nedelec_orientation_is_invariant_to_triangle_vertex_order ... ok
test scientific::tests::canonical_heat_source_is_smooth_and_finite ... ok
test reference_mixed::tests::stokes_has_an_exact_zero_pressure_block_and_symmetric_coupling ... ok
test scientific::tests::production_catalog_covers_agent_gate_spaces ... ok
test rsl::tests::parses_and_elaborates_heat_model ... ok
test scientific::tests::parses_structured_heat_source ... ok
test scientific::tests::property_expression_symbolic_derivative_matches_finite_difference ... ok
test scientific_bridge::tests::constrained_latex_lowers_to_scientific_expression_nodes ... ok
test scientific_weak::tests::pointwise_field_gradients_are_not_unlowered_higher_derivatives ... ok
test scientific_weak::tests::nonlinear_heat_lowers_without_named_physics_special_case ... ok
test scientific_weak::tests::electrothermal_aliases_lower_into_two_generic_blocks ... ok
test structural::dae::tests::detects_alias_class ... ok
test structural::scc::tests::components_are_reverse_topological ... ok
test form_v2::tests::scalar_adapter_is_lossless_serializable_and_claim_free ... ok
test structural::dae::tests::hidden_constraint_requests_differentiation ... ok
test structural::schedule::tests::coupled_solver_can_keep_raw_algebraic_loop ... ok
test structural::schedule::tests::lower_triangular_chain_is_explicit_in_order ... ok
test structural::schedule::tests::rlc_shape_has_explicit_loop_explicit_blocks ... ok
test structural::schedule::tests::dense_loop_is_torn_deterministically ... ok
test structural::schedule::tests::structural_singularity_reports_unmatched_rows_and_columns ... ok
test units::tests::parses_density ... ok
test units::tests::parses_thermal_conductivity ... ok
test structural::scc::tests::deep_chain_is_iterative_and_deterministic ... ok

test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

[1m[92m     Running[0m unittests src/bin/resolvent.rs (target/debug/deps/resolvent-642e06f4de5d96c5)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[1m[92m     Running[0m unittests src/bin/resolvent-science.rs (target/debug/deps/resolvent_science-eae597ab7bbebb1e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[1m[92m     Running[0m tests/plexus_parity.rs (target/debug/deps/plexus_parity-48d4eab7cf83e2f5)

running 5 tests
test lower_triangular_chain_is_all_explicit ... ok
test structurally_singular_system_reports_both_sides ... ok
test two_by_two_loop_tears_one_variable_and_preserves_untorn_block ... ok
test structural_results_are_deterministic ... ok
test matching_agrees_with_exhaustive_reference_on_all_three_by_three_graphs ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[1m[92m     Running[0m tests/r14_acceptance.rs (target/debug/deps/r14_acceptance-5de17015f34d0cbb)

running 3 tests
test every_current_semantic_declaration_has_a_nonempty_source_span ... ok
test whitespace_and_comments_do_not_change_semantic_digest ... ok
test format_roundtrip_preserves_all_current_scientific_v1_semantics ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[1m[92m     Running[0m tests/r18_coupling.rs (target/debug/deps/r18_coupling-fa1060a1d440ee61)

running 2 tests
test nested_property_and_constitutive_dependencies_reach_cross_blocks ... ok
test declaration_reordering_preserves_semantic_digest_and_coupling_graph ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[1m[92m     Running[0m tests/scientific_bridge.rs (target/debug/deps/scientific_bridge-f3c80a3ac30f51a0)

running 2 tests
test scientific_physics_lock_carries_property_provenance_and_digest_identity ... ok
test file_and_rust_macro_use_identical_scientific_semantics ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[1m[92m     Running[0m tests/scientific_language.rs (target/debug/deps/scientific_language-45555927eedb9fa9)

running 4 tests
test declaration_order_does_not_change_coupling_dependencies ... ok
test r14_corpus_has_fifty_invalid_modules ... ok
test formatting_is_idempotent_and_semantics_stable ... ok
test r14_corpus_has_fifty_valid_modules ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[1m[92m     Running[0m tests/scientific_stack.rs (target/debug/deps/scientific_stack-c7e901a5308b291a)

running 7 tests
test evidence_axes_do_not_collapse_and_profiles_are_order_independent ... ok
test expression_store_hash_conses_commutative_builders ... ok
test scope_broadening_requires_named_obligation ... ok
test kernel_proved_reification_requires_theorem_and_axiom_whitelist ... ok
test handle_bearing_artifact_hash_includes_its_semantic_context ... ok
test serialized_context_rebuilds_interning_indexes ... ok
test structural_projection_uses_common_system_ir ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[1m[92m   Doc-tests[0m resolvent

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```
