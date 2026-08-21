use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(pub u32);

        impl $name {
            pub const fn index(self) -> usize {
                self.0 as usize
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

id_type!(ObligationId);

/// Stable digest used at repository boundaries. The wire format deliberately carries
/// the algorithm so a future digest migration cannot silently reinterpret old records.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Digest {
    pub algorithm: String,
    pub hex: String,
}

impl Digest {
    pub fn blake3(bytes: &[u8]) -> Self {
        Self {
            algorithm: "blake3".into(),
            hex: blake3::hash(bytes).to_hex().to_string(),
        }
    }
}

/// Hash a deterministically serialized typed artifact after removing presentation-only spans.
///
/// This is deliberately not a general JSON canonicalizer: typed structs determine field order,
/// maps use deterministic key order, and array order remains semantically significant.
pub(crate) fn span_independent_digest(value: &impl Serialize) -> Digest {
    let mut value = serde_json::to_value(value).expect("artifact serialization is infallible");
    fn strip_source_spans(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.retain(|key, _| key != "span" && !key.ends_with("_span"));
                for child in map.values_mut() {
                    strip_source_spans(child);
                }
            }
            serde_json::Value::Array(children) => {
                for child in children {
                    strip_source_spans(child);
                }
            }
            _ => {}
        }
    }
    strip_source_spans(&mut value);
    Digest::blake3(&serde_json::to_vec(&value).expect("typed artifact serialization is infallible"))
}

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm, self.hex)
    }
}
