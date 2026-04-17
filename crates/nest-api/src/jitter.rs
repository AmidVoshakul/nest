//! Random jitter for timing side channel attack prevention
//!
//! Adds controlled random delays to all operations that could leak timing
//! information to attackers.

use rand::rngs::OsRng;
use rand::Rng;
use std::time::Duration;

/// Jitter configuration
#[derive(Debug, Clone, Copy)]
pub struct JitterConfig {
    /// Minimum jitter delay in milliseconds
    pub min_ms: u64,

    /// Maximum jitter delay in milliseconds
    pub max_ms: u64,

    /// Enable jitter
    pub enabled: bool,
}

impl Default for JitterConfig {
    fn default() -> Self {
        Self {
            min_ms: 0,
            max_ms: 50,
            enabled: true,
        }
    }
}

/// Timing jitter generator to prevent timing side channel attacks
#[derive(Debug, Default)]
pub struct JitterGenerator {
    config: JitterConfig,
    rng: OsRng,
}

impl JitterGenerator {
    /// Create new jitter generator with default configuration
    pub fn new() -> Self {
        Self::with_config(JitterConfig::default())
    }

    /// Create new jitter generator with custom configuration
    pub fn with_config(config: JitterConfig) -> Self {
        Self { config, rng: OsRng }
    }

    /// Generate a random delay duration
    pub fn generate_delay(&mut self) -> Duration {
        if !self.config.enabled {
            return Duration::ZERO;
        }

        let delay = self.rng.gen_range(self.config.min_ms..=self.config.max_ms);
        Duration::from_millis(delay)
    }

    /// Add random jitter to an existing duration
    pub fn add_jitter(&mut self, duration: Duration) -> Duration {
        if !self.config.enabled {
            return duration;
        }

        duration + self.generate_delay()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jitter_generation() {
        let mut jitter = JitterGenerator::new();

        for _ in 0..100 {
            let delay = jitter.generate_delay();
            assert!(delay <= Duration::from_millis(50));
            assert!(delay >= Duration::from_millis(0));
        }
    }

    #[test]
    fn test_disabled_jitter() {
        let mut jitter = JitterGenerator::with_config(JitterConfig {
            enabled: false,
            ..Default::default()
        });

        assert_eq!(jitter.generate_delay(), Duration::ZERO);
    }

    #[test]
    fn test_add_jitter_to_duration() {
        let mut jitter = JitterGenerator::new();
        let base = Duration::from_millis(100);

        let total = jitter.add_jitter(base);
        assert!(total >= base);
        assert!(total <= base + Duration::from_millis(50));
    }
}
