//! Linux namespace sandbox implementation

use libc::{self, CLONE_NEWNS, CLONE_NEWPID, CLONE_NEWNET, CLONE_NEWIPC, CLONE_NEWUTS, MS_NOEXEC, MS_NOSUID, MS_NODEV, MS_BIND, MS_REC, MS_PRIVATE, MNT_DETACH, SYS_pivot_root, SECCOMP_SET_MODE_FILTER, SECCOMP_MODE_FILTER};
use std::fs::File;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::process::CommandExt;
use std::os::unix::fs::PermissionsExt;
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

                    // Create temporary sandbox root directory
                    let tmp_dir = std::env::temp_dir().join(format!("nest-sandbox-{}", std::process::id()));
                    fs::create_dir_all(&tmp_dir)?;
                    fs::set_permissions(&tmp_dir, std::fs::Permissions::from_mode(0o755))?;

                    // Make root directory private mount
                    libc::mount(
                        std::ptr::null(),
                        b"/\0".as_ptr() as *const _,
                        std::ptr::null(),
                        MS_PRIVATE | MS_REC,
                        std::ptr::null(),
                    );

                    // Bind mount the temporary directory to itself for pivot_root
                    libc::mount(
                        tmp_dir.as_os_str().as_bytes().as_ptr() as *const _,
                        tmp_dir.as_os_str().as_bytes().as_ptr() as *const _,
                        std::ptr::null(),
                        MS_BIND | MS_REC,
                        std::ptr::null(),
                    );

                    // Mount proc filesystem
                    let proc_dir = tmp_dir.join("proc");
                    fs::create_dir_all(&proc_dir)?;
                    libc::mount(
                        b"proc\0".as_ptr() as *const _,
                        proc_dir.as_os_str().as_bytes().as_ptr() as *const _,
                        b"proc\0".as_ptr() as *const _,
                        MS_NOEXEC | MS_NOSUID | MS_NODEV,
                        std::ptr::null(),
                    );

                    // Create oldroot directory for pivot_root
                    let old_root = tmp_dir.join("oldroot");
                    fs::create_dir_all(&old_root)?;

                    // Change to new root directory
                    std::env::set_current_dir(&tmp_dir)?;

                    // Pivot root into the sandbox
                    libc::syscall(
                        SYS_pivot_root,
                        tmp_dir.as_os_str().as_bytes().as_ptr() as *const libc::c_char,
                        old_root.as_os_str().as_bytes().as_ptr() as *const libc::c_char,
                    );

                    // Chroot to the new root
                    std::env::set_current_dir("/")?;
                    libc::chroot(b"/\0".as_ptr() as *const _);

                    // Unmount old root to complete isolation
                    libc::umount2(b"/oldroot\0".as_ptr() as *const _, MNT_DETACH);
                    fs::remove_dir_all("/oldroot")?;

                    // Apply seccomp filter to allow only basic system calls
                    // This is a minimal filter - we'll expand it later
                    let mut filter = vec![
                        libc::sock_filter {
                            code: (libc::BPF_RET | libc::BPF_K) as u16,
                            k: 0x7fff0000, // Allow all syscalls for now - this is a placeholder
                            jt: 0,
                            jf: 0,
                        },
                    ];

                    let prog = libc::sock_fprog {
                        len: filter.len() as u16,
                        filter: filter.as_mut_ptr(),
                    };

                    libc::syscall(
                        libc::SYS_seccomp,
                        SECCOMP_SET_MODE_FILTER,
                        0,
                        &prog as *const libc::sock_fprog,
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