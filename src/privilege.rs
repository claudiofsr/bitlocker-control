use crate::{
    error::{BitError, BitResult},
    logger::Logger,
};
use std::{
    env,
    io::{self, Write},
    os::unix::process::CommandExt,
    process::Command,
};

/// Handles system privilege escalation requirements atomically.
pub struct PrivilegeManager<'a> {
    logger: &'a Logger,
}

impl<'a> PrivilegeManager<'a> {
    pub fn new(logger: &'a Logger) -> Self {
        Self { logger }
    }

    pub fn ensure_root(&self) -> BitResult<()> {
        // 1. Use libc to check the Effective User ID. UID 0 is always root.
        let is_root = unsafe { libc::geteuid() } == 0;

        if !is_root {
            self.logger
                .warning("Root privileges required. Prompting for sudo password...");

            // 2. Get the path to the current running binary so we can re-run it.
            let current_exe = env::current_exe().map_err(|e| {
                BitError::Command(format!("Failed to get current executable path: {}", e))
            })?;

            // 3. Collect all arguments passed to this program to forward them to the new process.
            let args: Vec<String> = env::args().skip(1).collect();

            // 4. Flush stdout/stderr to ensure no logs are lost before the process image is replaced.
            let _ = io::stdout().flush();
            let _ = io::stderr().flush();

            // 5. 'exec()' replaces the current process image with 'sudo <current_exe> <args>'.
            // If successful, this line never returns; the program "becomes" the new sudo process.
            let err = Command::new("sudo").arg(current_exe).args(args).exec();

            // 6. If execution reaches this point, 'exec()' failed (e.g., 'sudo' not found).
            // We return the specific 'Privilege' error wrapping the underlying IO error.
            return Err(BitError::Privilege(err));
        }

        // 7. If already root, proceed normally.
        Ok(())
    }
}
