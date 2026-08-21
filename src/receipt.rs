use crate::{AlgebraError, Expr, QPoly};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlgebraOperation {
    Canonicalize,
    Differentiate,
    Resultant,
    IsolateRealRoots,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgebraReceipt {
    pub schema: String,
    pub operation: AlgebraOperation,
    pub input_digest: String,
    pub output_digest: String,
}

impl AlgebraReceipt {
    pub fn for_expressions(
        operation: AlgebraOperation,
        input: &Expr,
        output: &Expr,
    ) -> Result<Self, AlgebraError> {
        Self::new(operation, input, output)
    }
    pub fn for_polynomial<T: Serialize>(
        operation: AlgebraOperation,
        input: &QPoly,
        output: &T,
    ) -> Result<Self, AlgebraError> {
        Self::new(operation, input, output)
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
