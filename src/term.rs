//! Caller-owned immutable structural terms.
//!
//! Term identity preserves retained syntax. Construction does not sort, flatten,
//! commute, cancel, factor, reassociate, or otherwise apply algebraic laws.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

use dashu::integer::{IBig, UBig};
use dashu::rational::RBig;
use thiserror::Error;

use crate::Rational;

const WIRE_HEADER: &[u8] = b"RESOLVENT-TERM\0\x01";
static NEXT_STORE_ID: AtomicU64 = AtomicU64::new(1);

/// Store-relative compact term handle. Stable identity is [`TermDigest`], not this value.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TermId {
    store: u64,
    index: u32,
}

impl TermId {
    pub fn index(self) -> usize {
        self.index as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymbolId(u32);

impl SymbolId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Stable cross-store identity for one retained structural term.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TermDigest([u8; 32]);

impl TermDigest {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for TermDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Explicit bound for construction, traversal, import, and canonical wire work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TermBudget {
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_children_per_node: usize,
    pub max_atom_bytes: usize,
    pub max_wire_bytes: usize,
}

impl Default for TermBudget {
    fn default() -> Self {
        Self {
            max_nodes: 1_000_000,
            max_depth: 100_000,
            max_children_per_node: 100_000,
            max_atom_bytes: 1 << 20,
            max_wire_bytes: 256 << 20,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SymbolName {
    pub namespace: String,
    pub name: String,
}

impl SymbolName {
    pub fn new(namespace: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            name: name.into(),
        }
    }
}

/// Canonical exact decimal `coefficient * 10^-scale`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ExactDecimal {
    coefficient: String,
    scale: i64,
}

impl ExactDecimal {
    pub fn parse(text: &str) -> Result<Self, TermError> {
        parse_decimal(text).map(|(coefficient, scale)| Self { coefficient, scale })
    }

    pub fn coefficient(&self) -> &str {
        &self.coefficient
    }

    pub fn scale(&self) -> i64 {
        self.scale
    }
}

/// Canonical decimal significand/exponent with an explicit approximation precision.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PrecisionReal {
    significand: String,
    exponent10: i64,
    precision_bits: u32,
}

impl PrecisionReal {
    pub fn new(significand: &str, exponent10: i64, precision_bits: u32) -> Result<Self, TermError> {
        if precision_bits == 0 {
            return Err(TermError::InvalidAtom("precision must be nonzero"));
        }
        let mut significand = canonical_integer(significand)?;
        let mut exponent10 = exponent10;
        normalize_coefficient(&mut significand, &mut exponent10)?;
        Ok(Self {
            significand,
            exponent10,
            precision_bits,
        })
    }

    pub fn significand(&self) -> &str {
        &self.significand
    }

    pub fn exponent10(&self) -> i64 {
        self.exponent10
    }

    pub fn precision_bits(&self) -> u32 {
        self.precision_bits
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SymbolicConstant {
    Pi,
    E,
    ImaginaryUnit,
    Infinity,
    ComplexInfinity,
    Undefined,
}

/// Canonical atom input. String-backed integer forms are validated at construction.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Atom {
    Integer(String),
    Rational(Rational),
    ExactDecimal(ExactDecimal),
    ExactIeee754Bits(u64),
    MachineFloatBits(u64),
    PrecisionReal(PrecisionReal),
    String(String),
    Bytes(Vec<u8>),
    Symbol(SymbolName),
    Boolean(bool),
    Constant(SymbolicConstant),
    /// De Bruijn index: zero names the nearest bound variable.
    BoundVariable(u32),
}

impl Atom {
    pub fn integer(text: &str) -> Result<Self, TermError> {
        Ok(Self::Integer(canonical_integer(text)?))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RelationOperator {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BooleanOperator {
    Not,
    And,
    Or,
    Xor,
    Implies,
    Equivalent,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum CollectionKind {
    Tuple,
    List,
    Array { shape: Vec<usize> },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuleKind {
    Immediate,
    Delayed,
    Pattern,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinderKind {
    Lambda,
    Sum,
    Product,
    Integral,
    Limit,
    Local,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PiecewiseCase {
    pub value: TermId,
    pub condition: TermId,
}

/// Minimal generic retained node vocabulary. Every vector is structurally ordered.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum TermNode {
    Atom(Atom),
    Apply {
        head: TermId,
        arguments: Vec<TermId>,
    },
    Relation {
        operator: RelationOperator,
        left: TermId,
        right: TermId,
    },
    Boolean {
        operator: BooleanOperator,
        arguments: Vec<TermId>,
    },
    Condition {
        expression: TermId,
        condition: TermId,
    },
    Piecewise {
        cases: Vec<PiecewiseCase>,
        otherwise: Option<TermId>,
    },
    Collection {
        kind: CollectionKind,
        elements: Vec<TermId>,
    },
    /// Entry order is retained syntax; this is not a mathematically unordered map.
    OrderedMap {
        entries: Vec<(TermId, TermId)>,
    },
    Index {
        target: TermId,
        indices: Vec<TermId>,
    },
    Slice {
        target: TermId,
        start: Option<TermId>,
        end: Option<TermId>,
        step: Option<TermId>,
    },
    Rule {
        kind: RuleKind,
        pattern: TermId,
        replacement: TermId,
        condition: Option<TermId>,
    },
    /// Bounds are outside the new scope. `body` uses de Bruijn indices.
    Binder {
        kind: BinderKind,
        variable_count: u32,
        bounds: Vec<TermId>,
        body: TermId,
    },
    Held {
        expression: TermId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreMetrics {
    pub terms: usize,
    pub symbols: usize,
    /// Portable logical schema bytes, excluding allocator/hash-table overhead.
    ///
    /// Nodes and unique symbol-table entries are charged separately by fixed
    /// widths documented in the RV1 contract; symbol names are charged once.
    pub logical_bytes: u64,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum TermError {
    #[error("term handle belongs to another store")]
    ForeignTerm,
    #[error("unknown term handle")]
    UnknownTerm,
    #[error("invalid atom: {0}")]
    InvalidAtom(&'static str),
    #[error("invalid structural node: {0}")]
    InvalidNode(&'static str),
    #[error("invalid binder: {0}")]
    InvalidBinder(&'static str),
    #[error("term budget exceeded for {resource}: limit {limit}")]
    BudgetExceeded {
        resource: &'static str,
        limit: usize,
    },
    #[error("term store exhausted its local handle space")]
    HandleSpaceExhausted,
    #[error("invalid canonical term wire: {0}")]
    InvalidWire(&'static str),
    #[error("term wire is well-formed but not canonical")]
    NonCanonicalWire,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum StoredAtom {
    Integer(String),
    Rational(Rational),
    ExactDecimal(ExactDecimal),
    ExactIeee754Bits(u64),
    MachineFloatBits(u64),
    PrecisionReal(PrecisionReal),
    String(String),
    Bytes(Vec<u8>),
    Symbol(SymbolName),
    Boolean(bool),
    Constant(SymbolicConstant),
    BoundVariable(u32),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum StoredNode {
    Atom(StoredAtom),
    Apply(TermId, Vec<TermId>),
    Relation(RelationOperator, TermId, TermId),
    Boolean(BooleanOperator, Vec<TermId>),
    Condition(TermId, TermId),
    Piecewise(Vec<PiecewiseCase>, Option<TermId>),
    Collection(CollectionKind, Vec<TermId>),
    OrderedMap(Vec<(TermId, TermId)>),
    Index(TermId, Vec<TermId>),
    Slice(TermId, Option<TermId>, Option<TermId>, Option<TermId>),
    Rule(RuleKind, TermId, TermId, Option<TermId>),
    Binder(BinderKind, u32, Vec<TermId>, TermId),
    Held(TermId),
}

/// Strong arena ownership for the first RV1 integration slice.
///
/// Construction requires `&mut self`; once shared immutably, ordinary Rust `Sync`
/// rules provide thread-safe concurrent reads. Compaction/weak retention is RV1-B2.
#[derive(Debug)]
pub struct TermStore {
    id: u64,
    nodes: Vec<StoredNode>,
    depths: Vec<usize>,
    interner: HashMap<StoredNode, u32>,
    symbols: Vec<SymbolName>,
    symbol_interner: HashMap<SymbolName, u32>,
    logical_node_bytes: u64,
    logical_symbol_bytes: u64,
}

impl TermStore {
    pub fn new() -> Result<Self, TermError> {
        let id = NEXT_STORE_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                (value < u64::MAX).then_some(value + 1)
            })
            .map_err(|_| TermError::HandleSpaceExhausted)?;
        Ok(Self {
            id,
            nodes: Vec::new(),
            depths: Vec::new(),
            interner: HashMap::new(),
            symbols: Vec::new(),
            symbol_interner: HashMap::new(),
            logical_node_bytes: 0,
            logical_symbol_bytes: 0,
        })
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn metrics(&self) -> StoreMetrics {
        StoreMetrics {
            terms: self.nodes.len(),
            symbols: self.symbols.len(),
            logical_bytes: self
                .logical_node_bytes
                .saturating_add(self.logical_symbol_bytes),
        }
    }

    pub fn symbol(&self, id: SymbolId) -> Option<&SymbolName> {
        self.symbols.get(id.index())
    }

    pub fn intern(&mut self, node: TermNode, budget: TermBudget) -> Result<TermId, TermError> {
        let stored = self.store_node(node, budget)?;
        let depth = node_depth(&stored, |term| self.depths.get(term.index()).copied())?;
        if depth > budget.max_depth {
            return Err(exceeded("depth", budget.max_depth));
        }
        if let Some(index) = self.interner.get(&stored) {
            return Ok(self.id(*index));
        }
        if self.nodes.len() >= budget.max_nodes {
            return Err(exceeded("nodes", budget.max_nodes));
        }
        self.commit_stored(stored, depth)
    }

    fn commit_stored(&mut self, stored: StoredNode, depth: usize) -> Result<TermId, TermError> {
        let index = u32::try_from(self.nodes.len()).map_err(|_| TermError::HandleSpaceExhausted)?;
        if let StoredNode::Atom(StoredAtom::Symbol(value)) = &stored
            && !self.symbol_interner.contains_key(value)
        {
            let symbol_index =
                u32::try_from(self.symbols.len()).map_err(|_| TermError::HandleSpaceExhausted)?;
            self.logical_symbol_bytes = self
                .logical_symbol_bytes
                .saturating_add(logical_symbol_bytes(value));
            self.symbols.push(value.clone());
            self.symbol_interner.insert(value.clone(), symbol_index);
        }
        self.logical_node_bytes = self
            .logical_node_bytes
            .saturating_add(logical_node_bytes(&stored));
        self.nodes.push(stored.clone());
        self.depths.push(depth);
        self.interner.insert(stored, index);
        Ok(self.id(index))
    }

    pub fn atom(&mut self, atom: Atom, budget: TermBudget) -> Result<TermId, TermError> {
        self.intern(TermNode::Atom(atom), budget)
    }

    pub fn node(&self, id: TermId) -> Result<TermNode, TermError> {
        Ok(self.public_node(self.get(id)?))
    }

    /// Deterministic child-first walk over the reachable DAG.
    pub fn topological(&self, root: TermId, budget: TermBudget) -> Result<Vec<TermId>, TermError> {
        self.get(root)?;
        let mut seen = HashSet::new();
        let mut output = Vec::new();
        let mut stack = vec![(root, false)];
        while let Some((term, expanded)) = stack.pop() {
            if expanded {
                output.push(term);
                continue;
            }
            if !seen.insert(term) {
                continue;
            }
            if seen.len() > budget.max_nodes {
                return Err(exceeded("nodes", budget.max_nodes));
            }
            stack.push((term, true));
            let children = children(self.get(term)?);
            if children.len() > budget.max_children_per_node {
                return Err(exceeded("children", budget.max_children_per_node));
            }
            for child in children.into_iter().rev() {
                stack.push((child, false));
            }
        }
        let mut depths = HashMap::with_capacity(output.len());
        for term in &output {
            let depth = children(self.get(*term)?)
                .iter()
                .map(|child| depths[child])
                .max()
                .unwrap_or(0usize)
                .checked_add(1)
                .ok_or_else(|| exceeded("depth", budget.max_depth))?;
            if depth > budget.max_depth {
                return Err(exceeded("depth", budget.max_depth));
            }
            depths.insert(*term, depth);
        }
        Ok(output)
    }

    pub fn canonical_bytes(&self, root: TermId, budget: TermBudget) -> Result<Vec<u8>, TermError> {
        let order = self.topological(root, budget)?;
        if self.required_outer_depth(&order)? != 0 {
            return Err(TermError::InvalidBinder(
                "canonical roots may not contain escaping de Bruijn indices",
            ));
        }
        let canonical_ids = order
            .iter()
            .enumerate()
            .map(|(index, term)| (*term, index as u64))
            .collect::<HashMap<_, _>>();
        let mut output = Vec::new();
        push_bytes(&mut output, WIRE_HEADER, budget)?;
        push_varint(&mut output, order.len() as u64, budget)?;
        for term in &order {
            encode_node(self.get(*term)?, &canonical_ids, &mut output, budget)?;
        }
        push_varint(
            &mut output,
            *canonical_ids.get(&root).expect("root is in traversal"),
            budget,
        )?;
        Ok(output)
    }

    pub fn digest(&self, root: TermId, budget: TermBudget) -> Result<TermDigest, TermError> {
        let bytes = self.canonical_bytes(root, budget)?;
        Ok(TermDigest(*blake3::hash(&bytes).as_bytes()))
    }

    /// Rebuild reachable structure into another store without copying local handles.
    pub fn import(
        &mut self,
        source: &TermStore,
        root: TermId,
        budget: TermBudget,
    ) -> Result<TermId, TermError> {
        let order = source.topological(root, budget)?;
        let mut rebuilt = HashMap::new();
        let mut planned = Vec::<(StoredNode, usize)>::new();
        let mut planned_ids = HashMap::<StoredNode, u32>::new();
        let base = self.nodes.len();
        for source_id in order {
            let mut node = source.node(source_id)?;
            remap_node(&mut node, &rebuilt)?;
            validate_node_shape(&node, budget)?;
            let stored = self.stored_from_node(node, budget)?;
            let depth = node_depth(&stored, |term| {
                self.depths.get(term.index()).copied().or_else(|| {
                    term.index()
                        .checked_sub(base)
                        .and_then(|index| planned.get(index).map(|(_, depth)| *depth))
                })
            })?;
            if depth > budget.max_depth {
                return Err(exceeded("depth", budget.max_depth));
            }
            let target = if let Some(index) = self.interner.get(&stored) {
                self.id(*index)
            } else if let Some(index) = planned_ids.get(&stored) {
                self.id(*index)
            } else {
                let next = base
                    .checked_add(planned.len())
                    .ok_or_else(|| exceeded("nodes", budget.max_nodes))?;
                if next >= budget.max_nodes {
                    return Err(exceeded("nodes", budget.max_nodes));
                }
                let index = u32::try_from(next).map_err(|_| TermError::HandleSpaceExhausted)?;
                planned_ids.insert(stored.clone(), index);
                planned.push((stored, depth));
                self.id(index)
            };
            rebuilt.insert(source_id, target);
        }
        let target = rebuilt.get(&root).copied().ok_or(TermError::UnknownTerm)?;
        let new_symbols = planned
            .iter()
            .filter_map(|(node, _)| match node {
                StoredNode::Atom(StoredAtom::Symbol(value))
                    if !self.symbol_interner.contains_key(value) =>
                {
                    Some(value)
                }
                _ => None,
            })
            .collect::<HashSet<_>>();
        if (self.symbols.len() as u128) + (new_symbols.len() as u128) > (u32::MAX as u128) + 1 {
            return Err(TermError::HandleSpaceExhausted);
        }
        for (stored, depth) in planned {
            self.commit_stored(stored, depth)?;
        }
        Ok(target)
    }

    fn id(&self, index: u32) -> TermId {
        TermId {
            store: self.id,
            index,
        }
    }

    fn get(&self, id: TermId) -> Result<&StoredNode, TermError> {
        if id.store != self.id {
            return Err(TermError::ForeignTerm);
        }
        self.nodes.get(id.index()).ok_or(TermError::UnknownTerm)
    }

    fn store_node(&self, node: TermNode, budget: TermBudget) -> Result<StoredNode, TermError> {
        validate_node_shape(&node, budget)?;
        for child in public_children(&node) {
            self.get(child)?;
        }
        self.stored_from_node(node, budget)
    }

    fn stored_from_node(
        &self,
        node: TermNode,
        budget: TermBudget,
    ) -> Result<StoredNode, TermError> {
        let stored = match node {
            TermNode::Atom(atom) => StoredNode::Atom(self.store_atom(atom, budget)?),
            TermNode::Apply { head, arguments } => StoredNode::Apply(head, arguments),
            TermNode::Relation {
                operator,
                left,
                right,
            } => StoredNode::Relation(operator, left, right),
            TermNode::Boolean {
                operator,
                arguments,
            } => StoredNode::Boolean(operator, arguments),
            TermNode::Condition {
                expression,
                condition,
            } => StoredNode::Condition(expression, condition),
            TermNode::Piecewise { cases, otherwise } => StoredNode::Piecewise(cases, otherwise),
            TermNode::Collection { kind, elements } => StoredNode::Collection(kind, elements),
            TermNode::OrderedMap { entries } => StoredNode::OrderedMap(entries),
            TermNode::Index { target, indices } => StoredNode::Index(target, indices),
            TermNode::Slice {
                target,
                start,
                end,
                step,
            } => StoredNode::Slice(target, start, end, step),
            TermNode::Rule {
                kind,
                pattern,
                replacement,
                condition,
            } => StoredNode::Rule(kind, pattern, replacement, condition),
            TermNode::Binder {
                kind,
                variable_count,
                bounds,
                body,
            } => {
                validate_binder_shape(kind, variable_count, bounds.len())?;
                StoredNode::Binder(kind, variable_count, bounds, body)
            }
            TermNode::Held { expression } => StoredNode::Held(expression),
        };
        Ok(stored)
    }

    fn store_atom(&self, atom: Atom, budget: TermBudget) -> Result<StoredAtom, TermError> {
        validate_atom(&atom, budget)?;
        Ok(match atom {
            Atom::Integer(value) => StoredAtom::Integer(value),
            Atom::Rational(value) => StoredAtom::Rational(value),
            Atom::ExactDecimal(value) => StoredAtom::ExactDecimal(value),
            Atom::ExactIeee754Bits(value) => StoredAtom::ExactIeee754Bits(value),
            Atom::MachineFloatBits(value) => StoredAtom::MachineFloatBits(value),
            Atom::PrecisionReal(value) => StoredAtom::PrecisionReal(value),
            Atom::String(value) => StoredAtom::String(value),
            Atom::Bytes(value) => StoredAtom::Bytes(value),
            Atom::Symbol(value) => StoredAtom::Symbol(value),
            Atom::Boolean(value) => StoredAtom::Boolean(value),
            Atom::Constant(value) => StoredAtom::Constant(value),
            Atom::BoundVariable(value) => StoredAtom::BoundVariable(value),
        })
    }

    fn public_node(&self, node: &StoredNode) -> TermNode {
        match node {
            StoredNode::Atom(atom) => TermNode::Atom(match atom {
                StoredAtom::Integer(value) => Atom::Integer(value.clone()),
                StoredAtom::Rational(value) => Atom::Rational(value.clone()),
                StoredAtom::ExactDecimal(value) => Atom::ExactDecimal(value.clone()),
                StoredAtom::ExactIeee754Bits(value) => Atom::ExactIeee754Bits(*value),
                StoredAtom::MachineFloatBits(value) => Atom::MachineFloatBits(*value),
                StoredAtom::PrecisionReal(value) => Atom::PrecisionReal(value.clone()),
                StoredAtom::String(value) => Atom::String(value.clone()),
                StoredAtom::Bytes(value) => Atom::Bytes(value.clone()),
                StoredAtom::Symbol(value) => Atom::Symbol(value.clone()),
                StoredAtom::Boolean(value) => Atom::Boolean(*value),
                StoredAtom::Constant(value) => Atom::Constant(*value),
                StoredAtom::BoundVariable(value) => Atom::BoundVariable(*value),
            }),
            StoredNode::Apply(head, arguments) => TermNode::Apply {
                head: *head,
                arguments: arguments.clone(),
            },
            StoredNode::Relation(operator, left, right) => TermNode::Relation {
                operator: *operator,
                left: *left,
                right: *right,
            },
            StoredNode::Boolean(operator, arguments) => TermNode::Boolean {
                operator: *operator,
                arguments: arguments.clone(),
            },
            StoredNode::Condition(expression, condition) => TermNode::Condition {
                expression: *expression,
                condition: *condition,
            },
            StoredNode::Piecewise(cases, otherwise) => TermNode::Piecewise {
                cases: cases.clone(),
                otherwise: *otherwise,
            },
            StoredNode::Collection(kind, elements) => TermNode::Collection {
                kind: kind.clone(),
                elements: elements.clone(),
            },
            StoredNode::OrderedMap(entries) => TermNode::OrderedMap {
                entries: entries.clone(),
            },
            StoredNode::Index(target, indices) => TermNode::Index {
                target: *target,
                indices: indices.clone(),
            },
            StoredNode::Slice(target, start, end, step) => TermNode::Slice {
                target: *target,
                start: *start,
                end: *end,
                step: *step,
            },
            StoredNode::Rule(kind, pattern, replacement, condition) => TermNode::Rule {
                kind: *kind,
                pattern: *pattern,
                replacement: *replacement,
                condition: *condition,
            },
            StoredNode::Binder(kind, variable_count, bounds, body) => TermNode::Binder {
                kind: *kind,
                variable_count: *variable_count,
                bounds: bounds.clone(),
                body: *body,
            },
            StoredNode::Held(expression) => TermNode::Held {
                expression: *expression,
            },
        }
    }

    fn required_outer_depth(&self, order: &[TermId]) -> Result<u32, TermError> {
        let mut required = HashMap::<TermId, u32>::with_capacity(order.len());
        for term in order {
            let node = self.get(*term)?;
            let need = match node {
                StoredNode::Atom(StoredAtom::BoundVariable(index)) => index
                    .checked_add(1)
                    .ok_or(TermError::InvalidBinder("bound-variable depth overflow"))?,
                StoredNode::Binder(_, variables, bounds, body) => {
                    let bound_need = bounds.iter().map(|term| required[term]).max().unwrap_or(0);
                    bound_need.max(required[body].saturating_sub(*variables))
                }
                _ => children(node)
                    .iter()
                    .map(|term| required[term])
                    .max()
                    .unwrap_or(0),
            };
            required.insert(*term, need);
        }
        Ok(required[order.last().ok_or(TermError::UnknownTerm)?])
    }
}

/// Decode a complete canonical term into a fresh caller-owned store.
pub fn decode_canonical_term(
    bytes: &[u8],
    budget: TermBudget,
) -> Result<(TermStore, TermId), TermError> {
    if bytes.len() > budget.max_wire_bytes {
        return Err(exceeded("wire bytes", budget.max_wire_bytes));
    }
    let mut cursor = Cursor::new(bytes);
    if cursor.take(WIRE_HEADER.len())? != WIRE_HEADER {
        return Err(TermError::InvalidWire("schema header"));
    }
    let count = usize::try_from(cursor.varint()?)
        .map_err(|_| TermError::InvalidWire("node count overflow"))?;
    if count == 0 {
        return Err(TermError::InvalidWire("empty term graph"));
    }
    if count > budget.max_nodes {
        return Err(exceeded("nodes", budget.max_nodes));
    }
    let mut store = TermStore::new()?;
    let mut canonical = Vec::with_capacity(count);
    for index in 0..count {
        let node = decode_node(&mut cursor, &canonical, budget)?;
        let term = store.intern(node, budget)?;
        if term.index() != index {
            return Err(TermError::NonCanonicalWire);
        }
        canonical.push(term);
    }
    let root = usize::try_from(cursor.varint()?)
        .map_err(|_| TermError::InvalidWire("root index overflow"))?;
    if root.checked_add(1) != Some(count) || !cursor.is_empty() {
        return Err(TermError::InvalidWire("root index or trailing bytes"));
    }
    let root = canonical[root];
    if store.canonical_bytes(root, budget)? != bytes {
        return Err(TermError::NonCanonicalWire);
    }
    Ok((store, root))
}

fn validate_atom(atom: &Atom, budget: TermBudget) -> Result<(), TermError> {
    let bytes = match atom {
        Atom::Integer(value) => {
            if canonical_integer(value)? != *value {
                return Err(TermError::InvalidAtom("non-canonical integer"));
            }
            value.len()
        }
        Atom::Rational(value) => {
            value.as_rbig().numerator().to_string().len()
                + value.as_rbig().denominator().to_string().len()
        }
        Atom::ExactDecimal(value) => {
            if canonical_integer(&value.coefficient)? != value.coefficient
                || (value.coefficient == "0" && value.scale != 0)
                || (value.coefficient != "0" && value.coefficient.ends_with('0'))
            {
                return Err(TermError::InvalidAtom("non-canonical exact decimal"));
            }
            value.coefficient.len()
        }
        Atom::PrecisionReal(value) => {
            let rebuilt =
                PrecisionReal::new(&value.significand, value.exponent10, value.precision_bits)?;
            if rebuilt != *value {
                return Err(TermError::InvalidAtom("non-canonical precision real"));
            }
            value.significand.len()
        }
        Atom::String(value) => value.len(),
        Atom::Bytes(value) => value.len(),
        Atom::Symbol(value) => {
            if value.name.trim().is_empty()
                || value.name.as_bytes().contains(&0)
                || value.namespace.as_bytes().contains(&0)
            {
                return Err(TermError::InvalidAtom("invalid symbol name"));
            }
            value.namespace.len() + value.name.len()
        }
        Atom::ExactIeee754Bits(_)
        | Atom::MachineFloatBits(_)
        | Atom::Boolean(_)
        | Atom::Constant(_)
        | Atom::BoundVariable(_) => 8,
    };
    if bytes > budget.max_atom_bytes {
        return Err(exceeded("atom bytes", budget.max_atom_bytes));
    }
    Ok(())
}

fn validate_node_shape(node: &TermNode, budget: TermBudget) -> Result<(), TermError> {
    let child_count = public_children(node).len();
    if child_count > budget.max_children_per_node {
        return Err(exceeded("children", budget.max_children_per_node));
    }
    match node {
        TermNode::Boolean {
            operator: BooleanOperator::Not,
            arguments,
        } if arguments.len() != 1 => Err(TermError::InvalidNode("Not requires one argument")),
        TermNode::Boolean {
            operator,
            arguments,
        } if *operator != BooleanOperator::Not && arguments.len() < 2 => Err(
            TermError::InvalidNode("non-unary boolean operators require two arguments"),
        ),
        TermNode::Piecewise { cases, otherwise } if cases.is_empty() && otherwise.is_none() => Err(
            TermError::InvalidNode("piecewise requires a case or default"),
        ),
        TermNode::Collection {
            kind: CollectionKind::Array { shape },
            elements,
        } => {
            if shape.len() > budget.max_children_per_node {
                return Err(exceeded("array rank", budget.max_children_per_node));
            }
            if shape.is_empty() || shape.contains(&0) {
                return Err(TermError::InvalidNode(
                    "array shape must have positive extents",
                ));
            }
            let size = shape
                .iter()
                .try_fold(1usize, |size, extent| size.checked_mul(*extent));
            if size != Some(elements.len()) {
                return Err(TermError::InvalidNode(
                    "array shape does not match element count",
                ));
            }
            Ok(())
        }
        TermNode::OrderedMap { entries } => {
            let mut keys = HashSet::new();
            if entries.iter().any(|(key, _)| !keys.insert(*key)) {
                return Err(TermError::InvalidNode(
                    "ordered map keys must be structurally unique",
                ));
            }
            Ok(())
        }
        TermNode::Index { indices, .. } if indices.is_empty() => {
            Err(TermError::InvalidNode("index requires at least one index"))
        }
        _ => Ok(()),
    }
}

fn validate_binder_shape(
    kind: BinderKind,
    variable_count: u32,
    bound_count: usize,
) -> Result<(), TermError> {
    if variable_count == 0 {
        return Err(TermError::InvalidBinder("variable count must be nonzero"));
    }
    let variables = variable_count as usize;
    let expected_bounds = match kind {
        BinderKind::Lambda => 0,
        BinderKind::Sum | BinderKind::Product | BinderKind::Integral => variables
            .checked_mul(2)
            .ok_or(TermError::InvalidBinder("binder arity overflow"))?,
        BinderKind::Limit | BinderKind::Local => variables,
    };
    if bound_count != expected_bounds {
        return Err(TermError::InvalidBinder("wrong bound arity"));
    }
    Ok(())
}

fn canonical_integer(text: &str) -> Result<String, TermError> {
    let integer =
        IBig::from_str(text).map_err(|_| TermError::InvalidAtom("invalid canonical integer"))?;
    let canonical = integer.to_string();
    if canonical != text {
        return Err(TermError::InvalidAtom("non-canonical integer spelling"));
    }
    Ok(canonical)
}

fn parse_decimal(text: &str) -> Result<(String, i64), TermError> {
    if text.is_empty() || text.as_bytes().contains(&0) {
        return Err(TermError::InvalidAtom("invalid exact decimal"));
    }
    let (mantissa, exponent) = match text.find(['e', 'E']) {
        Some(index) => {
            if text[index + 1..].contains(['e', 'E']) {
                return Err(TermError::InvalidAtom("invalid exact decimal exponent"));
            }
            let exponent = text[index + 1..]
                .parse::<i64>()
                .map_err(|_| TermError::InvalidAtom("invalid exact decimal exponent"))?;
            (&text[..index], exponent)
        }
        None => (text, 0),
    };
    let negative = mantissa.starts_with('-');
    let unsigned = mantissa.strip_prefix(['-', '+']).unwrap_or(mantissa);
    let mut parts = unsigned.split('.');
    let whole = parts.next().unwrap_or_default();
    let fraction = parts.next().unwrap_or_default();
    if parts.next().is_some()
        || whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(TermError::InvalidAtom("invalid exact decimal"));
    }
    let mut digits = format!("{whole}{fraction}");
    let mut scale = i64::try_from(fraction.len())
        .map_err(|_| TermError::InvalidAtom("exact decimal scale overflow"))?
        .checked_sub(exponent)
        .ok_or(TermError::InvalidAtom("exact decimal scale overflow"))?;
    while digits.len() > 1 && digits.ends_with('0') {
        digits.pop();
        scale = scale
            .checked_sub(1)
            .ok_or(TermError::InvalidAtom("exact decimal scale overflow"))?;
    }
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Ok(("0".into(), 0));
    }
    let coefficient = if negative {
        format!("-{digits}")
    } else {
        digits.to_owned()
    };
    Ok((coefficient, scale))
}

fn normalize_coefficient(coefficient: &mut String, exponent: &mut i64) -> Result<(), TermError> {
    if coefficient == "0" {
        *exponent = 0;
        return Ok(());
    }
    while coefficient.ends_with('0') {
        coefficient.pop();
        *exponent = exponent
            .checked_add(1)
            .ok_or(TermError::InvalidAtom("precision-real exponent overflow"))?;
    }
    Ok(())
}

// Portable logical accounting constants. These describe retained schema data,
// not Rust layout, allocator capacity, or hash-table overhead.
const LOGICAL_TAG_BYTES: u64 = 1;
const LOGICAL_LENGTH_BYTES: u64 = 8;
const LOGICAL_REFERENCE_BYTES: u64 = 8;
const LOGICAL_SYMBOL_REFERENCE_BYTES: u64 = 4;

fn logical_blob_bytes(length: usize) -> u64 {
    LOGICAL_LENGTH_BYTES.saturating_add(length as u64)
}

fn logical_symbol_bytes(value: &SymbolName) -> u64 {
    logical_blob_bytes(value.namespace.len()).saturating_add(logical_blob_bytes(value.name.len()))
}

fn logical_node_bytes(node: &StoredNode) -> u64 {
    let references = |count: usize| (count as u64).saturating_mul(LOGICAL_REFERENCE_BYTES);
    LOGICAL_TAG_BYTES.saturating_add(match node {
        StoredNode::Atom(atom) => LOGICAL_TAG_BYTES.saturating_add(match atom {
            StoredAtom::Integer(value) | StoredAtom::String(value) => {
                logical_blob_bytes(value.len())
            }
            StoredAtom::Rational(value) => {
                logical_blob_bytes(value.as_rbig().numerator().to_string().len()).saturating_add(
                    logical_blob_bytes(value.as_rbig().denominator().to_string().len()),
                )
            }
            StoredAtom::ExactDecimal(value) => {
                logical_blob_bytes(value.coefficient.len()).saturating_add(8)
            }
            StoredAtom::PrecisionReal(value) => logical_blob_bytes(value.significand.len())
                .saturating_add(8)
                .saturating_add(4),
            StoredAtom::Bytes(value) => logical_blob_bytes(value.len()),
            StoredAtom::Symbol(_) => LOGICAL_SYMBOL_REFERENCE_BYTES,
            StoredAtom::ExactIeee754Bits(_) | StoredAtom::MachineFloatBits(_) => 8,
            StoredAtom::Boolean(_) | StoredAtom::Constant(_) => 1,
            StoredAtom::BoundVariable(_) => 4,
        }),
        StoredNode::Apply(_, values) | StoredNode::Index(_, values) => {
            LOGICAL_LENGTH_BYTES.saturating_add(references(values.len().saturating_add(1)))
        }
        StoredNode::Relation(_, _, _) => LOGICAL_TAG_BYTES.saturating_add(references(2)),
        StoredNode::Condition(_, _) => references(2),
        StoredNode::Boolean(_, values) => LOGICAL_TAG_BYTES
            .saturating_add(LOGICAL_LENGTH_BYTES)
            .saturating_add(references(values.len())),
        StoredNode::Piecewise(values, otherwise) => LOGICAL_LENGTH_BYTES
            .saturating_add(references(values.len().saturating_mul(2)))
            .saturating_add(LOGICAL_TAG_BYTES)
            .saturating_add(references(usize::from(otherwise.is_some()))),
        StoredNode::Collection(kind, values) => {
            let shape = match kind {
                CollectionKind::Array { shape } => LOGICAL_LENGTH_BYTES
                    .saturating_add((shape.len() as u64).saturating_mul(LOGICAL_LENGTH_BYTES)),
                CollectionKind::Tuple | CollectionKind::List => 0,
            };
            LOGICAL_TAG_BYTES
                .saturating_add(shape)
                .saturating_add(LOGICAL_LENGTH_BYTES)
                .saturating_add(references(values.len()))
        }
        StoredNode::OrderedMap(values) => {
            LOGICAL_LENGTH_BYTES.saturating_add(references(values.len().saturating_mul(2)))
        }
        StoredNode::Slice(_, start, end, step) => references(1)
            .saturating_add(3 * LOGICAL_TAG_BYTES)
            .saturating_add(references(
                usize::from(start.is_some())
                    + usize::from(end.is_some())
                    + usize::from(step.is_some()),
            )),
        StoredNode::Rule(_, _, _, condition) => LOGICAL_TAG_BYTES
            .saturating_add(references(2))
            .saturating_add(LOGICAL_TAG_BYTES)
            .saturating_add(references(usize::from(condition.is_some()))),
        StoredNode::Binder(_, _, values, _) => LOGICAL_TAG_BYTES
            .saturating_add(4)
            .saturating_add(LOGICAL_LENGTH_BYTES)
            .saturating_add(references(values.len().saturating_add(1))),
        StoredNode::Held(_) => references(1),
    })
}

fn node_depth(
    node: &StoredNode,
    mut depth_of: impl FnMut(TermId) -> Option<usize>,
) -> Result<usize, TermError> {
    children(node)
        .into_iter()
        .map(|child| depth_of(child).ok_or(TermError::UnknownTerm))
        .try_fold(0usize, |maximum, depth| Ok(maximum.max(depth?)))?
        .checked_add(1)
        .ok_or_else(|| exceeded("depth", usize::MAX))
}

fn children(node: &StoredNode) -> Vec<TermId> {
    match node {
        StoredNode::Atom(_) => Vec::new(),
        StoredNode::Apply(head, arguments) => {
            let mut result = Vec::with_capacity(arguments.len() + 1);
            result.push(*head);
            result.extend(arguments);
            result
        }
        StoredNode::Relation(_, left, right) | StoredNode::Condition(left, right) => {
            vec![*left, *right]
        }
        StoredNode::Boolean(_, arguments) | StoredNode::Collection(_, arguments) => {
            arguments.clone()
        }
        StoredNode::Piecewise(cases, otherwise) => {
            let mut result = Vec::with_capacity(cases.len() * 2 + usize::from(otherwise.is_some()));
            for case in cases {
                result.push(case.value);
                result.push(case.condition);
            }
            result.extend(otherwise);
            result
        }
        StoredNode::OrderedMap(entries) => entries
            .iter()
            .flat_map(|(key, value)| [*key, *value])
            .collect(),
        StoredNode::Index(target, indices) => {
            let mut result = Vec::with_capacity(indices.len() + 1);
            result.push(*target);
            result.extend(indices);
            result
        }
        StoredNode::Slice(target, start, end, step) => {
            let mut result = vec![*target];
            result.extend(*start);
            result.extend(*end);
            result.extend(*step);
            result
        }
        StoredNode::Rule(_, pattern, replacement, condition) => {
            let mut result = vec![*pattern, *replacement];
            result.extend(*condition);
            result
        }
        StoredNode::Binder(_, _, bounds, body) => {
            let mut result = bounds.clone();
            result.push(*body);
            result
        }
        StoredNode::Held(expression) => vec![*expression],
    }
}

fn public_children(node: &TermNode) -> Vec<TermId> {
    match node {
        TermNode::Atom(_) => Vec::new(),
        TermNode::Apply { head, arguments } => {
            let mut result = vec![*head];
            result.extend(arguments);
            result
        }
        TermNode::Relation { left, right, .. }
        | TermNode::Condition {
            expression: left,
            condition: right,
        } => vec![*left, *right],
        TermNode::Boolean { arguments, .. }
        | TermNode::Collection {
            elements: arguments,
            ..
        } => arguments.clone(),
        TermNode::Piecewise { cases, otherwise } => {
            let mut result = Vec::new();
            for case in cases {
                result.extend([case.value, case.condition]);
            }
            result.extend(*otherwise);
            result
        }
        TermNode::OrderedMap { entries } => entries
            .iter()
            .flat_map(|(key, value)| [*key, *value])
            .collect(),
        TermNode::Index { target, indices } => {
            let mut result = vec![*target];
            result.extend(indices);
            result
        }
        TermNode::Slice {
            target,
            start,
            end,
            step,
        } => {
            let mut result = vec![*target];
            result.extend(*start);
            result.extend(*end);
            result.extend(*step);
            result
        }
        TermNode::Rule {
            pattern,
            replacement,
            condition,
            ..
        } => {
            let mut result = vec![*pattern, *replacement];
            result.extend(*condition);
            result
        }
        TermNode::Binder { bounds, body, .. } => {
            let mut result = bounds.clone();
            result.push(*body);
            result
        }
        TermNode::Held { expression } => vec![*expression],
    }
}

fn remap_node(node: &mut TermNode, rebuilt: &HashMap<TermId, TermId>) -> Result<(), TermError> {
    let remap = |id: &mut TermId| -> Result<(), TermError> {
        *id = rebuilt.get(id).copied().ok_or(TermError::UnknownTerm)?;
        Ok(())
    };
    match node {
        TermNode::Atom(_) => {}
        TermNode::Apply { head, arguments } => {
            remap(head)?;
            for child in arguments {
                remap(child)?;
            }
        }
        TermNode::Relation { left, right, .. } => {
            remap(left)?;
            remap(right)?;
        }
        TermNode::Boolean { arguments, .. }
        | TermNode::Collection {
            elements: arguments,
            ..
        } => {
            for child in arguments {
                remap(child)?;
            }
        }
        TermNode::Condition {
            expression,
            condition,
        } => {
            remap(expression)?;
            remap(condition)?;
        }
        TermNode::Piecewise { cases, otherwise } => {
            for case in cases {
                remap(&mut case.value)?;
                remap(&mut case.condition)?;
            }
            if let Some(otherwise) = otherwise {
                remap(otherwise)?;
            }
        }
        TermNode::OrderedMap { entries } => {
            for (key, value) in entries {
                remap(key)?;
                remap(value)?;
            }
        }
        TermNode::Index { target, indices } => {
            remap(target)?;
            for index in indices {
                remap(index)?;
            }
        }
        TermNode::Slice {
            target,
            start,
            end,
            step,
        } => {
            remap(target)?;
            for child in [start, end, step].into_iter().flatten() {
                remap(child)?;
            }
        }
        TermNode::Rule {
            pattern,
            replacement,
            condition,
            ..
        } => {
            remap(pattern)?;
            remap(replacement)?;
            if let Some(condition) = condition {
                remap(condition)?;
            }
        }
        TermNode::Binder { bounds, body, .. } => {
            for bound in bounds {
                remap(bound)?;
            }
            remap(body)?;
        }
        TermNode::Held { expression } => remap(expression)?,
    }
    Ok(())
}

fn exceeded(resource: &'static str, limit: usize) -> TermError {
    TermError::BudgetExceeded { resource, limit }
}

fn encode_node(
    node: &StoredNode,
    ids: &HashMap<TermId, u64>,
    output: &mut Vec<u8>,
    budget: TermBudget,
) -> Result<(), TermError> {
    macro_rules! id {
        ($term:expr) => {
            *ids.get(&$term).ok_or(TermError::UnknownTerm)?
        };
    }
    match node {
        StoredNode::Atom(atom) => {
            push_u8(output, 0x01, budget)?;
            encode_atom(atom, output, budget)?;
        }
        StoredNode::Apply(head, arguments) => {
            push_u8(output, 0x10, budget)?;
            push_varint(output, id!(*head), budget)?;
            encode_ids(arguments, ids, output, budget)?;
        }
        StoredNode::Relation(operator, left, right) => {
            push_u8(output, 0x11, budget)?;
            push_u8(output, relation_tag(*operator), budget)?;
            push_varint(output, id!(*left), budget)?;
            push_varint(output, id!(*right), budget)?;
        }
        StoredNode::Boolean(operator, arguments) => {
            push_u8(output, 0x12, budget)?;
            push_u8(output, boolean_tag(*operator), budget)?;
            encode_ids(arguments, ids, output, budget)?;
        }
        StoredNode::Condition(expression, condition) => {
            push_u8(output, 0x13, budget)?;
            push_varint(output, id!(*expression), budget)?;
            push_varint(output, id!(*condition), budget)?;
        }
        StoredNode::Piecewise(cases, otherwise) => {
            push_u8(output, 0x14, budget)?;
            push_varint(output, cases.len() as u64, budget)?;
            for case in cases {
                push_varint(output, id!(case.value), budget)?;
                push_varint(output, id!(case.condition), budget)?;
            }
            encode_optional(*otherwise, ids, output, budget)?;
        }
        StoredNode::Collection(kind, elements) => {
            push_u8(output, 0x15, budget)?;
            match kind {
                CollectionKind::Tuple => push_u8(output, 0, budget)?,
                CollectionKind::List => push_u8(output, 1, budget)?,
                CollectionKind::Array { shape } => {
                    push_u8(output, 2, budget)?;
                    push_varint(output, shape.len() as u64, budget)?;
                    for extent in shape {
                        push_varint(output, *extent as u64, budget)?;
                    }
                }
            }
            encode_ids(elements, ids, output, budget)?;
        }
        StoredNode::OrderedMap(entries) => {
            push_u8(output, 0x16, budget)?;
            push_varint(output, entries.len() as u64, budget)?;
            for (key, value) in entries {
                push_varint(output, id!(*key), budget)?;
                push_varint(output, id!(*value), budget)?;
            }
        }
        StoredNode::Index(target, indices) => {
            push_u8(output, 0x17, budget)?;
            push_varint(output, id!(*target), budget)?;
            encode_ids(indices, ids, output, budget)?;
        }
        StoredNode::Slice(target, start, end, step) => {
            push_u8(output, 0x18, budget)?;
            push_varint(output, id!(*target), budget)?;
            encode_optional(*start, ids, output, budget)?;
            encode_optional(*end, ids, output, budget)?;
            encode_optional(*step, ids, output, budget)?;
        }
        StoredNode::Rule(kind, pattern, replacement, condition) => {
            push_u8(output, 0x19, budget)?;
            push_u8(output, rule_tag(*kind), budget)?;
            push_varint(output, id!(*pattern), budget)?;
            push_varint(output, id!(*replacement), budget)?;
            encode_optional(*condition, ids, output, budget)?;
        }
        StoredNode::Binder(kind, count, bounds, body) => {
            push_u8(output, 0x1a, budget)?;
            push_u8(output, binder_tag(*kind), budget)?;
            push_varint(output, u64::from(*count), budget)?;
            encode_ids(bounds, ids, output, budget)?;
            push_varint(output, id!(*body), budget)?;
        }
        StoredNode::Held(expression) => {
            push_u8(output, 0x1b, budget)?;
            push_varint(output, id!(*expression), budget)?;
        }
    }
    Ok(())
}

fn encode_atom(
    atom: &StoredAtom,
    output: &mut Vec<u8>,
    budget: TermBudget,
) -> Result<(), TermError> {
    match atom {
        StoredAtom::Integer(value) => {
            push_u8(output, 0x01, budget)?;
            push_blob(output, value.as_bytes(), budget)?;
        }
        StoredAtom::Rational(value) => {
            push_u8(output, 0x02, budget)?;
            push_blob(
                output,
                value.as_rbig().numerator().to_string().as_bytes(),
                budget,
            )?;
            push_blob(
                output,
                value.as_rbig().denominator().to_string().as_bytes(),
                budget,
            )?;
        }
        StoredAtom::ExactDecimal(value) => {
            push_u8(output, 0x03, budget)?;
            push_blob(output, value.coefficient.as_bytes(), budget)?;
            push_zigzag(output, value.scale, budget)?;
        }
        StoredAtom::ExactIeee754Bits(bits) => {
            push_u8(output, 0x04, budget)?;
            push_bytes(output, &bits.to_be_bytes(), budget)?;
        }
        StoredAtom::MachineFloatBits(bits) => {
            push_u8(output, 0x05, budget)?;
            push_bytes(output, &bits.to_be_bytes(), budget)?;
        }
        StoredAtom::PrecisionReal(value) => {
            push_u8(output, 0x06, budget)?;
            push_blob(output, value.significand.as_bytes(), budget)?;
            push_zigzag(output, value.exponent10, budget)?;
            push_varint(output, u64::from(value.precision_bits), budget)?;
        }
        StoredAtom::String(value) => {
            push_u8(output, 0x07, budget)?;
            push_blob(output, value.as_bytes(), budget)?;
        }
        StoredAtom::Bytes(value) => {
            push_u8(output, 0x08, budget)?;
            push_blob(output, value, budget)?;
        }
        StoredAtom::Symbol(value) => {
            push_u8(output, 0x09, budget)?;
            push_blob(output, value.namespace.as_bytes(), budget)?;
            push_blob(output, value.name.as_bytes(), budget)?;
        }
        StoredAtom::Boolean(value) => {
            push_u8(output, 0x0a, budget)?;
            push_u8(output, u8::from(*value), budget)?;
        }
        StoredAtom::Constant(value) => {
            push_u8(output, 0x0b, budget)?;
            push_u8(output, constant_tag(*value), budget)?;
        }
        StoredAtom::BoundVariable(value) => {
            push_u8(output, 0x0c, budget)?;
            push_varint(output, u64::from(*value), budget)?;
        }
    }
    Ok(())
}

fn decode_node(
    cursor: &mut Cursor<'_>,
    terms: &[TermId],
    budget: TermBudget,
) -> Result<TermNode, TermError> {
    let id = |cursor: &mut Cursor<'_>| -> Result<TermId, TermError> {
        let index = usize::try_from(cursor.varint()?)
            .map_err(|_| TermError::InvalidWire("child index overflow"))?;
        terms
            .get(index)
            .copied()
            .ok_or(TermError::InvalidWire("forward or invalid child reference"))
    };
    Ok(match cursor.u8()? {
        0x01 => TermNode::Atom(decode_atom(cursor, budget)?),
        0x10 => TermNode::Apply {
            head: id(cursor)?,
            arguments: decode_ids(cursor, terms, budget)?,
        },
        0x11 => TermNode::Relation {
            operator: decode_relation(cursor.u8()?)?,
            left: id(cursor)?,
            right: id(cursor)?,
        },
        0x12 => TermNode::Boolean {
            operator: decode_boolean(cursor.u8()?)?,
            arguments: decode_ids(cursor, terms, budget)?,
        },
        0x13 => TermNode::Condition {
            expression: id(cursor)?,
            condition: id(cursor)?,
        },
        0x14 => {
            let count = bounded_count(cursor, budget)?;
            let mut cases = Vec::with_capacity(count);
            for _ in 0..count {
                cases.push(PiecewiseCase {
                    value: id(cursor)?,
                    condition: id(cursor)?,
                });
            }
            TermNode::Piecewise {
                cases,
                otherwise: decode_optional(cursor, terms)?,
            }
        }
        0x15 => {
            let kind = match cursor.u8()? {
                0 => CollectionKind::Tuple,
                1 => CollectionKind::List,
                2 => {
                    let count = bounded_count(cursor, budget)?;
                    let mut shape = Vec::with_capacity(count);
                    for _ in 0..count {
                        shape.push(
                            usize::try_from(cursor.varint()?)
                                .map_err(|_| TermError::InvalidWire("array extent overflow"))?,
                        );
                    }
                    CollectionKind::Array { shape }
                }
                _ => return Err(TermError::InvalidWire("collection tag")),
            };
            TermNode::Collection {
                kind,
                elements: decode_ids(cursor, terms, budget)?,
            }
        }
        0x16 => {
            let count = bounded_count(cursor, budget)?;
            let mut entries = Vec::with_capacity(count);
            for _ in 0..count {
                entries.push((id(cursor)?, id(cursor)?));
            }
            TermNode::OrderedMap { entries }
        }
        0x17 => TermNode::Index {
            target: id(cursor)?,
            indices: decode_ids(cursor, terms, budget)?,
        },
        0x18 => TermNode::Slice {
            target: id(cursor)?,
            start: decode_optional(cursor, terms)?,
            end: decode_optional(cursor, terms)?,
            step: decode_optional(cursor, terms)?,
        },
        0x19 => TermNode::Rule {
            kind: decode_rule(cursor.u8()?)?,
            pattern: id(cursor)?,
            replacement: id(cursor)?,
            condition: decode_optional(cursor, terms)?,
        },
        0x1a => {
            let kind = decode_binder(cursor.u8()?)?;
            let variable_count = u32::try_from(cursor.varint()?)
                .map_err(|_| TermError::InvalidWire("binder arity overflow"))?;
            TermNode::Binder {
                kind,
                variable_count,
                bounds: decode_ids(cursor, terms, budget)?,
                body: id(cursor)?,
            }
        }
        0x1b => TermNode::Held {
            expression: id(cursor)?,
        },
        _ => return Err(TermError::InvalidWire("node tag")),
    })
}

fn decode_atom(cursor: &mut Cursor<'_>, budget: TermBudget) -> Result<Atom, TermError> {
    let string = |cursor: &mut Cursor<'_>| -> Result<String, TermError> {
        let bytes = cursor.blob(budget.max_atom_bytes)?;
        String::from_utf8(bytes.to_vec()).map_err(|_| TermError::InvalidWire("atom utf-8"))
    };
    Ok(match cursor.u8()? {
        0x01 => Atom::Integer(string(cursor)?),
        0x02 => {
            let numerator = IBig::from_str(&string(cursor)?)
                .map_err(|_| TermError::InvalidWire("rational numerator"))?;
            let denominator = UBig::from_str(&string(cursor)?)
                .map_err(|_| TermError::InvalidWire("rational denominator"))?;
            if denominator == UBig::ZERO {
                return Err(TermError::InvalidWire("zero rational denominator"));
            }
            Atom::Rational(Rational::from(RBig::from_parts(numerator, denominator)))
        }
        0x03 => {
            let coefficient = string(cursor)?;
            let scale = cursor.zigzag()?;
            Atom::ExactDecimal(ExactDecimal { coefficient, scale })
        }
        0x04 => Atom::ExactIeee754Bits(u64::from_be_bytes(cursor.array()?)),
        0x05 => Atom::MachineFloatBits(u64::from_be_bytes(cursor.array()?)),
        0x06 => {
            let significand = string(cursor)?;
            let exponent10 = cursor.zigzag()?;
            let precision_bits = u32::try_from(cursor.varint()?)
                .map_err(|_| TermError::InvalidWire("precision overflow"))?;
            Atom::PrecisionReal(PrecisionReal {
                significand,
                exponent10,
                precision_bits,
            })
        }
        0x07 => Atom::String(string(cursor)?),
        0x08 => Atom::Bytes(cursor.blob(budget.max_atom_bytes)?.to_vec()),
        0x09 => Atom::Symbol(SymbolName::new(string(cursor)?, string(cursor)?)),
        0x0a => Atom::Boolean(match cursor.u8()? {
            0 => false,
            1 => true,
            _ => return Err(TermError::InvalidWire("boolean value")),
        }),
        0x0b => Atom::Constant(decode_constant(cursor.u8()?)?),
        0x0c => Atom::BoundVariable(
            u32::try_from(cursor.varint()?)
                .map_err(|_| TermError::InvalidWire("bound-variable overflow"))?,
        ),
        _ => return Err(TermError::InvalidWire("atom tag")),
    })
}

fn encode_ids(
    values: &[TermId],
    ids: &HashMap<TermId, u64>,
    output: &mut Vec<u8>,
    budget: TermBudget,
) -> Result<(), TermError> {
    push_varint(output, values.len() as u64, budget)?;
    for value in values {
        push_varint(
            output,
            *ids.get(value).ok_or(TermError::UnknownTerm)?,
            budget,
        )?;
    }
    Ok(())
}

fn encode_optional(
    value: Option<TermId>,
    ids: &HashMap<TermId, u64>,
    output: &mut Vec<u8>,
    budget: TermBudget,
) -> Result<(), TermError> {
    match value {
        Some(value) => {
            push_u8(output, 1, budget)?;
            push_varint(
                output,
                *ids.get(&value).ok_or(TermError::UnknownTerm)?,
                budget,
            )?;
        }
        None => push_u8(output, 0, budget)?,
    }
    Ok(())
}

fn decode_ids(
    cursor: &mut Cursor<'_>,
    terms: &[TermId],
    budget: TermBudget,
) -> Result<Vec<TermId>, TermError> {
    let count = bounded_count(cursor, budget)?;
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        let index = usize::try_from(cursor.varint()?)
            .map_err(|_| TermError::InvalidWire("child index overflow"))?;
        result.push(
            terms
                .get(index)
                .copied()
                .ok_or(TermError::InvalidWire("forward or invalid child reference"))?,
        );
    }
    Ok(result)
}

fn decode_optional(cursor: &mut Cursor<'_>, terms: &[TermId]) -> Result<Option<TermId>, TermError> {
    match cursor.u8()? {
        0 => Ok(None),
        1 => {
            let index = usize::try_from(cursor.varint()?)
                .map_err(|_| TermError::InvalidWire("child index overflow"))?;
            Ok(Some(terms.get(index).copied().ok_or(
                TermError::InvalidWire("forward or invalid child reference"),
            )?))
        }
        _ => Err(TermError::InvalidWire("optional tag")),
    }
}

fn bounded_count(cursor: &mut Cursor<'_>, budget: TermBudget) -> Result<usize, TermError> {
    let count = usize::try_from(cursor.varint()?)
        .map_err(|_| TermError::InvalidWire("child count overflow"))?;
    if count > budget.max_children_per_node {
        return Err(exceeded("children", budget.max_children_per_node));
    }
    Ok(count)
}

fn push_u8(output: &mut Vec<u8>, value: u8, budget: TermBudget) -> Result<(), TermError> {
    push_bytes(output, &[value], budget)
}

fn push_bytes(output: &mut Vec<u8>, value: &[u8], budget: TermBudget) -> Result<(), TermError> {
    let length = output
        .len()
        .checked_add(value.len())
        .ok_or_else(|| exceeded("wire bytes", budget.max_wire_bytes))?;
    if length > budget.max_wire_bytes {
        return Err(exceeded("wire bytes", budget.max_wire_bytes));
    }
    output.extend_from_slice(value);
    Ok(())
}

fn push_blob(output: &mut Vec<u8>, value: &[u8], budget: TermBudget) -> Result<(), TermError> {
    push_varint(output, value.len() as u64, budget)?;
    push_bytes(output, value, budget)
}

fn push_varint(output: &mut Vec<u8>, mut value: u64, budget: TermBudget) -> Result<(), TermError> {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        push_u8(output, byte, budget)?;
        if value == 0 {
            return Ok(());
        }
    }
}

fn push_zigzag(output: &mut Vec<u8>, value: i64, budget: TermBudget) -> Result<(), TermError> {
    let encoded = ((value << 1) ^ (value >> 63)) as u64;
    push_varint(output, encoded, budget)
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], TermError> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or(TermError::InvalidWire("offset overflow"))?;
        let result = self
            .bytes
            .get(self.offset..end)
            .ok_or(TermError::InvalidWire("truncated input"))?;
        self.offset = end;
        Ok(result)
    }

    fn u8(&mut self) -> Result<u8, TermError> {
        Ok(self.take(1)?[0])
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], TermError> {
        self.take(N)?
            .try_into()
            .map_err(|_| TermError::InvalidWire("truncated fixed-width value"))
    }

    fn varint(&mut self) -> Result<u64, TermError> {
        let start = self.offset;
        let mut value = 0u64;
        for shift in (0..70).step_by(7) {
            let byte = self.u8()?;
            if shift == 63 && byte > 1 {
                return Err(TermError::InvalidWire("varint overflow"));
            }
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                let length = self.offset - start;
                if length > 1 && byte == 0 {
                    return Err(TermError::InvalidWire("non-canonical varint"));
                }
                return Ok(value);
            }
        }
        Err(TermError::InvalidWire("varint overflow"))
    }

    fn zigzag(&mut self) -> Result<i64, TermError> {
        let value = self.varint()?;
        Ok(((value >> 1) as i64) ^ (-((value & 1) as i64)))
    }

    fn blob(&mut self, limit: usize) -> Result<&'a [u8], TermError> {
        let length = usize::try_from(self.varint()?)
            .map_err(|_| TermError::InvalidWire("blob length overflow"))?;
        if length > limit {
            return Err(exceeded("atom bytes", limit));
        }
        self.take(length)
    }
}

fn relation_tag(value: RelationOperator) -> u8 {
    match value {
        RelationOperator::Equal => 0,
        RelationOperator::NotEqual => 1,
        RelationOperator::Less => 2,
        RelationOperator::LessEqual => 3,
        RelationOperator::Greater => 4,
        RelationOperator::GreaterEqual => 5,
    }
}
fn boolean_tag(value: BooleanOperator) -> u8 {
    match value {
        BooleanOperator::Not => 0,
        BooleanOperator::And => 1,
        BooleanOperator::Or => 2,
        BooleanOperator::Xor => 3,
        BooleanOperator::Implies => 4,
        BooleanOperator::Equivalent => 5,
    }
}
fn rule_tag(value: RuleKind) -> u8 {
    match value {
        RuleKind::Immediate => 0,
        RuleKind::Delayed => 1,
        RuleKind::Pattern => 2,
    }
}
fn binder_tag(value: BinderKind) -> u8 {
    match value {
        BinderKind::Lambda => 0,
        BinderKind::Sum => 1,
        BinderKind::Product => 2,
        BinderKind::Integral => 3,
        BinderKind::Limit => 4,
        BinderKind::Local => 5,
    }
}
fn constant_tag(value: SymbolicConstant) -> u8 {
    match value {
        SymbolicConstant::Pi => 0,
        SymbolicConstant::E => 1,
        SymbolicConstant::ImaginaryUnit => 2,
        SymbolicConstant::Infinity => 3,
        SymbolicConstant::ComplexInfinity => 4,
        SymbolicConstant::Undefined => 5,
    }
}

fn decode_relation(value: u8) -> Result<RelationOperator, TermError> {
    match value {
        0 => Ok(RelationOperator::Equal),
        1 => Ok(RelationOperator::NotEqual),
        2 => Ok(RelationOperator::Less),
        3 => Ok(RelationOperator::LessEqual),
        4 => Ok(RelationOperator::Greater),
        5 => Ok(RelationOperator::GreaterEqual),
        _ => Err(TermError::InvalidWire("relation tag")),
    }
}

fn decode_boolean(value: u8) -> Result<BooleanOperator, TermError> {
    match value {
        0 => Ok(BooleanOperator::Not),
        1 => Ok(BooleanOperator::And),
        2 => Ok(BooleanOperator::Or),
        3 => Ok(BooleanOperator::Xor),
        4 => Ok(BooleanOperator::Implies),
        5 => Ok(BooleanOperator::Equivalent),
        _ => Err(TermError::InvalidWire("boolean tag")),
    }
}

fn decode_rule(value: u8) -> Result<RuleKind, TermError> {
    match value {
        0 => Ok(RuleKind::Immediate),
        1 => Ok(RuleKind::Delayed),
        2 => Ok(RuleKind::Pattern),
        _ => Err(TermError::InvalidWire("rule tag")),
    }
}

fn decode_binder(value: u8) -> Result<BinderKind, TermError> {
    match value {
        0 => Ok(BinderKind::Lambda),
        1 => Ok(BinderKind::Sum),
        2 => Ok(BinderKind::Product),
        3 => Ok(BinderKind::Integral),
        4 => Ok(BinderKind::Limit),
        5 => Ok(BinderKind::Local),
        _ => Err(TermError::InvalidWire("binder tag")),
    }
}

fn decode_constant(value: u8) -> Result<SymbolicConstant, TermError> {
    match value {
        0 => Ok(SymbolicConstant::Pi),
        1 => Ok(SymbolicConstant::E),
        2 => Ok(SymbolicConstant::ImaginaryUnit),
        3 => Ok(SymbolicConstant::Infinity),
        4 => Ok(SymbolicConstant::ComplexInfinity),
        5 => Ok(SymbolicConstant::Undefined),
        _ => Err(TermError::InvalidWire("constant tag")),
    }
}
