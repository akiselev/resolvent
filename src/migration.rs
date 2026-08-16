use crate::id::Digest;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Frozen behavior from a legacy/reference implementation. Migration is complete only when
/// the native Resolvent projection produces the same canonical payload or an explicitly
/// reviewed tolerance comparison for floating data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MigrationCase {
    pub id: String,
    pub source: String,
    pub schema: String,
    pub input: serde_json::Value,
    pub expected: serde_json::Value,
    pub digest: Digest,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

impl MigrationCase {
    pub fn freeze(
        id: impl Into<String>,
        source: impl Into<String>,
        schema: impl Into<String>,
        input: serde_json::Value,
        expected: serde_json::Value,
    ) -> Result<Self, serde_json::Error> {
        #[derive(Serialize)]
        struct Payload<'a> {
            input: &'a serde_json::Value,
            expected: &'a serde_json::Value,
        }
        let bytes = serde_json::to_vec(&Payload {
            input: &input,
            expected: &expected,
        })?;
        Ok(Self {
            id: id.into(),
            source: source.into(),
            schema: schema.into(),
            input,
            expected,
            digest: Digest::blake3(&bytes),
            metadata: BTreeMap::new(),
        })
    }
    pub fn verify_digest(&self) -> Result<bool, serde_json::Error> {
        #[derive(Serialize)]
        struct Payload<'a> {
            input: &'a serde_json::Value,
            expected: &'a serde_json::Value,
        }
        Ok(self.digest
            == Digest::blake3(&serde_json::to_vec(&Payload {
                input: &self.input,
                expected: &self.expected,
            })?))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NumericTolerance {
    pub absolute: f64,
    pub relative: f64,
}
impl NumericTolerance {
    pub const EXACT: Self = Self {
        absolute: 0.0,
        relative: 0.0,
    };
    pub fn accepts(&self, expected: f64, actual: f64) -> bool {
        let delta = (expected - actual).abs();
        delta
            <= self
                .absolute
                .max(self.relative * expected.abs().max(actual.abs()))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DifferentialMismatch {
    pub path: String,
    pub expected: serde_json::Value,
    pub actual: serde_json::Value,
}

pub fn compare_json(
    expected: &serde_json::Value,
    actual: &serde_json::Value,
    tol: NumericTolerance,
) -> Vec<DifferentialMismatch> {
    let mut out = vec![];
    compare_at("$", expected, actual, &tol, &mut out);
    out
}
fn compare_at(
    path: &str,
    e: &serde_json::Value,
    a: &serde_json::Value,
    tol: &NumericTolerance,
    out: &mut Vec<DifferentialMismatch>,
) {
    match (e, a) {
        (serde_json::Value::Number(en), serde_json::Value::Number(an))
            if en.as_f64().is_some() && an.as_f64().is_some() =>
        {
            if !tol.accepts(en.as_f64().unwrap(), an.as_f64().unwrap()) {
                out.push(DifferentialMismatch {
                    path: path.into(),
                    expected: e.clone(),
                    actual: a.clone(),
                })
            }
        }
        (serde_json::Value::Array(es), serde_json::Value::Array(as_)) if es.len() == as_.len() => {
            for (i, (e, a)) in es.iter().zip(as_).enumerate() {
                compare_at(&format!("{path}[{i}]"), e, a, tol, out)
            }
        }
        (serde_json::Value::Object(es), serde_json::Value::Object(as_)) => {
            for (k, e) in es {
                if let Some(a) = as_.get(k) {
                    compare_at(&format!("{path}.{k}"), e, a, tol, out)
                } else {
                    out.push(DifferentialMismatch {
                        path: format!("{path}.{k}"),
                        expected: e.clone(),
                        actual: serde_json::Value::Null,
                    })
                }
            }
            for (k, a) in as_ {
                if !es.contains_key(k) {
                    out.push(DifferentialMismatch {
                        path: format!("{path}.{k}"),
                        expected: serde_json::Value::Null,
                        actual: a.clone(),
                    })
                }
            }
        }
        _ if e == a => {}
        _ => out.push(DifferentialMismatch {
            path: path.into(),
            expected: e.clone(),
            actual: a.clone(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tolerance_is_local_and_explicit() {
        let e = serde_json::json!({"k":[1.0,2.0]});
        let a = serde_json::json!({"k":[1.0+1e-12,2.0]});
        assert!(
            compare_json(
                &e,
                &a,
                NumericTolerance {
                    absolute: 1e-10,
                    relative: 0.0
                }
            )
            .is_empty()
        );
        assert_eq!(compare_json(&e, &a, NumericTolerance::EXACT).len(), 1);
    }
    #[test]
    fn frozen_case_detects_tampering() {
        let mut c = MigrationCase::freeze(
            "x",
            "plexus",
            "v1",
            serde_json::json!([1]),
            serde_json::json!([2]),
        )
        .unwrap();
        assert!(c.verify_digest().unwrap());
        c.expected = serde_json::json!([3]);
        assert!(!c.verify_digest().unwrap());
    }
}
