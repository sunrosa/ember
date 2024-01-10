use std::ops::{Add, Deref, Div, Mul, Sub};

use num_traits::{Bounded, SaturatingAdd};

/// The error returned by some [`BoundedStat`] functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundedFloatError {
    /// Tried to set the [`current`](BoundedStat::current()) value to below zero.
    TooLow,
    /// Tried to set the [`current`](BoundedStat::current()) value above the [`max`](BoundedStat::max()).
    TooHigh,
    /// Tried to initialize the [`BoundedStat`] with a [`max`](BoundedStat::max()) below [`min`](BoundedStat::min()).
    InvalidBounds,
}

/// A [`f64`], with a configured maximum and minimum.
#[derive(Debug, Clone, Copy)]
pub struct BoundedFloat {
    /// The current value.
    current: f64,
    /// The minimum value.
    min: f64,
    /// The maximum value.
    max: f64,
}

impl BoundedFloat {
    /// Create a new [`BoundedFloat`]. If `current` is above `max`, this will return [`TooHigh`](BoundedStatError::TooHigh). If `current` is below [`min`](Self::min()), this will return [`TooLow`](BoundedStatError::TooLow). If `max` is below [`min`](Self::min()), this will return [`InvalidBounds`](BoundedStatError::InvalidBounds).
    ///
    /// # Parameters
    /// * `current` - The value to start with.
    /// * `min` - The minimum bound that the value cannot go beyond.
    /// * `max` - The maximum bound that the value cannot go beyond.
    pub fn new(current: f64, min: f64, max: f64) -> Result<Self, BoundedFloatError> {
        if max < min {
            return Err(BoundedFloatError::InvalidBounds);
        }
        if current < min {
            return Err(BoundedFloatError::TooLow);
        }
        if current > max {
            return Err(BoundedFloatError::TooHigh);
        }

        Ok(Self {
            current,
            max,
            min: min,
        })
    }

    /// Create a new [`BoundedFloat`] with a [`min`](Self::min()) of `0.0`. If `current` is above `max`, this will return [`TooHigh`](BoundedStatError::TooHigh). If `current` is below [`min`](Self::min()), this will return [`TooLow`](BoundedStatError::TooLow). If `max` is below [`min`](Self::min()), this will return [`InvalidBounds`](BoundedStatError::InvalidBounds).
    ///
    /// # Parameters
    /// * `current` - The value to start with.
    /// * `max` - The maximum bound that the value cannot go beyond.
    pub fn new_zero_min(current: f64, max: f64) -> Result<Self, BoundedFloatError> {
        let min = 0.0;

        Self::new(current, min, max)
    }

    /// Get the current value. This value is guaranteed to be above [`min`](Self::min()), and below [`max`](Self::max()).
    pub fn current(&self) -> f64 {
        self.current
    }

    /// Set the current value to `value`. Returns [`TooLow`](BoundedStatError::TooLow) if `value` is below [`min`](Self::min()), and [`TooHigh`](BoundedStatError::TooHigh) if `value` is above [`max`](Self::max).
    pub fn with_current(mut self, value: f64) -> Result<Self, BoundedFloatError> {
        if value < self.min() {
            return Err(BoundedFloatError::TooLow);
        }
        if value > self.max() {
            return Err(BoundedFloatError::TooHigh);
        }

        self.current = value;
        Ok(self)
    }

    /// Get the maximum value.
    pub fn max(&self) -> f64 {
        self.max
    }

    pub fn with_max(mut self, value: f64) -> Result<Self, BoundedFloatError> {
        if value < self.min() {
            return Err(BoundedFloatError::InvalidBounds);
        }

        self.max = value;
        Ok(self)
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn with_min(mut self, value: f64) -> Result<Self, BoundedFloatError> {
        if self.max() < value {
            return Err(BoundedFloatError::InvalidBounds);
        }

        self.min = value;
        Ok(self)
    }

    /// Set [`current`](Self::current) to `value`, without going below [`min`](Self::min()), or above [`max`](Self::max()).
    pub fn saturating_set(mut self, value: f64) -> Self {
        self = match self.with_current(value) {
            Ok(s) => s,
            Err(BoundedFloatError::TooLow) => self.with_current(self.min()).unwrap(),
            Err(BoundedFloatError::TooHigh) => self.with_current(self.max()).unwrap(),
            _ => unreachable!(),
        };
        self
    }

    /// Add `value` to [`current`](Self::current), without going beyond [`max`](Self::max()).
    fn saturating_add(mut self, value: f64) -> Self {
        self = self.saturating_set(self.current() + value);
        self
    }

    /// Subtract `value` from [`current`](Self::current), without going below [`min`](Self::min()).
    fn saturating_sub(mut self, value: f64) -> Self {
        self = self.saturating_set(self.current() - value);
        self
    }

    fn saturating_mul(mut self, value: f64) -> Self {
        self = self.saturating_set(self.current() * value);
        self
    }

    fn saturating_div(mut self, value: f64) -> Self {
        self = self.saturating_set(self.current() / value);
        self
    }
}

impl Deref for BoundedFloat {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl Add for BoundedFloat {
    type Output = BoundedFloat;

    fn add(self, rhs: Self) -> Self::Output {
        self.saturating_add(rhs.current())
    }
}

impl Add<f64> for BoundedFloat {
    type Output = BoundedFloat;

    fn add(self, rhs: f64) -> Self::Output {
        self.saturating_add(rhs)
    }
}

impl Sub for BoundedFloat {
    type Output = BoundedFloat;

    fn sub(self, rhs: Self) -> Self::Output {
        self.saturating_sub(rhs.current())
    }
}

impl Sub<f64> for BoundedFloat {
    type Output = BoundedFloat;

    fn sub(self, rhs: f64) -> Self::Output {
        self.saturating_sub(rhs)
    }
}

impl Mul for BoundedFloat {
    type Output = BoundedFloat;

    fn mul(self, rhs: Self) -> Self::Output {
        self.saturating_mul(rhs.current())
    }
}

impl Mul<f64> for BoundedFloat {
    type Output = BoundedFloat;

    fn mul(self, rhs: f64) -> Self::Output {
        self.saturating_mul(rhs)
    }
}

impl Div for BoundedFloat {
    type Output = BoundedFloat;

    fn div(self, rhs: Self) -> Self::Output {
        self.saturating_div(rhs.current())
    }
}

impl Div<f64> for BoundedFloat {
    type Output = BoundedFloat;

    fn div(self, rhs: f64) -> Self::Output {
        self.saturating_div(rhs)
    }
}

/// Get the weighted mean of a [`Vec`] of [`f64`] values together with [`f64`] weights.
///
/// # Returns
/// The weighted mean of the [`Vec`].
pub fn weighted_mean(data: Vec<(f64, f64)>) -> f64 {
    let mut sum = 0.0;
    let mut weighting_factor_sum = 0.0;

    for point in data {
        sum += point.0 * point.1;
        weighting_factor_sum += point.1;
    }

    sum / weighting_factor_sum
}

#[cfg(test)]
mod test {
    use super::*;

    mod bounded_stat {
        use super::*;

        #[test]
        fn new_max_below_zero() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, -1.0).err().unwrap(),
                BoundedFloatError::InvalidBounds
            );
        }

        #[test]
        fn new_too_low() {
            assert_eq!(
                BoundedFloat::new_zero_min(-1.0, 1.0).err().unwrap(),
                BoundedFloatError::TooLow
            );
        }

        #[test]
        fn new_too_high() {
            assert_eq!(
                BoundedFloat::new_zero_min(2.0, 1.0).err().unwrap(),
                BoundedFloatError::TooHigh
            )
        }

        #[test]
        fn saturating_add() {
            assert_eq!(
                BoundedFloat::new_zero_min(0.0, 2.0)
                    .unwrap()
                    .saturating_add(5.0)
                    .current(),
                2.0
            )
        }

        #[test]
        fn saturating_sub() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, 2.0)
                    .unwrap()
                    .saturating_sub(5.0)
                    .current(),
                0.0
            )
        }

        #[test]
        fn saturating_set() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, 5.0)
                    .unwrap()
                    .saturating_set(3.0)
                    .current(),
                3.0
            )
        }

        #[test]
        fn saturating_set_min_0() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, 5.0)
                    .unwrap()
                    .saturating_set(-1.0)
                    .current(),
                0.0
            )
        }

        #[test]
        fn saturating_set_min_1() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, 5.0)
                    .unwrap()
                    .saturating_set(0.0)
                    .current(),
                0.0
            )
        }

        #[test]
        fn saturating_set_max_0() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, 5.0)
                    .unwrap()
                    .saturating_set(10.0)
                    .current(),
                5.0
            )
        }

        #[test]
        fn saturating_set_max_1() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, 5.0)
                    .unwrap()
                    .saturating_set(5.0)
                    .current(),
                5.0
            )
        }
    }
}
