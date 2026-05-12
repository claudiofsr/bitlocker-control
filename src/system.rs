use crate::config::CommandResult;
use crate::error::{BitError, BitResult};
use crate::logger::Logger;
use std::process::Command as StdCommand;

/// Executes external binaries and captures their outputs safely.
pub struct SystemExecutor<'a> {
    logger: &'a Logger,
}

impl<'a> SystemExecutor<'a> {
    pub fn new(logger: &'a Logger) -> Self {
        Self { logger }
    }

    /// Executes a command and returns an atomic CommandResult.
    pub fn run(&self, cmd_args: &[&str]) -> BitResult<CommandResult> {
        self.logger
            .info(&format!("Executing: {}", cmd_args.join(" ")));

        let mut command = StdCommand::new(cmd_args[0]);
        if cmd_args.len() > 1 {
            command.args(&cmd_args[1..]);
        }

        // We map the IO Error directly to BitError::Command to provide contextual meaning
        let output = command.output().map_err(|e| {
            BitError::Command(format!("Failed to execute '{}': {}", cmd_args[0], e))
        })?;

        let exit_code = output.status.code().unwrap_or(1);

        // Prioritize stderr if it's an error, otherwise use stdout.
        let out_str = if !output.stderr.is_empty() {
            String::from_utf8_lossy(&output.stderr).trim().to_string()
        } else {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        };

        Ok(CommandResult {
            exit_code,
            output: out_str,
        })
    }
}
