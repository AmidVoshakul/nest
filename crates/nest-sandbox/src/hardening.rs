//! Process hardening and secure initialization
//!
//! Implements kernel level process protection mechanisms to prevent
//! debugging, memory dumping, and other attacks against the process.

use libc::{self, c_int};
use std::io::Result;

/// Disable process debugging and core dumps using PR_SET_DUMPABLE
pub fn disable_debugging() -> Result<()> {
    // PR_SET_DUMPABLE = 4, value = 0
    let res = unsafe { libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0) };

    if res < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Lock all process memory into RAM to prevent swapping to disk
pub fn lock_memory() -> Result<()> {
    // MCL_CURRENT | MCL_FUTURE | MCL_ONFAULT
    let flags = libc::MCL_CURRENT | libc::MCL_FUTURE | libc::MCL_ONFAULT;

    let res = unsafe { libc::mlockall(flags) };

    if res < 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

/// Close all file descriptors except stdin, stdout, stderr
pub fn close_all_fds() -> Result<()> {
    // Get maximum possible file descriptor number
    let max_fd = unsafe { libc::sysconf(libc::_SC_OPEN_MAX) };

    if max_fd < 0 {
        return Err(std::io::Error::last_os_error());
    }

    // Close all fds >= 3
    for fd in 3..max_fd as c_int {
        unsafe {
            libc::close(fd);
        }
    }

    Ok(())
}

/// Reset all signal handlers to default
pub fn reset_signal_handlers() -> Result<()> {
    // Reset standard signals 1-31
    for sig in 1..=31 {
        unsafe {
            libc::signal(sig, libc::SIG_DFL);
        }
    }

    Ok(())
}

/// Apply all process hardening protections
pub fn apply_all_hardening() -> Result<()> {
    disable_debugging()?;
    lock_memory()?;
    close_all_fds()?;
    reset_signal_handlers()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Requires root privileges
    fn test_disable_debugging() {
        assert!(disable_debugging().is_ok());
    }

    #[test]
    #[ignore] // Requires root privileges
    fn test_lock_memory() {
        assert!(lock_memory().is_ok());
    }

    #[test]
    fn test_close_all_fds() {
        // This will close all fds >=3, should always succeed
        assert!(close_all_fds().is_ok());
    }

    #[test]
    fn test_reset_signal_handlers() {
        assert!(reset_signal_handlers().is_ok());
    }
}
