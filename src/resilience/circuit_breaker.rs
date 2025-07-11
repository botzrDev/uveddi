use crate::error::RenderingServiceError;
use std::time::{Duration, Instant};

/// Represents the state of a circuit breaker
#[derive(Debug, PartialEq, Eq)]
pub enum State {
    /// Circuit is closed and requests are allowed
    Closed,
    /// Circuit is open and requests are blocked
    Open {
        /// Timestamp when the circuit was opened
        opened_at: Instant,
    },
    /// Circuit is half-open, allowing limited requests to test recovery
    HalfOpen,
}

/// Circuit breaker implementation for handling service failures
pub struct CircuitBreaker {
    /// Current state of the circuit breaker
    state: State,
    /// Number of consecutive failures required to open the circuit
    failure_threshold: usize,
    /// Duration to keep the circuit open before allowing test requests
    reset_timeout: Duration,
    /// Count of consecutive failures that triggered circuit opening
    consecutive_failures: usize,
}

impl CircuitBreaker {
    /// Creates a new CircuitBreaker with specified failure threshold and reset timeout
    pub fn new(failure_threshold: usize, reset_timeout: Duration) -> Self {
        CircuitBreaker {
            state: State::Closed,
            failure_threshold,
            reset_timeout,
            consecutive_failures: 0,
        }
    }

    /// Records a failure and updates circuit state based on error severity
    pub fn record_failure(&mut self, error: &RenderingServiceError) {
        // UV-172: Only count errors that should open the circuit
        if error.should_open_circuit() {
            self.consecutive_failures += 1;
        }

        // Transition to Open state if threshold is reached
        if self.consecutive_failures >= self.failure_threshold {
            self.state = State::Open {
                opened_at: Instant::now(),
            };
        }
    }

    /// Records a successful request and resets failure count
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.state = State::Closed;
    }

    /// Checks if requests are allowed based on current circuit state
    pub fn allow_request(&self) -> bool {
        match &self.state {
            State::Closed => true,
            State::Open { opened_at } => {
                // Allow requests after reset timeout expires
                opened_at.elapsed() >= self.reset_timeout
            }
            State::HalfOpen => {
                // Allow limited requests in HalfOpen state
                true
            }
        }
    }

    /// Returns true if circuit is closed
    pub fn is_closed(&self) -> bool {
        matches!(self.state, State::Closed)
    }

    /// Returns true if circuit is open
    pub fn is_open(&self) -> bool {
        matches!(self.state, State::Open { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::RenderingServiceError;

    // Create mock critical error (should open circuit)
    fn critical_error() -> RenderingServiceError {
        RenderingServiceError::ServiceUnavailable
    }

    // Create mock non-critical error (should not open circuit)
    fn non_critical_error() -> RenderingServiceError {
        RenderingServiceError::InvalidMermaidSyntax {
            line: None,
            details: "Non-critical failure".to_string(),
        }
    }

    #[test]
    fn should_not_open_circuit_on_non_critical_errors() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(10));

        // Record multiple non-critical errors
        for _ in 0..5 {
            cb.record_failure(&non_critical_error());
        }

        assert!(cb.is_closed(), "Circuit should remain closed");
    }

    #[test]
    fn should_open_circuit_on_critical_errors() {
        let mut cb = CircuitBreaker::new(3, Duration::from_secs(10));

        // Record critical errors
        cb.record_failure(&critical_error());
        cb.record_failure(&critical_error());
        assert!(cb.is_closed(), "Should remain closed before threshold");

        cb.record_failure(&critical_error());
        assert!(cb.is_open(), "Should open after reaching threshold");
    }

    #[test]
    fn should_reset_after_success() {
        let mut cb = CircuitBreaker::new(2, Duration::from_secs(10));

        // Record failures
        cb.record_failure(&critical_error());
        cb.record_failure(&critical_error());
        assert!(cb.is_open(), "Circuit should be open");

        // Record success
        cb.record_success();
        assert!(cb.is_closed(), "Circuit should close after success");
    }
}
