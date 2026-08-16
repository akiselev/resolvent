//! Continuum and variational form dialect.
//!
//! Forms preserve field/domain/test/trial semantics long enough for mathematically correct
//! transformations. They reference scalar/tensor [`crate::ExprId`] payloads instead of
//! cloning a second algebraic expression language.

use crate::ExprId;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FieldId(pub u32);
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DomainId(pub u32);
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FunctionSpaceId(pub u32);
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct FormExprId(pub u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FieldRole {
    State,
    Trial,
    Test,
    Coefficient,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum DifferentialOperator {
    Grad,
    Div,
    Curl,
    Dt,
    Trace,
    NormalDerivative,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Measure {
    Volume(DomainId),
    Boundary(DomainId),
    Interface(DomainId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Field {
    pub id: FieldId,
    pub name: String,
    pub space: FunctionSpaceId,
    pub role: FieldRole,
    pub scalar_symbol: ExprId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FormExpr {
    Field(FieldId),
    Scalar(ExprId),
    Differential {
        op: DifferentialOperator,
        arg: FormExprId,
    },
    Add(Vec<FormExprId>),
    Mul(Vec<FormExprId>),
    Inner(FormExprId, FormExprId),
    Contract(FormExprId, FormExprId),
    Integral {
        integrand: FormExprId,
        measure: Measure,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FormProgram {
    pub fields: Vec<Field>,
    pub expressions: Vec<FormExpr>,
    pub residuals: Vec<FormExprId>,
}

impl FormProgram {
    pub fn push(&mut self, expr: FormExpr) -> FormExprId {
        let id = FormExprId(self.expressions.len() as u32);
        self.expressions.push(expr);
        id
    }

    pub fn expr(&self, id: FormExprId) -> Option<&FormExpr> {
        self.expressions.get(id.0 as usize)
    }
}
