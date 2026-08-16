use crate::{EquationId, ExprId, FormId, ModelId, OperatorId, Refinement, ScientificSpec};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExprNode {
    Symbol(String),
    Integer(i128),
    Apply { function: String, args: Vec<ExprId> },
    Add(Vec<ExprId>),
    Mul(Vec<ExprId>),
    Neg(ExprId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EquationNode {
    /// Residual expression; zero means the equation is satisfied.
    pub residual: ExprId,
    pub name: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelNode {
    pub spec: ScientificSpec,
    pub equations: Vec<EquationId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormNode {
    Integral { domain: String, integrand: ExprId },
    Sum(Vec<FormId>),
    Named { kind: String, payload: ExprId },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperatorNode {
    Residual {
        name: String,
    },
    Mass {
        name: String,
    },
    Block {
        name: String,
        children: Vec<OperatorId>,
    },
    MatrixFree {
        name: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextError {
    MissingExpression(ExprId),
    MissingEquation(EquationId),
    MissingForm(FormId),
    MissingOperator(OperatorId),
}

/// Caller-owned semantic arena shared by the Resolvent dialects.
///
/// The initial implementation is deliberately simple. Hash-consing, canonical bytes, exact
/// coefficient domains, and specialized stores can be introduced behind these handles
/// without changing the rule that stores are caller-owned and stage identities are typed.
#[derive(Clone, Debug, Default)]
pub struct Context {
    expressions: Vec<ExprNode>,
    equations: Vec<EquationNode>,
    models: Vec<ModelNode>,
    forms: Vec<FormNode>,
    operators: Vec<OperatorNode>,
    refinements: Vec<Refinement>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern_expr(&mut self, node: ExprNode) -> ExprId {
        let id = ExprId::new(self.expressions.len() as u64);
        self.expressions.push(node);
        id
    }

    pub fn expr(&self, id: ExprId) -> Result<&ExprNode, ContextError> {
        self.expressions
            .get(id.get() as usize)
            .ok_or(ContextError::MissingExpression(id))
    }

    pub fn add_equation(&mut self, node: EquationNode) -> EquationId {
        let id = EquationId::new(self.equations.len() as u64);
        self.equations.push(node);
        id
    }

    pub fn equation(&self, id: EquationId) -> Result<&EquationNode, ContextError> {
        self.equations
            .get(id.get() as usize)
            .ok_or(ContextError::MissingEquation(id))
    }

    pub fn add_model(&mut self, node: ModelNode) -> ModelId {
        let id = ModelId::new(self.models.len() as u64);
        self.models.push(node);
        id
    }

    pub fn model(&self, id: ModelId) -> Option<&ModelNode> {
        self.models.get(id.get() as usize)
    }

    pub fn add_form(&mut self, node: FormNode) -> FormId {
        let id = FormId::new(self.forms.len() as u64);
        self.forms.push(node);
        id
    }

    pub fn form(&self, id: FormId) -> Result<&FormNode, ContextError> {
        self.forms
            .get(id.get() as usize)
            .ok_or(ContextError::MissingForm(id))
    }

    pub fn add_operator(&mut self, node: OperatorNode) -> OperatorId {
        let id = OperatorId::new(self.operators.len() as u64);
        self.operators.push(node);
        id
    }

    pub fn operator(&self, id: OperatorId) -> Result<&OperatorNode, ContextError> {
        self.operators
            .get(id.get() as usize)
            .ok_or(ContextError::MissingOperator(id))
    }

    pub fn record_refinement(&mut self, refinement: Refinement) -> usize {
        let id = self.refinements.len();
        self.refinements.push(refinement);
        id
    }

    pub fn refinements(&self) -> &[Refinement] {
        &self.refinements
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stage_handles_do_not_alias() {
        let mut ctx = Context::new();
        let x = ctx.intern_expr(ExprNode::Symbol("x".into()));
        let eq = ctx.add_equation(EquationNode {
            residual: x,
            name: Some("x_zero".into()),
        });
        assert_eq!(ctx.expr(x).unwrap(), &ExprNode::Symbol("x".into()));
        assert_eq!(ctx.equation(eq).unwrap().residual, x);
    }
}
