//! Progress bar logger for cargo-style output with quiet mode support.

use std::io::IsTerminal;

use indicatif::{
    ProgressBar,
    ProgressStyle,
};

/// Logger for handling output with quiet mode and cargo-style progress bars.
///
/// This logger is designed for operations with known progress (like processing
/// multiple files). It uses progress bars rather than spinners.
pub struct ProgressLogger {
    quiet: bool,
    progress: Option<ProgressBar>,
}

impl ProgressLogger {
    /// Create a new progress logger.
    ///
    /// * `quiet` - If true, suppresses all output
    pub fn new(quiet: bool) -> Self {
        Self {
            quiet,
            progress: None,
        }
    }

    /// Check if progress should be shown based on cargo's term.progress.when
    /// setting (respects CARGO_TERM_PROGRESS_WHEN environment variable).
    ///
    /// Returns `true` if progress should be shown, `false` otherwise.
    #[allow(clippy::disallowed_methods)] // CLI tool needs direct env access
    pub fn should_show_progress(&self) -> bool {
        if self.quiet {
            return false;
        }
        // Respect cargo's term.progress.when setting
        // Values: "auto" (default), "always", "never"
        match std::env::var("CARGO_TERM_PROGRESS_WHEN")
            .as_deref()
            .unwrap_or("auto")
        {
            "never" => false,
            "always" => true,
            "auto" => {
                // Auto: show if stdout is a TTY (interactive terminal)
                std::io::stdout().is_terminal()
            }
            _ => {
                // Default to auto behavior for unknown values
                std::io::stdout().is_terminal()
            }
        }
    }

    /// Set a status message with a progress bar (ephemeral, like cargo's
    /// "Compiling").
    ///
    /// * `total` - Total number of items to process
    pub fn set_progress(&mut self, total: u64) {
        if !self.should_show_progress() {
            return;
        }
        let pb = ProgressBar::new(total);
        // Match cargo's progress bar style
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} {msg} [{bar:40.cyan/blue}] {pos}/{len}")
                .unwrap()
                .progress_chars("#>-"),
        );
        self.progress = Some(pb);
    }

    /// Update progress status message.
    pub fn set_message(&self, msg: &str) {
        if let Some(pb) = &self.progress {
            pb.set_message(msg.to_string());
        }
    }

    /// Increment progress by 1.
    pub fn inc(&self) {
        if let Some(pb) = &self.progress {
            pb.inc(1);
        }
    }

    /// Print a permanent message (will be kept in output).
    ///
    /// Format matches cargo's style: "   ✓ message" or "   message"
    pub fn println(&mut self, msg: &str) {
        if !self.quiet {
            // If we have an active progress bar, suspend it while printing
            if let Some(pb) = &self.progress {
                pb.suspend(|| {
                    println!("{}", msg);
                });
            } else {
                println!("{}", msg);
            }
        }
    }

    /// Print a status message in cargo's style: "   Compiling crate-name".
    pub fn status(&mut self, action: &str, target: &str) {
        if !self.quiet {
            if let Some(pb) = &self.progress {
                pb.suspend(|| {
                    println!("   {} {}", action, target);
                });
            } else {
                println!("   {} {}", action, target);
            }
        }
    }

    /// Clear/finish the progress bar.
    pub fn finish(&mut self) {
        if let Some(pb) = self.progress.take() {
            pb.finish_and_clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_logger_new() {
        let logger = ProgressLogger::new(false);
        assert!(!logger.quiet);
        assert!(logger.progress.is_none());
    }

    #[test]
    fn test_progress_logger_quiet() {
        let logger = ProgressLogger::new(true);
        assert!(logger.quiet);
        assert!(!logger.should_show_progress());
    }

    #[test]
    fn test_progress_logger_set_progress() {
        let mut logger = ProgressLogger::new(false);
        logger.set_progress(10);
        // Progress bar should be created if TTY or CARGO_TERM_PROGRESS_WHEN
        // allows (we can't easily test this without mocking, but the
        // function should complete)
    }

    #[test]
    fn test_progress_logger_inc() {
        let mut logger = ProgressLogger::new(false);
        logger.set_progress(10);
        logger.inc();
        // Should not panic
    }

    #[test]
    fn test_progress_logger_finish() {
        let mut logger = ProgressLogger::new(false);
        logger.set_progress(10);
        logger.finish();
        assert!(logger.progress.is_none());
    }
}
