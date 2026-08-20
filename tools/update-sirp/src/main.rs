use std::{env, fs, process::ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .ok_or("usage: update-sirp <sirp.json> <snapshot.json>")?;
    let output = args
        .next()
        .ok_or("usage: update-sirp <sirp.json> <snapshot.json>")?;
    let bytes = fs::read(&input).map_err(|e| format!("{input}: {e}"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|e| format!("{input}: {e}"))?;
    let canonical = canonicalize(value);
    let payload = serde_json::to_vec_pretty(&canonical).map_err(|e| e.to_string())?;
    let digest = blake3::hash(&payload).to_hex().to_string();
    let envelope = serde_json::json!({
        "schema": "resolvent-sirp-snapshot/1",
        "source_digest": digest,
        "data": canonical,
    });
    let mut encoded = serde_json::to_vec_pretty(&envelope).map_err(|e| e.to_string())?;
    encoded.push(b'\n');
    fs::write(&output, encoded).map_err(|e| format!("{output}: {e}"))?;
    Ok(())
}

fn canonicalize(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Array(xs) => {
            serde_json::Value::Array(xs.into_iter().map(canonicalize).collect())
        }
        serde_json::Value::Object(map) => {
            let mut entries: Vec<_> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            let mut out = serde_json::Map::new();
            for (k, v) in entries {
                out.insert(k, canonicalize(v));
            }
            serde_json::Value::Object(out)
        }
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonicalization_sorts_object_keys_recursively() {
        let v = serde_json::json!({"z":1,"a":{"b":2,"a":1}});
        let encoded = serde_json::to_string(&canonicalize(v)).unwrap();
        assert_eq!(encoded, r#"{"a":{"a":1,"b":2},"z":1}"#);
    }
}
