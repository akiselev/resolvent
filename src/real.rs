//! `Real<E>`: the resolvent number (DESIGN.md §3.5).
//!
//! A `Real` is a cheaply-clonable handle to a DAG node holding an eagerly
//! computed [`Interval`] and a lazily computed exact value. Comparisons take
//! the identity shortcut, then the interval rung, then force exactness.
//!
//! Concurrency protocol (per node, "five steps"): check the exact cell;
//! lock the op and **re-check**; ensure operand exacts (iteratively, no lock
//! held across nodes); compute under this node's lock only (touching nothing
//! but already-set operand cells); publish, tighten the interval, prune.
//! At most one node lock is ever held at a time, so no waits-for cycle can
//! form even when threads force overlapping sub-DAGs.
//!
//! Evaluation and teardown are **iterative** — CGAL's recursive
//! `update_exact()`/destructor stack overflow on deep DAGs is designed out.

use crate::exact::{ExactField, ExactRing, Rational, RingOps};
use crate::interval::{AtomicInterval, Interval};
use crate::uncertain::{Sign, UOrd, USign, Uncertain};
use crate::{AlgebraBudget, AlgebraError};
use core::cmp::Ordering;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

/// The exact value of a child whose exact cell is already set.
///
/// `force` is two-phase: a node is only revisited with `expanded == true`
/// after every child with an unset exact cell has been pushed *above* it on
/// the LIFO stack and evaluated. So by the time any `Op` is evaluated, every
/// operand's `OnceLock` is populated, and `exact_ref` cannot be `None`.
/// Nothing ever clears an exact cell — `OnceLock` has no such operation — so
/// the property is stable once established.
#[allow(clippy::expect_used)] // invariant established by `force`'s two phases
fn child_exact<E: ExactField>(r: &Real<E>) -> &E {
    r.exact_ref()
        .expect("force() evaluates every child before its parent")
}

/// Lock a node's op cell, recovering rather than propagating poison.
///
/// The guarded datum is an `Option<Op<E>>` that is only ever replaced by
/// `None` *after* the exact cell has been published, so a panic inside
/// `Op::eval` (user `Formula` code, or `ExactField::div` by zero) leaves it
/// untouched and internally consistent. Recovering keeps one poisoned
/// evaluation from bringing down every later reader of the same DAG.
fn lock_op<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// A formula body written once, generically over the number type — the
/// single source of truth for both the interval and the exact instantiation
/// (DESIGN.md §3.5). Implementors are plain structs whose fields are the
/// formula's captured constants.
pub trait Formula: Send + Sync + 'static {
    /// Evaluate over borrowed operands.
    fn apply<T: RingOps>(&self, args: &[&T]) -> T;
}

/// Multi-output formula: a construction with several output coordinates
/// computed in ONE exact pass (a lazy geometric object, à la CGAL's
/// `Lazy_rep_n`).
pub trait TupleFormula: Send + Sync + 'static {
    /// Number of outputs.
    fn arity(&self) -> usize;
    /// Evaluate all outputs over borrowed operands.
    fn apply<T: RingOps>(&self, args: &[&T], out: &mut Vec<T>);
}

/// Object-safe erasure of [`Formula`] — blanket-implemented, so the two
/// instantiations can never diverge.
trait ErasedFormula<E>: Send + Sync {
    fn eval_exact(&self, args: &[&E]) -> E;
    fn eval_interval(&self, args: &[&Interval]) -> Interval;
}

impl<E: ExactRing, F: Formula> ErasedFormula<E> for F {
    fn eval_exact(&self, args: &[&E]) -> E {
        self.apply::<E>(args)
    }
    fn eval_interval(&self, args: &[&Interval]) -> Interval {
        self.apply::<Interval>(args)
    }
}

trait ErasedTupleFormula<E>: Send + Sync {
    fn arity(&self) -> usize;
    fn eval_exact(&self, args: &[&E], out: &mut Vec<E>);
    fn eval_interval(&self, args: &[&Interval], out: &mut Vec<Interval>);
}

impl<E: ExactRing, F: TupleFormula> ErasedTupleFormula<E> for F {
    fn arity(&self) -> usize {
        TupleFormula::arity(self)
    }
    fn eval_exact(&self, args: &[&E], out: &mut Vec<E>) {
        self.apply::<E>(args, out);
    }
    fn eval_interval(&self, args: &[&Interval], out: &mut Vec<Interval>) {
        self.apply::<Interval>(args, out);
    }
}

/// The operation recipe of a node; dropped after exact evaluation (pruning).
enum Op<E: ExactField> {
    Neg(Real<E>),
    Add(Real<E>, Real<E>),
    Sub(Real<E>, Real<E>),
    Mul(Real<E>, Real<E>),
    Div(Real<E>, Real<E>),
    Min(Real<E>, Real<E>),
    Max(Real<E>, Real<E>),
    Expr {
        operands: Box<[Real<E>]>,
        f: Box<dyn ErasedFormula<E>>,
    },
}

impl<E: ExactField> Op<E> {
    fn children(&self) -> Vec<Real<E>> {
        match self {
            Op::Neg(a) => vec![a.clone()],
            Op::Add(a, b)
            | Op::Sub(a, b)
            | Op::Mul(a, b)
            | Op::Div(a, b)
            | Op::Min(a, b)
            | Op::Max(a, b) => vec![a.clone(), b.clone()],
            Op::Expr { operands, .. } => operands.to_vec(),
        }
    }

    fn into_children(self) -> Vec<Real<E>> {
        match self {
            Op::Neg(a) => vec![a],
            Op::Add(a, b)
            | Op::Sub(a, b)
            | Op::Mul(a, b)
            | Op::Div(a, b)
            | Op::Min(a, b)
            | Op::Max(a, b) => vec![a, b],
            Op::Expr { operands, .. } => operands.into_vec(),
        }
    }

    /// Evaluate exactly. Precondition: every child's exact cell is set.
    fn eval(&self) -> E {
        let x = child_exact::<E>;
        match self {
            Op::Neg(a) => x(a).neg(),
            Op::Add(a, b) => x(a).add(x(b)),
            Op::Sub(a, b) => x(a).sub(x(b)),
            Op::Mul(a, b) => x(a).mul(x(b)),
            Op::Div(a, b) => x(a).div(x(b)),
            Op::Min(a, b) => x(a).min(x(b)).clone(),
            Op::Max(a, b) => x(a).max(x(b)).clone(),
            Op::Expr { operands, f } => {
                let args: Vec<&E> = operands.iter().map(x).collect();
                f.eval_exact(&args)
            }
        }
    }
}

struct Node<E: ExactField> {
    approx: AtomicInterval,
    exact: OnceLock<E>,
    op: Mutex<Option<Op<E>>>,
}

struct TupleOp<E: ExactField> {
    operands: Box<[Real<E>]>,
    f: Box<dyn ErasedTupleFormula<E>>,
}

struct TupleNode<E: ExactField> {
    approx: Box<[AtomicInterval]>,
    exact: OnceLock<Box<[E]>>,
    op: Mutex<Option<TupleOp<E>>>,
}

/// Iterative teardown: when the last handle to a node dies, its children are
/// unwrapped into a worklist instead of recursing (deep chains must not
/// overflow the stack on drop).
impl<E: ExactField> Drop for Node<E> {
    fn drop(&mut self) {
        let mut work: Vec<Real<E>> = Vec::new();
        if let Ok(Some(op)) = self.op.get_mut().map(Option::take) {
            work.extend(op.into_children());
        }
        drain_children(&mut work);
    }
}

impl<E: ExactField> Drop for TupleNode<E> {
    fn drop(&mut self) {
        let mut work: Vec<Real<E>> = Vec::new();
        if let Ok(Some(op)) = self.op.get_mut().map(Option::take) {
            work.extend(op.operands.into_vec());
        }
        drain_children(&mut work);
    }
}

fn drain_children<E: ExactField>(work: &mut Vec<Real<E>>) {
    while let Some(child) = work.pop() {
        match child.0 {
            Repr::Scalar(arc) => {
                if let Some(mut node) = Arc::into_inner(arc) {
                    // We own the node: harvest its children so ITS drop
                    // finds an empty op and cannot recurse.
                    if let Ok(Some(op)) = node.op.get_mut().map(Option::take) {
                        work.extend(op.into_children());
                    }
                }
            }
            Repr::Slot(arc, _) => {
                if let Some(mut node) = Arc::into_inner(arc)
                    && let Ok(Some(op)) = node.op.get_mut().map(Option::take)
                {
                    work.extend(op.operands.into_vec());
                }
            }
        }
    }
}

enum Repr<E: ExactField> {
    Scalar(Arc<Node<E>>),
    Slot(Arc<TupleNode<E>>, u32),
}

impl<E: ExactField> Clone for Repr<E> {
    fn clone(&self) -> Self {
        match self {
            Repr::Scalar(a) => Repr::Scalar(a.clone()),
            Repr::Slot(a, i) => Repr::Slot(a.clone(), *i),
        }
    }
}

/// A resolvent real number over the exact field `E` (DESIGN.md §3.5).
pub struct Real<E: ExactField = Rational>(Repr<E>);

impl<E: ExactField> Clone for Real<E> {
    fn clone(&self) -> Self {
        Real(self.0.clone())
    }
}

impl<E: ExactField> Real<E> {
    /// Leaf from a finite double (`None` on NaN/±∞ — the crate's fallible
    /// float ingress boundary).
    pub fn from_f64(x: f64) -> Option<Real<E>> {
        let e = E::from_f64(x)?;
        Some(Real::from_exact_with_approx(e, Interval::point(x)))
    }

    /// Leaf from an exact value.
    pub fn from_exact(e: E) -> Real<E> {
        let iv = e.to_interval();
        Real::from_exact_with_approx(e, iv)
    }

    fn from_exact_with_approx(e: E, iv: Interval) -> Real<E> {
        Real(Repr::Scalar(Arc::new(Node {
            approx: AtomicInterval::new(iv),
            // Born exact: `OnceLock::from` fills the cell at construction, so
            // there is no "already set" case to handle.
            exact: OnceLock::from(e),
            op: Mutex::new(None),
        })))
    }

    fn from_op(iv: Interval, op: Op<E>) -> Real<E> {
        Real(Repr::Scalar(Arc::new(Node {
            approx: AtomicInterval::new(iv),
            exact: OnceLock::new(),
            op: Mutex::new(Some(op)),
        })))
    }

    /// Whole-formula node: ONE node for an arbitrary expression over the
    /// operands (coarse nodes, not one per `+`).
    pub fn expr(f: impl Formula, operands: &[Real<E>]) -> Real<E> {
        let f: Box<dyn ErasedFormula<E>> = Box::new(f);
        let ivs: Vec<Interval> = operands.iter().map(|r| r.approx()).collect();
        let refs: Vec<&Interval> = ivs.iter().collect();
        let iv = f.eval_interval(&refs);
        Real::from_op(
            iv,
            Op::Expr {
                operands: operands.to_vec().into_boxed_slice(),
                f,
            },
        )
    }

    /// Multi-output construction: all coordinates computed in one exact
    /// pass; the returned `Real`s are (node, slot) projections sharing one
    /// exact cell.
    pub fn construct(f: impl TupleFormula, operands: &[Real<E>]) -> Vec<Real<E>> {
        let ivs: Vec<Interval> = operands.iter().map(|r| r.approx()).collect();
        let refs: Vec<&Interval> = ivs.iter().collect();
        let mut out = Vec::with_capacity(f.arity());
        ErasedTupleFormula::<E>::eval_interval(&f, &refs, &mut out);
        let approx: Box<[AtomicInterval]> = out.iter().map(|iv| AtomicInterval::new(*iv)).collect();
        let arity = approx.len();
        let node = Arc::new(TupleNode {
            approx,
            exact: OnceLock::new(),
            op: Mutex::new(Some(TupleOp {
                operands: operands.to_vec().into_boxed_slice(),
                f: Box::new(f),
            })),
        });
        (0..arity as u32)
            .map(|i| Real(Repr::Slot(node.clone(), i)))
            .collect()
    }

    /// Filtered sign of a formula over operands, allocating **no** node when
    /// the interval rung already decides.
    pub fn sign_of(f: &impl Formula, operands: &[Real<E>]) -> Sign {
        let ivs: Vec<Interval> = operands.iter().map(|r| r.approx()).collect();
        let refs: Vec<&Interval> = ivs.iter().collect();
        if let Uncertain::Certain(s) = f.apply::<Interval>(&refs).sign() {
            return s;
        }
        let exacts: Vec<&E> = operands.iter().map(|r| r.exact()).collect();
        f.apply::<E>(&exacts).sign()
    }

    /// Current interval enclosure (never forces exactness).
    pub fn approx(&self) -> Interval {
        match &self.0 {
            Repr::Scalar(n) => n.approx.load(),
            Repr::Slot(n, i) => n.approx[*i as usize].load(),
        }
    }

    /// Midpoint of the enclosure — documented-lossy.
    pub fn to_f64_lossy(&self) -> f64 {
        self.approx().midpoint()
    }

    /// Filtered sign (never forces exactness).
    pub fn try_sign_filtered(&self) -> USign {
        self.approx().sign()
    }

    /// Exact sign (forces exactness only if the filter cannot decide).
    pub fn sign(&self) -> Sign {
        self.try_sign_filtered().certain_or(|| self.exact().sign())
    }

    fn exact_ref(&self) -> Option<&E> {
        match &self.0 {
            Repr::Scalar(n) => n.exact.get(),
            Repr::Slot(n, i) => n.exact.get().map(|xs| &xs[*i as usize]),
        }
    }

    /// The exact value, computed (iteratively) on first use and memoized.
    ///
    /// `force` returns only once `self`'s own exact cell is set: its stack is
    /// seeded with `self`, and the loop pops a node without setting its exact
    /// cell only when that cell is *already* set (the `continue` arms). So the
    /// second `exact_ref` is `Some` on every path, and `OnceLock` never
    /// un-sets what it published.
    #[allow(clippy::expect_used)] // `force` post-condition, argued above
    pub fn exact(&self) -> &E {
        if self.exact_ref().is_none() {
            self.force(usize::MAX)
                .expect("unbounded lazy forcing cannot exhaust its node budget");
        }
        self.exact_ref()
            .expect("force() sets the exact cell of its root")
    }

    /// Compute the exact value while bounding the number of lazy nodes forced.
    ///
    /// Work completed before exhaustion remains safely memoized; retrying with
    /// a larger budget continues from that state without changing the value.
    pub fn exact_with_budget(&self, budget: AlgebraBudget) -> Result<&E, AlgebraError> {
        if self.exact_ref().is_none() {
            self.force(budget.max_lazy_nodes)?;
        }
        self.exact_ref().ok_or(AlgebraError::BudgetExceeded {
            operation: "forcing a lazy exact DAG",
            limit: budget.max_lazy_nodes,
        })
    }

    /// Iterative DAG evaluation (explicit stack; one node lock at a time).
    fn force(&self, limit: usize) -> Result<(), AlgebraError> {
        enum Task<E: ExactField> {
            Node(Real<E>, bool), // (handle, expanded)
        }
        let mut stack = vec![Task::Node(self.clone(), false)];
        let mut forced = 0usize;
        while let Some(Task::Node(r, expanded)) = stack.pop() {
            if r.exact_ref().is_some() {
                continue;
            }
            if !expanded {
                // Phase 1: snapshot children (lock, clone handles, unlock).
                let children: Vec<Real<E>> = match &r.0 {
                    Repr::Scalar(n) => {
                        let g = lock_op(&n.op);
                        if n.exact.get().is_some() {
                            continue; // raced: another thread finished
                        }
                        match g.as_ref() {
                            Some(op) => op.children(),
                            // op pruned but exact unset can only mean a
                            // leaf constructed exact-at-birth: handled by
                            // the exact_ref check above.
                            None => continue,
                        }
                    }
                    Repr::Slot(n, _) => {
                        let g = lock_op(&n.op);
                        if n.exact.get().is_some() {
                            continue;
                        }
                        match g.as_ref() {
                            Some(op) => op.operands.to_vec(),
                            None => continue,
                        }
                    }
                };
                stack.push(Task::Node(r.clone(), true));
                for c in children {
                    if c.exact_ref().is_none() {
                        stack.push(Task::Node(c, false));
                    }
                }
            } else {
                if forced >= limit {
                    return Err(AlgebraError::BudgetExceeded {
                        operation: "forcing a lazy exact DAG",
                        limit,
                    });
                }
                forced += 1;
                // Phase 2: children exacts are set; compute under this
                // node's lock only (reads only already-set OnceLocks).
                //
                // The `else { continue }` arms are the same case Phase 1
                // already skips: the op is pruned only *after* the exact cell
                // is published, and both happen under this lock, so an unset
                // exact cell implies an unpruned op. Skipping rather than
                // panicking keeps the two phases spelled the same way.
                match &r.0 {
                    Repr::Scalar(n) => {
                        let mut g = lock_op(&n.op);
                        if n.exact.get().is_some() {
                            continue; // raced
                        }
                        let Some(op) = g.as_ref() else { continue };
                        let e = op.eval();
                        n.approx.refine(e.to_interval());
                        // Sole setter: we hold the lock and re-checked `get()`
                        // above, so the closure is the one that runs.
                        n.exact.get_or_init(|| e);
                        *g = None; // prune
                    }
                    Repr::Slot(n, _) => {
                        let mut g = lock_op(&n.op);
                        if n.exact.get().is_some() {
                            continue;
                        }
                        let Some(op) = g.as_ref() else { continue };
                        let args: Vec<&E> = op.operands.iter().map(child_exact).collect();
                        let mut out = Vec::with_capacity(op.f.arity());
                        op.f.eval_exact(&args, &mut out);
                        for (slot, e) in out.iter().enumerate() {
                            n.approx[slot].refine(e.to_interval());
                        }
                        n.exact.get_or_init(|| out.into_boxed_slice());
                        *g = None; // prune
                    }
                }
            }
        }
        Ok(())
    }

    /// Identity: same node (and slot). `x.cmp(x)` never forces exactness.
    pub fn same_handle(&self, other: &Real<E>) -> bool {
        match (&self.0, &other.0) {
            (Repr::Scalar(a), Repr::Scalar(b)) => Arc::ptr_eq(a, b),
            (Repr::Slot(a, i), Repr::Slot(b, j)) => Arc::ptr_eq(a, b) && i == j,
            _ => false,
        }
    }

    /// Certified comparison without forcing exactness.
    pub fn try_cmp_filtered(&self, other: &Real<E>) -> UOrd {
        if self.same_handle(other) {
            return Uncertain::Certain(Ordering::Equal);
        }
        self.approx().cmp_interval(other.approx())
    }

    /// Total comparison: identity → interval → exact.
    pub fn cmp_real(&self, other: &Real<E>) -> Ordering {
        self.try_cmp_filtered(other)
            .certain_or(|| self.exact().cmp(other.exact()))
    }

    /// Lazy minimum (stays lazy when the filter cannot order the operands).
    #[must_use]
    pub fn min_real(&self, other: &Real<E>) -> Real<E> {
        match self.try_cmp_filtered(other) {
            Uncertain::Certain(Ordering::Less | Ordering::Equal) => self.clone(),
            Uncertain::Certain(Ordering::Greater) => other.clone(),
            Uncertain::Unknown => Real::from_op(
                self.approx().min_interval(other.approx()),
                Op::Min(self.clone(), other.clone()),
            ),
        }
    }

    /// Lazy absolute value: `max(self, -self)` — stays lazy when the filter
    /// cannot order `self` against `-self`, and is exact-closed (`|q|` is
    /// rational for rational `q`).
    #[must_use]
    pub fn abs(&self) -> Real<E> {
        self.max_real(&(-self))
    }

    /// Lazy maximum.
    #[must_use]
    pub fn max_real(&self, other: &Real<E>) -> Real<E> {
        match self.try_cmp_filtered(other) {
            Uncertain::Certain(Ordering::Less | Ordering::Equal) => other.clone(),
            Uncertain::Certain(Ordering::Greater) => self.clone(),
            Uncertain::Unknown => Real::from_op(
                self.approx().max_interval(other.approx()),
                Op::Max(self.clone(), other.clone()),
            ),
        }
    }
}

macro_rules! real_binop {
    ($trait:ident, $method:ident, $op:ident, $ivop:tt) => {
        impl<E: ExactField> core::ops::$trait for &Real<E> {
            type Output = Real<E>;
            fn $method(self, rhs: &Real<E>) -> Real<E> {
                Real::from_op(
                    self.approx() $ivop rhs.approx(),
                    Op::$op(self.clone(), rhs.clone()),
                )
            }
        }
        impl<E: ExactField> core::ops::$trait for Real<E> {
            type Output = Real<E>;
            fn $method(self, rhs: Real<E>) -> Real<E> {
                (&self).$method(&rhs)
            }
        }
    };
}

real_binop!(Add, add, Add, +);
real_binop!(Sub, sub, Sub, -);
real_binop!(Mul, mul, Mul, *);
real_binop!(Div, div, Div, /);

impl<E: ExactField> core::ops::Neg for &Real<E> {
    type Output = Real<E>;
    fn neg(self) -> Real<E> {
        Real::from_op(-self.approx(), Op::Neg(self.clone()))
    }
}

impl<E: ExactField> core::ops::Neg for Real<E> {
    type Output = Real<E>;
    fn neg(self) -> Real<E> {
        -&self
    }
}

impl<E: ExactField> PartialEq for Real<E> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp_real(other) == Ordering::Equal
    }
}
impl<E: ExactField> Eq for Real<E> {}
impl<E: ExactField> PartialOrd for Real<E> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<E: ExactField> Ord for Real<E> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_real(other)
    }
}

impl<E: ExactField> core::fmt::Debug for Real<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Real({:?})", self.approx())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exact::Rational;

    type R = Real<Rational>;

    fn r(x: f64) -> R {
        R::from_f64(x).unwrap()
    }

    #[test]
    fn basic_arithmetic_and_sign() {
        let x = &(&r(0.1) + &r(0.2)) - &r(0.3);
        // Filter cannot decide (result within an ulp of zero) — exact must.
        assert_eq!(x.try_sign_filtered(), Uncertain::Unknown);
        // Exact: fl(0.1)+fl(0.2)-fl(0.3) ≠ 0 as rationals.
        assert_ne!(x.sign(), Sign::Zero);
    }

    #[test]
    fn exact_zero_detected() {
        let x = &(&r(1.5) + &r(2.25)) - &r(3.75); // all exact doubles
        assert_eq!(x.sign(), Sign::Zero);
    }

    #[test]
    fn identity_shortcut_no_forcing() {
        let x = &r(0.1) / &r(0.3);
        assert_eq!(x.cmp_real(&x), Ordering::Equal);
        // Not forced: op still present (peek through repr).
        match &x.0 {
            Repr::Scalar(n) => assert!(n.exact.get().is_none()),
            _ => unreachable!(),
        }
    }

    #[test]
    fn deep_chain_eval_and_drop() {
        // 200k-node chain: recursive eval or drop would overflow the stack.
        let mut x = r(1.0);
        for _ in 0..200_000 {
            x = &x + &r(1.0);
        }
        assert_eq!(x.sign(), Sign::Positive);
        assert_eq!(x.exact().clone(), crate::exact::Rational::from_i64(200_001));
        drop(x); // iterative teardown
    }

    #[test]
    fn deep_chain_drop_unevaluated() {
        let mut x = r(1.0);
        for _ in 0..200_000 {
            x = &x * &r(1.0);
        }
        drop(x); // must not overflow the stack
    }

    #[test]
    fn approx_tightens_after_exact() {
        let x = &r(1.0) / &r(3.0);
        let before = x.approx();
        let _ = x.exact();
        let after = x.approx();
        assert!(after.sup() - after.inf() <= before.sup() - before.inf());
        assert!(after.sup() - after.inf() <= f64::EPSILON);
    }

    #[test]
    fn lazy_min_stays_lazy_on_overlap() {
        let a = &r(1.0) / &r(3.0);
        let b = &(&r(1.0) / &r(3.0)) + &(&r(1e-40) * &r(1.0));
        let m = a.min_real(&b);
        // Overlapping intervals: node created; exact resolves correctly.
        assert_eq!(m.cmp_real(&a), Ordering::Equal);
    }

    struct Det2;
    impl Formula for Det2 {
        fn apply<T: RingOps>(&self, v: &[&T]) -> T {
            v[0].mul(v[3]).sub(&v[1].mul(v[2]))
        }
    }

    #[test]
    fn expr_node_and_sign_of() {
        let ops = [r(2.0), r(6.0), r(1.0), r(3.0)];
        assert_eq!(R::sign_of(&Det2, &ops), Sign::Zero);
        let x = R::expr(Det2, &ops);
        assert_eq!(x.sign(), Sign::Zero);
    }

    struct MidAndSum;
    impl TupleFormula for MidAndSum {
        fn arity(&self) -> usize {
            2
        }
        fn apply<T: RingOps>(&self, v: &[&T], out: &mut Vec<T>) {
            out.push(v[0].add(v[1]));
            out.push(v[0].sub(v[1]));
        }
    }

    #[test]
    fn tuple_construction_shares_one_pass() {
        let outs = R::construct(MidAndSum, &[r(0.1), r(0.2)]);
        assert_eq!(outs.len(), 2);
        assert_eq!(outs[0].sign(), Sign::Positive);
        assert_eq!(outs[1].sign(), Sign::Negative);
        // Same underlying node.
        match (&outs[0].0, &outs[1].0) {
            (Repr::Slot(a, 0), Repr::Slot(b, 1)) => assert!(Arc::ptr_eq(a, b)),
            _ => unreachable!(),
        }
    }

    #[test]
    fn threads_force_overlapping_dags() {
        // Both threads FORCE exactness (`exact()`, not the filtered path) so
        // the shared sub-DAG is evaluated concurrently — the interleaving
        // named in M1's exit criteria.
        let shared = &r(0.1) / &r(0.3);
        let a = &shared + &r(1.0);
        let b = &shared - &r(1.0);
        std::thread::scope(|s| {
            s.spawn(|| assert_eq!(a.exact().sign(), Sign::Positive));
            s.spawn(|| assert_eq!(b.exact().sign(), Sign::Negative));
        });
        // The shared node was forced exactly once and memoized.
        assert!(shared.exact_ref().is_some());
        let expected = Rational::from_f64(0.1)
            .unwrap()
            .div(&Rational::from_f64(0.3).unwrap());
        assert_eq!(shared.exact_ref().unwrap(), &expected);
    }
}
