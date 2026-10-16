//! Rung 3: exact expansions with fixed inline capacity.

use super::{MARGIN_DOWN, MAX_MAGNITUDE, MAX_PRODUCT_OPERAND, RungArith, within};
use crate::eft::two_sum;
use crate::exact::RingOps;
use crate::expansion::two_prod_dekker;
use crate::uncertain::Sign;

/// An exact sum of at most `N` nonoverlapping `f64` components (Shewchuk
/// expansion): increasing magnitude, zero-eliminated, so the value is the sum
/// of [`components`](Self::components) and the sign is the sign of the last.
///
/// The storage is inline. When an operation would need more than `N`
/// components, would overflow, or meets an operand beyond the rung's guard
/// thresholds ([`crate::rungs`]), the result is flagged
/// [`must_escalate`](Self::must_escalate): it holds no value, has no sign, and
/// the flag is absorbed by every later operation. An expansion that is not
/// flagged is exact.
#[derive(Clone, Copy, Debug)]
pub struct StackExpansion<const N: usize> {
    comps: [f64; N],
    len: usize,
    escalate: bool,
}

impl<const N: usize> StackExpansion<N> {
    fn empty() -> Self {
        StackExpansion {
            comps: [0.0; N],
            len: 0,
            escalate: false,
        }
    }

    fn poisoned() -> Self {
        StackExpansion {
            escalate: true,
            ..Self::empty()
        }
    }

    /// Append a component (largest last); zero is eliminated, and a full
    /// buffer flags the expansion.
    fn push(&mut self, c: f64) {
        if c == 0.0 {
            return;
        }
        if self.len == N {
            self.escalate = true;
        } else {
            self.comps[self.len] = c;
            self.len += 1;
        }
    }

    /// The expansion of one input. Flagged if `x` is non-finite or beyond
    /// [`MAX_MAGNITUDE`](super::MAX_MAGNITUDE) (or `N == 0` and `x != 0`).
    pub fn from_f64(x: f64) -> Self {
        if !within(x, MAX_MAGNITUDE) {
            return Self::poisoned();
        }
        let mut s = Self::empty();
        s.push(x);
        s
    }

    /// The components, increasing in magnitude, none zero. Meaningless (empty)
    /// when [`must_escalate`](Self::must_escalate).
    pub fn components(&self) -> &[f64] {
        &self.comps[..self.len]
    }

    /// `true` when this value could not be represented exactly; use a higher
    /// rung.
    pub fn must_escalate(&self) -> bool {
        self.escalate
    }

    /// The exact sign, or `None` when flagged.
    pub fn sign(&self) -> Option<Sign> {
        if self.escalate {
            return None;
        }
        // Nonoverlapping, increasing: the last component dominates.
        Some(match self.components().last() {
            None => Sign::Zero,
            Some(&c) if c > 0.0 => Sign::Positive,
            Some(_) => Sign::Negative,
        })
    }

    /// A certified lower bound on the exact `|value|`: `0.0` when flagged or
    /// zero.
    ///
    /// The components below the largest sum to less than twice the next
    /// largest in magnitude (they are nonoverlapping), so
    /// `|value| >= |top| - 2 |next|`.
    pub fn margin(&self) -> f64 {
        let n = self.len;
        if self.escalate || n == 0 {
            return 0.0;
        }
        let top = self.comps[n - 1].abs();
        if n == 1 {
            return top;
        }
        ((top - 2.0 * self.comps[n - 2].abs()) * MARGIN_DOWN).max(0.0)
    }

    /// Grow by one `f64`, exactly (Shewchuk GROW-EXPANSION).
    fn grow(&self, b: f64) -> Self {
        if self.escalate || !within(b, MAX_MAGNITUDE) {
            return Self::poisoned();
        }
        let mut out = Self::empty();
        let mut q = b;
        for &c in self.components() {
            let (s, h) = two_sum(q, c);
            if !within(s, MAX_MAGNITUDE) || !h.is_finite() {
                return Self::poisoned();
            }
            out.push(h);
            q = s;
        }
        out.push(q);
        if out.escalate { Self::poisoned() } else { out }
    }

    /// Multiply by one `f64`, exactly (Shewchuk SCALE-EXPANSION).
    fn scale(&self, b: f64) -> Self {
        if self.escalate
            || b.abs() > MAX_PRODUCT_OPERAND
            || self.top_magnitude() > MAX_PRODUCT_OPERAND
        {
            return Self::poisoned();
        }
        if self.len == 0 || b == 0.0 {
            return Self::empty();
        }
        let mut out = Self::empty();
        let Some((mut q, h)) = two_prod_dekker(self.comps[0], b) else {
            return Self::poisoned();
        };
        out.push(h);
        for &c in &self.components()[1..] {
            let Some((t, t0)) = two_prod_dekker(c, b) else {
                return Self::poisoned();
            };
            let (s, h1) = two_sum(q, t0);
            out.push(h1);
            let (q2, h2) = two_sum(t, s);
            out.push(h2);
            q = q2;
        }
        out.push(q);
        if out.escalate { Self::poisoned() } else { out }
    }

    fn top_magnitude(&self) -> f64 {
        if self.len == 0 {
            0.0
        } else {
            self.comps[self.len - 1].abs()
        }
    }
}

impl<const N: usize> RingOps for StackExpansion<N> {
    fn zero() -> Self {
        Self::empty()
    }

    fn from_i32(i: i32) -> Self {
        Self::from_f64(f64::from(i))
    }

    fn add(&self, rhs: &Self) -> Self {
        if self.escalate || rhs.escalate {
            return Self::poisoned();
        }
        let mut acc = *self;
        for &c in rhs.components() {
            acc = acc.grow(c);
            if acc.escalate {
                break;
            }
        }
        acc
    }

    fn sub(&self, rhs: &Self) -> Self {
        self.add(&rhs.neg())
    }

    fn mul(&self, rhs: &Self) -> Self {
        if self.escalate || rhs.escalate {
            return Self::poisoned();
        }
        // Scale the shorter expansion by each component of the longer.
        let (short, long) = if self.len <= rhs.len {
            (self, rhs)
        } else {
            (rhs, self)
        };
        let mut acc = Self::empty();
        for &b in long.components() {
            acc = acc.add(&short.scale(b));
            if acc.escalate {
                break;
            }
        }
        acc
    }

    fn neg(&self) -> Self {
        let mut out = *self;
        for c in &mut out.comps[..out.len] {
            *c = -*c;
        }
        out
    }
}

impl<const N: usize> RungArith for StackExpansion<N> {
    fn lift(x: f64) -> Self {
        Self::from_f64(x)
    }
}
