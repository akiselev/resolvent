use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParityTolerance {
    pub absolute: f64,
    pub relative: f64,
}

impl Default for ParityTolerance {
    fn default() -> Self {
        Self {
            absolute: 0.0,
            relative: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OracleKind {
    Residua,
    Plexus,
    Solverang,
    HandReference,
    External(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ParityCase {
    pub id: String,
    pub oracle: OracleKind,
    pub capability: String,
    #[serde(default)]
    pub source_paths: Vec<String>,
    #[serde(default)]
    pub expected_artifacts: Vec<String>,
    #[serde(default)]
    pub tolerance: ParityTolerance,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ParityManifest {
    pub schema_version: String,
    #[serde(default)]
    pub cases: Vec<ParityCase>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NumericParity {
    pub equal: bool,
    pub max_absolute: f64,
    pub max_relative: f64,
    pub worst_index: Option<usize>,
    pub compared: usize,
}

pub fn compare_f64(expected: &[f64], actual: &[f64], tolerance: &ParityTolerance) -> NumericParity {
    if expected.len() != actual.len() {
        return NumericParity {
            equal: false,
            max_absolute: f64::INFINITY,
            max_relative: f64::INFINITY,
            worst_index: None,
            compared: expected.len().min(actual.len()),
        };
    }
    let mut result = NumericParity {
        equal: true,
        max_absolute: 0.0,
        max_relative: 0.0,
        worst_index: None,
        compared: expected.len(),
    };
    for (index, (&want, &got)) in expected.iter().zip(actual).enumerate() {
        let abs = (want - got).abs();
        let scale = want.abs().max(got.abs());
        let rel = if scale == 0.0 { 0.0 } else { abs / scale };
        if abs > result.max_absolute || rel > result.max_relative {
            result.worst_index = Some(index);
        }
        result.max_absolute = result.max_absolute.max(abs);
        result.max_relative = result.max_relative.max(rel);
        if !want.is_finite()
            || !got.is_finite()
            || (abs > tolerance.absolute && rel > tolerance.relative)
        {
            result.equal = false;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parity_obeys_abs_or_relative_contract() {
        let result = compare_f64(
            &[1000.0],
            &[1000.0001],
            &ParityTolerance {
                absolute: 1e-6,
                relative: 1e-6,
            },
        );
        assert!(result.equal);
    }
}
