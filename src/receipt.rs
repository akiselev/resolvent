use crate::{AlgebraError, Expr, Polynomial};
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
        Ok(Self::new(operation, input, output))
    }
    pub fn for_polynomial<T: Serialize>(
        operation: AlgebraOperation,
        input: &Polynomial,
        output: &T,
    ) -> Result<Self, AlgebraError> {
        Ok(Self::new(operation, input, output))
    }
    fn new<I: Serialize, O: Serialize>(operation: AlgebraOperation, input: &I, output: &O) -> Self {
        Self {
            schema: "resolvent-algebra-receipt/1".into(),
            operation,
            input_digest: digest(input),
            output_digest: digest(output),
        }
    }
}

fn digest(value: &impl Serialize) -> String {
    blake3::hash(&serde_json::to_vec(value).expect("algebra values serialize"))
        .to_hex()
        .to_string()
}
