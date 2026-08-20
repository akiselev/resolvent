fn infer_expression(
    payload: &VariationalArtifactPayloadV2,
    expression: &FormExprV2,
    measure: &MeasureV2,
    restricted: bool,
) -> Result<TypedValueV2, FormV2Error> {
    match expression {
        FormExprV2::ScientificScalar { expression } => {
            if contains_differential_operator(expression) {
                return Err(FormV2Error::UnloweredTerm {
                    stage: "scientific_scalar".into(),
                    detail: format!("differential expression remained local: {expression:?}"),
                });
            }
            Ok(TypedValueV2 {
                value_type: TensorTypeV2::scalar(payload.form.scalar_kind),
                domain_frame: None,
            })
        }
        FormExprV2::Argument { id } => {
            require_side(measure, restricted, id.to_string())?;
            let argument = payload
                .form
                .arguments
                .iter()
                .find(|argument| argument.id == *id)
                .ok_or_else(|| FormV2Error::MissingReference {
                    kind: "argument".into(),
                    id: id.to_string(),
                })?;
            let space = space(payload, &argument.space)?;
            Ok(TypedValueV2 {
                value_type: argument.value_type.clone(),
                domain_frame: Some(space.spatial_frame.clone()),
            })
        }
        FormExprV2::Coefficient { field: id } => {
            require_side(measure, restricted, id.to_string())?;
            let coefficient = payload
                .form
                .coefficients
                .iter()
                .find(|coefficient| coefficient.field == *id)
                .ok_or_else(|| FormV2Error::MissingReference {
                    kind: "coefficient".into(),
                    id: id.to_string(),
                })?;
            let space = space(payload, &coefficient.space)?;
            Ok(TypedValueV2 {
                value_type: coefficient.value_type.clone(),
                domain_frame: Some(space.spatial_frame.clone()),
            })
        }
        FormExprV2::Constant { id } => {
            let constant = payload
                .form
                .constants
                .iter()
                .find(|constant| constant.id == *id)
                .ok_or_else(|| FormV2Error::MissingReference {
                    kind: "constant".into(),
                    id: id.to_string(),
                })?;
            Ok(TypedValueV2 {
                value_type: constant.value_type.clone(),
                domain_frame: None,
            })
        }
        FormExprV2::Neg { value }
        | FormExprV2::TimeDerivative { value }
        | FormExprV2::Conjugate { value } => {
            infer_expression(payload, value, measure, restricted)
        }
        FormExprV2::Add { values } => {
            let mut values = values.iter();
            let first = values.next().ok_or_else(|| FormV2Error::EmptyOperation {
                operation: "add".into(),
            })?;
            let mut typed = infer_expression(payload, first, measure, restricted)?;
            for value in values {
                let right = infer_expression(payload, value, measure, restricted)?;
                if typed.value_type != right.value_type {
                    return Err(FormV2Error::TypeMismatch {
                        operation: "add".into(),
                    });
                }
                typed.domain_frame = merge_frames(typed.domain_frame, right.domain_frame)?;
            }
            Ok(typed)
        }
        FormExprV2::Product { values } => {
            if values.is_empty() {
                return Err(FormV2Error::EmptyOperation {
                    operation: "product".into(),
                });
            }
            let mut non_scalar = None::<TensorTypeV2>;
            let mut domain_frame = None;
            for value in values {
                let typed = infer_expression(payload, value, measure, restricted)?;
                domain_frame = merge_frames(domain_frame, typed.domain_frame)?;
                if !typed.value_type.is_scalar() {
                    if non_scalar.is_some() {
                        return Err(FormV2Error::InvalidContraction {
                            operation: "product of multiple tensor operands".into(),
                        });
                    }
                    non_scalar = Some(typed.value_type);
                }
            }
            Ok(TypedValueV2 {
                value_type: non_scalar
                    .unwrap_or_else(|| TensorTypeV2::scalar(payload.form.scalar_kind)),
                domain_frame,
            })
        }
        FormExprV2::Apply { function, args } => {
            let mut domain_frame = None;
            for argument in args {
                let typed = infer_expression(payload, argument, measure, restricted)?;
                if !typed.value_type.is_scalar() {
                    return Err(FormV2Error::TypeMismatch {
                        operation: format!("scalar function `{function}`"),
                    });
                }
                domain_frame = merge_frames(domain_frame, typed.domain_frame)?;
            }
            Ok(TypedValueV2 {
                value_type: TensorTypeV2::scalar(payload.form.scalar_kind),
                domain_frame,
            })
        }
        FormExprV2::Gradient { value, frame: id } => {
            let mut typed = infer_expression(payload, value, measure, restricted)?;
            frame(payload, id)?;
            if let Some(domain_frame) = &typed.domain_frame
                && domain_frame != id
            {
                return Err(FormV2Error::FrameMismatch {
                    left: domain_frame.to_string(),
                    right: id.to_string(),
                });
            }
            typed.value_type.axes.push(AxisKindV2::Spatial {
                frame: id.clone(),
                variance: VarianceV2::Covariant,
            });
            typed.domain_frame = Some(id.clone());
            Ok(typed)
        }
        FormExprV2::Dot { left, right } => {
            let left = infer_expression(payload, left, measure, restricted)?;
            let right = infer_expression(payload, right, measure, restricted)?;
            if left.value_type.axes.len() != 1
                || right.value_type.axes.len() != 1
                || left.value_type.axes != right.value_type.axes
            {
                return Err(FormV2Error::InvalidContraction {
                    operation: "dot".into(),
                });
            }
            Ok(TypedValueV2 {
                value_type: TensorTypeV2::scalar(payload.form.scalar_kind),
                domain_frame: merge_frames(left.domain_frame, right.domain_frame)?,
            })
        }
        FormExprV2::Inner { left, right } => {
            let left = infer_expression(payload, left, measure, restricted)?;
            let right = infer_expression(payload, right, measure, restricted)?;
            if left.value_type != right.value_type {
                return Err(FormV2Error::InvalidContraction {
                    operation: "inner".into(),
                });
            }
            Ok(TypedValueV2 {
                value_type: TensorTypeV2::scalar(payload.form.scalar_kind),
                domain_frame: merge_frames(left.domain_frame, right.domain_frame)?,
            })
        }
        FormExprV2::Contract {
            left,
            right,
            left_axis,
            right_axis,
        } => {
            let left = infer_expression(payload, left, measure, restricted)?;
            let right = infer_expression(payload, right, measure, restricted)?;
            let left_contracted = left.value_type.axes.get(*left_axis);
            let right_contracted = right.value_type.axes.get(*right_axis);
            if left_contracted.is_none() || left_contracted != right_contracted {
                return Err(FormV2Error::InvalidContraction {
                    operation: format!("contract axes {left_axis}/{right_axis}"),
                });
            }
            let mut axes = left.value_type.axes.clone();
            axes.remove(*left_axis);
            let mut right_axes = right.value_type.axes.clone();
            right_axes.remove(*right_axis);
            axes.extend(right_axes);
            Ok(TypedValueV2 {
                value_type: TensorTypeV2 {
                    scalar: payload.form.scalar_kind,
                    axes,
                    quantity: QuantityTypeV2::Unspecified,
                },
                domain_frame: merge_frames(left.domain_frame, right.domain_frame)?,
            })
        }
        FormExprV2::Transpose { value } | FormExprV2::HermitianTranspose { value } => {
            let mut typed = infer_expression(payload, value, measure, restricted)?;
            if typed.value_type.axes.len() != 2 {
                return Err(FormV2Error::InvalidContraction {
                    operation: "transpose requires rank two".into(),
                });
            }
            typed.value_type.axes.swap(0, 1);
            Ok(typed)
        }
        FormExprV2::Restrict { value, side } => {
            if !measure.allows_side(*side) {
                return Err(FormV2Error::InvalidSide {
                    measure: measure.kind_name().into(),
                    side: *side,
                });
            }
            infer_expression(payload, value, measure, true)
        }
    }
}
