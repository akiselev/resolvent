//! `f64` interval arithmetic without global rounding-mode state
//! (DESIGN.md §3.2).
//!
//! Default tier (always on): `+`/`-` use TwoSum one-sided widening (exact
//! results stay points); `*`, `/`, `sqrt`, `square` use round-to-nearest
//! plus blind outward widening (`next_down`/`next_up`), which is sound for
//! any correctly-rounded operation and is overflow/underflow-safe by
//! construction. Bounds may be `±∞`; they are never NaN.
//!
//! The invariant every operation preserves: **the exact real result of the
//! operation on any reals enclosed by the operands is enclosed by the
//! result.**

use crate::eft;
use crate::uncertain::{Sign, UOrd, USign, Uncertain};
use core::cmp::Ordering;
use core::ops::{Add, Div, Mul, Neg, Sub};
use core::sync::atomic::{AtomicU64, Ordering as AtOrd};

/// A closed interval `[inf, sup]` of reals, `inf ≤ sup`, bounds non-NaN
/// (infinite bounds allowed).
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Interval {
    inf: f64,
    sup: f64,
}

impl Interval {
    /// The whole real line.
    pub const WHOLE: Interval = Interval {
        inf: f64::NEG_INFINITY,
        sup: f64::INFINITY,
    };

    /// Exact point interval. Panics on a non-finite value (use
    /// [`Interval::try_point`]).
    ///
    /// # Panics
    /// This is an **API-contract panic**, not an internal invariant: the
    /// caller promises a finite `x`, exactly as `Vec` indexing promises an
    /// in-range index. The total form is [`Interval::try_point`], and every
    /// caller that cannot promise finiteness is expected to use it.
    #[allow(clippy::expect_used)] // documented contract; `try_point` is the total form
    pub fn point(x: f64) -> Interval {
        Interval::try_point(x).expect("Interval::point: not finite")
    }

    /// Exact point interval; `None` unless `x` is **finite**.
    ///
    /// `±∞` is rejected, not just NaN. A *point* at infinity encloses no real
    /// number, and admitting one breaks the crate's "bounds are never NaN"
    /// invariant one operation later: `point(∞) - point(∞)` and
    /// `point(∞) / point(∞)` both yield `[NaN, NaN]`. From there `bound_mul`'s
    /// `0 × ∞ → NaN → 0.0` fixup turns every candidate bound into `0.0` while
    /// `corners_can_be` reads NaN as negative, so multiplying by a zero
    /// interval certifies `Sign::Zero` for a value that has no sign at all.
    ///
    /// Unbounded *intervals* remain available via [`Interval::WHOLE`] and
    /// [`Interval::new`]; it is the degenerate point that is meaningless.
    pub fn try_point(x: f64) -> Option<Interval> {
        if x.is_finite() {
            Some(Interval { inf: x, sup: x })
        } else {
            None
        }
    }

    /// Interval from explicit bounds. Panics if NaN or `lo > hi`.
    pub fn new(lo: f64, hi: f64) -> Interval {
        assert!(!lo.is_nan() && !hi.is_nan(), "Interval::new(NaN)");
        assert!(lo <= hi, "Interval::new: lo > hi ({lo} > {hi})");
        Interval { inf: lo, sup: hi }
    }

    /// Lower bound.
    pub fn inf(self) -> f64 {
        self.inf
    }

    /// Upper bound.
    pub fn sup(self) -> f64 {
        self.sup
    }

    /// `true` iff the interval is a single point (an exactly-known value).
    pub fn is_point(self) -> bool {
        self.inf == self.sup
    }

    /// Midpoint (documented-lossy representative; for display/estimation).
    pub fn midpoint(self) -> f64 {
        if self.inf == f64::NEG_INFINITY && self.sup == f64::INFINITY {
            0.0
        } else if self.inf == f64::NEG_INFINITY {
            f64::MIN
        } else if self.sup == f64::INFINITY {
            f64::MAX
        } else if self.inf == self.sup {
            self.inf
        } else {
            // Avoids overflow of (inf + sup). Clamped because halving is not
            // safe against subnormal underflow: `bits(1)/2` ties-to-even DOWN
            // to 0 (below `inf`), and `bits(3)/2` ties-to-even UP to `bits(2)`,
            // whose double is `bits(4)` (above `sup`). A representative outside
            // its own enclosure is a different claim than a rounded one — it is
            // what made `Real::to_f64_lossy` return 0.0 for a value whose
            // `sign()` is `Positive`.
            let m = self.inf / 2.0 + self.sup / 2.0;
            m.clamp(self.inf, self.sup)
        }
    }

    /// Does the interval contain `x`?
    pub fn contains(self, x: f64) -> bool {
        self.inf <= x && x <= self.sup
    }

    /// Does the interval contain zero?
    pub fn contains_zero(self) -> bool {
        self.contains(0.0)
    }

    /// Intersection, if non-empty. Used when refining a cached enclosure —
    /// monotone refinement keeps every historical bound valid.
    pub fn intersect(self, rhs: Interval) -> Option<Interval> {
        let lo = self.inf.max(rhs.inf);
        let hi = self.sup.min(rhs.sup);
        if lo <= hi {
            Some(Interval { inf: lo, sup: hi })
        } else {
            None
        }
    }

    /// Certified sign.
    ///
    /// Contract (DESIGN.md §3.2): `Certain` iff `inf > 0`, or `sup < 0`, or
    /// `inf == sup == 0`. An interval with zero as a non-degenerate endpoint
    /// (`[0, x]`, `[x, 0]`) is `Unknown` — the true value could be zero *or*
    /// nonzero, which is exactly the degenerate case filters exist to catch.
    pub fn sign(self) -> USign {
        if self.inf > 0.0 {
            Uncertain::Certain(Sign::Positive)
        } else if self.sup < 0.0 {
            Uncertain::Certain(Sign::Negative)
        } else if self.inf == 0.0 && self.sup == 0.0 {
            Uncertain::Certain(Sign::Zero)
        } else {
            Uncertain::Unknown
        }
    }

    /// Certified comparison.
    pub fn cmp_interval(self, rhs: Interval) -> UOrd {
        if self.sup < rhs.inf {
            Uncertain::Certain(Ordering::Less)
        } else if self.inf > rhs.sup {
            Uncertain::Certain(Ordering::Greater)
        } else if self.is_point() && rhs.is_point() && self.inf == rhs.inf {
            Uncertain::Certain(Ordering::Equal)
        } else {
            Uncertain::Unknown
        }
    }

    /// Absolute value (exact — no widening: `|·|` of a bound is exact).
    #[must_use]
    pub fn abs(self) -> Interval {
        if self.inf >= 0.0 {
            self
        } else if self.sup <= 0.0 {
            -self
        } else {
            Interval {
                inf: 0.0,
                sup: self.sup.max(-self.inf),
            }
        }
    }

    /// Minimum (exact: min of correctly-known bounds needs no widening).
    #[must_use]
    pub fn min_interval(self, rhs: Interval) -> Interval {
        Interval {
            inf: self.inf.min(rhs.inf),
            sup: self.sup.min(rhs.sup),
        }
    }

    /// Maximum.
    #[must_use]
    pub fn max_interval(self, rhs: Interval) -> Interval {
        Interval {
            inf: self.inf.max(rhs.inf),
            sup: self.sup.max(rhs.sup),
        }
    }

    /// Square. Tighter than `self * self` (respects the correlation:
    /// `[-1,2]² = [0,4]`, not `[-2,4]`). The zero lower bound of a
    /// straddling interval is exact and not widened.
    #[must_use]
    pub fn square(self) -> Interval {
        let (lo_mag, hi_mag) = if self.inf >= 0.0 {
            (self.inf, self.sup)
        } else if self.sup <= 0.0 {
            (-self.sup, -self.inf)
        } else {
            (0.0, self.sup.max(-self.inf))
        };
        let lo = if lo_mag == 0.0 {
            0.0 // exact
        } else {
            eft::blind_lo(lo_mag * lo_mag).max(0.0)
        };
        Interval {
            inf: lo,
            sup: eft::blind_hi(hi_mag * hi_mag),
        }
    }

    /// Square root. Precondition: `sup ≥ 0` (panics otherwise). If the
    /// interval straddles zero the negative part is clamped (the enclosed
    /// exact value is assumed ≥ 0 by the caller's precondition).
    #[must_use]
    pub fn sqrt(self) -> Interval {
        assert!(
            self.sup >= 0.0,
            "Interval::sqrt of a certainly-negative interval"
        );
        let lo_in = self.inf.max(0.0);
        let lo = if lo_in == 0.0 {
            0.0 // sqrt(0) exact
        } else {
            eft::blind_lo(lo_in.sqrt()).max(0.0)
        };
        Interval {
            inf: lo,
            sup: eft::blind_hi(self.sup.sqrt()),
        }
    }
}

/// An [`Interval`] cache refined **monotonically**, readable through `&self`
/// with no lock and no blocking.
///
/// This is the crate's one shared-enclosure primitive: [`Real`](crate::Real)
/// holds one per DAG node and [`RealRoot`](crate::RealRoot) holds one per
/// isolated root, so a filter tier can read an enclosure through a shared
/// reference while some other thread is refining it.
///
/// # Why racy reads are still enclosures
///
/// Every value ever stored in `inf` is a valid **lower** bound on the number
/// (each store is the `max` of the current lower bound and a new valid one)
/// and every value ever stored in `sup` is a valid **upper** bound. The two
/// fields are loaded independently, so a reader can observe a *torn* pair —
/// an old `inf` with a new `sup`, or the reverse. That pair is still a valid
/// enclosure, because each component is independently valid, and `inf ≤ sup`
/// still holds for any old/new mix: both sequences are monotone toward each
/// other and neither ever crosses the true value. A stale read therefore
/// costs precision (a filter that could have certified returns `Unknown` and
/// the caller escalates) and can never cost soundness.
///
/// `Relaxed` is the right ordering for exactly that reason: nothing else is
/// published alongside these bounds, so there is no happens-before edge for a
/// stronger ordering to establish (DESIGN.md §2 pitfall 2).
#[derive(Debug)]
pub struct AtomicInterval {
    inf: AtomicU64,
    sup: AtomicU64,
}

impl AtomicInterval {
    /// A cache initialised to `iv`.
    pub fn new(iv: Interval) -> AtomicInterval {
        AtomicInterval {
            inf: AtomicU64::new(iv.inf().to_bits()),
            sup: AtomicU64::new(iv.sup().to_bits()),
        }
    }

    /// The current enclosure. Never blocks.
    pub fn load(&self) -> Interval {
        let inf = f64::from_bits(self.inf.load(AtOrd::Relaxed));
        let sup = f64::from_bits(self.sup.load(AtOrd::Relaxed));
        // A torn pair (old inf, new sup) is still an enclosure because
        // refinement is monotone; inf ≤ sup holds for any old/new mix.
        Interval::new(inf, sup)
    }

    /// Refine with a tighter enclosure (new ⊆ current must hold up to
    /// races; intersection keeps every write individually valid).
    pub fn refine(&self, tighter: Interval) {
        let cur = self.load();
        if let Some(meet) = cur.intersect(tighter) {
            self.inf.store(meet.inf().to_bits(), AtOrd::Relaxed);
            self.sup.store(meet.sup().to_bits(), AtOrd::Relaxed);
        }
        // A disjoint intersection is impossible for sound refinements; if it
        // ever happened we keep the current bounds (never widen).
    }
}

impl Clone for AtomicInterval {
    fn clone(&self) -> AtomicInterval {
        AtomicInterval::new(self.load())
    }
}

impl Neg for Interval {
    type Output = Interval;
    fn neg(self) -> Interval {
        // Exact.
        Interval {
            inf: -self.sup,
            sup: -self.inf,
        }
    }
}

impl Add for Interval {
    type Output = Interval;
    fn add(self, rhs: Interval) -> Interval {
        Interval {
            inf: eft::add_lo(self.inf, rhs.inf),
            sup: eft::add_hi(self.sup, rhs.sup),
        }
    }
}

impl Sub for Interval {
    type Output = Interval;
    fn sub(self, rhs: Interval) -> Interval {
        Interval {
            inf: eft::add_lo(self.inf, -rhs.sup),
            sup: eft::add_hi(self.sup, -rhs.inf),
        }
    }
}

/// Candidate product of two bounds; `0 × ∞ → NaN` is fixed up to `0` (the
/// exact product of a zero endpoint with anything is zero).
#[inline]
fn bound_mul(a: f64, b: f64) -> f64 {
    let p = a * b;
    if p.is_nan() { 0.0 } else { p }
}

/// Can any corner pair `(x, y)` of the two intervals have a true product
/// strictly positive / strictly negative? Decides whether a zero candidate
/// bound is sound to keep unwidened: keeping `sup = 0` is sound iff no true
/// product can exceed 0 (no same-sign nonzero corner); keeping `inf = 0` is
/// sound iff no true product can be negative (no opposite-sign nonzero
/// corner). This is immune to underflow: a positive product that underflows
/// to `+0.0` still has a same-sign corner, forcing the widen.
#[inline]
fn corners_can_be(a: Interval, b: Interval, positive: bool) -> bool {
    let ax = [a.inf, a.sup];
    let bx = [b.inf, b.sup];
    for x in ax {
        for y in bx {
            if x != 0.0 && y != 0.0 {
                let same = (x > 0.0) == (y > 0.0);
                if same == positive {
                    return true;
                }
            }
        }
    }
    false
}

impl Mul for Interval {
    type Output = Interval;
    fn mul(self, rhs: Interval) -> Interval {
        // Four candidate products with 0×∞ fixups; min/max, then blind
        // outward widening (next_down(+∞)=MAX keeps overflow sound). A zero
        // bound stays exact only when the sign analysis proves no true
        // product lies beyond it.
        let p1 = bound_mul(self.inf, rhs.inf);
        let p2 = bound_mul(self.inf, rhs.sup);
        let p3 = bound_mul(self.sup, rhs.inf);
        let p4 = bound_mul(self.sup, rhs.sup);
        let lo = p1.min(p2).min(p3).min(p4);
        let hi = p1.max(p2).max(p3).max(p4);
        let inf = if lo == 0.0 && !corners_can_be(self, rhs, false) {
            0.0
        } else {
            eft::blind_lo(lo)
        };
        let sup = if hi == 0.0 && !corners_can_be(self, rhs, true) {
            0.0
        } else {
            eft::blind_hi(hi)
        };
        Interval { inf, sup }
    }
}

impl Div for Interval {
    type Output = Interval;
    fn div(self, rhs: Interval) -> Interval {
        if rhs.contains_zero() {
            // Divisor may be zero: no finite enclosure (CGAL semantics).
            return Interval::WHOLE;
        }
        // rhs has a definite nonzero sign; quotient signs mirror products.
        let q1 = self.inf / rhs.inf;
        let q2 = self.inf / rhs.sup;
        let q3 = self.sup / rhs.inf;
        let q4 = self.sup / rhs.sup;
        let lo = q1.min(q2).min(q3).min(q4);
        let hi = q1.max(q2).max(q3).max(q4);
        let inf = if lo == 0.0 && !corners_can_be(self, rhs, false) {
            0.0
        } else {
            eft::blind_lo(lo)
        };
        let sup = if hi == 0.0 && !corners_can_be(self, rhs, true) {
            0.0
        } else {
            eft::blind_hi(hi)
        };
        Interval { inf, sup }
    }
}

impl From<f64> for Interval {
    /// Point interval of a finite double; panics on NaN.
    fn from(x: f64) -> Interval {
        Interval::point(x)
    }
}

impl From<i32> for Interval {
    fn from(i: i32) -> Interval {
        Interval::point(f64::from(i)) // exact: i32 ⊂ f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn iv(a: f64, b: f64) -> Interval {
        Interval::new(a, b)
    }

    #[test]
    fn add_keeps_exact_points() {
        let r = Interval::point(1.5) + Interval::point(2.25);
        assert!(r.is_point());
        assert_eq!(r.inf(), 3.75);
    }

    #[test]
    fn add_widens_inexact_side_only() {
        let r = Interval::point(0.1) + Interval::point(0.2);
        // Result must enclose the true 0.3₁₀ which is NOT 0.1+0.2 in f64.
        assert!(!r.is_point());
        assert!(r.sup() - r.inf() <= f64::EPSILON); // 1 ulp wide
    }

    #[test]
    fn mul_zero_exact() {
        let r = Interval::point(0.0) * iv(1.0, f64::INFINITY);
        assert!(r.contains(0.0));
        assert_eq!(r.inf(), 0.0);
        assert_eq!(r.sup(), 0.0);
    }

    #[test]
    fn mul_underflow_widens() {
        let tiny = f64::MIN_POSITIVE; // 2^-1022
        let r = Interval::point(tiny) * Interval::point(tiny);
        // True product 2^-2044 is below the subnormal range: rounded to 0,
        // but 0 must NOT be claimed exact.
        assert!(r.inf() <= 0.0 && r.sup() > 0.0);
        assert!(!r.is_point());
    }

    #[test]
    fn mul_overflow_sound() {
        let r = Interval::point(f64::MAX) * Interval::point(2.0);
        assert_eq!(r.sup(), f64::INFINITY);
        assert!(r.inf() >= f64::MAX); // true value > MAX
    }

    #[test]
    fn div_by_zero_straddle() {
        let r = iv(1.0, 2.0) / iv(-1.0, 1.0);
        assert_eq!(r, Interval::WHOLE);
    }

    #[test]
    fn square_correlation() {
        let r = iv(-1.0, 2.0).square();
        assert_eq!(r.inf(), 0.0); // exact
        assert!(r.sup() >= 4.0);
        assert!(r.sup() <= 4.0 + 4.0 * f64::EPSILON);
    }

    #[test]
    fn sign_contract_boundary_zero() {
        assert_eq!(iv(0.0, 5.0).sign(), Uncertain::Unknown);
        assert_eq!(iv(-5.0, 0.0).sign(), Uncertain::Unknown);
        assert_eq!(iv(0.0, 0.0).sign(), Uncertain::Certain(Sign::Zero));
        assert_eq!(iv(0.5, 5.0).sign(), Uncertain::Certain(Sign::Positive));
        assert_eq!(iv(-5.0, -0.5).sign(), Uncertain::Certain(Sign::Negative));
        assert_eq!(iv(-1.0, 1.0).sign(), Uncertain::Unknown);
    }

    #[test]
    fn cmp_touching_bounds_unknown() {
        // sup == rhs.inf but not points: could be equal or less.
        assert_eq!(iv(0.0, 1.0).cmp_interval(iv(1.0, 2.0)), Uncertain::Unknown);
        assert_eq!(
            Interval::point(1.0).cmp_interval(Interval::point(1.0)),
            Uncertain::Certain(Ordering::Equal)
        );
        assert_eq!(
            iv(0.0, 0.5).cmp_interval(iv(1.0, 2.0)),
            Uncertain::Certain(Ordering::Less)
        );
    }
}
