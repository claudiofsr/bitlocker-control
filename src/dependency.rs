use crate::error::{BitError, BitResult};
use crate::logger::Logger;
use colored::*;
use std::collections::HashMap;

/// Checks for required system dependencies and provides Arch Linux install instructions.
pub struct DependencyManager<'a> {
    logger: &'a Logger,
    required_tools: HashMap<&'static str, &'static str>,
}

impl<'a> DependencyManager<'a> {
    pub fn new(logger: &'a Logger) -> Self {
        let mut required_tools = HashMap::new();
        required_tools.insert("dislocker", "dislocker");
        required_tools.insert("mount", "util-linux");
        required_tools.insert("umount", "util-linux");
        required_tools.insert("lsblk", "util-linux");
        required_tools.insert("eject", "util-linux");

        Self {
            logger,
            required_tools,
        }
    }

    /// Verifies if all required binaries are in the system PATH.
    pub fn check_all(&self) -> BitResult<()> {
        let mut missing = Vec::new();

        for (cmd, pkg) in &self.required_tools {
            if which::which(cmd).is_err() {
                missing.push((*cmd, *pkg));
            }
        }

        if !missing.is_empty() {
            self.logger.error("Missing required system dependencies!");
            self.print_install_instructions(&missing);

            let missing_cmds: Vec<&str> = missing.iter().map(|(cmd, _)| *cmd).collect();
            return Err(BitError::Dependency(format!(
                "The following tools are missing: {:?}",
                missing_cmds
            )));
        }

        Ok(())
    }

    /// Generates contextual installation instructions for Arch/Manjaro.
    fn print_install_instructions(&self, missing: &[(&str, &str)]) {
        println!(
            "\n{}",
            "Please install the missing dependencies on your Arch/Manjaro system:".yellow()
        );

        let has_aur = missing.iter().any(|&(_, pkg)| pkg == "dislocker");
        let mut standard_pkgs: Vec<&str> = missing
            .iter()
            .filter(|&&(_, pkg)| pkg != "dislocker")
            .map(|&(_, pkg)| pkg)
            .collect();

        standard_pkgs.dedup(); // Remove duplicates

        if !standard_pkgs.is_empty() {
            let pkgs_str = standard_pkgs.join(" ");
            println!("  {}", "# Standard packages via pacman:".dimmed());
            println!("  {}", format!("sudo pacman -S {}", pkgs_str).green());
        }

        if has_aur {
            println!("\n  {}", "# AUR packages via yay or paru:".dimmed());
            println!("  {}", "yay -S dislocker".green());
            println!("  {}", "  -- OR --".dimmed());
            println!("  {}", "paru -S dislocker".green());
        }
        println!();
    }
}
