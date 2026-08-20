fn require_side(
    measure: &MeasureV2,
    restricted: bool,
    operand: String,
) -> Result<(), FormV2Error> {
    if measure.requires_explicit_side() && !restricted {
        return Err(FormV2Error::MissingSide {
            measure: measure.kind_name().into(),
            operand,
        });
    }
    Ok(())
}

fn merge_frames(
    left: Option<FrameIdV2>,
    right: Option<FrameIdV2>,
) -> Result<Option<FrameIdV2>, FormV2Error> {
    match (left, right) {
        (Some(left), Some(right)) if left != right => Err(FormV2Error::FrameMismatch {
            left: left.to_string(),
            right: right.to_string(),
        }),
        (Some(frame), _) | (_, Some(frame)) => Ok(Some(frame)),
        (None, None) => Ok(None),
    }
}

fn space<'a>(
    payload: &'a VariationalArtifactPayloadV2,
    id: &SpaceRequirementIdV2,
) -> Result<&'a SpaceRequirementV2, FormV2Error> {
    payload
        .spaces
        .iter()
        .find(|space| space.id == *id)
        .ok_or_else(|| FormV2Error::MissingReference {
            kind: "space_requirement".into(),
            id: id.to_string(),
        })
}

fn frame<'a>(
    payload: &'a VariationalArtifactPayloadV2,
    id: &FrameIdV2,
) -> Result<&'a FrameV2, FormV2Error> {
    payload
        .frames
        .iter()
        .find(|frame| frame.id == *id)
        .ok_or_else(|| FormV2Error::MissingReference {
            kind: "frame".into(),
            id: id.to_string(),
        })
}

fn index_set_ref<'a>(
    payload: &'a VariationalArtifactPayloadV2,
    id: &IndexSetIdV2,
) -> Result<&'a IndexSetV2, FormV2Error> {
    payload
        .index_sets
        .iter()
        .find(|set| set.id == *id)
        .ok_or_else(|| FormV2Error::MissingReference {
            kind: "index_set".into(),
            id: id.to_string(),
        })
}
