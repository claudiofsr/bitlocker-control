use serde::{Deserialize, Serialize};

/// Represents the application's configuration state.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub device: String,
    pub password: Option<String>,
    pub verbose: bool,
    pub mount_options: String,
    pub bitlocker_mount_dir: String,
    pub final_mount_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            device: "/dev/sda1".to_string(),
            password: None, // <-- Semantic representation of "no password set"
            verbose: true,
            mount_options: "loop,rw,users".to_string(),
            bitlocker_mount_dir: "/mnt/bitlocker".to_string(),
            final_mount_dir: "/mnt/mount".to_string(),
        }
    }
}

impl Config {
    /// Validates if the configuration is still using the placeholder values.
    /// This prevents the system from triggering unnecessary OS-level errors
    /// when the user hasn't configured their drive yet.
    pub fn is_dummy(&self) -> bool {
        self.password.is_none()
    }
}

/// Atomic data structure for standardizing system command outputs.
#[derive(Debug, Clone)]
pub struct CommandResult {
    pub exit_code: i32,
    pub output: String,
}

impl CommandResult {
    /// Helper method to quickly verify if the command succeeded.
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
}
