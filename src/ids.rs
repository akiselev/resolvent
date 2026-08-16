use core::fmt;

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(pub u64);

        impl $name {
            pub const fn new(raw: u64) -> Self {
                Self(raw)
            }

            pub const fn get(self) -> u64 {
                self.0
            }
        }
    };
}

id_type!(ExprId);
id_type!(EquationId);
id_type!(ModelId);
id_type!(FormId);
id_type!(OperatorId);
id_type!(ObservableId);
id_type!(ScopeId);

/// Content identity carried through every lowering receipt.
///
/// Resolvent deliberately does not prescribe the hash algorithm in the semantic API. The
/// producer records the algorithm in [`crate::Provenance`]; this type only carries the
/// digest bytes so identities cannot be confused with human-readable names.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ArtifactHash(pub [u8; 32]);

impl ArtifactHash {
    pub const ZERO: Self = Self([0; 32]);

    pub const fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl fmt::Debug for ArtifactHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ArtifactHash(")?;
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        f.write_str(")")
    }
}
