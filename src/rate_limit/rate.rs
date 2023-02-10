use std::time::Duration;

/// A rate of requests per time period.
#[derive(Debug, Copy, Clone)]
pub struct Rate {
    num: usize,
    per: Duration,
}

impl Rate {
    /// Create a new rate.
    ///
    /// # Panics
    ///
    /// This function panics if `num` or `per` is 0.
    pub fn new(num: usize, per: Duration) -> Self {
        assert!(num > 0);
        assert!(per > Duration::from_millis(0));

        Rate { num, per }
    }

    pub(crate) fn num(&self) -> usize {
        self.num
    }

    pub(crate) fn per(&self) -> Duration {
        self.per
    }
}