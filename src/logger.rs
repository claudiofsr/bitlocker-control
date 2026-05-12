use colored::*;

/// Handles standard output with semantic formatting.
/// Educational Note: By using a struct instead of static methods,
/// we can inject the verbosity state rather than passing it to every function.
pub struct Logger {
    pub verbose: bool,
}

impl Logger {
    pub fn new(verbose: bool) -> Self {
        Self { verbose }
    }

    pub fn info(&self, msg: &str) {
        if self.verbose {
            println!("{} {}", "[INFO]".blue().bold(), msg);
        }
    }

    pub fn success(&self, msg: &str) {
        println!("{} {}", "[SUCCESS]".green().bold(), msg);
    }

    pub fn warning(&self, msg: &str) {
        println!("{} {}", "[WARN]".yellow().bold(), msg);
    }

    pub fn error(&self, msg: &str) {
        eprintln!("{} {}", "[ERROR]".red().bold(), msg);
    }

    pub fn step(&self, msg: &str) {
        if self.verbose {
            println!("{} {}", "➜".cyan().bold(), msg.bold());
        }
    }

    /// Prints unformatted text (useful for command outputs).
    pub fn raw(&self, msg: &str) {
        println!("{}", msg);
    }
}
