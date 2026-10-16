//! Arithmetic rungs for adaptive polynomial-sign predicates.
//!
//! One predicate text, written once against [`RungArith`] (a [`RingOps`] ring
//! over `f64` inputs), is evaluated at increasing precision until a sign is
//! certified:
//!
//! | rung | type | decides |
//! |---|---|---|
//! | [`Rung::Bound`] (1) | [`RunningError`]: `f64` plus a rigorous running error bound | generic signs, and exact zeros when every operation is exact |
//! | [`Rung::DoubleDouble`] (2) | [`DdBound`]: double-double plus a rigorous bound | near-degenerate nonzero signs |
//! | [`Rung::Expansion`] (3) | [`StackExpansion`]: exact, fixed inline capacity | exact zeros from cancellation |
//! | [`Rung::Rational`] (4) | [`Rational`] | everything the cheaper rungs refused |
//!
//! Rung 0 (structural detection from geometric definitions) belongs to the
//! consumer and is not part of this crate.
//!
//! # What "certified" means
//!
//! The reference value of every rung is the *exact real value of the predicate
//! text evaluated over the exact binary values of the `f64` inputs*. Rungs 1
//! and 2 carry an absolute bound `e` with `|v - exact| <= e`; they certify a
//! sign only when `|v|` exceeds `e`. Rung 3 is the exact value. A rung that
//! cannot guarantee its invariant (a guard below tripped, an expansion ran out
//! of capacity) says so and the driver escalates; **no rung ever returns a
//! wrong sign in place of "uncertain"**.
//!
//! # Guards
//!
//! All three inexact rungs share the thresholds below. A value or operation
//! outside them is *poisoned*: the bound becomes infinite (rungs 1, 2) or the
//! escalate flag is raised (rung 3), and poison is absorbed by every later
//! operation.
//!
//! * [`MAX_MAGNITUDE`] = `1e290`: no `f64` value, input or result, may exceed
//!   it. The error-free `two_sum` computes `s - a`, which overflows for large
//!   opposite-sign operands even when the sum is finite; capping every
//!   operand at `1e290` leaves a factor `1.8e18` of headroom, so no
//!   intermediate of an addition, nor the doubled result `2e290`, can
//!   overflow.
//! * [`MAX_PRODUCT_OPERAND`] = `1e140`: operands of a product. Their product
//!   is at most `1e280`, inside the sum cap, and far inside the Veltkamp split
//!   limit `2^996 ~ 6.7e299`.
//! * [`MIN_PRODUCT`] = `2^-969`: a nonzero product below this is refused. The
//!   Dekker error term carries bits down to `2^(exp(p) - 106)`; below `2^-969`
//!   they fall off the subnormal range and the "exact" residual silently
//!   rounds. This is the threshold of [`crate::expansion::two_prod`].
//!
//! The bounds themselves are computed in `f64`. Each bound is a sum of at most
//! six nonnegative terms, so it is rounded up with the factor
//! `1 + 2^-48` (which dominates the accumulated `(1 - 2^-53)^6` rounding) and,
//! if any term could have underflowed, a floor of `2^-1000` (which dominates
//! the absolute error of an underflowing term, at most `2^-1075` each). A
//! bound whose every operation was exact stays exactly zero: that is what lets
//! rung 1 certify an exact zero.
//!
//! # Local origin
//!
//! Rung 1's error is *semi-static*: it is proportional to the magnitude of the
//! operands of each product, so predicates should be written over differences
//! from a local origin (as in `(b - a) x (c - a)`) rather than absolute
//! coordinates. Write the subtraction inside the predicate text. It is then
//! tracked exactly (its rounding residual enters the bound), and for clustered
//! data the differences are exact (Sterbenz) so the bound stays zero. A caller
//! that translates `f64` inputs *before* calling must make that translation
//! exact; a rounded translation changes the question being asked and the bound
//! no longer covers the original coordinates.
//!
//! # Determinism
//!
//! Every operation is `+`, `-`, `*` and the Dekker/Veltkamp product from
//! [`crate::expansion`]; `mul_add` is never used and Rust does not contract
//! `a * b + c`, so results are bit-identical with and without the `fma` target
//! feature. There is no global state, thread-local, or call history.

mod dd_bound;
mod running_error;
mod stack_expansion;

pub use dd_bound::DdBound;
pub use running_error::RunningError;
pub use stack_expansion::StackExpansion;

use crate::exact::{ExactRing, Rational, RingOps};
use crate::uncertain::{Sign, Uncertain};
use smallvec::SmallVec;

/// Largest magnitude any `f64` value may have on rungs 1 to 3.
pub const MAX_MAGNITUDE: f64 = 1e290;
/// Largest magnitude of an operand of a product on rungs 1 to 3.
pub const MAX_PRODUCT_OPERAND: f64 = 1e140;
/// Smallest nonzero product magnitude rungs 1 to 3 accept, `2^-969`.
pub const MIN_PRODUCT: f64 = f64::MIN_POSITIVE * 9_007_199_254_740_992.0;
/// Inline component capacity of the expansion used by [`adaptive_sign`].
pub const EXPANSION_CAPACITY: usize = 32;

/// `|x| <= limit`; false for NaN.
#[inline]
fn within(x: f64, limit: f64) -> bool {
    x.abs() <= limit
}

/// `1 + 2^-48`: rounds a freshly summed bound up.
const BOUND_UP: f64 = 1.0 + 1.0 / 281_474_976_710_656.0;
/// `2^-1000`: added to a nonzero bound to cover underflow of its terms.
const BOUND_FLOOR: f64 = f64::MIN_POSITIVE * 4_194_304.0;
/// `1 - 2^-51`: rounds a margin down.
const MARGIN_DOWN: f64 = 1.0 - 1.0 / 2_251_799_813_685_248.0;

/// Round the sum `sum` of nonnegative bound terms up to a valid bound.
/// NaN and infinity map to infinity (no information).
///
/// `sum == 0` means the bound is exactly zero only when no term could have
/// underflowed to zero; callers whose terms are products of nonzero factors
/// pass `may_underflow = true` to keep the floor.
#[inline]
fn round_up_bound(sum: f64, may_underflow: bool) -> f64 {
    if !sum.is_finite() {
        f64::INFINITY
    } else if sum == 0.0 && !may_underflow {
        0.0
    } else {
        sum * BOUND_UP + BOUND_FLOOR
    }
}

/// A rung of the adaptive ladder. Rung 0 (structural) is the consumer's.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rung {
    /// `f64` plus a running error bound.
    Bound = 1,
    /// Double-double plus a bound.
    DoubleDouble = 2,
    /// Exact stack-allocated expansion.
    Expansion = 3,
    /// Exact rational.
    Rational = 4,
}

/// The number type a predicate is evaluated over at one rung.
pub trait RungArith: RingOps + Clone {
    /// Embed a finite `f64` input exactly. The inexact rungs accept any value
    /// and poison one that violates their guards; the rational rung requires
    /// a finite value (the drivers check this).
    fn lift(x: f64) -> Self;
}

impl RungArith for Rational {
    #[allow(clippy::expect_used)] // documented contract: the drivers check finiteness first
    fn lift(x: f64) -> Self {
        Rational::from_f64(x).expect("RungArith::lift requires a finite input")
    }
}

/// A polynomial predicate: one text evaluated over any [`RungArith`].
///
/// `eval` must be a polynomial in its inputs built from `add`, `sub`, `mul`,
/// `neg` and `from_i32`, perform the same operations in the same order for
/// every `N`, and be free of side effects.
pub trait PolyPredicate {
    /// Number of `f64` inputs `eval` reads.
    fn arity(&self) -> usize;
    /// Evaluate the predicate polynomial over `inputs` (length [`arity`]).
    ///
    /// [`arity`]: PolyPredicate::arity
    fn eval<N: RungArith>(&self, inputs: &[N]) -> N;
}

/// A certified sign with the rung that decided it.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct AdaptiveSign {
    /// The exact sign of the predicate over the inputs.
    pub sign: Sign,
    /// The rung that certified it.
    pub rung: Rung,
    /// A certified lower bound on `|value|`, in the predicate's own units
    /// (`0.0` for [`Sign::Zero`]). It is never above the exact `|value|`; a
    /// rung that certifies a sign with a thin margin reports a thin margin.
    pub margin: f64,
}

/// Input rejected before any rung ran.
#[derive(Copy, Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum AdaptiveError {
    /// An input was NaN or infinite; the predicate has no exact value.
    #[error("predicate input is not finite")]
    NonFiniteInput,
    /// The input slice does not match [`PolyPredicate::arity`].
    #[error("predicate expects {expected} inputs, got {got}")]
    Arity {
        /// The predicate's arity.
        expected: usize,
        /// The length supplied.
        got: usize,
    },
}

fn validate<P: PolyPredicate>(p: &P, inputs: &[f64]) -> Result<(), AdaptiveError> {
    if inputs.len() != p.arity() {
        return Err(AdaptiveError::Arity {
            expected: p.arity(),
            got: inputs.len(),
        });
    }
    if inputs.iter().any(|x| !x.is_finite()) {
        return Err(AdaptiveError::NonFiniteInput);
    }
    Ok(())
}

/// Predicates up to this many inputs lift without touching the heap.
const INLINE_INPUTS: usize = 16;

fn lifted<N: RungArith>(inputs: &[f64]) -> SmallVec<[N; INLINE_INPUTS]> {
    inputs.iter().map(|&x| N::lift(x)).collect()
}

/// A lower bound of `|r|` as an `f64`, for the rational rung.
fn rational_margin(r: &Rational) -> f64 {
    let mag = if ExactRing::sign(r) == Sign::Negative {
        r.neg()
    } else {
        r.clone()
    };
    let approx = mag.to_f64_lossy();
    if approx.is_nan() {
        0.0
    } else if approx == f64::INFINITY {
        f64::MAX
    } else {
        // Nearest rounding may land above the true value; step down once.
        approx.next_down().max(0.0)
    }
}

/// The rational rung; `inputs` must already be validated.
fn rational_sign<P: PolyPredicate>(p: &P, inputs: &[f64]) -> AdaptiveSign {
    let v = p.eval(&lifted::<Rational>(inputs));
    let sign = ExactRing::sign(&v);
    AdaptiveSign {
        sign,
        rung: Rung::Rational,
        margin: if sign == Sign::Zero {
            0.0
        } else {
            rational_margin(&v)
        },
    }
}

/// Evaluate `p` at exactly one rung. `Ok(None)` means that rung could not
/// certify a sign (inconclusive bound, tripped guard, or full expansion); the
/// rational rung always returns `Some`.
pub fn eval_rung<P: PolyPredicate>(
    p: &P,
    inputs: &[f64],
    rung: Rung,
) -> Result<Option<AdaptiveSign>, AdaptiveError> {
    validate(p, inputs)?;
    Ok(match rung {
        Rung::Bound => {
            let v = p.eval(&lifted::<RunningError>(inputs));
            match v.sign() {
                Uncertain::Certain(sign) => Some(AdaptiveSign {
                    sign,
                    rung,
                    margin: v.margin(),
                }),
                Uncertain::Unknown => None,
            }
        }
        Rung::DoubleDouble => {
            let v = p.eval(&lifted::<DdBound>(inputs));
            match v.sign() {
                Uncertain::Certain(sign) => Some(AdaptiveSign {
                    sign,
                    rung,
                    margin: v.margin(),
                }),
                Uncertain::Unknown => None,
            }
        }
        Rung::Expansion => {
            let v = p.eval(&lifted::<StackExpansion<EXPANSION_CAPACITY>>(inputs));
            v.sign().map(|sign| AdaptiveSign {
                sign,
                rung,
                margin: v.margin(),
            })
        }
        Rung::Rational => Some(rational_sign(p, inputs)),
    })
}

/// [`adaptive_sign`] starting at `start` instead of rung 1: for replaying a
/// decision at a forced higher rung. The result's rung is never below `start`.
pub fn adaptive_sign_from<P: PolyPredicate>(
    p: &P,
    inputs: &[f64],
    start: Rung,
) -> Result<AdaptiveSign, AdaptiveError> {
    validate(p, inputs)?;
    for rung in [Rung::Bound, Rung::DoubleDouble, Rung::Expansion] {
        if rung >= start
            && let Some(a) = eval_rung(p, inputs, rung)?
        {
            return Ok(a);
        }
    }
    Ok(rational_sign(p, inputs))
}

/// The exact sign of `p` over `inputs`, escalating rung 1 -> 2 -> 3 -> 4 until
/// one certifies it. Never wrong; the rung and margin are deterministic
/// functions of `p` and `inputs` alone.
pub fn adaptive_sign<P: PolyPredicate>(
    p: &P,
    inputs: &[f64],
) -> Result<AdaptiveSign, AdaptiveError> {
    adaptive_sign_from(p, inputs, Rung::Bound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_the_documented_powers() {
        assert_eq!(MIN_PRODUCT, 2f64.powi(-969));
        assert_eq!(BOUND_UP, 1.0 + 2f64.powi(-48));
        assert_eq!(BOUND_FLOOR, 2f64.powi(-1000));
        assert_eq!(MARGIN_DOWN, 1.0 - 2f64.powi(-51));
        assert_eq!(dd_bound::U2, 2f64.powi(-106));
    }

    #[test]
    fn bound_rounding_keeps_exact_zero_and_poisons_non_numbers() {
        assert_eq!(round_up_bound(0.0, false), 0.0);
        assert_eq!(round_up_bound(0.0, true), BOUND_FLOOR);
        assert_eq!(round_up_bound(f64::NAN, false), f64::INFINITY);
        assert_eq!(round_up_bound(f64::INFINITY, true), f64::INFINITY);
        assert!(round_up_bound(1.0, false) > 1.0);
    }
}
