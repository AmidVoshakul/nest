//! Subprocess sandbox utilities

use std::process::Command;

/// List of safe environment variables that are allowed to be inherited by child processes
const SAFE_ENV_VARS: &[&str] = &[
    "PATH", "HOME", "TMPDIR", "TMP", "TEMP", "LANG", "LC_ALL", "TERM",
];

/// Windows-specific safe environment variables
#[cfg(windows)]
const SAFE_ENV_VARS_WINDOWS: &[&str] = &[
    "USERPROFILE",
    "SYSTEMROOT",
    "APPDATA",
    "LOCALAPPDATA",
    "COMSPEC",
    "WINDIR",
    "PATHEXT",
];

/// Prepare a command to run in a sandboxed environment with cleared environment
pub fn sandbox_command(command: &str, allowed_env_vars: &[String]) -> Command {
    let mut cmd = Command::new(command);

    // Clear ALL inherited environment variables
    cmd.env_clear();

    // Re-add safe default variables
    for var in SAFE_ENV_VARS {
        if let Ok(val) = std::env::var(var) {
            cmd.env(var, val);
        }
    }

    // Add Windows-specific safe variables
    #[cfg(windows)]
    for var in SAFE_ENV_VARS_WINDOWS {
        if let Ok(val) = std::env::var(var) {
            cmd.env(var, val);
        }
    }

    // Add caller-specified allowed variables
    for var in allowed_env_vars {
        if let Ok(val) = std::env::var(var) {
            cmd.env(var, val);
        }
    }

    cmd
}

/// Validate that an executable path does not contain path traversal attempts
pub fn validate_executable_path(path: &str) -> Result<(), crate::error::Error> {
    use std::path::{Component, Path};

    let p = Path::new(path);

    for component in p.components() {
        if matches!(component, Component::ParentDir) {
            return Err(crate::error::Error::PermissionDenied(format!(
                "executable path '{}' contains '..' component which is not allowed",
                path
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_executable_path() {
        assert!(validate_executable_path("ls").is_ok());
        assert!(validate_executable_path("/usr/bin/ls").is_ok());
        assert!(validate_executable_path("../bin/ls").is_err());
        assert!(validate_executable_path("/usr/bin/../../bin/ls").is_err());
    }

    #[test]
    fn test_sandbox_command_env_clear() {
        let _cmd = sandbox_command("echo", &[]);

        // Command should have env cleared
        // We can't test the actual environment in unit tests easily,
        // but this ensures the function compiles and runs
    }
}
