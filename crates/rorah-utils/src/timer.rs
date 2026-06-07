//! Timing utilities for benchmarking.

use std::time::{Duration, Instant};

/// Simple timer for measuring execution time.
pub struct Timer {
    start: Instant,
    label: String,
}

impl Timer {
    /// Create and start a new timer.
    pub fn start(label: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            label: label.into(),
        }
    }

    /// Get elapsed time.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Stop timer and print elapsed time.
    pub fn stop(self) {
        let elapsed = self.elapsed();
        println!("[{}] took {:?}", self.label, elapsed);
    }

    /// Stop and return elapsed time in milliseconds.
    pub fn stop_ms(self) -> u128 {
        self.elapsed().as_millis()
    }
}

/// Measure execution time of a function.
pub fn time_it<F, R>(label: &str, f: F) -> (R, Duration)
where
    F: FnOnce() -> R,
{
    let timer = Timer::start(label);
    let result = f();
    let elapsed = timer.elapsed();
    (result, elapsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_timer() {
        let timer = Timer::start("test");
        sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();

        assert!(elapsed.as_millis() >= 10);
    }

    #[test]
    fn test_time_it() {
        let (result, elapsed) = time_it("computation", || {
            sleep(Duration::from_millis(5));
            42
        });

        assert_eq!(result, 42);
        assert!(elapsed.as_millis() >= 5);
    }
}