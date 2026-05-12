use colored::*;

fn main() {
    // Delegates full operation to the library layer
    if let Err(e) = bitlocker_control::run() {
        // Formats the gracefully propagated application error
        eprintln!("{} {}", "[ERROR]".red().bold(), e);
        std::process::exit(1);
    }
}
