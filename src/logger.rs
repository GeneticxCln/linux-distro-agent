use std::fmt::Display;

/// Logger that handles different output levels based on CLI flags
#[derive(Clone)]
pub struct Logger {
    pub verbose: bool,
    pub quiet: bool,
}

impl Logger {
    pub fn new(verbose: bool, quiet: bool) -> Self {
        Self { verbose, quiet }
    }

    /// Print information that should always be shown (unless in quiet mode)
    pub fn info<T: Display>(&self, message: T) {
        if !self.quiet {
            println!("{message}");
        }
    }

    /// Print verbose information (only shown in verbose mode)
    pub fn verbose<T: Display>(&self, message: T) {
        if self.verbose {
            println!("[VERBOSE] {message}");
        }
    }

    /// Print debug information (only shown in verbose mode)
    #[allow(dead_code)]
    pub fn debug<T: Display>(&self, message: T) {
        if self.verbose {
            println!("[DEBUG] {message}");
        }
    }

    /// Print warnings (shown unless in quiet mode)
    pub fn warn<T: Display>(&self, message: T) {
        if !self.quiet {
            eprintln!("[WARNING] {message}");
        }
    }

    /// Print errors (always shown, even in quiet mode)
    pub fn error<T: Display>(&self, message: T) {
        eprintln!("[ERROR] {message}");
    }

    /// Print success messages (shown unless in quiet mode)
    pub fn success<T: Display>(&self, message: T) {
        if !self.quiet {
            println!("✓ {message}");
        }
    }

    /// Print formatted output (respects quiet mode)
    pub fn output<T: Display>(&self, message: T) {
        if !self.quiet {
            println!("{message}");
        }
    }

    /// Print JSON or structured data (always shown, even in quiet mode for command output)
    pub fn json<T: Display>(&self, message: T) {
        println!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logger_verbose_mode() {
        let logger = Logger::new(true, false);
        // These would normally print, but we can't easily test stdout in unit tests
        // The test mainly ensures the struct works correctly
        assert!(logger.verbose);
        assert!(!logger.quiet);
    }

    #[test]
    fn test_logger_quiet_mode() {
        let logger = Logger::new(false, true);
        assert!(!logger.verbose);
        assert!(logger.quiet);
    }

    #[test]
    fn test_logger_normal_mode() {
        let logger = Logger::new(false, false);
        assert!(!logger.verbose);
        assert!(!logger.quiet);
    }
}
