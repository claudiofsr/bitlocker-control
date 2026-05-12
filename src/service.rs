use crate::config::{CommandResult, Config};
use crate::error::{BitError, BitResult};
use crate::logger::Logger;
use crate::system::SystemExecutor;
use colored::*;
use std::path::Path;

/// Coordinates the core business logic of BitLocker volume management.
pub struct BitLockerService<'a> {
    config: &'a Config,
    executor: &'a SystemExecutor<'a>,
    logger: &'a Logger,
}

impl<'a> BitLockerService<'a> {
    pub fn new(config: &'a Config, executor: &'a SystemExecutor<'a>, logger: &'a Logger) -> Self {
        Self {
            config,
            executor,
            logger,
        }
    }

    fn ensure_paths_exist(&self) -> BitResult<()> {
        for path_str in [
            &self.config.bitlocker_mount_dir,
            &self.config.final_mount_dir,
        ] {
            let path = Path::new(path_str);
            if !path.exists() {
                self.logger
                    .step(&format!("Creating directory: {}", path_str));
                std::fs::create_dir_all(path)?;
            }
        }
        Ok(())
    }

    fn decrypt_volume(&self) -> BitResult<()> {
        if let Some(pwd) = &self.config.password {
            let auth = format!("-u{}", pwd);
            let cmd = [
                "dislocker",
                "-v",
                "-V",
                &self.config.device,
                &auth,
                "--",
                &self.config.bitlocker_mount_dir,
            ];

            let result = self.executor.run(&cmd)?;
            if !result.is_success() {
                let msg = format!("Decryption failed:\n{}", result.output.dimmed());
                self.logger.error(&msg);
                return Err(BitError::Decryption(result.output));
            }
            Ok(())
        } else {
            // Safely abort if password is None,
            Err(BitError::Command(
                "Cannot decrypt: No password configured.".to_string(),
            ))
        }
    }

    fn mount_loop_device(&self) -> BitResult<()> {
        let dislocker_file = format!("{}/dislocker-file", self.config.bitlocker_mount_dir);
        let cmd = [
            "mount",
            "-o",
            &self.config.mount_options,
            &dislocker_file,
            &self.config.final_mount_dir,
        ];

        let result = self.executor.run(&cmd)?;
        if !result.is_success() {
            self.logger
                .error(&format!("Loop mount failed: {}", result.output));
            return Err(BitError::Mount(result.output));
        }
        Ok(())
    }

    /// Use Case 1: Mount a BitLocker drive.
    pub fn mount(&self) -> BitResult<()> {
        self.logger
            .step(&format!("Mounting {}...", self.config.device));
        self.ensure_paths_exist()?;

        self.decrypt_volume()?;

        if let Err(e) = self.mount_loop_device() {
            // Rollback: Unmount FUSE layer if loop mount failed
            let _ = self
                .executor
                .run(&["umount", &self.config.bitlocker_mount_dir]);
            return Err(e);
        }

        self.logger.success(&format!(
            "Drive successfully available at: {}",
            self.config.final_mount_dir
        ));
        Ok(())
    }

    /// Use Case 2: Safely unmount and optionally eject.
    pub fn unmount(&self, eject_hardware: bool) -> BitResult<()> {
        self.logger.step("Starting unmount cleanup...");
        let mut has_errors = false;
        let mut err_msg = String::new();

        let targets = [
            &self.config.final_mount_dir,
            &self.config.bitlocker_mount_dir,
        ];

        for target in targets {
            let result = self
                .executor
                .run(&["umount", target])
                .unwrap_or_else(|e| CommandResult {
                    exit_code: 1,
                    output: e.to_string(),
                });

            if !result.is_success() && !result.output.to_lowercase().contains("not mounted") {
                self.logger
                    .warning(&format!("Could not unmount {}: {}", target, result.output));
                has_errors = true;
                err_msg.push_str(&format!("Failed to unmount {}. ", target));
            } else if result.is_success() {
                self.logger.info(&format!("Unmounted: {}", target));
            }
        }

        self.logger.success("Cleanup complete.");

        if eject_hardware {
            self.logger
                .step(&format!("Ejecting {}...", self.config.device));
            let result = self
                .executor
                .run(&["eject", &self.config.device])
                .unwrap_or_else(|e| CommandResult {
                    exit_code: 1,
                    output: e.to_string(),
                });

            if result.is_success() {
                self.logger.success("Hardware ejected safely.");
            } else {
                self.logger
                    .error(&format!("Eject failed: {}", result.output));
                has_errors = true;
                err_msg.push_str("Failed to eject hardware.");
            }
        }

        if has_errors {
            Err(BitError::Mount(err_msg))
        } else {
            Ok(())
        }
    }

    /// Use Case 3: Display block device layout.
    pub fn show_status(&self) -> BitResult<()> {
        self.logger.step("Retrieving block device status...");
        let res = self.executor.run(&["lsblk", "-f"])?;

        if res.is_success() {
            self.logger.raw(&format!("\n{}\n", res.output.dimmed()));
            Ok(())
        } else {
            Err(BitError::Command(format!(
                "Could not retrieve lsblk status: {}",
                res.output
            )))
        }
    }
}
