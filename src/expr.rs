use crate::id::{ExprId, SymbolId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Exact-by-construction expression literal. Arbitrary precision values are carried as
/// canonical decimal strings until the algebra backend owns their concrete representation.
/// `FloatBits` means the exact IEEE-754 dyadic value represented by those bits; it never
/// means "approximately this decimal".
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScalarLiteral {
    Integer(String),
    Rational { numerator: String, denominator: String },
    FloatBits(u64),
    NamedConstant(String),
}

impl ScalarLiteral {
    pub fn integer(value: i64) -> Self {
        Self::Integer(value.to_string())
    }

    pub fn f64_exact(value: f64) -> Option<Self> {
        value.is_finite().then(|| Self::FloatBits(value.to_bits()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub role: SymbolRole,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dimension: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolRole {
    Independent,
    State,
    Parameter,
    Algebraic,
    Auxiliary,
}

/// Generic mathematical expression IR. It deliberately does not contain mesh, basis,
/// quadrature, test-function, or solver concepts. Higher scientific dialects reference
/// `ExprId` rather than cloning or reparsing scalar expressions.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExprNode {
    Literal(ScalarLiteral),
    Symbol(SymbolId),
    Neg(ExprId),
    Add(Vec<ExprId>),
    Mul(Vec<ExprId>),
    Div { numerator: ExprId, denominator: ExprId },
    PowI { base: ExprId, exponent: i32 },
    Apply { function: String, args: Vec<ExprId> },
    /// Semantic derivative. Structural/system passes may later replace state derivatives
    /// with derivative variables; executable AD belongs below this layer.
    Derivative { expr: ExprId, with_respect_to: SymbolId, order: u8 },
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ExprStore {
    nodes: Vec<ExprNode>,
    #[serde(skip)]
    index: BTreeMap<ExprNode, ExprId>,
}

impl ExprStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn rebuild_index(&mut self) {
        self.index.clear();
        for (i, node) in self.nodes.iter().cloned().enumerate() {
            self.index.insert(node, ExprId(i as u32));
        }
    }

    pub fn intern(&mut self, node: ExprNode) -> ExprId {
        if let Some(id) = self.index.get(&node) {
            return *id;
        }
        let id = ExprId(self.nodes.len() as u32);
        self.nodes.push(node.clone());
        self.index.insert(node, id);
        id
    }

    pub fn get(&self, id: ExprId) -> Option<&ExprNode> {
        self.nodes.get(id.index())
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn literal(&mut self, literal: ScalarLiteral) -> ExprId {
        self.intern(ExprNode::Literal(literal))
    }

    pub fn symbol(&mut self, symbol: SymbolId) -> ExprId {
        self.intern(ExprNode::Symbol(symbol))
    }

    pub fn add(&mut self, terms: impl IntoIterator<Item = ExprId>) -> ExprId {
        let mut terms: Vec<_> = terms.into_iter().collect();
        terms.sort_unstable();
        self.intern(ExprNode::Add(terms))
    }

    pub fn mul(&mut self, factors: impl IntoIterator<Item = ExprId>) -> ExprId {
        let mut factors: Vec<_> = factors.into_iter().collect();
        factors.sort_unstable();
        self.intern(ExprNode::Mul(factors))
    }
}

/// Caller-owned table. No process-global symbol interner is permitted.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
    #[serde(skip)]
    names: BTreeMap<String, SymbolId>,
}

impl SymbolTable {
    pub fn declare(&mut self, symbol: Symbol) -> SymbolId {
        if let Some(id) = self.names.get(&symbol.name) {
            return *id;
        }
        let id = SymbolId(self.symbols.len() as u32);
        self.names.insert(symbol.name.clone(), id);
        self.symbols.push(symbol);
        id
    }

    pub fn get(&self, id: SymbolId) -> Option<&Symbol> {
        self.symbols.get(id.index())
    }

    pub fn rebuild_index(&mut self) {
        self.names.clear();
        for (i, symbol) in self.symbols.iter().enumerate() {
            self.names.insert(symbol.name.clone(), SymbolId(i as u32));
        }
    }
}
