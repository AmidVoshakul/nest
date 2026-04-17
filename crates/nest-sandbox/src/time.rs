//! Time normalization and high precision timer blocking for sandbox environments
//!
//! Prevents microtiming attacks and side channel exploitation by normalizing
//! all observed time inside the sandbox.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Time normalization configuration
#[derive(Debug, Clone, Copy)]
pub struct TimeNormalizationConfig {
    /// Granularity of time observations in milliseconds
    pub granularity_ms: u64,
    
    /// Enable time normalization
    pub enabled: bool,
}

impl Default for TimeNormalizationConfig {
    fn default() -> Self {
        Self {
            granularity_ms: 1000, // 1 second granularity
            enabled: true,
        }
    }
}

/// Time normalizer that prevents high precision timing attacks
#[derive(Debug, Clone)]
pub struct TimeNormalizer {
    config: TimeNormalizationConfig,
    base_time: Instant,
    base_system_time: SystemTime,
}

impl Default for TimeNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TimeNormalizer {
    /// Create new time normalizer with default configuration
    pub fn new() -> Self {
        Self::with_config(TimeNormalizationConfig::default())
    }
    
    /// Create new time normalizer with custom configuration
    pub fn with_config(config: TimeNormalizationConfig) -> Self {
        Self {
            config,
            base_time: Instant::now(),
            base_system_time: SystemTime::now(),
        }
    }
    
    /// Get normalized monotonic time
    pub fn now(&self) -> Instant {
        if !self.config.enabled {
            return Instant::now();
        }
        
        let elapsed = self.base_time.elapsed();
        let normalized_ms = (elapsed.as_millis() as u64 / self.config.granularity_ms) * self.config.granularity_ms;
        self.base_time + Duration::from_millis(normalized_ms)
    }
    
    /// Get normalized system time
    pub fn system_time(&self) -> SystemTime {
        if !self.config.enabled {
            return SystemTime::now();
        }
        
        let elapsed = self.base_system_time.elapsed().unwrap_or_default();
        let normalized_ms = (elapsed.as_millis() as u64 / self.config.granularity_ms) * self.config.granularity_ms;
        self.base_system_time + Duration::from_millis(normalized_ms)
    }
    
    /// Get normalized duration since epoch
    pub fn unix_time(&self) -> Duration {
        self.system_time()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
    }
    
    /// Get current time in seconds since epoch (normalized)
    pub fn unix_time_seconds(&self) -> u64 {
        self.unix_time().as_secs()
    }
    
    /// Sleep for at least the specified duration (normalized)
    pub async fn sleep(&self, duration: Duration) {
        if !self.config.enabled {
            tokio::time::sleep(duration).await;
            return;
        }
        
        // Normalize sleep duration to granularity
        let ms = duration.as_millis() as u64;
        let normalized_ms = ((ms + self.config.granularity_ms - 1) / self.config.granularity_ms) * self.config.granularity_ms;
        
        tokio::time::sleep(Duration::from_millis(normalized_ms)).await;
    }
}

/// Global time normalization for sandbox environments
pub mod global {
    use super::*;
    use std::sync::OnceLock;
    
    static GLOBAL_NORMALIZER: OnceLock<TimeNormalizer> = OnceLock::new();
    
    /// Initialize global time normalizer
    pub fn init(config: TimeNormalizationConfig) {
        GLOBAL_NORMALIZER.get_or_init(|| TimeNormalizer::with_config(config));
    }
    
    /// Get global normalized time
    pub fn now() -> Instant {
        GLOBAL_NORMALIZER.get()
            .map(|n| n.now())
            .unwrap_or_else(Instant::now)
    }
    
    /// Get global normalized system time
    pub fn system_time() -> SystemTime {
        GLOBAL_NORMALIZER.get()
            .map(|n| n.system_time())
            .unwrap_or_else(SystemTime::now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_time_normalization_granularity() {
        let normalizer = TimeNormalizer::with_config(TimeNormalizationConfig {
            granularity_ms: 1000,
            enabled: true,
        });
        
        // Time should always be rounded to nearest second
        let now = normalizer.now();
        let elapsed = now.duration_since(normalizer.base_time).as_millis();
        
        assert_eq!(elapsed % 1000, 0);
    }

    #[test]
    fn test_disabled_normalization() {
        let normalizer = TimeNormalizer::with_config(TimeNormalizationConfig {
            granularity_ms: 1000,
            enabled: false,
        });
        
        let now1 = normalizer.now();
        std::thread::sleep(Duration::from_millis(100));
        let now2 = normalizer.now();
        
        // Should see accurate time when disabled
        assert!(now2 > now1);
    }

    #[test]
    fn test_unix_time_normalization() {
        let normalizer = TimeNormalizer::with_config(TimeNormalizationConfig {
            granularity_ms: 1000,
            enabled: true,
        });
        
        let time = normalizer.unix_time();
        assert_eq!(time.subsec_millis(), 0);
    }
}
