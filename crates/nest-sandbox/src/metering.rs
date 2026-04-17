//! Dual execution metering system
//!
//! Implements two independent metering mechanisms:
//! 1. Fuel metering - deterministic instruction count
//! 2. Epoch interruption - wall clock timeout
//!
//! Both run simultaneously, each catches what the other misses.

use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct MeteringConfig {
    /// Maximum number of WASM instructions allowed
    pub fuel_limit: u64,

    /// Maximum wall clock execution time
    pub timeout_secs: u64,
}

impl Default for MeteringConfig {
    fn default() -> Self {
        Self {
            fuel_limit: 1_000_000,
            timeout_secs: 30,
        }
    }
}

/// Dual metering system
#[derive(Debug)]
pub struct DualMeter {
    config: MeteringConfig,
    fuel_consumed: u64,
    start_time: std::time::Instant,
}

impl Default for DualMeter {
    fn default() -> Self {
        Self::new()
    }
}

impl DualMeter {
    /// Create new dual meter with default configuration
    pub fn new() -> Self {
        Self::with_config(MeteringConfig::default())
    }

    /// Create new dual meter with custom configuration
    pub fn with_config(config: MeteringConfig) -> Self {
        Self {
            config,
            fuel_consumed: 0,
            start_time: std::time::Instant::now(),
        }
    }

    /// Consume fuel, return true if limit exceeded
    pub fn consume_fuel(&mut self, amount: u64) -> bool {
        self.fuel_consumed += amount;
        self.fuel_consumed >= self.config.fuel_limit
    }

    /// Check if timeout has been exceeded
    pub fn check_timeout(&self) -> bool {
        self.start_time.elapsed() >= Duration::from_secs(self.config.timeout_secs)
    }

    /// Check if execution should be terminated
    pub fn should_terminate(&self) -> bool {
        self.fuel_consumed >= self.config.fuel_limit || self.check_timeout()
    }

    /// Get current fuel consumption
    pub fn fuel_consumed(&self) -> u64 {
        self.fuel_consumed
    }

    /// Get elapsed time
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Reset meter for new execution
    pub fn reset(&mut self) {
        self.fuel_consumed = 0;
        self.start_time = std::time::Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_fuel_metering() {
        let mut meter = DualMeter::with_config(MeteringConfig {
            fuel_limit: 1000,
            timeout_secs: 10,
        });

        assert!(!meter.consume_fuel(500));
        assert_eq!(meter.fuel_consumed(), 500);

        assert!(!meter.consume_fuel(499));
        assert_eq!(meter.fuel_consumed(), 999);

        assert!(meter.consume_fuel(1));
        assert_eq!(meter.fuel_consumed(), 1000);
        assert!(meter.should_terminate());
    }

    #[test]
    fn test_timeout_metering() {
        let meter = DualMeter::with_config(MeteringConfig {
            fuel_limit: 1_000_000,
            timeout_secs: 1,
        });

        assert!(!meter.check_timeout());

        sleep(Duration::from_secs(2));

        assert!(meter.check_timeout());
        assert!(meter.should_terminate());
    }

    #[test]
    fn test_reset() {
        let mut meter = DualMeter::new();

        meter.consume_fuel(1000);
        sleep(Duration::from_millis(100));

        assert_eq!(meter.fuel_consumed(), 1000);
        assert!(meter.elapsed() > Duration::default());

        meter.reset();

        assert_eq!(meter.fuel_consumed(), 0);
        assert!(meter.elapsed() < Duration::from_millis(10));
    }
}
