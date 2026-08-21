//! One-root numbers `a + b·√r` (DESIGN.md §3.4).
//!
//! `SqrtExt` is a **coordinate type for curve geometries** — deliberately
//! *not* an [`ExactRing`](crate::ExactRing)/[`Real`](crate::Real) backend: roots are runtime
//! values, so same-root arithmetic is a checked condition (`Option` returns
//! at construction time), never a deferred panic. Cross-root *comparison*
//! is total: repeated squaring reduces it to same-root signs, with no
//! separation bounds.

use crate::exact::{ExactField, RingOps};
use crate::interval::Interval;
use crate::uncertain::Sign;
use core::cmp::Ordering;

/// `a + b·√r` over the exact field `T`; `b == 0` encodes a plain field
/// element (any `r`). Invariant: `r ≥ 0`, and `b == 0` normalizes `r` to 0.
#[derive(Clone, Debug)]
pub struct SqrtExt<T> {
    a: T,
    b: T,
    r: T,
}

impl<T: ExactField> SqrtExt<T> {
    /// A rational (extension-free) value.
    pub fn rational(a: T) -> SqrtExt<T> {
        SqrtExt {
            a,
            b: T::zero(),
            r: T::zero(),
        }
    }

    /// `a + b·√r`. Panics if `r` is negative (caller bug: radicands come
    /// from discriminants already known non-negative).
    pub fn new(a: T, b: T, r: T) -> SqrtExt<T> {
        assert!(
            r.sign() != Sign::Negative,
            "SqrtExt::new: negative radicand"
        );
        if b.sign() == Sign::Zero || r.sign() == Sign::Zero {
            // Normalize: value is exactly `a`.
            SqrtExt::rational(a)
        } else {
            SqrtExt { a, b, r }
        }
    }

    /// Rational part.
    pub fn a(&self) -> &T {
        &self.a
    }

    /// Root coefficient.
    pub fn b(&self) -> &T {
        &self.b
    }

    /// Radicand (0 when the value is rational).
    pub fn root(&self) -> &T {
        &self.r
    }

    /// Is the value extension-free?
    pub fn is_rational(&self) -> bool {
        self.b.sign() == Sign::Zero
    }

    /// Do two values live in the same extension (same root, or either side
    /// rational)?
    pub fn same_root(&self, rhs: &SqrtExt<T>) -> bool {
        self.is_rational() || rhs.is_rational() || self.r == rhs.r
    }

    fn unified_root(&self, rhs: &SqrtExt<T>) -> T {
        if self.is_rational() {
            rhs.r.clone()
        } else {
            self.r.clone()
        }
    }

    /// Checked addition: `None` if the roots differ.
    pub fn checked_add(&self, rhs: &SqrtExt<T>) -> Option<SqrtExt<T>> {
        if !self.same_root(rhs) {
            return None;
        }
        let r = self.unified_root(rhs);
        Some(SqrtExt::new(self.a.add(&rhs.a), self.b.add(&rhs.b), r))
    }

    /// Checked subtraction.
    pub fn checked_sub(&self, rhs: &SqrtExt<T>) -> Option<SqrtExt<T>> {
        if !self.same_root(rhs) {
            return None;
        }
        let r = self.unified_root(rhs);
        Some(SqrtExt::new(self.a.sub(&rhs.a), self.b.sub(&rhs.b), r))
    }

    /// Checked multiplication:
    /// `(a1 + b1√r)(a2 + b2√r) = (a1a2 + b1b2·r) + (a1b2 + a2b1)√r`.
    pub fn checked_mul(&self, rhs: &SqrtExt<T>) -> Option<SqrtExt<T>> {
        if !self.same_root(rhs) {
            return None;
        }
        let r = self.unified_root(rhs);
        let a = self.a.mul(&rhs.a).add(&self.b.mul(&rhs.b).mul(&r));
        let b = self.a.mul(&rhs.b).add(&self.b.mul(&rhs.a));
        Some(SqrtExt::new(a, b, r))
    }

    /// Checked division (multiply by the conjugate). `None` if roots differ;
    /// panics on a zero divisor.
    pub fn checked_div(&self, rhs: &SqrtExt<T>) -> Option<SqrtExt<T>> {
        if !self.same_root(rhs) {
            return None;
        }
        assert!(rhs.sign() != Sign::Zero, "SqrtExt::checked_div by zero");
        let r = self.unified_root(rhs);
        // Conjugate norm: a2² − b2²·r.
        let norm = rhs.a.mul(&rhs.a).sub(&rhs.b.mul(&rhs.b).mul(&r));
        if norm.sign() == Sign::Zero {
            // Degenerate: √r is rational (= |a2/b2|), divisor is rational.
            // a2² = b2²r ⇒ √r = |a2|/|b2|; divisor = a2 + b2·√r = 2·a2
            // when sign(a2) == sign(b2·√r) — compute directly:
            let sqrt_r = abs(&rhs.a.div(&rhs.b));
            let divisor = rhs.a.add(&rhs.b.mul(&sqrt_r));
            // divisor nonzero because rhs ≠ 0.
            let self_val_a = self.a.div(&divisor);
            let self_val_b = self.b.div(&divisor);
            return Some(SqrtExt::new(self_val_a, self_val_b, r));
        }
        // self · conj(rhs) / norm. `conj` carries the root already unified
        // with `self`, so `checked_mul` cannot reject it — but `?` says that
        // without a panic, and this function already returns `Option`.
        let conj = SqrtExt::new(rhs.a.clone(), rhs.b.neg(), r.clone());
        let num = self.checked_mul(&conj)?;
        Some(SqrtExt::new(num.a.div(&norm), num.b.div(&norm), r))
    }

    /// Negation.
    #[must_use]
    pub fn neg_ext(&self) -> SqrtExt<T> {
        SqrtExt {
            a: self.a.neg(),
            b: self.b.neg(),
            r: self.r.clone(),
        }
    }

    /// Exact sign, by ring operations only (repeated squaring — no
    /// separation bounds needed).
    pub fn sign(&self) -> Sign {
        let sa = self.a.sign();
        let sb = self.b.sign();
        if sb == Sign::Zero {
            return sa;
        }
        if sa == Sign::Zero {
            return sb; // b√r has b's sign (r > 0 here by normalization)
        }
        if sa == sb {
            return sa;
        }
        // Opposite signs: |a| vs |b|√r ⇔ a² vs b²·r.
        let a2 = self.a.mul(&self.a);
        let b2r = self.b.mul(&self.b).mul(&self.r);
        match a2.cmp(&b2r) {
            Ordering::Greater => sa,
            Ordering::Less => sb,
            Ordering::Equal => Sign::Zero,
        }
    }

    /// Same-root comparison (panics on root mismatch — use
    /// [`SqrtExt::cmp_cross`] for the total cross-root order).
    ///
    /// # Panics
    /// **API-contract panic**, stated in the summary line: the two values
    /// must live in the same extension. [`SqrtExt::cmp_cross`] is the total
    /// form and is what callers with arbitrary roots should use;
    /// [`SqrtExt::same_root`] is the cheap precondition test.
    #[allow(clippy::expect_used)] // documented contract; `cmp_cross` is the total form
    pub fn cmp_same_root(&self, rhs: &SqrtExt<T>) -> Ordering {
        self.checked_sub(rhs)
            .expect("cmp_same_root: differing roots")
            .sign()
            .as_ordering()
    }

    /// Total comparison across arbitrary roots: sign of
    /// `(a1 − a2) + b1√r1 − b2√r2`, decided by comparing
    /// `L = (a1 − a2) + b1√r1` (one-root) against `R = b2√r2` via one
    /// squaring: `L²` is again one-root over `r1`, `R²` is rational.
    ///
    /// Neither `expect` below can fire, and both follow from
    /// [`SqrtExt::same_root`] alone (`self.is_rational() || rhs.is_rational()
    /// || self.r == rhs.r`): `l.checked_mul(&l)` compares `l`'s root with
    /// itself, and the `checked_sub` operand is built by
    /// [`SqrtExt::rational`], whose `is_rational()` is true by construction.
    /// Both are total operations wearing a checked signature because *other*
    /// callers pass unrelated roots.
    #[allow(clippy::expect_used)] // `x·x` and `x − rational` always unify — argued above
    pub fn cmp_cross(&self, rhs: &SqrtExt<T>) -> Ordering {
        if self.same_root(rhs) {
            return self.cmp_same_root(rhs);
        }
        let l = SqrtExt::new(self.a.sub(&rhs.a), self.b.clone(), self.r.clone());
        let sl = l.sign();
        let sr = rhs.b.sign(); // sign of b2√r2 (r2 > 0 since rhs not rational)
        match (sl, sr) {
            (Sign::Zero, Sign::Zero) => Ordering::Equal,
            (Sign::Zero, s) => s.negated().as_ordering(),
            (s, Sign::Zero) => s.as_ordering(),
            (sl, sr) if sl != sr => {
                if sl == Sign::Positive {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }
            (s, _) => {
                // Same nonzero sign: compare squares (both one-root-free
                // after squaring R; L² stays one-root over r1).
                let l2 = l.checked_mul(&l).expect("same root");
                let r2v = rhs.b.mul(&rhs.b).mul(&rhs.r); // rational
                let diff = l2
                    .checked_sub(&SqrtExt::rational(r2v))
                    .expect("rational rhs");
                match (diff.sign(), s) {
                    (Sign::Zero, _) => Ordering::Equal,
                    (Sign::Positive, Sign::Positive) => Ordering::Greater,
                    (Sign::Positive, _) => Ordering::Less,
                    (Sign::Negative, Sign::Positive) => Ordering::Less,
                    (Sign::Negative, _) => Ordering::Greater,
                }
            }
        }
    }

    /// Outward-correct interval enclosure.
    pub fn to_interval(&self) -> Interval {
        if self.is_rational() {
            return self.a.to_interval();
        }
        self.a
            .to_interval()
            .add(&self.b.to_interval().mul(&self.r.to_interval().sqrt()))
    }
}

fn abs<T: ExactField>(x: &T) -> T {
    if x.sign() == Sign::Negative {
        x.neg()
    } else {
        x.clone()
    }
}

impl<T: ExactField> PartialEq for SqrtExt<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp_cross(other) == Ordering::Equal
    }
}
impl<T: ExactField> Eq for SqrtExt<T> {}
impl<T: ExactField> PartialOrd for SqrtExt<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T: ExactField> Ord for SqrtExt<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_cross(other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::exact::Rational;

    type S = SqrtExt<Rational>;

    fn q(n: i64, d: i64) -> Rational {
        Rational::from_ratio(n, d)
    }

    fn sqrt_of(r: i64) -> S {
        S::new(q(0, 1), q(1, 1), q(r, 1))
    }

    #[test]
    fn signs() {
        // 1 - √2 < 0 ; 2 - √2 > 0 ; 2 - √4 == 0.
        assert_eq!(S::new(q(1, 1), q(-1, 1), q(2, 1)).sign(), Sign::Negative);
        assert_eq!(S::new(q(2, 1), q(-1, 1), q(2, 1)).sign(), Sign::Positive);
        assert_eq!(S::new(q(2, 1), q(-1, 1), q(4, 1)).sign(), Sign::Zero);
    }

    #[test]
    fn cross_root_compare() {
        // √2 < √3 ; 1 + √2 < √8 ; 1 + √2 > √5
        assert_eq!(sqrt_of(2).cmp_cross(&sqrt_of(3)), Ordering::Less);
        let one_plus_sqrt2 = S::new(q(1, 1), q(1, 1), q(2, 1));
        assert_eq!(one_plus_sqrt2.cmp_cross(&sqrt_of(8)), Ordering::Less);
        assert_eq!(one_plus_sqrt2.cmp_cross(&sqrt_of(5)), Ordering::Greater);
        // (1 + √2)² = 3 + 2√2 vs 3 + √8: equal (√8 = 2√2).
        let l = S::new(q(3, 1), q(2, 1), q(2, 1));
        let r = S::new(q(3, 1), q(1, 1), q(8, 1));
        assert_eq!(l.cmp_cross(&r), Ordering::Equal);
    }

    #[test]
    fn division_conjugate() {
        // (1 + √2) / (1 + √2) == 1.
        let x = S::new(q(1, 1), q(1, 1), q(2, 1));
        let one = x.checked_div(&x).unwrap();
        assert_eq!(one.cmp_cross(&S::rational(q(1, 1))), Ordering::Equal);
        // (3 + √4) / (1 + √4) = 5/3 — degenerate norm (√4 rational).
        let a = S::new(q(3, 1), q(1, 1), q(4, 1));
        let b = S::new(q(1, 1), q(1, 1), q(4, 1));
        let quot = a.checked_div(&b).unwrap();
        assert_eq!(quot.cmp_cross(&S::rational(q(5, 3))), Ordering::Equal);
    }

    #[test]
    fn mixed_roots_checked() {
        assert!(sqrt_of(2).checked_add(&sqrt_of(3)).is_none());
        assert!(sqrt_of(2).checked_add(&S::rational(q(1, 1))).is_some());
    }

    #[test]
    fn interval_encloses() {
        let x = S::new(q(1, 1), q(1, 1), q(2, 1)); // 1 + √2 ≈ 2.41421356
        let iv = x.to_interval();
        assert!(iv.contains(2.414_213_562_373_095));
        assert!(iv.sup() - iv.inf() < 1e-12);
    }
}
