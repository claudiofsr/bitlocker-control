use crate::repository::ConfigRepository;
use clap::{
    ArgAction, Parser, Subcommand,
    builder::styling::{AnsiColor, Effects, Styles},
};
use colored::*; // <-- Adicionado para usar .yellow(), .dimmed(), etc.

/// Custom Clap styling to mimic the Python version's beautiful colored help menu.
fn get_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .usage(AnsiColor::Yellow.on_default() | Effects::BOLD)
        .literal(AnsiColor::Green.on_default())
        .placeholder(AnsiColor::Cyan.on_default())
}

/// Dynamically builds the extra help menu section with named colors
/// and injects the actual system path of the config file.
fn get_after_help() -> String {
    let config_path = ConfigRepository::get_config_path().display().to_string();

    // 1. Inicia a string base com o arquivo de configuração
    let mut help_text = format!(
        "{}\n  {}\n\n{}\n",
        "Config file:".yellow().bold(),
        config_path.blue().bold(),
        "Examples:".yellow().bold()
    );

    // 2. Define os exemplos como um Array de Tuplas estruturadas ("Comentário", "Comando")
    let examples = [
        (
            "# Standard mount using config defaults",
            "bitlocker-control mount",
        ),
        (
            "# Mount a specific device with verbose output",
            "bitlocker-control -d /dev/sda1 -v mount",
        ),
        (
            "# Unmount the device and safely eject it",
            "bitlocker-control --eject unmount",
        ),
        (
            "# Show current block devices and mount status",
            "bitlocker-control show",
        ),
        (
            "# Show current configuration content",
            "bitlocker-control --config",
        ),
    ];

    // 3. Itera sobre a lista aplicando as cores de forma centralizada e idiomática
    for (comment, cmd) in examples {
        help_text.push_str(&format!(
            "  {}\n  {}\n\n",
            comment.dimmed(),
            cmd.green().bold()
        ));
    }

    // Remove as quebras de linha excedentes no final da string
    help_text.trim_end().to_string()
}

const APPLET_TEMPLATE: &str = "\
{before-help}
{about}
{usage-heading} {usage}

{all-args}
{after-help}";

#[derive(Parser)]
#[command(
    version,
    about,
    help_template = APPLET_TEMPLATE,
    styles = get_styles(),
    after_help = get_after_help(),
    arg_required_else_help = true,
)]
pub struct Cli {
    /// Show config file path and content
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub config: bool,

    /// Target block device
    #[arg(short, long)]
    pub device: Option<String>,

    /// Eject drive after unmounting
    #[arg(short, long, action = ArgAction::SetTrue)]
    pub eject: bool,

    /// BitLocker password
    #[arg(short, long)]
    pub password: Option<String>,

    /// Show verbose messages
    #[arg(short, long)]
    pub verbose: bool,

    /// Suppress non-error messages
    #[arg(short, long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, PartialEq)]
pub enum Commands {
    /// Decrypt and mount drive
    Mount,
    /// Safely unmount drive
    Unmount,
    /// Show block devices (lsblk)
    Show,
}
