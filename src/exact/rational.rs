//! Exact rationals over dashu's `RBig` (DESIGN.md §3.3).

use crate::exact::{ExactField, ExactRing, RingOps};
use crate::interval::Interval;
use crate::uncertain::Sign;
use dashu::base::Sign as DSign;
use dashu::integer::{IBig, UBig};
use dashu::rational::RBig;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// An exact arbitrary-precision rational number.
///
/// The default exact backend of the crate: a field (constructions divide),
/// with the outward-correct [`ExactRing::to_interval`] bridge implemented by
/// exact comparison rather than trusting any float-conversion rounding mode.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rational(RBig);

impl Rational {
    /// Zero.
    pub fn zero() -> Rational {
        Rational(RBig::ZERO)
    }

    /// One.
    pub fn one() -> Rational {
        Rational(RBig::ONE)
    }

    /// Whether this value is exactly zero.
    pub fn is_zero(&self) -> bool {
        self.sign() == Sign::Zero
    }

    /// Whether this value is exactly one.
    pub fn is_one(&self) -> bool {
        self == &Rational::one()
    }

    /// Exact value of a finite `f64`; `None` on NaN/±∞.
    ///
    /// Implemented by exact mantissa/exponent decomposition — every finite
    /// double is a dyadic rational.
    pub fn from_f64(x: f64) -> Option<Rational> {
        if !x.is_finite() {
            return None;
        }
        // Exact: RBig::try_from(f64) is dashu's exact conversion.
        RBig::try_from(x).ok().map(Rational)
    }

    /// From an integer.
    pub fn from_i64(i: i64) -> Rational {
        Rational(RBig::from(i))
    }

    /// From a numerator/denominator pair. Panics if `den == 0`.
    pub fn from_ratio(num: i64, den: i64) -> Rational {
        Self::try_from_ratio(num, den).expect("Rational::from_ratio: zero denominator")
    }

    /// From a numerator/denominator pair, or `None` when `den == 0`.
    pub fn try_from_ratio(num: i64, den: i64) -> Option<Rational> {
        (den != 0).then(|| Rational(RBig::from(num) / RBig::from(den)))
    }

    /// Borrow the underlying dashu value.
    pub fn as_rbig(&self) -> &RBig {
        &self.0
    }

    /// Nearest `f64` (documented-lossy).
    pub fn to_f64_lossy(&self) -> f64 {
        self.0.to_f64().value()
    }

    /// The exact rational square root, when one exists in ℚ.
    ///
    /// `None` for negatives and for every non-square — there is no rounding
    /// and no tolerance, so a `Some` is a proof that `self` is a square.
    pub fn sqrt_exact(&self) -> Option<Rational> {
        use dashu::base::SquareRootRem;
        use dashu::integer::{IBig, UBig};
        match self.sign() {
            Sign::Negative => return None,
            Sign::Zero => return Some(Rational::zero()),
            Sign::Positive => {}
        }
        let num: UBig = UBig::try_from(self.0.numerator().clone()).ok()?;
        let den = self.0.denominator().clone();
        let (sn, rn) = num.sqrt_rem();
        if rn != UBig::ZERO {
            return None;
        }
        let (sd, rd) = den.sqrt_rem();
        if rd != UBig::ZERO {
            return None;
        }
        Some(Rational(RBig::from_parts(IBig::from(sn), sd)))
    }

    /// The **square class** of a nonzero rational: the squarefree integer `s`
    /// with `self = s·q²` for some `q ∈ ℚ`. This is the `δ` that names the
    /// quadratic extension `ℚ(√self) = ℚ(√δ)`.
    ///
    /// Factoring is by trial division and is **capped**: an unfactored cofactor
    /// is carried into `s` unchanged, so the returned `s` always satisfies
    /// `self/s ∈ (ℚ*)²` but need not be squarefree when the cofactor has two
    /// equal large prime factors. Zero maps to zero.
    pub fn square_class(&self) -> Rational {
        use dashu::integer::{IBig, UBig};
        if self.sign() == Sign::Zero {
            return Rational::zero();
        }
        // n/d and n·d have the same square class (they differ by d²).
        let prod = self.0.numerator() * IBig::from(self.0.denominator().clone());
        let neg = prod.sign() == DSign::Negative;
        let mut m: UBig = UBig::try_from(if neg { -prod } else { prod }).unwrap_or(UBig::ONE);
        let mut out = UBig::ONE;
        let mut p = UBig::from(2u8);
        let cap = UBig::from(100_000u32);
        while &p * &p <= m && p <= cap {
            let mut e = 0u32;
            while (&m % &p) == UBig::ZERO {
                m /= &p;
                e += 1;
            }
            if e % 2 == 1 {
                out *= &p;
            }
            p += UBig::ONE;
        }
        out *= m;
        let i = IBig::from(out);
        Rational(RBig::from_parts(if neg { -i } else { i }, UBig::ONE))
    }

    /// Total bit size — numerator bits plus denominator bits. The coefficient-
    /// growth metric for exact algebraic pipelines.
    pub fn bit_size(&self) -> u64 {
        use dashu::base::BitTest;
        use dashu::integer::UBig;
        let n = self.0.numerator();
        let nb = if n.sign() == DSign::Negative {
            UBig::try_from(-n.clone()).map(|u| u.bit_len()).unwrap_or(1)
        } else {
            UBig::try_from(n.clone()).map(|u| u.bit_len()).unwrap_or(1)
        };
        (nb.max(1) + self.0.denominator().bit_len()) as u64
    }

    /// Multiplicative inverse. Panics on exact zero.
    pub fn recip(&self) -> Rational {
        self.checked_recip().expect("Rational::recip of zero")
    }

    /// Multiplicative inverse, or `None` for exact zero.
    pub fn checked_recip(&self) -> Option<Rational> {
        (self.sign() != Sign::Zero).then(|| ExactField::div(&Rational::one(), self))
    }

    /// Integer power, including negative powers for nonzero values.
    pub fn pow(&self, exponent: i32) -> Rational {
        self.checked_pow(exponent)
            .expect("Rational::pow: negative power of zero")
    }

    /// Integer power, or `None` for a negative power of exact zero.
    pub fn checked_pow(&self, exponent: i32) -> Option<Rational> {
        if exponent < 0 {
            return self
                .checked_recip()
                .map(|reciprocal| reciprocal.pow_unsigned(exponent.unsigned_abs()));
        }
        Some(self.pow_unsigned(exponent as u32))
    }

    /// Nonnegative integer power over the full `u32` exponent range.
    pub fn pow_unsigned(&self, exponent: u32) -> Rational {
        let mut base = self.clone();
        let mut power = exponent;
        let mut result = Rational::one();
        while power != 0 {
            if power & 1 == 1 {
                result = RingOps::mul(&result, &base);
            }
            power >>= 1;
            if power != 0 {
                base = RingOps::mul(&base, &base);
            }
        }
        result
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RationalWire {
    numerator: String,
    denominator: String,
}

impl Serialize for Rational {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        RationalWire {
            numerator: self.0.numerator().to_string(),
            denominator: self.0.denominator().to_string(),
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Rational {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = RationalWire::deserialize(deserializer)?;
        let numerator = IBig::from_str(&wire.numerator).map_err(serde::de::Error::custom)?;
        let denominator = UBig::from_str(&wire.denominator).map_err(serde::de::Error::custom)?;
        if denominator == UBig::ZERO {
            return Err(serde::de::Error::custom(
                "rational denominator must be positive",
            ));
        }
        let value = Rational(RBig::from_parts(numerator, denominator));
        if value.0.numerator().to_string() != wire.numerator
            || value.0.denominator().to_string() != wire.denominator
        {
            return Err(serde::de::Error::custom("non-canonical rational encoding"));
        }
        Ok(value)
    }
}

impl From<RBig> for Rational {
    fn from(r: RBig) -> Rational {
        Rational(r)
    }
}

impl RingOps for Rational {
    fn zero() -> Self {
        Rational(RBig::ZERO)
    }
    fn from_i32(i: i32) -> Self {
        Rational(RBig::from(i))
    }
    fn add(&self, rhs: &Self) -> Self {
        Rational(&self.0 + &rhs.0)
    }
    fn sub(&self, rhs: &Self) -> Self {
        Rational(&self.0 - &rhs.0)
    }
    fn mul(&self, rhs: &Self) -> Self {
        Rational(&self.0 * &rhs.0)
    }
    fn neg(&self) -> Self {
        Rational(-self.0.clone())
    }
}

impl ExactRing for Rational {
    fn from_f64(x: f64) -> Option<Self> {
        Rational::from_f64(x)
    }

    fn sign(&self) -> Sign {
        match self.0.sign() {
            DSign::Positive => {
                if self.0.is_zero() {
                    Sign::Zero
                } else {
                    Sign::Positive
                }
            }
            DSign::Negative => Sign::Negative,
        }
    }

    /// Outward-correct enclosure, from the *direction* `RBig::to_f64` reports
    /// rather than a second exact comparison.
    ///
    /// `RBig::to_f64` is dashu's **correctly-rounded** conversion and returns
    /// `Approximation<f64, Sign>`, where `Inexact(v, s)` means
    /// `s == sign(v − self)` (it is `sign` when the magnitude rounded away
    /// from zero and `−sign` when it rounded toward zero). So one ulp on the
    /// side `s` names is exactly the widening this bridge needs:
    ///
    /// - `Exact(v)` ⟹ `self == v` ⟹ the point interval, which is what makes
    ///   an interval filter able to certify `Equal` at all.
    /// - `Inexact(v, Positive)` ⟹ `v > self` ⟹ `[next_down(v), v]`.
    /// - `Inexact(v, Negative)` ⟹ `v < self` ⟹ `[v, next_up(v)]`.
    ///
    /// Overflow and underflow fall out of the same rule with no special
    /// cases: `+∞` can only be reported `Inexact(_, Positive)`, and
    /// `next_down(∞) == f64::MAX`, giving `[MAX, ∞]`; a value that underflows
    /// to `+0` is `Inexact(0, Negative)`, giving `[0, 5e-324]`.
    ///
    /// **This is the crate's hottest bridge** — every interval filter over
    /// rational data converts one coefficient per term through it — and the
    /// obvious implementation (re-embed `v` with `from_f64` and compare) costs
    /// a rational construction (with its gcd reduction) plus a
    /// cross-multiplying comparison *per coefficient*. `to_interval_matches_
    /// exact_comparison` pins the two against each other so the cheap form
    /// cannot drift from the definition.
    fn to_interval(&self) -> Interval {
        use dashu::base::Approximation;
        match self.0.to_f64() {
            Approximation::Exact(v) => Interval::point(v),
            Approximation::Inexact(v, DSign::Positive) => Interval::new(v.next_down(), v),
            Approximation::Inexact(v, DSign::Negative) => Interval::new(v, v.next_up()),
        }
    }
}

impl ExactField for Rational {
    fn div(&self, rhs: &Self) -> Self {
        assert!(!rhs.0.is_zero(), "Rational::div by zero");
        Rational(&self.0 / &rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uncertain::Uncertain;
    use core::cmp::Ordering;

    #[test]
    fn from_f64_rejects_nonfinite() {
        assert!(Rational::from_f64(f64::NAN).is_none());
        assert!(Rational::from_f64(f64::INFINITY).is_none());
        assert!(Rational::from_f64(f64::NEG_INFINITY).is_none());
    }

    #[test]
    fn to_interval_exact_double() {
        let q = Rational::from_f64(0.5).unwrap();
        let iv = q.to_interval();
        assert!(iv.is_point());
        assert_eq!(iv.inf(), 0.5);
    }

    #[test]
    fn to_interval_inexact_third() {
        let q = Rational::from_ratio(1, 3);
        let iv = q.to_interval();
        assert!(!iv.is_point());
        // Enclosure: 1/3 lies strictly inside.
        let lo = Rational::from_f64(iv.inf()).unwrap();
        let hi = Rational::from_f64(iv.sup()).unwrap();
        assert!(lo < q && q < hi);
        assert_eq!(iv.sup(), iv.inf().next_up()); // 1 ulp wide
    }

    #[test]
    fn to_interval_huge_and_tiny() {
        // > MAX
        let big = Rational::from_f64(f64::MAX)
            .unwrap()
            .mul(&Rational::from_i64(2));
        let iv = big.to_interval();
        assert_eq!(iv.sup(), f64::INFINITY);
        assert!(iv.inf() >= f64::MAX);
        // Below the subnormal range: enclosure must straddle correctly.
        let tiny = Rational::from_ratio(1, i64::MAX)
            .mul(&Rational::from_ratio(1, i64::MAX))
            .mul(&Rational::from_ratio(1, i64::MAX)); // ~1e-56... still normal
        let tiny = tiny.clone().mul(&tiny).mul(&tiny); // ~1e-341: subnormal-or-below
        let ivt = tiny.to_interval();
        let lo = Rational::from_f64(ivt.inf()).unwrap();
        let hi = Rational::from_f64(ivt.sup()).unwrap();
        assert!(lo <= tiny && tiny <= hi);
    }

    /// The definition [`ExactRing::to_interval`] used to *be*: round to
    /// nearest, re-embed exactly, compare, widen the side that needs it.
    fn to_interval_by_exact_comparison(q: &Rational) -> Interval {
        let v = q.0.to_f64().value();
        if v == f64::INFINITY {
            return Interval::new(f64::MAX, f64::INFINITY);
        }
        if v == f64::NEG_INFINITY {
            return Interval::new(f64::NEG_INFINITY, f64::MIN);
        }
        let back = Rational::from_f64(v).unwrap();
        match back.cmp(q) {
            Ordering::Equal => Interval::point(v),
            Ordering::Less => Interval::new(v, v.next_up()),
            Ordering::Greater => Interval::new(v.next_down(), v),
        }
    }

    /// The cheap bridge must agree with that definition **bit for bit** —
    /// including at the overflow, underflow and exactly-representable edges,
    /// which is where trusting a reported rounding direction could go wrong.
    #[test]
    fn to_interval_matches_exact_comparison() {
        let mut cases: Vec<Rational> = Vec::new();
        for n in -40i64..=40 {
            for d in 1i64..=17 {
                cases.push(Rational::from_ratio(n, d));
            }
        }
        for x in [
            0.0,
            -0.0,
            1.0,
            -1.0,
            0.5,
            f64::MAX,
            f64::MIN,
            f64::MIN_POSITIVE,
            5e-324,
            -5e-324,
            core::f64::consts::PI,
        ] {
            let q = Rational::from_f64(x).unwrap();
            cases.push(q.clone());
            cases.push(q.clone().mul(&Rational::from_i64(3)));
            cases.push(q.clone().mul(&Rational::from_ratio(1, 3)));
            cases.push(q.add(&Rational::from_ratio(1, 7)));
        }
        // Beyond the double range in both directions, and below it.
        let huge = Rational::from_f64(f64::MAX)
            .unwrap()
            .mul(&Rational::from_i64(4));
        cases.push(huge.clone());
        cases.push(huge.neg());
        let tiny = Rational::from_f64(5e-324)
            .unwrap()
            .mul(&Rational::from_ratio(1, 5));
        cases.push(tiny.clone());
        cases.push(tiny.neg());

        for q in &cases {
            let fast = q.to_interval();
            let slow = to_interval_by_exact_comparison(q);
            assert_eq!(
                (fast.inf(), fast.sup()),
                (slow.inf(), slow.sup()),
                "to_interval disagreed on {q:?}"
            );
            // And it really encloses: the bounds re-embed around `q`.
            if fast.inf().is_finite() {
                assert!(Rational::from_f64(fast.inf()).unwrap() <= *q);
            }
            if fast.sup().is_finite() {
                assert!(*q <= Rational::from_f64(fast.sup()).unwrap());
            }
        }
    }

    #[test]
    fn sign_and_interval_sign_agree() {
        for (n, d) in [(1i64, 3i64), (-7, 2), (0, 1), (5, 7), (-1, 1000000007)] {
            let q = Rational::from_ratio(n, d);
            let iv = q.to_interval();
            match iv.sign() {
                Uncertain::Certain(s) => assert_eq!(s, q.sign()),
                Uncertain::Unknown => {
                    // Only possible when the enclosure touches zero: the
                    // exact value must then be within one ulp of zero.
                    assert!(iv.contains_zero());
                }
            }
        }
    }
}
