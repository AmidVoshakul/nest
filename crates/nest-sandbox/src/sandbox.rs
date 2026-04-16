//! Linux namespace sandbox implementation

use libc::{self, CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWNET, CLONE_NEWIPC, CLONE_NEWUTS, MS_NOEXEC, MS_NOSUID, MS_NODEV};
use std::fs::File;
use std::os::unix::process::CommandExt;
use nest_api::error::{Result, Error};

/// Sandbox implementation using Linux namespaces
pub struct Sandbox {
    config: super::SandboxConfig,
    pid: Option<u32>,
    root_fd: Option<File>,
}

impl Sandbox {
    /// Create a new sandbox with the given configuration
    pub fn new(config: super::SandboxConfig) -> Result<Self> {
        Ok(Self {
            config,
            pid: None,
            root_fd: None,
        })
    }

    /// Execute a command inside an isolated sandbox
    pub async fn execute(&mut self, command: &str, args: &[&str]) -> Result<super::SandboxOutput> {
        use tokio::process::Command;

        // Create a new process in isolated namespaces
        let child = unsafe {
            Command::new(command)
                .args(args)
                .pre_exec(|| {
                    // Unshare all namespaces
                    if libc::unshare(CLONE_NEWNS | CLONE_NEWPID | CLONE_NEWNET | CLONE_NEWIPC | CLONE_NEWUTS) != 0 {
                        return Err(std::io::Error::last_os_error());
                    }

                    // Mount proc filesystem
                    libc::mount(
                        b"proc\0".as_ptr() as *const _,
                        b"/proc\0".as_ptr() as *const _,
                        b"proc\0".as_ptr() as *const _,
                        MS_NOEXEC | MS_NOSUID | MS_NODEV,
                        std::ptr::null(),
                    );

                    Ok(())
                })
                .spawn()
                .map_err(|e| Error::Sandbox(e.to_string()))?
        };

        let output = child.wait_with_output()
            .await
            .map_err(|e| Error::Sandbox(e.to_string()))?;

        Ok(super::SandboxOutput {
            stdout: output.stdout,
            stderr: output.stderr,
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    /// Kill the sandbox and all processes inside it
    pub fn kill(&mut self) -> Result<()> {
        if let Some(pid) = self.pid {
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
        }
        Ok(())
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = self.kill();
    }
}