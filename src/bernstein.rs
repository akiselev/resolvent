//! Bernstein-coefficient exact enclosure + de Casteljau subdivision
//! (DESIGN.md §3.6, M6 deliverable 3): certified range/sign bounds for
//! polynomials over an interval, producing *certified-enclosure*
//! fail-closed verdicts where exact decision is not rationalizable.
//!
//! The convex-hull property makes the Bernstein coefficients over
//! `[lo, hi]` a proof: the polynomial's range over the interval is
//! contained in `[min coeff, max coeff]`, with the first and last
//! coefficients EQUAL to the endpoint values. Subdivision (exact
//! de Casteljau over ℚ) converges quadratically to the true range —
//! verdicts are `Certain` when the coefficients agree in sign and
//! `Unknown` otherwise, never a silent approximation.

use crate::AlgebraError;
use crate::exact::{ExactField, ExactRing, Rational, RingOps};
use crate::interval::Interval;
use crate::roots::QPoly;
use crate::uncertain::{Sign, USign, Uncertain};

/// An exact Bernstein representation of a polynomial over `[lo, hi]`.
#[derive(Clone, Debug)]
pub struct Bernstein {
    /// Bernstein coefficients `b_0 … b_n` over `[lo, hi]`.
    ///
    /// **Invariant: never empty.** `from_power` pushes `n + 1 ≥ 1` entries
    /// (`for k in 0..=n`, with `n = p.degree().unwrap_or(0)`, so the zero
    /// polynomial still gets one coefficient), and `subdivide_at` — the only
    /// other constructor — pushes one entry per de Casteljau level plus the
    /// initial one, preserving the length. `value_at_lo`/`value_at_hi` rest
    /// on this.
    coeffs: Vec<Rational>,
    lo: Rational,
    hi: Rational,
}

/// Exact binomial coefficient as a `Rational`.
fn binom(n: usize, k: usize) -> Rational {
    let mut acc = Rational::one();
    for i in 0..k {
        acc = acc
            .mul(&Rational::from_i64((n - i) as i64))
            .div(&Rational::from_i64((i + 1) as i64));
    }
    acc
}

impl Bernstein {
    /// Convert a power-basis polynomial to its exact Bernstein form over
    /// `[lo, hi]` (`lo < hi`). The zero polynomial gets one zero
    /// coefficient.
    pub fn from_power(p: &QPoly, lo: &Rational, hi: &Rational) -> Bernstein {
        Self::try_from_power(p, lo, hi).expect("empty or inverted interval")
    }

    /// Fallible form of [`Bernstein::from_power`].
    pub fn try_from_power(
        p: &QPoly,
        lo: &Rational,
        hi: &Rational,
    ) -> Result<Bernstein, AlgebraError> {
        if lo >= hi {
            return Err(AlgebraError::InvalidInterval);
        }
        // Power basis on [0,1]: q(t) = p(lo + (hi−lo)·t).
        let width = hi.sub(lo);
        let n = p.degree().unwrap_or(0);
        let q: Vec<Rational> = {
            // compose_affine is private to roots; Horner it here.
            let mut acc: Vec<Rational> = vec![Rational::zero()];
            for i in (0..=n).rev() {
                // acc = acc·(lo + width·t) + p_i
                let mut next = vec![Rational::zero(); acc.len() + 1];
                for (j, c) in acc.iter().enumerate() {
                    next[j] = next[j].add(&c.mul(lo));
                    next[j + 1] = next[j + 1].add(&c.mul(&width));
                }
                next[0] = next[0].add(&p.coeff(i));
                while next.last().is_some_and(|c| c.sign() == Sign::Zero) && next.len() > 1 {
                    next.pop();
                }
                acc = next;
            }
            acc
        };
        // b_k = Σ_{i≤k} C(k,i)/C(n,i) · q_i.
        let mut coeffs = Vec::with_capacity(n + 1);
        for k in 0..=n {
            let mut b = Rational::zero();
            for (i, qi) in q.iter().enumerate().take(k + 1) {
                if i > n {
                    break;
                }
                b = b.add(&binom(k, i).div(&binom(n, i)).mul(qi));
            }
            coeffs.push(b);
        }
        Ok(Bernstein {
            coeffs,
            lo: lo.clone(),
            hi: hi.clone(),
        })
    }

    /// The interval this representation lives on.
    pub fn interval(&self) -> (&Rational, &Rational) {
        (&self.lo, &self.hi)
    }

    /// The Bernstein coefficients (exact).
    pub fn coeffs(&self) -> &[Rational] {
        &self.coeffs
    }

    /// Exact value at the left endpoint (`b_0`).
    #[allow(clippy::expect_used)] // `coeffs` is non-empty by construction — see the field
    pub fn value_at_lo(&self) -> &Rational {
        self.coeffs
            .first()
            .expect("Bernstein::coeffs is never empty")
    }

    /// Exact value at the right endpoint (`b_n`).
    #[allow(clippy::expect_used)] // `coeffs` is non-empty by construction — see the field
    pub fn value_at_hi(&self) -> &Rational {
        self.coeffs
            .last()
            .expect("Bernstein::coeffs is never empty")
    }

    /// Certified range bound over `[lo, hi]`: the polynomial's range is
    /// CONTAINED in `[min, max]` of the coefficients (convex hull
    /// property); the bound touches the truth at both endpoints.
    pub fn range_bound(&self) -> (Rational, Rational) {
        let mut min = self.coeffs[0].clone();
        let mut max = self.coeffs[0].clone();
        for c in &self.coeffs[1..] {
            if *c < min {
                min = c.clone();
            }
            if *c > max {
                max = c.clone();
            }
        }
        (min, max)
    }

    /// Outward-correct double enclosure of the certified range bound.
    pub fn range_interval(&self) -> Interval {
        let (min, max) = self.range_bound();
        let lo = min.to_interval();
        let hi = max.to_interval();
        Interval::new(lo.inf(), hi.sup())
    }

    /// Certified sign of the polynomial over the WHOLE interval:
    /// `Certain` when every coefficient agrees (strictly positive /
    /// strictly negative / identically zero), `Unknown` when the
    /// coefficients mix signs — the certified-enclosure fail-closed
    /// verdict (subdivide to refine).
    pub fn sign_over(&self) -> USign {
        let mut pos = false;
        let mut neg = false;
        let mut zero = false;
        for c in &self.coeffs {
            match c.sign() {
                Sign::Positive => pos = true,
                Sign::Negative => neg = true,
                Sign::Zero => zero = true,
            }
        }
        match (pos, neg, zero) {
            (true, false, false) => Uncertain::Certain(Sign::Positive),
            (false, true, false) => Uncertain::Certain(Sign::Negative),
            (false, false, true) => Uncertain::Certain(Sign::Zero),
            _ => Uncertain::Unknown,
        }
    }

    /// Exact de Casteljau subdivision at parameter `t ∈ (0, 1)` of the
    /// current interval: two Bernstein forms over the sub-intervals,
    /// stitching exactly at the split point.
    pub fn subdivide_at(&self, t: &Rational) -> (Bernstein, Bernstein) {
        self.try_subdivide_at(t)
            .expect("split parameter must be in (0,1)")
    }

    /// Fallible form of [`Bernstein::subdivide_at`].
    pub fn try_subdivide_at(&self, t: &Rational) -> Result<(Bernstein, Bernstein), AlgebraError> {
        if t.sign() != Sign::Positive || *t >= Rational::one() {
            return Err(AlgebraError::InvalidInterval);
        }
        let one_minus = Rational::one().sub(t);
        let n = self.coeffs.len();
        let mut tri = self.coeffs.clone();
        let mut left = Vec::with_capacity(n);
        let mut right_rev = Vec::with_capacity(n);
        left.push(tri[0].clone());
        right_rev.push(tri[n - 1].clone());
        for level in 1..n {
            for i in 0..n - level {
                tri[i] = tri[i].mul(&one_minus).add(&tri[i + 1].mul(t));
            }
            left.push(tri[0].clone());
            right_rev.push(tri[n - 1 - level].clone());
        }
        right_rev.reverse();
        let split = self.lo.add(&self.hi.sub(&self.lo).mul(t));
        Ok((
            Bernstein {
                coeffs: left,
                lo: self.lo.clone(),
                hi: split.clone(),
            },
            Bernstein {
                coeffs: right_rev,
                lo: split,
                hi: self.hi.clone(),
            },
        ))
    }

    /// Midpoint subdivision.
    pub fn subdivide(&self) -> (Bernstein, Bernstein) {
        self.subdivide_at(&Rational::from_ratio(1, 2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rat(n: i64, d: i64) -> Rational {
        Rational::from_ratio(n, d)
    }

    #[test]
    fn endpoint_exactness_and_hull() {
        // p = x² over [−1, 1]: true range [0, 1].
        let p = QPoly::from_i64s(&[0, 0, 1]);
        let b = Bernstein::from_power(&p, &rat(-1, 1), &rat(1, 1));
        assert_eq!(b.value_at_lo(), &rat(1, 1)); // p(−1)
        assert_eq!(b.value_at_hi(), &rat(1, 1)); // p(1)
        let (min, max) = b.range_bound();
        assert!(min <= rat(0, 1), "hull bounds the true minimum");
        assert_eq!(max, rat(1, 1), "endpoint maxima are exact");
    }

    #[test]
    fn certified_signs() {
        // x² + 3 > 0 on [−1, 1]: certified Positive immediately.
        let p = QPoly::from_i64s(&[3, 0, 1]);
        let b = Bernstein::from_power(&p, &rat(-1, 1), &rat(1, 1));
        assert_eq!(b.sign_over(), Uncertain::Certain(Sign::Positive));
        // x² + 1 > 0 on [−1, 1], but its degree-2 hull touches zero
        // (coefficients [2, 0, 2]): honestly Unknown until ONE
        // subdivision certifies both halves — the refine loop consumers
        // run.
        let p1 = QPoly::from_i64s(&[1, 0, 1]);
        let b1 = Bernstein::from_power(&p1, &rat(-1, 1), &rat(1, 1));
        assert_eq!(b1.sign_over(), Uncertain::Unknown);
        let (l, r) = b1.subdivide();
        assert_eq!(l.sign_over(), Uncertain::Certain(Sign::Positive));
        assert_eq!(r.sign_over(), Uncertain::Certain(Sign::Positive));
        // x² touches zero: fail-closed Unknown (coefficients mix).
        let q = QPoly::from_i64s(&[0, 0, 1]);
        let bq = Bernstein::from_power(&q, &rat(-1, 1), &rat(1, 1));
        assert_eq!(bq.sign_over(), Uncertain::Unknown);
        // −x² − 3: certified Negative.
        let r = QPoly::from_i64s(&[-3, 0, -1]);
        let br = Bernstein::from_power(&r, &rat(-1, 1), &rat(1, 1));
        assert_eq!(br.sign_over(), Uncertain::Certain(Sign::Negative));
        // The zero polynomial: certified Zero.
        let z = QPoly::zero_poly();
        let bz = Bernstein::from_power(&z, &rat(0, 1), &rat(1, 1));
        assert_eq!(bz.sign_over(), Uncertain::Certain(Sign::Zero));
    }

    #[test]
    fn subdivision_stitches_and_tightens() {
        // p = x³ − x over [−2, 2].
        let p = QPoly::from_i64s(&[0, -1, 0, 1]);
        let b = Bernstein::from_power(&p, &rat(-2, 1), &rat(2, 1));
        let (l, r) = b.subdivide();
        assert_eq!(l.interval().1, r.interval().0, "shared split point");
        assert_eq!(l.value_at_lo(), &p.eval(&rat(-2, 1)));
        assert_eq!(r.value_at_hi(), &p.eval(&rat(2, 1)));
        assert_eq!(l.value_at_hi(), &p.eval(&rat(0, 1)), "stitch is exact");
        // Subdivision only tightens the hull.
        let (min0, max0) = b.range_bound();
        for piece in [&l, &r] {
            let (min, max) = piece.range_bound();
            assert!(min >= min0 && max <= max0);
        }
        // Refining around the local max at x = −1/√3 (value 2/(3√3) ≈
        // 0.3849): after a few subdivisions the global bound is close.
        let mut pieces = vec![b];
        for _ in 0..8 {
            pieces = pieces
                .into_iter()
                .flat_map(|q| {
                    let (a, c) = q.subdivide();
                    [a, c]
                })
                .collect();
        }
        // The local max at x = −1/√3 has value 2/(3√3) ≈ 0.3849; the
        // pieces covering [−1, 0] must converge to it from above.
        let true_max = 0.3849001794597505; // 2/(3√3), f64 approx
        let interior_max = pieces
            .iter()
            .filter(|q| *q.interval().0 >= rat(-1, 1) && *q.interval().1 <= rat(0, 1))
            .map(|q| q.range_bound().1)
            .max()
            .unwrap();
        let gi = interior_max.to_interval().sup();
        assert!(gi >= true_max - 1e-9 && gi <= true_max + 1e-3);
    }

    #[test]
    fn asymmetric_split() {
        let p = QPoly::from_i64s(&[1, 2, 3]);
        let b = Bernstein::from_power(&p, &rat(0, 1), &rat(1, 1));
        let (l, r) = b.subdivide_at(&rat(1, 3));
        assert_eq!(l.interval().1, &rat(1, 3));
        assert_eq!(l.value_at_hi(), &p.eval(&rat(1, 3)));
        assert_eq!(r.value_at_lo(), &p.eval(&rat(1, 3)));
    }
}
