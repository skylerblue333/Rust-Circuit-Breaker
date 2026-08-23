pub use circuit_breaker::{CircuitBreaker, State};

pub mod circuit_breaker {
    use std::time::{Duration, Instant};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum State {
        Closed,
        Open,
        HalfOpen,
    }

    pub struct CircuitBreaker {
        pub state: State,
        pub failure_count: u32,
        pub failure_threshold: u32,
        pub reset_timeout: Duration,
        pub last_failure_time: Option<Instant>,
        half_open_in_flight: bool,
    }

    impl CircuitBreaker {
        pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
            assert!(failure_threshold > 0, "failure threshold must be positive");
            Self {
                state: State::Closed,
                failure_count: 0,
                failure_threshold,
                reset_timeout,
                last_failure_time: None,
                half_open_in_flight: false,
            }
        }

        /// Returns true when an operation may start.
        ///
        /// The state lock can be held briefly by callers, allowing the actual
        /// upstream operation to run without blocking other requests.
        pub fn allow(&mut self) -> bool {
            self.update_state();
            match self.state {
                State::Open => false,
                State::Closed => true,
                State::HalfOpen => {
                    if self.half_open_in_flight {
                        false
                    } else {
                        self.half_open_in_flight = true;
                        true
                    }
                }
            }
        }

        pub fn record_success(&mut self) {
            self.failure_count = 0;
            self.last_failure_time = None;
            self.half_open_in_flight = false;
            self.state = State::Closed;
        }

        pub fn record_failure(&mut self) {
            self.failure_count = self.failure_count.saturating_add(1);
            self.last_failure_time = Some(Instant::now());
            self.half_open_in_flight = false;
            if self.failure_count >= self.failure_threshold {
                self.state = State::Open;
            }
        }

        pub fn execute<F, T, E>(&mut self, operation: F) -> Result<T, &'static str>
        where
            F: FnOnce() -> Result<T, E>,
        {
            if !self.allow() {
                return Err("Circuit is OPEN - Fast failing");
            }

            match operation() {
                Ok(result) => {
                    self.record_success();
                    Ok(result)
                }
                Err(_) => {
                    self.record_failure();
                    Err("Operation failed")
                }
            }
        }

        fn update_state(&mut self) {
            if self.state == State::Open {
                if let Some(last_fail) = self.last_failure_time {
                    if last_fail.elapsed() >= self.reset_timeout {
                        self.state = State::HalfOpen;
                        self.half_open_in_flight = false;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{CircuitBreaker, State};
    use std::time::Duration;

    #[test]
    fn test_circuit_breaker_state_transitions() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(20));
        assert_eq!(cb.state, State::Closed);

        assert!(!cb.execute(|| -> Result<(), ()> { Err(()) }).is_ok());
        assert_eq!(cb.state, State::Closed);

        assert!(cb.execute(|| -> Result<(), ()> { Err(()) }).is_err());
        assert_eq!(cb.state, State::Open);

        assert!(cb.execute(|| -> Result<(), ()> { Ok(()) }).is_err());
        std::thread::sleep(Duration::from_millis(30));

        assert!(cb.allow());
        assert_eq!(cb.state, State::HalfOpen);
        assert!(!cb.allow(), "only one half-open probe may run");
        cb.record_success();
        assert_eq!(cb.state, State::Closed);
    }

    #[test]
    #[should_panic(expected = "failure threshold must be positive")]
    fn rejects_zero_threshold() {
        let _ = CircuitBreaker::new(0, Duration::from_secs(1));
    }

    #[test]
    fn success_resets_failure_count() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(1));
        let _ = cb.execute(|| -> Result<(), ()> { Err(()) });
        assert_eq!(cb.failure_count, 1);
        let _ = cb.execute(|| -> Result<(), ()> { Ok(()) });
        assert_eq!(cb.failure_count, 0);
        assert_eq!(cb.state, State::Closed);
    }
}
