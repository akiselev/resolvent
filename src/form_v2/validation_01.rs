#[derive(Clone, Debug)]
struct TypedValueV2 {
    value_type: TensorTypeV2,
    domain_frame: Option<FrameIdV2>,
}

fn validate_payload(
    payload: &VariationalArtifactPayloadV2,
    require_target_digest: bool,
) -> Result<(), FormV2Error> {
    if payload.form.schema != VARIATIONAL_FORM_V2_SCHEMA {
        return Err(FormV2Error::Schema {
            expected: VARIATIONAL_FORM_V2_SCHEMA.into(),
            got: payload.form.schema.clone(),
        });
    }
    if payload.receipt.schema != FORMULATION_RECEIPT_V2_SCHEMA {
        return Err(FormV2Error::Schema {
            expected: FORMULATION_RECEIPT_V2_SCHEMA.into(),
            got: payload.receipt.schema.clone(),
        });
    }
    if payload.form.derivation != payload.derivation.id {
        return Err(FormV2Error::MissingReference {
            kind: "formulation_derivation".into(),
            id: payload.form.derivation.to_string(),
        });
    }

    unique_ids(
        payload.frames.iter().map(|frame| frame.id.to_string()),
        "frame",
    )?;
    unique_ids(
        payload.index_sets.iter().map(|set| set.id.to_string()),
        "index_set",
    )?;
    unique_ids(
        payload.spaces.iter().map(|space| space.id.to_string()),
        "space_requirement",
    )?;
    unique_ids(
        payload
            .form
            .arguments
            .iter()
            .map(|argument| argument.id.to_string()),
        "argument",
    )?;
    unique_ids(
        payload
            .form
            .coefficients
            .iter()
            .map(|coefficient| coefficient.field.to_string()),
        "coefficient",
    )?;
    unique_ids(
        payload
            .form
            .constants
            .iter()
            .map(|constant| constant.id.to_string()),
        "constant",
    )?;
    unique_ids(
        payload.form.integrals.iter().map(|integral| integral.id.clone()),
        "integral",
    )?;

    for frame in &payload.frames {
        if frame.dimension == 0 {
            return Err(FormV2Error::InvalidExtent {
                kind: "frame".into(),
                id: frame.id.to_string(),
            });
        }
    }
    for index_set in &payload.index_sets {
        if index_set.extent == 0 {
            return Err(FormV2Error::InvalidExtent {
                kind: "index_set".into(),
                id: index_set.id.to_string(),
            });
        }
    }
    for space in &payload.spaces {
        frame(payload, &space.spatial_frame)?;
        validate_tensor_type(payload, &space.value_type, payload.form.scalar_kind)?;
    }

    if payload.form.arity != payload.form.computed_arity() {
        return Err(FormV2Error::ArityMismatch {
            declared: payload.form.arity,
            computed: payload.form.computed_arity(),
        });
    }

    let mut number_parts = BTreeSet::new();
    let mut numbers = BTreeSet::new();
    for argument in &payload.form.arguments {
        if !number_parts.insert((argument.number, argument.part)) {
            return Err(FormV2Error::DuplicateArgumentPart {
                number: argument.number,
                part: argument.part,
            });
        }
        numbers.insert(argument.number);
        let space = space(payload, &argument.space)?;
        if space.value_type != argument.value_type {
            return Err(FormV2Error::TypeMismatch {
                operation: format!("argument `{}` space", argument.id),
            });
        }
        validate_tensor_type(payload, &argument.value_type, payload.form.scalar_kind)?;
    }
    if let Some(maximum) = numbers.iter().next_back().copied() {
        for number in 0..=maximum {
            if !numbers.contains(&number) {
                return Err(FormV2Error::ArityGap { number });
            }
        }
    }

    for coefficient in &payload.form.coefficients {
        let space = space(payload, &coefficient.space)?;
        if space.value_type != coefficient.value_type {
            return Err(FormV2Error::TypeMismatch {
                operation: format!("coefficient `{}` space", coefficient.field),
            });
        }
        validate_tensor_type(payload, &coefficient.value_type, payload.form.scalar_kind)?;
    }
    for constant in &payload.form.constants {
        validate_tensor_type(payload, &constant.value_type, payload.form.scalar_kind)?;
    }

    for integral in &payload.form.integrals {
        let typed = infer_expression(payload, &integral.integrand, &integral.measure, false)?;
        if !typed.value_type.is_scalar() {
            return Err(FormV2Error::NonScalarIntegrand {
                integral: integral.id.clone(),
                axes: typed.value_type.axes.len(),
            });
        }
    }

    for status in derivative_statuses(&payload.derivatives) {
        if let DerivativeArtifactStatusV2::Generated { evidence, .. } = status
            && evidence.is_empty()
        {
            return Err(FormV2Error::UnevidencedClaim {
                claim: "derivative artifact".into(),
            });
        }
    }
    for claim in &payload.operator_claims {
        if claim.evidence.is_empty() {
            return Err(FormV2Error::UnevidencedClaim {
                claim: claim.property.clone(),
            });
        }
    }

    if let Some(compatibility) = &payload.scalar_h1_compatibility {
        if compatibility.schema != "resolvent-scalar-h1-compatibility/2" {
            return Err(FormV2Error::Schema {
                expected: "resolvent-scalar-h1-compatibility/2".into(),
                got: compatibility.schema.clone(),
            });
        }
        let digest = digest_serialized(&compatibility.program)?;
        if digest != compatibility.source_digest
            || digest != payload.receipt.source_digest
        {
            return Err(FormV2Error::DigestMismatch {
                which: "compatibility source".into(),
            });
        }
    }

    if require_target_digest && payload.receipt.target_semantic_digest.hex.is_empty() {
        return Err(FormV2Error::DigestMismatch {
            which: "receipt target".into(),
        });
    }
    Ok(())
}

fn derivative_statuses(
    derivatives: &DerivativeArtifactsV2,
) -> impl Iterator<Item = &DerivativeArtifactStatusV2> {
    [
        &derivatives.exact_jacobian,
        &derivatives.dynamic_jacobian,
        &derivatives.preconditioning_jacobian,
        &derivatives.jvp,
        &derivatives.vjp,
    ]
    .into_iter()
    .chain(derivatives.parameter_actions.iter())
}

fn unique_ids(
    ids: impl IntoIterator<Item = String>,
    kind: &str,
) -> Result<(), FormV2Error> {
    let mut seen = BTreeSet::new();
    for id in ids {
        if !seen.insert(id.clone()) {
            return Err(FormV2Error::DuplicateId {
                kind: kind.into(),
                id,
            });
        }
    }
    Ok(())
}

fn validate_tensor_type(
    payload: &VariationalArtifactPayloadV2,
    value_type: &TensorTypeV2,
    scalar_kind: ScalarKindV2,
) -> Result<(), FormV2Error> {
    if value_type.scalar != scalar_kind {
        return Err(FormV2Error::ScalarKindMismatch {
            expected: scalar_kind,
            got: value_type.scalar,
        });
    }
    for axis in &value_type.axes {
        match axis {
            AxisKindV2::Spatial { frame: id, .. } => {
                frame(payload, id)?;
            }
            AxisKindV2::Species { index_set }
            | AxisKindV2::SlipSystem { index_set }
            | AxisKindV2::NetworkNode { index_set }
            | AxisKindV2::NetworkBranch { index_set }
            | AxisKindV2::MaterialComponent { index_set } => {
                index_set_ref(payload, index_set)?;
            }
            AxisKindV2::Algebraic { extent } if *extent == 0 => {
                return Err(FormV2Error::InvalidExtent {
                    kind: "algebraic_axis".into(),
                    id: "0".into(),
                });
            }
            AxisKindV2::Algebraic { .. } => {}
        }
    }
    Ok(())
}
