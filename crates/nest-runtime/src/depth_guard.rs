//! Maximum tool call depth guard
//!
//! Prevents infinite recursive tool call loops and stack overflow attacks.

#[derive(Debug, Clone, Copy)]
pub struct DepthGuardConfig {
    /// Maximum allowed call depth
    pub max_depth: u32,

    /// Enable depth checking
    pub enabled: bool,
}

impl Default for DepthGuardConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            enabled: true,
        }
    }
}

/// Depth guard for limiting recursive tool call depth
#[derive(Debug, Default)]
pub struct DepthGuard {
    config: DepthGuardConfig,
    current_depth: u32,
}

impl DepthGuard {
    /// Create new depth guard with default configuration
    pub fn new() -> Self {
        Self::with_config(DepthGuardConfig::default())
    }

    /// Create new depth guard with custom configuration
    pub fn with_config(config: DepthGuardConfig) -> Self {
        Self {
            config,
            current_depth: 0,
        }
    }

    /// Enter a new call level
    pub fn enter(&mut self) -> Result<(), &'static str> {
        if !self.config.enabled {
            return Ok(());
        }

        if self.current_depth >= self.config.max_depth {
            return Err("Maximum tool call depth exceeded");
        }

        self.current_depth += 1;
        Ok(())
    }

    /// Exit a call level
    pub fn exit(&mut self) {
        if self.current_depth > 0 {
            self.current_depth -= 1;
        }
    }

    /// Get current call depth
    pub fn current_depth(&self) -> u32 {
        self.current_depth
    }

    /// Check if next call would exceed depth limit
    pub fn would_exceed(&self) -> bool {
        self.config.enabled && self.current_depth >= self.config.max_depth
    }

    /// Reset depth counter
    pub fn reset(&mut self) {
        self.current_depth = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_guard_basic() {
        let mut guard = DepthGuard::with_config(DepthGuardConfig {
            max_depth: 3,
            enabled: true,
        });

        assert_eq!(guard.current_depth(), 0);
        assert!(guard.enter().is_ok());
        assert_eq!(guard.current_depth(), 1);
        assert!(guard.enter().is_ok());
        assert_eq!(guard.current_depth(), 2);
        assert!(guard.enter().is_ok());
        assert_eq!(guard.current_depth(), 3);

        // Fourth call should fail
        assert!(guard.enter().is_err());
        assert_eq!(guard.current_depth(), 3);
    }

    #[test]
    fn test_depth_guard_exit() {
        let mut guard = DepthGuard::new();

        guard.enter().unwrap();
        guard.enter().unwrap();
        assert_eq!(guard.current_depth(), 2);

        guard.exit();
        assert_eq!(guard.current_depth(), 1);

        guard.exit();
        assert_eq!(guard.current_depth(), 0);
    }

    #[test]
    fn test_depth_guard_disabled() {
        let mut guard = DepthGuard::with_config(DepthGuardConfig {
            max_depth: 1,
            enabled: false,
        });

        // Should allow unlimited calls when disabled
        for _ in 0..100 {
            assert!(guard.enter().is_ok());
        }
    }

    #[test]
    fn test_depth_guard_reset() {
        let mut guard = DepthGuard::new();

        guard.enter().unwrap();
        guard.enter().unwrap();
        assert_eq!(guard.current_depth(), 2);

        guard.reset();
        assert_eq!(guard.current_depth(), 0);
    }
}
