use crate::{AlgebraError, Expr, QPoly, Rational};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlgebraOperation {
    Canonicalize,
    Differentiate,
    Resultant,
    IsolateRealRoots,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AlgebraReceipt {
    pub schema: String,
    pub operation: AlgebraOperation,
    pub input_digest: String,
    pub output_digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AlgebraReceiptWire {
    schema: String,
    operation: AlgebraOperation,
    input_digest: String,
    output_digest: String,
}

impl<'de> Deserialize<'de> for AlgebraReceipt {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = AlgebraReceiptWire::deserialize(deserializer)?;
        if wire.schema != "resolvent-algebra-receipt/1"
            || !valid_digest(&wire.input_digest)
            || !valid_digest(&wire.output_digest)
        {
            return Err(serde::de::Error::custom("invalid algebra receipt"));
        }
        Ok(Self {
            schema: wire.schema,
            operation: wire.operation,
            input_digest: wire.input_digest,
            output_digest: wire.output_digest,
        })
    }
}

fn valid_digest(digest: &str) -> bool {
    digest.len() == 64
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

impl AlgebraReceipt {
    pub fn for_expressions(
        operation: AlgebraOperation,
        input: &Expr,
        output: &Expr,
    ) -> Result<Self, AlgebraError> {
        Self::new(operation, input, output)
    }
    /// Receipt v1 for a polynomial operation with a rational scalar output.
    ///
    /// Both digest projections are frozen. Richer polynomial/root outputs
    /// require a new receipt schema instead of reusing this method generically.
    pub fn for_polynomial(
        operation: AlgebraOperation,
        input: &QPoly,
        output: &Rational,
    ) -> Result<Self, AlgebraError> {
        // Receipt v1 predates the versioned public QPoly wire schema. Keep its
        // original `{ "coeffs": [...] }` digest projection so making QPoly's
        // transport schema explicit does not silently rewrite durable receipt
        // identity.
        #[derive(Serialize)]
        struct PolynomialReceiptV1<'a> {
            coeffs: &'a [Rational],
        }
        Ok(Self {
            schema: "resolvent-algebra-receipt/1".into(),
            operation,
            input_digest: digest(&PolynomialReceiptV1 {
                coeffs: input.coefficients(),
            })?,
            output_digest: digest(output)?,
        })
    }
    fn new<I: Serialize, O: Serialize>(
        operation: AlgebraOperation,
        input: &I,
        output: &O,
    ) -> Result<Self, AlgebraError> {
        Ok(Self {
            schema: "resolvent-algebra-receipt/1".into(),
            operation,
            input_digest: digest(input)?,
            output_digest: digest(output)?,
        })
    }
}

fn digest(value: &impl Serialize) -> Result<String, AlgebraError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|error| AlgebraError::Serialization(error.to_string()))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}
