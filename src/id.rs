use serde::{Deserialize, Serialize};
use std::fmt;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(
            Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
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

id_type!(SymbolId);
id_type!(ExprId);
id_type!(SystemId);
id_type!(FieldId);
id_type!(FormId);
id_type!(DiscreteProgramId);
id_type!(OperatorId);
id_type!(RefinementId);
id_type!(ObligationId);
id_type!(ObservableId);

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

impl fmt::Display for Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm, self.hex)
    }
}
