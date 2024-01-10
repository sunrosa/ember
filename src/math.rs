use std::ops::{Add, Deref, Div, Mul, Sub};

use thiserror::Error;

/// The error returned by some [`BoundedFloat`] functions.
#[derive(Debug, Clone, Copy, Error)]
pub enum BoundedFloatError {
    /// Tried to set the [`current`](BoundedFloat::current()) value to below the [`min`](BoundedFloat::min()).
    ///
    /// # Parameters
    /// * `current` - The current value attempted to be set
    /// * `minimum` - The minimum value
    #[error("Tried to set the current value ({cur}) below minimum ({min})")]
    TooLow { cur: f64, min: f64 },

    /// Tried to set the [`current`](BoundedFloat::current()) value above the [`max`](BoundedFloat::max()).
    ///
    /// # Parameters
    /// * `current` - The current value attempted to be set
    /// * `maximum` - The maximum value
    #[error("Tried to set the current value ({cur}) above maximum ({max})")]
    TooHigh { cur: f64, max: f64 },

    /// Tried to initialize the [`BoundedFloat`] with a [`max`](BoundedFloat::max()) below [`min`](BoundedFloat::min()).
    ///
    /// # Parameters
    /// * `minimum` - The minimum value
    /// * `maximum` - The maximum value
    #[error("Tried to set the maximum ({max}) below the minimum ({min})")]
    InvalidBounds { min: f64, max: f64 },
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
    /// Create a new [`BoundedFloat`]. If `current` is above `max`, this will return [`TooHigh`](BoundedFloatError::TooHigh). If `current` is below [`min`](Self::min()), this will return [`TooLow`](BoundedFloatError::TooLow). If `max` is below [`min`](Self::min()), this will return [`InvalidBounds`](BoundedFloatError::InvalidBounds).
    ///
    /// # Parameters
    /// * `current` - The value to start with.
    /// * `min` - The minimum bound that the value cannot go beyond.
    /// * `max` - The maximum bound that the value cannot go beyond.
    pub fn new(current: f64, min: f64, max: f64) -> Result<Self, BoundedFloatError> {
        if max < min {
            return Err(BoundedFloatError::InvalidBounds { max, min });
        }
        if current < min {
            return Err(BoundedFloatError::TooLow { cur: current, min });
        }
        if current > max {
            return Err(BoundedFloatError::TooHigh { cur: current, max });
        }

        Ok(Self { current, max, min })
    }

    /// Create a new [`BoundedFloat`] with a [`min`](Self::min()) of `0.0`. If `current` is above `max`, this will return [`TooHigh`](BoundedFloatError::TooHigh). If `current` is below [`min`](Self::min()), this will return [`TooLow`](BoundedFloatError::TooLow). If `max` is below [`min`](Self::min()), this will return [`InvalidBounds`](BoundedFloatError::InvalidBounds).
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

    /// Set the current value to `value`. Returns [`TooLow`](BoundedFloatError::TooLow) if `value` is below [`min`](Self::min()), and [`TooHigh`](BoundedFloatError::TooHigh) if `value` is above [`max`](Self::max).
    pub fn with_current(mut self, value: f64) -> Result<Self, BoundedFloatError> {
        if value < self.min() {
            return Err(BoundedFloatError::TooLow {
                cur: value,
                min: self.min(),
            });
        }
        if value > self.max() {
            return Err(BoundedFloatError::TooHigh {
                cur: value,
                max: self.max(),
            });
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
            return Err(BoundedFloatError::InvalidBounds {
                max: self.max(),
                min: self.min(),
            });
        }

        self.max = value;
        Ok(self)
    }

    pub fn min(&self) -> f64 {
        self.min
    }

    pub fn with_min(mut self, value: f64) -> Result<Self, BoundedFloatError> {
        if self.max() < value {
            return Err(BoundedFloatError::InvalidBounds {
                max: self.max(),
                min: self.min(),
            });
        }

        self.min = value;
        Ok(self)
    }

    /// Set [`current`](Self::current) to `value`, without going below [`min`](Self::min()), or above [`max`](Self::max()).
    pub fn saturating_set(mut self, value: f64) -> Self {
        self = match self.with_current(value) {
            Ok(s) => s,
            Err(BoundedFloatError::TooLow { cur: _, min: _ }) => {
                self.with_current(self.min()).unwrap()
            }
            Err(BoundedFloatError::TooHigh { cur: _, max: _ }) => {
                self.with_current(self.max()).unwrap()
            }
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

impl PartialEq<f64> for BoundedFloat {
    fn eq(&self, other: &f64) -> bool {
        self.current() == *other
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

    use std::assert_matches::assert_matches;

    mod bounded_stat {
        use super::*;

        #[test]
        fn new_max_below_zero() {
            assert_matches!(
                BoundedFloat::new_zero_min(1.0, -1.0).err().unwrap(),
                BoundedFloatError::InvalidBounds { max: _, min: _ }
            );
        }

        #[test]
        fn new_too_low() {
            assert_matches!(
                BoundedFloat::new_zero_min(-1.0, 1.0).err().unwrap(),
                BoundedFloatError::TooLow { cur: _, min: _ }
            );
        }

        #[test]
        fn new_too_high() {
            assert_matches!(
                BoundedFloat::new_zero_min(2.0, 1.0).err().unwrap(),
                BoundedFloatError::TooHigh { cur: _, max: _ }
            )
        }

        #[test]
        fn saturating_add() {
            assert_eq!(BoundedFloat::new_zero_min(0.0, 2.0).unwrap() + 1.2, 1.2)
        }

        #[test]
        fn saturating_add_max() {
            assert_eq!(BoundedFloat::new_zero_min(0.0, 2.0).unwrap() + 5.0, 2.0)
        }

        #[test]
        fn saturating_sub() {
            assert_eq!(BoundedFloat::new_zero_min(1.0, 2.0).unwrap() - 0.5, 0.5)
        }

        #[test]
        fn saturating_sub_min() {
            assert_eq!(BoundedFloat::new_zero_min(1.0, 2.0).unwrap() - 5.0, 0.0)
        }

        #[test]
        fn saturating_mul() {
            assert_eq!(BoundedFloat::new_zero_min(3.0, 20.0).unwrap() * 4.0, 12.0);
        }

        #[test]
        fn saturating_mul_max() {
            assert_eq!(BoundedFloat::new_zero_min(3.0, 10.0).unwrap() * 4.0, 10.0);
        }

        #[test]
        fn saturating_div() {
            assert_eq!(BoundedFloat::new_zero_min(12.0, 20.0).unwrap() / 3.0, 4.0);
        }

        #[test]
        fn saturating_set() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, 5.0)
                    .unwrap()
                    .saturating_set(3.0),
                3.0
            )
        }

        #[test]
        fn saturating_set_min_0() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, 5.0)
                    .unwrap()
                    .saturating_set(-1.0),
                0.0
            )
        }

        #[test]
        fn saturating_set_min_1() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, 5.0)
                    .unwrap()
                    .saturating_set(0.0),
                0.0
            )
        }

        #[test]
        fn saturating_set_max_0() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, 5.0)
                    .unwrap()
                    .saturating_set(10.0),
                5.0
            )
        }

        #[test]
        fn saturating_set_max_1() {
            assert_eq!(
                BoundedFloat::new_zero_min(1.0, 5.0)
                    .unwrap()
                    .saturating_set(5.0),
                5.0
            )
        }
    }
}
