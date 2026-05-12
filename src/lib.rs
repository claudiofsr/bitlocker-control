pub mod cli;
pub mod config;
pub mod dependency;
pub mod error;
pub mod logger;
pub mod privilege;
pub mod repository;
pub mod service;
pub mod system;

use clap::{Parser, error::ErrorKind as ClapErrorKind};
use cli::{Cli, Commands};
use dependency::DependencyManager;
use error::{BitError, BitResult};
use logger::Logger;
use privilege::PrivilegeManager;
use repository::ConfigRepository;
use service::BitLockerService;
use std::ffi::OsString;
use system::SystemExecutor;

/// Main library entry point used by the production binary.
pub fn run() -> BitResult<()> {
    run_with_args(std::env::args_os())
}

/// Core orchestrator. Receives injected CLI arguments.
pub fn run_with_args<I, T>(args: I) -> BitResult<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    let temp_logger = Logger::new(false);
    let repo = ConfigRepository::new(&temp_logger);
    let mut app_config = repo.load().unwrap_or_default();

    // Educational Note: We ensure the configuration file is physically created
    // on disk as early as possible. This guarantees that even if the user runs
    // a command that fails, they instantly have a tangible file to inspect and edit.
    if !repo.config_path.exists() {
        repo.save(&app_config)?;
        temp_logger.success("Default configuration file generated.");
        temp_logger.info(&format!(
            "Please review your settings at: {}",
            repo.config_path.display()
        ));
    }

    let cli = match Cli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => {
            // If the user just asked for --help or --version, print it and return success.
            if e.kind() == ClapErrorKind::DisplayHelp || e.kind() == ClapErrorKind::DisplayVersion {
                let _ = e.print();
                return Ok(());
            }

            // Print the clap-generated help or error menu
            let _ = e.print();

            // Differentiate between missing arguments and invalid (garbage) arguments
            let error_msg = match e.kind() {
                ClapErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => "Missing CLI arguments.",
                _ => "Invalid CLI arguments provided.",
            };

            return Err(BitError::Command(error_msg.into()));
        }
    };

    if cli.config {
        // Display the configuration with pretty colors
        repo.display_pretty_json(&app_config)?;
        return Ok(());
    }

    let command = match cli.command {
        Some(cmd) => cmd,
        None => return Err(BitError::Command("Missing CLI arguments.".into())),
    };

    if let Some(dev) = cli.device {
        app_config.device = dev;
    }
    if let Some(pwd) = cli.password {
        app_config.password = Some(pwd);
    }
    if cli.verbose {
        app_config.verbose = true;
    }
    if cli.quiet {
        app_config.verbose = false;
    }

    let logger = Logger::new(app_config.verbose);
    let executor = SystemExecutor::new(&logger);
    let privilege_manager = PrivilegeManager::new(&logger);
    let dependency_manager = DependencyManager::new(&logger);
    let bl_service = BitLockerService::new(&app_config, &executor, &logger);

    dependency_manager.check_all()?;

    if command == Commands::Mount || command == Commands::Unmount {
        if app_config.is_dummy() {
            return Err(BitError::Command(format!(
                "Please configure your device and password in {}",
                repo.config_path.display()
            )));
        }
        privilege_manager.ensure_root()?;
    }

    match command {
        Commands::Mount => bl_service.mount()?,
        Commands::Unmount => bl_service.unmount(cli.eject)?,
        Commands::Show => {
            bl_service.show_status()?;
            return Ok(());
        }
    };

    repo.save(&app_config)?;
    logger.info("Configuration updated with last successful parameters.");

    Ok(())
}
