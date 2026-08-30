//! RV1-C2: a narrow, receipt-producing algebra-operation surface over
//! caller-owned [`TermStore`] roots.
//!
//! This is the surface a consumer's operation-specific projection (for
//! example Scientia's GX-A2 projection) calls after it has already projected
//! its own scientific expression identity into Resolvent [`Term`](crate::TermId)
//! atoms/nodes. Resolvent stays consumer-neutral here: nothing in this module
//! knows about scientific roles, units, or any consumer semantic arena.
//!
//! ## Operation contract
//!
//! [`TermAlgebraOperation::Differentiate`] and [`TermAlgebraOperation::Simplify`]
//! are implemented by a lossless round trip through the existing exact
//! [`Expr`] differentiator/canonicalizer (RV0). The scalar subset they
//! recognize is:
//!
//! - the three exact literal atom classes (RV1-C3): `Integer`, `Rational`,
//!   `ExactDecimal`;
//! - free symbol atoms;
//! - `Apply` nodes whose head is a symbol in the reserved
//!   `"Resolvent`"` namespace, with the names `Add`, `Sub`, `Mul`, `Div`,
//!   `Neg`, `Pow` (integer exponent), `Sin`, `Cos`, `Exp`, `Log`, `Ln`,
//!   `Sqrt`.
//!
//! Any other `Apply` head, or an `Apply` whose arity does not match one of
//! the names above, is outside the operation's contract. Relations, boolean
//! connectives, conditions, collections, ordered maps, indexing/slicing,
//! rules, binders, and held syntax are likewise outside the arithmetic
//! contract; see the per-operation behavior below.
//!
//! [`TermAlgebraOperation::Simplify`] never refuses: it recurses through
//! `Relation`, `Boolean`, `Condition`, `Piecewise`, `Collection`,
//! `OrderedMap`, `Index`, `Slice`, and `Rule` wrapper nodes, canonicalizing
//! every maximal scalar-eligible subtree it finds and leaving everything else
//! (including any construct outside the contract) exactly as it was. This is
//! what "preserve comparisons/conditions/piecewise/collections/... structurally"
//! means for `Simplify` (RV1-C2, extended by GX-F6). Two node kinds stay
//! deliberately opaque rather than recursing:
//!
//! - `OrderedMap` simplifies each entry's *value*, never its *key*: keys are
//!   retained-syntax identity (`TermStore` enforces structural key
//!   uniqueness), and simplifying a key could collapse two distinct keys
//!   into a structural duplicate that no longer decodes;
//! - `Held` and `Binder` are never recursed into: `Held` exists precisely to
//!   suppress evaluation of its contents, and a `Binder` body uses de Bruijn
//!   indices whose scope this narrow scalar round trip does not model, so
//!   both are preserved byte-for-byte.
//!
//! [`TermAlgebraOperation::Differentiate`] refuses (does not silently zero)
//! any construct outside the contract, with the stable
//! [`TermAlgebraRefusal::code`] `ALGEBRA_OUTSIDE_OPERATION`. Two exceptions
//! differentiate structurally instead of refusing:
//!
//! - `Piecewise` differentiates branchwise when every case's condition is
//!   free of the differentiation variable (checked structurally via
//!   [`TermStore::free_symbols`]); a variable-dependent condition refuses
//!   instead of silently moving the branch point;
//! - `Collection` differentiates elementwise when every element is itself
//!   scalar-eligible (or another `Collection` of scalar-eligible elements),
//!   which is what a vector-valued case binding such as `[0, 0, 0]` or
//!   `[sin(x), 0]` needs. An element outside the scalar contract refuses
//!   with *that element's* construct label, not a generic `"Collection"`
//!   label.
//!
//! `OrderedMap`, `Index`, `Slice`, and `Rule` have no well-defined
//! elementwise derivative in this narrow contract (an ordered map's keys and
//! an index/slice's positions are not differentiable quantities), so
//! `Differentiate` refuses them with their own construct label, unlike
//! `Simplify`.
//!
//! [`TermAlgebraOperation::Substitute`] does not go through [`Expr`] at all:
//! it resolves each binding's symbol to its (hash-consed) atom [`TermId`]
//! and delegates to the existing binder-safe [`TermStore::substitute_closed`]
//! (RV1-B3), which already preserves every node kind — including
//! `Collection`, `OrderedMap`, `Index`, `Slice`, and `Rule` — structurally.
//!
//! ## Literal spelling is normalized, value is not
//!
//! [`Expr`] has exactly one exact numeric variant (`Expr::Rational`), so any
//! subtree that round-trips through it collapses `Integer`/`Rational`/
//! `ExactDecimal` atoms to a single `Atom::Rational` on the way back out. The
//! exact *value* is always preserved (RV1-C3's `Rational` is the shared
//! lossless representation, and this round trip never touches `f64`), but the
//! literal's original spelling does not survive `Simplify`/`Differentiate`:
//! an input decimal `5.0` may come back as the atom `5` (rational `5/1`).
//! `Substitute` never does this, since it does not go through [`Expr`].
//!
//! ## `Pow` of an exact literal base folds
//!
//! [`Expr::canonicalize`] (RV0, shared by `Simplify` and by the trailing
//! canonicalization `Differentiate` performs on every derivative) folds
//! `Pow` of an exact literal base and a nonzero, non-unit integer exponent to
//! an exact literal: `5^2` simplifies to the atom `25`, and `(1/2)^-1` to
//! `2`. `0^0` folds to `1` (matching the existing convention in
//! [`Expr::evaluate`]); `0` raised to a negative exponent is a genuine
//! division by exact zero and the operation fails with
//! [`AlgebraError::DivisionByZero`] rather than silently producing a value.
//! The fold is charged against `AlgebraBudget::max_coefficient_bits` so an
//! adversarial exponent cannot force unbounded exact-integer growth. A `Pow`
//! whose base is not itself an exact literal (a symbol, a compound
//! expression, or a function call) is left as a structural `Pow` node,
//! unchanged from before.

use std::collections::HashMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::AlgebraWork;
use crate::{
    AlgebraBudget, AlgebraError, Atom, CollectionKind, Expr, PiecewiseCase, SymbolName, TermBudget,
    TermDigest, TermError, TermId, TermNode, TermStats, TermStore,
};

/// Reserved namespace for the arithmetic/function operator symbols this
/// operation surface recognizes as `Apply` heads. Consumers projecting scalar
/// algebra into this contract must spell recognized operators in this
/// namespace; any other namespace (or non-symbol head) is outside the
/// contract.
pub const OPERATOR_NAMESPACE: &str = "Resolvent`";

/// A requested term-level algebra operation (RV1-C2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TermAlgebraOperation {
    /// Symbolic derivative with respect to one free symbol.
    Differentiate { variable: SymbolName },
    /// Canonicalize every maximal scalar-eligible subtree; everything else is
    /// preserved structurally.
    Simplify,
    /// Replace every occurrence of each named free symbol with a closed
    /// replacement term. Implemented via [`TermStore::substitute_closed`].
    Substitute { bindings: Vec<(SymbolName, TermId)> },
}

/// A term construct outside the requested operation's contract.
///
/// `construct` is a short, stable-shaped diagnostic label (for example
/// `"Relation"`, `"Piecewise[variable-dependent condition]"`, or
/// `"Apply[Resolvent`Foo/2]"`); it is diagnostic text, not itself a wire
/// identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TermAlgebraRefusal {
    OutsideOperation { construct: String },
}

impl TermAlgebraRefusal {
    /// The stable refusal code, shared with the GX-CONTRACTS C9 registry.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::OutsideOperation { .. } => "ALGEBRA_OUTSIDE_OPERATION",
        }
    }
}

impl fmt::Display for TermAlgebraRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutsideOperation { construct } => {
                write!(f, "{}: {construct}", self.code())
            }
        }
    }
}

/// Failure of [`TermStore::apply_algebra_operation`]: either a structural
/// refusal (the request was understood but is outside the operation's
/// contract) or a genuine resource/store error.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum TermAlgebraError {
    #[error("{0}")]
    Refused(TermAlgebraRefusal),
    #[error(transparent)]
    Term(#[from] TermError),
    #[error(transparent)]
    Algebra(#[from] AlgebraError),
}

/// What budgets were in effect and how much of them the operation used.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TermAlgebraBudgetReport {
    pub term_budget: TermBudget,
    pub algebra_budget: AlgebraBudget,
    /// Nodes charged against `algebra_budget.max_expression_nodes` while
    /// projecting to/from the exact differentiator/canonicalizer.
    pub expression_nodes_used: usize,
    /// Structural size/depth/sharing of the output root.
    pub output: TermStats,
}

/// The operation actually performed, recorded for the receipt. Distinct from
/// [`TermAlgebraOperation`]: it carries only durable, serializable data (no
/// local [`TermId`] handles), matching RV1's rule that no Resolvent term
/// handle becomes durable identity.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TermAlgebraReceiptOperation {
    Simplify,
    Differentiate {
        namespace: String,
        name: String,
    },
    Substitute {
        /// `(namespace, name)` pairs, in request order.
        bindings: Vec<(String, String)>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TermAlgebraReceipt {
    pub schema: String,
    pub operation: TermAlgebraReceiptOperation,
    pub input_digest: String,
    pub output_digest: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TermAlgebraReceiptWire {
    schema: String,
    operation: TermAlgebraReceiptOperation,
    input_digest: String,
    output_digest: String,
}

impl<'de> Deserialize<'de> for TermAlgebraReceipt {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = TermAlgebraReceiptWire::deserialize(deserializer)?;
        if wire.schema != TERM_ALGEBRA_RECEIPT_SCHEMA
            || !crate::receipt::valid_digest(&wire.input_digest)
            || !crate::receipt::valid_digest(&wire.output_digest)
        {
            return Err(serde::de::Error::custom("invalid term algebra receipt"));
        }
        Ok(Self {
            schema: wire.schema,
            operation: wire.operation,
            input_digest: wire.input_digest,
            output_digest: wire.output_digest,
        })
    }
}

const TERM_ALGEBRA_RECEIPT_SCHEMA: &str = "resolvent-term-algebra-receipt/1";

impl TermAlgebraReceipt {
    fn new(
        operation: TermAlgebraReceiptOperation,
        input_digest: TermDigest,
        output_digest: TermDigest,
    ) -> Self {
        Self {
            schema: TERM_ALGEBRA_RECEIPT_SCHEMA.into(),
            operation,
            input_digest: input_digest.to_string(),
            output_digest: output_digest.to_string(),
        }
    }
}

/// Result of one [`TermAlgebraOperation`] applied to a [`TermStore`] root.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TermAlgebraOutcome {
    /// The result, re-imported into the same [`TermStore`] the request root
    /// came from.
    pub root: TermId,
    /// Non-obvious mathematical assumptions the operation relied on (for
    /// example, that a piecewise branch's condition does not depend on the
    /// differentiation variable). Empty when the operation made none.
    pub assumptions: Vec<String>,
    pub budget: TermAlgebraBudgetReport,
    pub receipt: TermAlgebraReceipt,
}

impl TermStore {
    /// Apply one [`TermAlgebraOperation`] to `root`, producing a receipt.
    ///
    /// See the module documentation for the recognized scalar subset and the
    /// per-operation refusal/structural-preservation contract.
    pub fn apply_algebra_operation(
        &mut self,
        root: TermId,
        operation: &TermAlgebraOperation,
        term_budget: TermBudget,
        algebra_budget: AlgebraBudget,
    ) -> Result<TermAlgebraOutcome, TermAlgebraError> {
        let input_digest = self.digest(root, term_budget)?;
        let mut work = AlgebraWork::new(algebra_budget, "term algebra operation");
        let mut assumptions = Vec::new();
        let (output_root, receipt_operation) = match operation {
            TermAlgebraOperation::Simplify => {
                let output = simplify_term(self, root, term_budget, algebra_budget, &mut work)?;
                (output, TermAlgebraReceiptOperation::Simplify)
            }
            TermAlgebraOperation::Differentiate { variable } => {
                let output = differentiate_term(
                    self,
                    root,
                    variable,
                    term_budget,
                    algebra_budget,
                    &mut work,
                    &mut assumptions,
                )?;
                (
                    output,
                    TermAlgebraReceiptOperation::Differentiate {
                        namespace: variable.namespace.clone(),
                        name: variable.name.clone(),
                    },
                )
            }
            TermAlgebraOperation::Substitute { bindings } => {
                let mut pairs = Vec::with_capacity(bindings.len());
                let mut tags = Vec::with_capacity(bindings.len());
                for (name, value) in bindings {
                    let symbol_id = self.atom(Atom::Symbol(name.clone()), term_budget)?;
                    pairs.push((symbol_id, *value));
                    tags.push((name.namespace.clone(), name.name.clone()));
                }
                let output = self.substitute_closed(root, &pairs, term_budget)?;
                (
                    output,
                    TermAlgebraReceiptOperation::Substitute { bindings: tags },
                )
            }
        };
        let output_digest = self.digest(output_root, term_budget)?;
        let stats = self.stats(output_root, term_budget)?;
        Ok(TermAlgebraOutcome {
            root: output_root,
            assumptions,
            budget: TermAlgebraBudgetReport {
                term_budget,
                algebra_budget,
                expression_nodes_used: work.used(),
                output: stats,
            },
            receipt: TermAlgebraReceipt::new(receipt_operation, input_digest, output_digest),
        })
    }
}

/// Outcome of trying to interpret one term as part of the recognized scalar
/// vocabulary: either an [`Expr`] fragment, a diagnosis that the term is
/// outside the vocabulary (with a label for [`TermAlgebraRefusal`]), or a
/// genuine budget/store failure.
enum ScalarProbe {
    NotScalar(String),
    Failed(TermAlgebraError),
}

impl From<TermError> for ScalarProbe {
    fn from(error: TermError) -> Self {
        Self::Failed(error.into())
    }
}

impl From<AlgebraError> for ScalarProbe {
    fn from(error: AlgebraError) -> Self {
        Self::Failed(error.into())
    }
}

fn term_to_expr(
    store: &TermStore,
    term: TermId,
    symbols: &mut Vec<SymbolName>,
    symbol_index: &mut HashMap<SymbolName, usize>,
    work: &mut AlgebraWork,
) -> Result<Expr, ScalarProbe> {
    work.spend(1)?;
    let node = store.node(term)?;
    match &node {
        TermNode::Atom(atom @ (Atom::Integer(_) | Atom::Rational(_) | Atom::ExactDecimal(_))) => {
            let value = atom
                .exact_rational_value()
                .expect("integer/rational/exact-decimal atoms always have an exact rational value");
            Ok(Expr::Rational(value))
        }
        TermNode::Atom(Atom::Symbol(name)) => {
            let index = intern_symbol(name, symbols, symbol_index);
            Ok(Expr::Symbol(synthetic_symbol(index)))
        }
        TermNode::Apply { head, arguments } => match operator_name(store, *head) {
            Some(op_name) => scalar_apply(store, &op_name, arguments, symbols, symbol_index, work),
            None => Err(ScalarProbe::NotScalar(apply_construct_label(store, *head))),
        },
        _ => Err(ScalarProbe::NotScalar(node_kind_label(&node).to_string())),
    }
}

fn synthetic_symbol(index: usize) -> String {
    format!("t{index}")
}

fn intern_symbol(
    name: &SymbolName,
    symbols: &mut Vec<SymbolName>,
    symbol_index: &mut HashMap<SymbolName, usize>,
) -> usize {
    if let Some(index) = symbol_index.get(name) {
        return *index;
    }
    let index = symbols.len();
    symbols.push(name.clone());
    symbol_index.insert(name.clone(), index);
    index
}

fn operator_name(store: &TermStore, head: TermId) -> Option<String> {
    match store.node(head).ok()? {
        TermNode::Atom(Atom::Symbol(name)) if name.namespace == OPERATOR_NAMESPACE => {
            Some(name.name)
        }
        _ => None,
    }
}

fn convert_all(
    store: &TermStore,
    arguments: &[TermId],
    symbols: &mut Vec<SymbolName>,
    symbol_index: &mut HashMap<SymbolName, usize>,
    work: &mut AlgebraWork,
) -> Result<Vec<Expr>, ScalarProbe> {
    arguments
        .iter()
        .map(|argument| term_to_expr(store, *argument, symbols, symbol_index, work))
        .collect()
}

fn scalar_apply(
    store: &TermStore,
    op_name: &str,
    arguments: &[TermId],
    symbols: &mut Vec<SymbolName>,
    symbol_index: &mut HashMap<SymbolName, usize>,
    work: &mut AlgebraWork,
) -> Result<Expr, ScalarProbe> {
    match (op_name, arguments.len()) {
        ("Add", _) => Ok(Expr::add(convert_all(
            store,
            arguments,
            symbols,
            symbol_index,
            work,
        )?)),
        ("Mul", _) => Ok(Expr::mul(convert_all(
            store,
            arguments,
            symbols,
            symbol_index,
            work,
        )?)),
        ("Sub", 2) => {
            let left = term_to_expr(store, arguments[0], symbols, symbol_index, work)?;
            let right = term_to_expr(store, arguments[1], symbols, symbol_index, work)?;
            Ok(Expr::add([left, Expr::mul([Expr::integer(-1), right])]))
        }
        ("Div", 2) => {
            let left = term_to_expr(store, arguments[0], symbols, symbol_index, work)?;
            let right = term_to_expr(store, arguments[1], symbols, symbol_index, work)?;
            Ok(Expr::mul([left, right.pow(-1)]))
        }
        ("Neg", 1) => {
            let value = term_to_expr(store, arguments[0], symbols, symbol_index, work)?;
            Ok(Expr::mul([Expr::integer(-1), value]))
        }
        ("Pow", 2) => {
            let base = term_to_expr(store, arguments[0], symbols, symbol_index, work)?;
            let exponent = scalar_integer_exponent(store, arguments[1])?;
            Ok(base.pow(exponent))
        }
        ("Sin", 1) => scalar_function(store, "sin", arguments[0], symbols, symbol_index, work),
        ("Cos", 1) => scalar_function(store, "cos", arguments[0], symbols, symbol_index, work),
        ("Exp", 1) => scalar_function(store, "exp", arguments[0], symbols, symbol_index, work),
        ("Sqrt", 1) => scalar_function(store, "sqrt", arguments[0], symbols, symbol_index, work),
        ("Log", 1) => scalar_function(store, "log", arguments[0], symbols, symbol_index, work),
        ("Ln", 1) => scalar_function(store, "ln", arguments[0], symbols, symbol_index, work),
        _ => Err(ScalarProbe::NotScalar(format!(
            "Apply[{OPERATOR_NAMESPACE}{op_name}/{}]",
            arguments.len()
        ))),
    }
}

fn scalar_function(
    store: &TermStore,
    name: &str,
    argument: TermId,
    symbols: &mut Vec<SymbolName>,
    symbol_index: &mut HashMap<SymbolName, usize>,
    work: &mut AlgebraWork,
) -> Result<Expr, ScalarProbe> {
    let argument = term_to_expr(store, argument, symbols, symbol_index, work)?;
    Ok(Expr::function(name, [argument]))
}

fn scalar_integer_exponent(store: &TermStore, term: TermId) -> Result<i32, ScalarProbe> {
    match store.node(term)? {
        TermNode::Atom(Atom::Integer(text)) => text
            .parse::<i32>()
            .map_err(|_| ScalarProbe::NotScalar("Pow[exponent out of range]".to_string())),
        _ => Err(ScalarProbe::NotScalar(
            "Pow[non-integer exponent]".to_string(),
        )),
    }
}

fn apply_construct_label(store: &TermStore, head: TermId) -> String {
    match store.node(head) {
        Ok(TermNode::Atom(Atom::Symbol(name))) => {
            format!("Apply[{}{}]", name.namespace, name.name)
        }
        _ => "Apply[non-symbol head]".to_string(),
    }
}

fn node_kind_label(node: &TermNode) -> &'static str {
    match node {
        TermNode::Atom(atom) => atom_kind_label(atom),
        TermNode::Apply { .. } => "Apply",
        TermNode::Relation { .. } => "Relation",
        TermNode::Boolean { .. } => "Boolean",
        TermNode::Condition { .. } => "Condition",
        TermNode::Piecewise { .. } => "Piecewise",
        TermNode::Collection { .. } => "Collection",
        TermNode::OrderedMap { .. } => "OrderedMap",
        TermNode::Index { .. } => "Index",
        TermNode::Slice { .. } => "Slice",
        TermNode::Rule { .. } => "Rule",
        TermNode::Binder { .. } => "Binder",
        TermNode::Held { .. } => "Held",
    }
}

fn atom_kind_label(atom: &Atom) -> &'static str {
    match atom {
        Atom::Integer(_) | Atom::Rational(_) | Atom::ExactDecimal(_) | Atom::Symbol(_) => {
            "Atom[scalar]"
        }
        Atom::ExactIeee754Bits(_) => "Atom[ExactIeee754Bits]",
        Atom::MachineFloatBits(_) => "Atom[MachineFloatBits]",
        Atom::PrecisionReal(_) => "Atom[PrecisionReal]",
        Atom::String(_) => "Atom[String]",
        Atom::Bytes(_) => "Atom[Bytes]",
        Atom::Boolean(_) => "Atom[Boolean]",
        Atom::Constant(_) => "Atom[Constant]",
        Atom::BoundVariable(_) => "Atom[BoundVariable]",
    }
}

fn expr_to_term(
    store: &mut TermStore,
    expr: &Expr,
    symbols: &[SymbolName],
    term_budget: TermBudget,
) -> Result<TermId, TermError> {
    match expr {
        Expr::Rational(value) => store.exact_rational(value.clone(), term_budget),
        Expr::Symbol(name) => {
            let index: usize = name[1..]
                .parse()
                .expect("synthetic symbol names are always `t{index}`");
            store.atom(Atom::Symbol(symbols[index].clone()), term_budget)
        }
        Expr::Add(terms) => build_apply(store, "Add", terms, symbols, term_budget),
        Expr::Mul(factors) => build_apply(store, "Mul", factors, symbols, term_budget),
        Expr::Pow { base, exponent } => {
            let base = expr_to_term(store, base, symbols, term_budget)?;
            let exponent = store.exact_integer(i128::from(*exponent), term_budget)?;
            let head = operator_symbol(store, "Pow", term_budget)?;
            store.intern(
                TermNode::Apply {
                    head,
                    arguments: vec![base, exponent],
                },
                term_budget,
            )
        }
        Expr::Function { name, args } => {
            let head_name = match name.as_str() {
                "sin" => "Sin",
                "cos" => "Cos",
                "exp" => "Exp",
                "sqrt" => "Sqrt",
                "log" => "Log",
                "ln" => "Ln",
                other => other,
            };
            let arguments = args
                .iter()
                .map(|argument| expr_to_term(store, argument, symbols, term_budget))
                .collect::<Result<Vec<_>, _>>()?;
            let head = operator_symbol(store, head_name, term_budget)?;
            store.intern(TermNode::Apply { head, arguments }, term_budget)
        }
    }
}

fn build_apply(
    store: &mut TermStore,
    name: &str,
    children: &[Expr],
    symbols: &[SymbolName],
    term_budget: TermBudget,
) -> Result<TermId, TermError> {
    let arguments = children
        .iter()
        .map(|child| expr_to_term(store, child, symbols, term_budget))
        .collect::<Result<Vec<_>, _>>()?;
    let head = operator_symbol(store, name, term_budget)?;
    store.intern(TermNode::Apply { head, arguments }, term_budget)
}

fn operator_symbol(
    store: &mut TermStore,
    name: &str,
    term_budget: TermBudget,
) -> Result<TermId, TermError> {
    store.atom(
        Atom::Symbol(SymbolName::new(OPERATOR_NAMESPACE, name)),
        term_budget,
    )
}

/// [`TermAlgebraOperation::Simplify`]: never refuses. Canonicalizes every
/// maximal scalar-eligible subtree and preserves everything else.
fn simplify_term(
    store: &mut TermStore,
    term: TermId,
    term_budget: TermBudget,
    algebra_budget: AlgebraBudget,
    work: &mut AlgebraWork,
) -> Result<TermId, TermAlgebraError> {
    let mut symbols = Vec::new();
    let mut symbol_index = HashMap::new();
    match term_to_expr(store, term, &mut symbols, &mut symbol_index, work) {
        Ok(expr) => {
            let simplified = expr.canonicalize(algebra_budget)?;
            Ok(expr_to_term(store, &simplified, &symbols, term_budget)?)
        }
        Err(ScalarProbe::Failed(error)) => Err(error),
        Err(ScalarProbe::NotScalar(_)) => {
            simplify_structural(store, term, term_budget, algebra_budget, work)
        }
    }
}

fn simplify_structural(
    store: &mut TermStore,
    term: TermId,
    term_budget: TermBudget,
    algebra_budget: AlgebraBudget,
    work: &mut AlgebraWork,
) -> Result<TermId, TermAlgebraError> {
    let node = store.node(term)?;
    match node {
        TermNode::Relation {
            operator,
            left,
            right,
        } => {
            let left = simplify_term(store, left, term_budget, algebra_budget, work)?;
            let right = simplify_term(store, right, term_budget, algebra_budget, work)?;
            Ok(store.intern(
                TermNode::Relation {
                    operator,
                    left,
                    right,
                },
                term_budget,
            )?)
        }
        TermNode::Boolean {
            operator,
            arguments,
        } => {
            let arguments = arguments
                .into_iter()
                .map(|argument| simplify_term(store, argument, term_budget, algebra_budget, work))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(store.intern(
                TermNode::Boolean {
                    operator,
                    arguments,
                },
                term_budget,
            )?)
        }
        TermNode::Condition {
            expression,
            condition,
        } => {
            let expression = simplify_term(store, expression, term_budget, algebra_budget, work)?;
            let condition = simplify_term(store, condition, term_budget, algebra_budget, work)?;
            Ok(store.intern(
                TermNode::Condition {
                    expression,
                    condition,
                },
                term_budget,
            )?)
        }
        TermNode::Piecewise { cases, otherwise } => {
            let cases = cases
                .into_iter()
                .map(|case| {
                    Ok(PiecewiseCase {
                        value: simplify_term(store, case.value, term_budget, algebra_budget, work)?,
                        condition: simplify_term(
                            store,
                            case.condition,
                            term_budget,
                            algebra_budget,
                            work,
                        )?,
                    })
                })
                .collect::<Result<Vec<_>, TermAlgebraError>>()?;
            let otherwise = otherwise
                .map(|value| simplify_term(store, value, term_budget, algebra_budget, work))
                .transpose()?;
            Ok(store.intern(TermNode::Piecewise { cases, otherwise }, term_budget)?)
        }
        TermNode::Apply { head, arguments } => {
            let head = simplify_term(store, head, term_budget, algebra_budget, work)?;
            let arguments = arguments
                .into_iter()
                .map(|argument| simplify_term(store, argument, term_budget, algebra_budget, work))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(store.intern(TermNode::Apply { head, arguments }, term_budget)?)
        }
        TermNode::Collection { kind, elements } => {
            let elements = elements
                .into_iter()
                .map(|element| simplify_term(store, element, term_budget, algebra_budget, work))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(store.intern(TermNode::Collection { kind, elements }, term_budget)?)
        }
        TermNode::OrderedMap { entries } => {
            // Keys are retained-syntax identity (structurally unique); only
            // values are simplified. See the module documentation.
            let entries = entries
                .into_iter()
                .map(|(key, value)| {
                    let value = simplify_term(store, value, term_budget, algebra_budget, work)?;
                    Ok((key, value))
                })
                .collect::<Result<Vec<_>, TermAlgebraError>>()?;
            Ok(store.intern(TermNode::OrderedMap { entries }, term_budget)?)
        }
        TermNode::Index { target, indices } => {
            let target = simplify_term(store, target, term_budget, algebra_budget, work)?;
            let indices = indices
                .into_iter()
                .map(|index| simplify_term(store, index, term_budget, algebra_budget, work))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(store.intern(TermNode::Index { target, indices }, term_budget)?)
        }
        TermNode::Slice {
            target,
            start,
            end,
            step,
        } => {
            let target = simplify_term(store, target, term_budget, algebra_budget, work)?;
            let start = start
                .map(|value| simplify_term(store, value, term_budget, algebra_budget, work))
                .transpose()?;
            let end = end
                .map(|value| simplify_term(store, value, term_budget, algebra_budget, work))
                .transpose()?;
            let step = step
                .map(|value| simplify_term(store, value, term_budget, algebra_budget, work))
                .transpose()?;
            Ok(store.intern(
                TermNode::Slice {
                    target,
                    start,
                    end,
                    step,
                },
                term_budget,
            )?)
        }
        TermNode::Rule {
            kind,
            pattern,
            replacement,
            condition,
        } => {
            let pattern = simplify_term(store, pattern, term_budget, algebra_budget, work)?;
            let replacement = simplify_term(store, replacement, term_budget, algebra_budget, work)?;
            let condition = condition
                .map(|value| simplify_term(store, value, term_budget, algebra_budget, work))
                .transpose()?;
            Ok(store.intern(
                TermNode::Rule {
                    kind,
                    pattern,
                    replacement,
                    condition,
                },
                term_budget,
            )?)
        }
        // `Held` and `Binder` stay opaque: `Held` exists to suppress
        // evaluation of its contents, and a `Binder` body's de Bruijn scope
        // is not modeled by this narrow scalar round trip. Both, and every
        // other non-scalar atom, are preserved structurally unchanged.
        _ => Ok(term),
    }
}

/// [`TermAlgebraOperation::Differentiate`]: refuses (does not zero) any
/// construct outside the contract, except `Piecewise` with
/// variable-independent conditions (differentiates branchwise) and
/// `Collection` of scalar-eligible elements (differentiates elementwise).
fn differentiate_term(
    store: &mut TermStore,
    term: TermId,
    variable: &SymbolName,
    term_budget: TermBudget,
    algebra_budget: AlgebraBudget,
    work: &mut AlgebraWork,
    assumptions: &mut Vec<String>,
) -> Result<TermId, TermAlgebraError> {
    let mut symbols = Vec::new();
    let mut symbol_index = HashMap::new();
    match term_to_expr(store, term, &mut symbols, &mut symbol_index, work) {
        Ok(expr) => {
            let variable_name = symbol_index
                .get(variable)
                .map(|index| synthetic_symbol(*index))
                // The variable never appears in `expr`, so every symbol
                // comparison inside `Expr::differentiate` correctly yields
                // zero; this sentinel can never collide with a real
                // synthetic name (those are always `t<digits>`).
                .unwrap_or_else(|| "$unused-differentiation-variable".to_string());
            let derivative = expr.differentiate(&variable_name, algebra_budget)?;
            Ok(expr_to_term(store, &derivative, &symbols, term_budget)?)
        }
        Err(ScalarProbe::Failed(error)) => Err(error),
        Err(ScalarProbe::NotScalar(construct)) => {
            let node = store.node(term)?;
            match node {
                TermNode::Piecewise { cases, otherwise } => differentiate_piecewise(
                    store,
                    cases,
                    otherwise,
                    variable,
                    term_budget,
                    algebra_budget,
                    work,
                    assumptions,
                ),
                TermNode::Collection { kind, elements } => differentiate_collection(
                    store,
                    kind,
                    elements,
                    variable,
                    term_budget,
                    algebra_budget,
                    work,
                    assumptions,
                ),
                _ => Err(TermAlgebraError::Refused(
                    TermAlgebraRefusal::OutsideOperation { construct },
                )),
            }
        }
    }
}

/// `Collection` differentiates elementwise when every element is itself
/// scalar-eligible (or another `Collection` of scalar-eligible elements,
/// via recursion): this is what a vector-valued case binding such as
/// `[0, 0, 0]` or `[sin(x), 0]` needs. An element outside the scalar
/// contract refuses with that element's own construct label.
#[allow(clippy::too_many_arguments)]
fn differentiate_collection(
    store: &mut TermStore,
    kind: CollectionKind,
    elements: Vec<TermId>,
    variable: &SymbolName,
    term_budget: TermBudget,
    algebra_budget: AlgebraBudget,
    work: &mut AlgebraWork,
    assumptions: &mut Vec<String>,
) -> Result<TermId, TermAlgebraError> {
    let elements = elements
        .into_iter()
        .map(|element| {
            differentiate_term(
                store,
                element,
                variable,
                term_budget,
                algebra_budget,
                work,
                assumptions,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(store.intern(TermNode::Collection { kind, elements }, term_budget)?)
}

#[allow(clippy::too_many_arguments)]
fn differentiate_piecewise(
    store: &mut TermStore,
    cases: Vec<PiecewiseCase>,
    otherwise: Option<TermId>,
    variable: &SymbolName,
    term_budget: TermBudget,
    algebra_budget: AlgebraBudget,
    work: &mut AlgebraWork,
    assumptions: &mut Vec<String>,
) -> Result<TermId, TermAlgebraError> {
    let mut differentiated = Vec::with_capacity(cases.len());
    for case in cases {
        let condition_symbols = store.free_symbols(case.condition, term_budget)?;
        if condition_symbols.contains(variable) {
            return Err(TermAlgebraError::Refused(
                TermAlgebraRefusal::OutsideOperation {
                    construct: "Piecewise[variable-dependent condition]".to_string(),
                },
            ));
        }
        assumptions.push(format!(
            "piecewise branch condition does not depend on `{}{}`, so it differentiates branchwise",
            variable.namespace, variable.name
        ));
        let value = differentiate_term(
            store,
            case.value,
            variable,
            term_budget,
            algebra_budget,
            work,
            assumptions,
        )?;
        differentiated.push(PiecewiseCase {
            value,
            condition: case.condition,
        });
    }
    let otherwise = otherwise
        .map(|value| {
            differentiate_term(
                store,
                value,
                variable,
                term_budget,
                algebra_budget,
                work,
                assumptions,
            )
        })
        .transpose()?;
    Ok(store.intern(
        TermNode::Piecewise {
            cases: differentiated,
            otherwise,
        },
        term_budget,
    )?)
}
