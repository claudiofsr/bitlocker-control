use crate::config::Config;
use crate::error::BitResult;
use crate::logger::Logger;
use colored::*;
use regex::Regex;
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    os::unix::fs::{OpenOptionsExt, chown},
    path::{Path, PathBuf},
};

/// Manages reading and writing Config to disk.
pub struct ConfigRepository<'a> {
    logger: &'a Logger,
    pub config_path: PathBuf,
}

impl<'a> ConfigRepository<'a> {
    pub fn new(logger: &'a Logger) -> Self {
        Self {
            logger,
            config_path: Self::get_config_path(),
        }
    }

    /// Detects the real user's home to avoid saving config in /root/.
    pub fn get_user_home() -> PathBuf {
        if let Ok(sudo_user) = env::var("SUDO_USER") {
            if sudo_user == "root" {
                PathBuf::from("/root")
            } else {
                PathBuf::from(format!("/home/{}", sudo_user))
            }
        } else {
            PathBuf::from(env::var("HOME").unwrap_or_else(|_| "/root".to_string()))
        }
    }

    pub fn get_config_path() -> PathBuf {
        Self::get_user_home()
            .join(".config")
            .join("bitlocker-control")
            .join("config.json")
    }

    /// Loads configuration from JSON or returns defaults.
    pub fn load(&self) -> BitResult<Config> {
        if !self.config_path.exists() {
            return Ok(Config::default());
        }

        match fs::read_to_string(&self.config_path) {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(config) => Ok(config),
                Err(e) => {
                    self.logger.error(&format!(
                        "Failed to parse config, using defaults. Error: {}",
                        e
                    ));
                    Ok(Config::default())
                }
            },
            Err(e) => {
                self.logger
                    .error(&format!("Could not read config file: {}", e));
                Ok(Config::default())
            }
        }
    }

    /// Helper Function (Idiomatic Rust):
    /// Silently attempts to get the SUDO_UID and SUDO_GID environment variables
    /// and applies ownership changes. If the variables don't exist, it does nothing.
    fn chown_to_sudo_user(path: &Path) {
        // A closure to cleanly fetch, parse, and convert to Option<u32>
        let get_sudo_id =
            |var_name: &str| -> Option<u32> { env::var(var_name).ok()?.parse::<u32>().ok() };

        if let (Some(uid), Some(gid)) = (get_sudo_id("SUDO_UID"), get_sudo_id("SUDO_GID")) {
            let _ = chown(path, Some(uid), Some(gid));
        }
    }

    /// Persists the Config state to JSON.
    pub fn save(&self, config: &Config) -> BitResult<()> {
        // 1. Create directories if they don't exist
        if let Some(parent) = self.config_path.parent()
            && !parent.exists()
        {
            fs::create_dir_all(parent)?;

            // Revert ownership to the real user if escalated via sudo
            Self::chown_to_sudo_user(parent);
        }

        let json = serde_json::to_string_pretty(config)?;

        // Educational Note: We securely create the file with strict permissions
        // (chmod 600) directly at the OS level to prevent brief plaintext exposure.
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600) // Read and write for the owner only (-rw-------)
            .open(&self.config_path)?;

        file.write_all(json.as_bytes())?;

        // Revert ownership of the file itself
        Self::chown_to_sudo_user(&self.config_path);

        Ok(())
    }

    /// Prints the configuration with colored JSON keys.
    pub fn display_pretty_json(&self, config: &Config) -> BitResult<()> {
        println!(
            "{} {}",
            "Configuration Path:".yellow().bold(),
            self.config_path.display().to_string().blue()
        );

        let json_str = serde_json::to_string_pretty(config)?;

        // Match JSON keys to colorize them
        let re = Regex::new(r#"(".*?")(\s*:)"#).unwrap();
        let colored_json = re.replace_all(&json_str, |caps: &regex::Captures| {
            format!("{}{}", caps[1].green().bold(), &caps[2])
        });

        println!("\n{}\n", colored_json);
        Ok(())
    }
}
