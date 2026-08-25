use serde::Serialize;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Success,
    Failure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Snapshot {
    pub state: State,
    pub failure_count: u32,
    pub failure_threshold: u32,
    pub half_open_probe_in_flight: bool,
}

#[derive(Debug)]
pub struct CircuitBreaker {
    state: State,
    failure_count: u32,
    failure_threshold: u32,
    reset_timeout: Duration,
    last_failure_time: Option<Instant>,
    half_open_probe_in_flight: bool,
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, reset_timeout: Duration) -> Result<Self, &'static str> {
        if failure_threshold == 0 {
            return Err("failure threshold must be positive");
        }
        if reset_timeout.is_zero() {
            return Err("reset timeout must be positive");
        }

        Ok(Self {
            state: State::Closed,
            failure_count: 0,
            failure_threshold,
            reset_timeout,
            last_failure_time: None,
            half_open_probe_in_flight: false,
        })
    }

    pub fn snapshot(&mut self) -> Snapshot {
        self.refresh_state();
        Snapshot {
            state: self.state,
            failure_count: self.failure_count,
            failure_threshold: self.failure_threshold,
            half_open_probe_in_flight: self.half_open_probe_in_flight,
        }
    }

    /// Reserves permission for one upstream attempt.
    ///
    /// In half-open state exactly one probe may be in flight. Callers must
    /// report that attempt through `record_outcome` before another probe can run.
    pub fn allow(&mut self) -> bool {
        self.refresh_state();
        match self.state {
            State::Open => false,
            State::Closed => true,
            State::HalfOpen if self.half_open_probe_in_flight => false,
            State::HalfOpen => {
                self.half_open_probe_in_flight = true;
                true
            }
        }
    }

    pub fn record_outcome(&mut self, outcome: Outcome) {
        match outcome {
            Outcome::Success => self.record_success(),
            Outcome::Failure => self.record_failure(),
        }
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.last_failure_time = None;
        self.half_open_probe_in_flight = false;
        self.state = State::Closed;
    }

    pub fn record_failure(&mut self) {
        self.failure_count = self.failure_count.saturating_add(1);
        self.last_failure_time = Some(Instant::now());
        self.half_open_probe_in_flight = false;
        if self.state == State::HalfOpen || self.failure_count >= self.failure_threshold {
            self.state = State::Open;
        }
    }

    fn refresh_state(&mut self) {
        if self.state != State::Open {
            return;
        }
        if self
            .last_failure_time
            .is_some_and(|instant| instant.elapsed() >= self.reset_timeout)
        {
            self.state = State::HalfOpen;
            self.half_open_probe_in_flight = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CircuitBreaker, Outcome, State};
    use std::time::Duration;

    #[test]
    fn rejects_invalid_configuration() {
        assert!(CircuitBreaker::new(0, Duration::from_secs(1)).is_err());
        assert!(CircuitBreaker::new(1, Duration::ZERO).is_err());
    }

    #[test]
    fn opens_after_threshold_and_recovers_with_one_probe() {
        let mut breaker = CircuitBreaker::new(2, Duration::from_millis(15)).unwrap();
        assert!(breaker.allow());
        breaker.record_outcome(Outcome::Failure);
        assert_eq!(breaker.snapshot().state, State::Closed);

        assert!(breaker.allow());
        breaker.record_outcome(Outcome::Failure);
        assert_eq!(breaker.snapshot().state, State::Open);
        assert!(!breaker.allow());

        std::thread::sleep(Duration::from_millis(20));
        assert!(breaker.allow());
        assert_eq!(breaker.snapshot().state, State::HalfOpen);
        assert!(!breaker.allow());

        breaker.record_outcome(Outcome::Success);
        let snapshot = breaker.snapshot();
        assert_eq!(snapshot.state, State::Closed);
        assert_eq!(snapshot.failure_count, 0);
    }

    #[test]
    fn failed_half_open_probe_reopens_immediately() {
        let mut breaker = CircuitBreaker::new(1, Duration::from_millis(5)).unwrap();
        assert!(breaker.allow());
        breaker.record_failure();
        std::thread::sleep(Duration::from_millis(8));
        assert!(breaker.allow());
        breaker.record_failure();
        assert_eq!(breaker.snapshot().state, State::Open);
    }

    #[test]
    fn success_clears_partial_failures() {
        let mut breaker = CircuitBreaker::new(3, Duration::from_secs(1)).unwrap();
        breaker.record_failure();
        assert_eq!(breaker.snapshot().failure_count, 1);
        breaker.record_success();
        assert_eq!(breaker.snapshot().failure_count, 0);
    }
}
