//! Opt-in counters for the certify-or-escalate ladder (`--features metrics`).
//!
//! The whole point of a filter tier is a *number*: what fraction of the
//! predicates the sweep asks for are decided by the `f64` enclosure without
//! touching rational arithmetic, and how many locks that saves. Asserting
//! "the filter helps" is not evidence; counting is.
//!
//! Without the `metrics` feature every function here is an empty `#[inline]`
//! body and [`snapshot`] reports zeros, so the instrumented call sites cost
//! nothing in a normal build. With it they are `Relaxed` increments on
//! process-global atomics — the counts are exact (fetch_add is atomic), only
//! the *interleaving* is unordered, which no reader cares about.

#[cfg(feature = "metrics")]
use core::sync::atomic::{AtomicU64, Ordering as AtOrd};

/// A reading of the ladder counters.
///
/// Every field is cumulative since process start (or since the last
/// [`reset`]). `*_hit` counts predicates the interval filter decided;
/// `*_miss` counts the ones that escalated to exact rational work.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    /// [`RealRoot`](crate::RealRoot) leaf ladders (`sign_of`, `cmp_rational`,
    /// `cmp_root`) decided by the cached enclosure.
    pub root_hit: u64,
    /// The same ladders that had to escalate.
    pub root_miss: u64,
    /// `sign_radical1`/`sign_radical2` decided by interval evaluation of the
    /// whole radical expression.
    pub radical_hit: u64,
    /// The same, escalated to the exact squaring ladder.
    pub radical_miss: u64,
    /// Chart-geometry abscissa predicates (`cmp_abscissa`, `cmp_rational`,
    /// `sign_of`, `cmp_loose`, `cmp_yrep`) decided by the lock-free
    /// enclosure. **This is the sweep-line inner-loop number.**
    pub abscissa_hit: u64,
    /// The same, escalated — each of these takes the exact path.
    pub abscissa_miss: u64,
    /// Mutex acquisitions on shared refinable root state.
    pub locks: u64,
}

impl Counters {
    /// Fraction of abscissa predicates the filter decided, in `[0, 1]`;
    /// `None` when none were asked.
    pub fn abscissa_hit_rate(&self) -> Option<f64> {
        let n = self.abscissa_hit + self.abscissa_miss;
        (n > 0).then(|| self.abscissa_hit as f64 / n as f64)
    }

    /// Fraction of `RealRoot` leaf ladders the filter decided.
    pub fn root_hit_rate(&self) -> Option<f64> {
        let n = self.root_hit + self.root_miss;
        (n > 0).then(|| self.root_hit as f64 / n as f64)
    }

    /// Fraction of radical sign ladders the filter decided.
    pub fn radical_hit_rate(&self) -> Option<f64> {
        let n = self.radical_hit + self.radical_miss;
        (n > 0).then(|| self.radical_hit as f64 / n as f64)
    }
}

#[cfg(feature = "metrics")]
mod cells {
    use super::AtomicU64;
    pub static ROOT_HIT: AtomicU64 = AtomicU64::new(0);
    pub static ROOT_MISS: AtomicU64 = AtomicU64::new(0);
    pub static RADICAL_HIT: AtomicU64 = AtomicU64::new(0);
    pub static RADICAL_MISS: AtomicU64 = AtomicU64::new(0);
    pub static ABSCISSA_HIT: AtomicU64 = AtomicU64::new(0);
    pub static ABSCISSA_MISS: AtomicU64 = AtomicU64::new(0);
    pub static LOCKS: AtomicU64 = AtomicU64::new(0);
}

/// Record the outcome of a [`RealRoot`](crate::RealRoot) leaf ladder.
#[inline]
pub fn root(hit: bool) {
    #[cfg(feature = "metrics")]
    {
        let c = if hit {
            &cells::ROOT_HIT
        } else {
            &cells::ROOT_MISS
        };
        c.fetch_add(1, AtOrd::Relaxed);
    }
    let _ = hit;
}

/// Record the outcome of a radical sign ladder.
#[inline]
pub fn radical(hit: bool) {
    #[cfg(feature = "metrics")]
    {
        let c = if hit {
            &cells::RADICAL_HIT
        } else {
            &cells::RADICAL_MISS
        };
        c.fetch_add(1, AtOrd::Relaxed);
    }
    let _ = hit;
}

/// Record the outcome of a chart-geometry abscissa predicate.
#[inline]
pub fn abscissa(hit: bool) {
    #[cfg(feature = "metrics")]
    {
        let c = if hit {
            &cells::ABSCISSA_HIT
        } else {
            &cells::ABSCISSA_MISS
        };
        c.fetch_add(1, AtOrd::Relaxed);
    }
    let _ = hit;
}

/// Record one mutex acquisition on shared refinable root state.
#[inline]
pub fn lock() {
    #[cfg(feature = "metrics")]
    cells::LOCKS.fetch_add(1, AtOrd::Relaxed);
}

/// Read every counter. All zeros without the `metrics` feature.
pub fn snapshot() -> Counters {
    #[cfg(feature = "metrics")]
    {
        Counters {
            root_hit: cells::ROOT_HIT.load(AtOrd::Relaxed),
            root_miss: cells::ROOT_MISS.load(AtOrd::Relaxed),
            radical_hit: cells::RADICAL_HIT.load(AtOrd::Relaxed),
            radical_miss: cells::RADICAL_MISS.load(AtOrd::Relaxed),
            abscissa_hit: cells::ABSCISSA_HIT.load(AtOrd::Relaxed),
            abscissa_miss: cells::ABSCISSA_MISS.load(AtOrd::Relaxed),
            locks: cells::LOCKS.load(AtOrd::Relaxed),
        }
    }
    #[cfg(not(feature = "metrics"))]
    Counters::default()
}

/// Are the counters live? `false` unless the `metrics` feature is on.
pub fn enabled() -> bool {
    cfg!(feature = "metrics")
}

/// Zero every counter.
pub fn reset() {
    #[cfg(feature = "metrics")]
    {
        for c in [
            &cells::ROOT_HIT,
            &cells::ROOT_MISS,
            &cells::RADICAL_HIT,
            &cells::RADICAL_MISS,
            &cells::ABSCISSA_HIT,
            &cells::ABSCISSA_MISS,
            &cells::LOCKS,
        ] {
            c.store(0, AtOrd::Relaxed);
        }
    }
}
