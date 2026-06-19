use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq)]
enum State {
    Closed,
    Open,
    HalfOpen,
}

struct CircuitBreaker {
    state: State,
    failure_threshold: u32,
    failure_count: u32,
    reset_timeout: Duration,
    last_failure_time: Option<Instant>,
}

impl CircuitBreaker {
    fn new(failure_threshold: u32, reset_timeout: Duration) -> Self {
        CircuitBreaker {
            state: State::Closed,
            failure_threshold,
            failure_count: 0,
            reset_timeout,
            last_failure_time: None,
        }
    }

    fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = State::Closed;
    }

    fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());
        if self.failure_count >= self.failure_threshold {
            self.state = State::Open;
        }
    }

    fn allow_request(&mut self) -> bool {
        match self.state {
            State::Closed => true,
            State::Open => {
                if let Some(time) = self.last_failure_time {
                    if time.elapsed() > self.reset_timeout {
                        self.state = State::HalfOpen;
                        return true;
                    }
                }
                false
            }
            State::HalfOpen => true,
        }
    }
}

fn main() {
    let cb = Arc::new(Mutex::new(CircuitBreaker::new(3, Duration::from_secs(5))));
    println!("Circuit breaker initialized.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_flow() {
        let mut cb = CircuitBreaker::new(2, Duration::from_millis(100));
        
        assert!(cb.allow_request());
        cb.record_failure();
        
        assert!(cb.allow_request());
        cb.record_failure();
        
        // Threshold reached, should be Open
        assert_eq!(cb.state, State::Open);
        assert!(!cb.allow_request());
        
        // Wait for timeout
        std::thread::sleep(Duration::from_millis(150));
        
        // Should transition to HalfOpen and allow request
        assert!(cb.allow_request());
        assert_eq!(cb.state, State::HalfOpen);
        
        // Success resets to Closed
        cb.record_success();
        assert_eq!(cb.state, State::Closed);
    }
}
