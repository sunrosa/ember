/// The error returned by some [`BoundedStat`] functions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundedStatError {
    /// Tried to set the [`current`](BoundedStat::current()) value to below zero.
    TooLow,
    /// Tried to set the [`current`](BoundedStat::current()) value above the [`max`](BoundedStat::max()).
    TooHigh,
    /// Tried to initialize the [`BoundedStat`] with a [`max`](BoundedStat::max()) below zero.
    MaxBelowZero,
}

/// A statistic, such as hit points, with a configured maximum, and a minimum of zero.
#[derive(Debug, Clone, Copy)]
pub struct BoundedStat {
    /// The current value.
    current: f64,
    /// The maximum value.
    max: f64,
}

impl BoundedStat {
    /// Create a new [`BoundedStat`]. If `current` is above `max`, this will return [`TooHigh`](BoundedStatError::TooHigh). If `current` is below `0.0`, this will return [`TooLow`](BoundedStatError::TooLow). If `max` is below `0.0`, this will return [`MaxBelowZero`](BoundedStatError::MaxBelowZero).
    pub fn new(current: f64, max: f64) -> Result<Self, BoundedStatError> {
        if max < 0.0 {
            return Err(BoundedStatError::MaxBelowZero);
        }
        if current < 0.0 {
            return Err(BoundedStatError::TooLow);
        }
        if current > max {
            return Err(BoundedStatError::TooHigh);
        }

        Ok(Self { current, max })
    }

    /// Get the current value. This value is guaranteed to be above `0.0`, and below [`max`](Self::max).
    pub fn current(&self) -> f64 {
        self.current
    }

    /// Set the current value to `value`. Returns [`TooLow`](BoundedStatError::TooLow) if `value` is below `0.0`, and [`TooHigh`](BoundedStatError::TooHigh) if `value` is above [`max`](Self::max).
    pub fn with_current(mut self, value: f64) -> Result<Self, BoundedStatError> {
        if value < 0.0 {
            return Err(BoundedStatError::TooLow);
        }
        if value > self.max() {
            return Err(BoundedStatError::TooHigh);
        }

        self.current = value;
        Ok(self)
    }

    /// Get the maximum value.
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Set [`current`](Self::current) to `value`, without going below `0.0`, or above [`max`](Self::max).
    pub fn saturating_set(mut self, value: f64) -> Self {
        self = match self.with_current(value) {
            Ok(s) => s,
            Err(BoundedStatError::TooLow) => self.with_current(0.0).unwrap(),
            Err(BoundedStatError::TooHigh) => self.with_current(self.max()).unwrap(),
            _ => unreachable!(),
        };
        self
    }

    /// Add `value` to [`current`](Self::current), without going beyond [`max`](Self::max).
    pub fn saturating_add(mut self, value: f64) -> Self {
        self = self.saturating_set(self.current() + value);
        self
    }

    /// Subtract `value` from [`current`](Self::current), without going below `0.0`.
    pub fn saturating_sub(mut self, value: f64) -> Self {
        self = self.saturating_set(self.current() - value);
        self
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
                BoundedStat::new(1.0, -1.0).err().unwrap(),
                BoundedStatError::MaxBelowZero
            );
        }

        #[test]
        fn new_too_low() {
            assert_eq!(
                BoundedStat::new(-1.0, 1.0).err().unwrap(),
                BoundedStatError::TooLow
            );
        }

        #[test]
        fn new_too_high() {
            assert_eq!(
                BoundedStat::new(2.0, 1.0).err().unwrap(),
                BoundedStatError::TooHigh
            )
        }

        #[test]
        fn saturating_add() {
            assert_eq!(
                BoundedStat::new(0.0, 2.0)
                    .unwrap()
                    .saturating_add(5.0)
                    .current(),
                2.0
            )
        }

        #[test]
        fn saturating_sub() {
            assert_eq!(
                BoundedStat::new(1.0, 2.0)
                    .unwrap()
                    .saturating_sub(5.0)
                    .current(),
                0.0
            )
        }

        #[test]
        fn saturating_set() {
            assert_eq!(
                BoundedStat::new(1.0, 5.0)
                    .unwrap()
                    .saturating_set(3.0)
                    .current(),
                3.0
            )
        }

        #[test]
        fn saturating_set_min_0() {
            assert_eq!(
                BoundedStat::new(1.0, 5.0)
                    .unwrap()
                    .saturating_set(-1.0)
                    .current(),
                0.0
            )
        }

        #[test]
        fn saturating_set_min_1() {
            assert_eq!(
                BoundedStat::new(1.0, 5.0)
                    .unwrap()
                    .saturating_set(0.0)
                    .current(),
                0.0
            )
        }

        #[test]
        fn saturating_set_max_0() {
            assert_eq!(
                BoundedStat::new(1.0, 5.0)
                    .unwrap()
                    .saturating_set(10.0)
                    .current(),
                5.0
            )
        }

        #[test]
        fn saturating_set_max_1() {
            assert_eq!(
                BoundedStat::new(1.0, 5.0)
                    .unwrap()
                    .saturating_set(5.0)
                    .current(),
                5.0
            )
        }
    }
}
