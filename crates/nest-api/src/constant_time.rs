//! Constant time operations for security critical comparisons
//!
//! All functions in this module run in constant time regardless of input values
//! to prevent timing side-channel attacks.

use subtle::{ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess};

/// Constant time equality comparison for byte slices
pub fn equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.ct_eq(b).into()
}

/// Constant time inequality comparison
pub fn not_equal(a: &[u8], b: &[u8]) -> bool {
    !equal(a, b)
}

/// Constant time less than comparison for unsigned integers
pub fn less_than(a: u64, b: u64) -> bool {
    a.ct_lt(&b).into()
}

/// Constant time greater than comparison for unsigned integers
pub fn greater_than(a: u64, b: u64) -> bool {
    a.ct_gt(&b).into()
}

/// Constant time less than or equal comparison
pub fn less_than_or_equal(a: u64, b: u64) -> bool {
    !greater_than(a, b)
}

/// Constant time greater than or equal comparison
pub fn greater_than_or_equal(a: u64, b: u64) -> bool {
    !less_than(a, b)
}

/// Compare two strings in constant time
pub fn str_equal(a: &str, b: &str) -> bool {
    equal(a.as_bytes(), b.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal() {
        assert!(equal(b"test", b"test"));
        assert!(!equal(b"test", b"tes"));
        assert!(!equal(b"test", b"tesx"));
        assert!(!equal(b"test", b"text"));
    }

    #[test]
    fn test_less_than() {
        assert!(less_than(1, 2));
        assert!(!less_than(2, 1));
        assert!(!less_than(2, 2));
    }

    #[test]
    fn test_greater_than() {
        assert!(greater_than(2, 1));
        assert!(!greater_than(1, 2));
        assert!(!greater_than(2, 2));
    }

    #[test]
    fn test_str_equal() {
        assert!(str_equal("test", "test"));
        assert!(!str_equal("test", "tes"));
        assert!(!str_equal("test", "tesx"));
    }
}
