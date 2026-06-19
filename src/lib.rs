pub use circuit_breaker::{CircuitBreaker, State};
pub mod circuit_breaker {
    use std::time::{Duration, Instant};

    #[derive(Debug, Clone, Copy, PartialEq)]
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
    }

    impl CircuitBreaker {
        pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
            Self {
                state: State::Closed,
                failure_count: 0,
                failure_threshold,
                reset_timeout,
                last_failure_time: None,
            }
        }

        pub fn execute<F, T, E>(&mut self, operation: F) -> Result<T, &'static str>
        where
            F: FnOnce() -> Result<T, E>,
        {
            self.update_state();

            match self.state {
                State::Open => Err("Circuit is OPEN - Fast failing"),
                State::HalfOpen | State::Closed => {
                    match operation() {
                        Ok(result) => {
                            self.on_success();
                            Ok(result)
                        }
                        Err(_) => {
                            self.on_failure();
                            Err("Operation failed")
                        }
                    }
                }
            }
        }

        fn update_state(&mut self) {
            if self.state == State::Open {
                if let Some(last_fail) = self.last_failure_time {
                    if last_fail.elapsed() >= self.reset_timeout {
                        self.state = State::HalfOpen;
                    }
                }
            }
        }

        fn on_success(&mut self) {
            self.failure_count = 0;
            self.state = State::Closed;
        }

        fn on_failure(&mut self) {
            self.failure_count += 1;
            self.last_failure_time = Some(Instant::now());
            
            if self.failure_count >= self.failure_threshold {
                self.state = State::Open;
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::CircuitBreaker;
    use std::time::Duration;

    #[test]
    fn test_circuit_breaker_state_transitions() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(100));

        // Initial state
        assert_eq!(cb.state, crate::State::Closed);

        // Fail 1 - Should stay closed
        let _ = cb.execute(|| -> Result<(), ()> { Err(()) });
        assert_eq!(cb.state, crate::State::Closed);

        // Fail 2 - Should trip to Open
        let _ = cb.execute(|| -> Result<(), ()> { Err(()) });
        assert_eq!(cb.state, crate::State::Open);

        // Call while Open - Should fast fail
        let res = cb.execute(|| -> Result<(), ()> { Ok(()) });
        assert!(res.is_err());

        // Wait for reset timeout
        std::thread::sleep(Duration::from_millis(150));

        // Next call should test HalfOpen
        let _ = cb.execute(|| -> Result<(), ()> { Ok(()) });
        assert_eq!(cb.state, crate::State::Closed);
    }
}
