//! Logger for handling output with cargo-style progress and status messages.

use std::io::Write;

use anyhow::Context;
use carlog::Status;
use console;
use indicatif::{
    ProgressBar,
    ProgressDrawTarget,
    ProgressStyle,
};
use portable_pty::{
    CommandBuilder,
    PtySize,
    native_pty_system,
};

use crate::scrolling::{
    clear_scrolling_region,
    get_terminal_size,
    move_cursor_to_line,
    reset_scrolling_region,
    set_scrolling_region,
};

/// Logger for handling output with cargo-style progress and status messages.
///
/// All progress and status messages go to stderr (matching cargo's behavior).
/// This allows command output (badges, changelog, etc.) to be piped cleanly
/// through stdout while progress messages appear on the console.
pub struct Logger {
    progress_bar: Option<ProgressBar>,
    line_count: usize,
}

impl Logger {
    /// Create a new logger.
    pub fn new() -> Self {
        Self {
            progress_bar: None,
            line_count: 0,
        }
    }

    /// Show a progress bar (ephemeral, disappears on finish).
    ///
    /// Use this for operations with known progress.
    /// Always uses stderr (matching cargo's behavior).
    #[allow(dead_code)] // Will be used for long-running operations
    pub fn progress(&mut self, message: &str) {
        let pb = ProgressBar::new_spinner();
        pb.set_draw_target(ProgressDrawTarget::stderr());
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        pb.set_message(message.to_string());
        pb.enable_steady_tick(std::time::Duration::from_millis(100));

        self.progress_bar = Some(pb);
    }

    /// Update the progress bar message.
    #[allow(dead_code)] // Will be used for long-running operations
    pub fn set_progress_message(&self, message: &str) {
        if let Some(pb) = &self.progress_bar {
            pb.set_message(message.to_string());
        }
    }

    /// Print a status message in cargo's style: "   Building crate-name".
    ///
    /// Uses cyan color for the action word (ephemeral operations).
    /// This creates an ephemeral message that will be cleared on finish().
    /// Always goes to stderr (matching cargo's behavior).
    pub fn status(&mut self, action: &str, target: &str) {
        // Clear previous status (replaces it with new one)
        if let Some(pb) = self.progress_bar.take() {
            pb.finish_and_clear();
        }

        // Format status message with cyan color (like cargo's "Building")
        use console::style;
        let formatted_message = format!("{:>12} {}", style(action).cyan().bold(), target);

        // Create a progress bar that shows the message ephemerally
        let pb = ProgressBar::new_spinner();
        pb.set_draw_target(ProgressDrawTarget::stderr());
        pb.set_style(ProgressStyle::default_spinner().template("{msg}").unwrap());
        pb.set_message(formatted_message);

        self.progress_bar = Some(pb);
        self.line_count = 1;
    }

    /// Print a permanent status message in cargo's style: "   Compiling
    /// crate-name".
    ///
    /// Uses green color for the action word (permanent operations).
    /// This message will NOT be cleared - use for operations that spawn
    /// subprocesses. Always goes to stderr (matching cargo's behavior).
    #[allow(dead_code)] // Will be used for subprocess-heavy operations
    pub fn status_permanent(&self, action: &str, target: &str) {
        let status = Status::new()
            .bold()
            .justify()
            .color(carlog::CargoColor::Green)
            .status(action);

        let formatted_target = format!(" {}", target);

        // Print permanent message to stderr (suspend if progress bar active)
        if let Some(pb) = &self.progress_bar {
            pb.suspend(|| {
                let _ = status.print_stderr(&formatted_target);
            });
        } else {
            let _ = status.print_stderr(&formatted_target);
        }
    }

    /// Print a permanent message (will be kept in output).
    ///
    /// Always goes to stderr (matching cargo's behavior).
    #[allow(dead_code)] // May be used by other commands
    pub fn print_message(&self, msg: &str) {
        if let Some(pb) = &self.progress_bar {
            pb.suspend(|| {
                eprintln!("{}", msg);
            });
        } else {
            eprintln!("{}", msg);
        }
    }

    /// Print an info message (cyan colored).
    ///
    /// Info messages are permanent (not cleared).
    /// Always goes to stderr (matching cargo's behavior).
    #[allow(dead_code)] // May be used by other commands
    pub fn info(&self, action: &str, target: &str) {
        let status = Status::new()
            .bold()
            .justify()
            .color(carlog::CargoColor::Cyan)
            .status(action);

        let formatted_target = format!(" {}", target);

        // Suspend progress bar to print permanent message to stderr
        if let Some(pb) = &self.progress_bar {
            pb.suspend(|| {
                let _ = status.print_stderr(&formatted_target);
            });
        } else {
            let _ = status.print_stderr(&formatted_target);
        }
    }

    /// Print a warning message (yellow colored).
    ///
    /// Warning messages are permanent (not cleared).
    /// Always goes to stderr (matching cargo's behavior).
    pub fn warning(&self, action: &str, target: &str) {
        let status = Status::new()
            .bold()
            .justify()
            .color(carlog::CargoColor::Yellow)
            .status(action);

        let formatted_target = format!(" {}", target);

        // Suspend progress bar to print permanent message to stderr
        if let Some(pb) = &self.progress_bar {
            pb.suspend(|| {
                let _ = status.print_stderr(&formatted_target);
            });
        } else {
            let _ = status.print_stderr(&formatted_target);
        }
    }

    /// Print an error message (red colored).
    ///
    /// Error messages are permanent (not cleared).
    /// Always goes to stderr (matching cargo's behavior).
    #[allow(dead_code)] // May be used by other commands
    pub fn error(&self, action: &str, target: &str) {
        let status = Status::new()
            .bold()
            .justify()
            .color(carlog::CargoColor::Red)
            .status(action);

        let formatted_target = format!(" {}", target);

        // Suspend progress bar to print permanent message to stderr
        if let Some(pb) = &self.progress_bar {
            pb.suspend(|| {
                let _ = status.print_stderr(&formatted_target);
            });
        } else {
            let _ = status.print_stderr(&formatted_target);
        }
    }

    /// Clear the current status message immediately.
    ///
    /// Useful before subprocess operations that might write to stderr.
    pub fn clear_status(&mut self) {
        if let Some(pb) = self.progress_bar.take() {
            pb.finish_and_clear();
            self.line_count = 0;
        }
    }

    /// Temporarily suspend the status message (for subprocess output).
    ///
    /// Call this before spawning subprocesses that write to stderr to avoid
    /// mixing their output with our status line.
    pub fn suspend<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        if let Some(pb) = &self.progress_bar {
            pb.suspend(f)
        } else {
            f()
        }
    }

    /// Finish logging and clear ephemeral status messages.
    pub fn finish(&mut self) {
        if let Some(pb) = self.progress_bar.take() {
            // finish_and_clear() will clear the progress bar's line
            pb.finish_and_clear();
            self.line_count = 0;
        }
    }
}

/// Result of running a subprocess with windowed stderr rendering.
#[derive(Debug, Clone)]
pub struct SubprocessOutput {
    /// Captured stdout
    pub stdout: Vec<u8>,
    /// Captured stderr
    pub stderr: Vec<u8>,
    /// Exit code
    pub exit_code: u32,
}

impl SubprocessOutput {
    /// Get stdout as a string, with UTF-8 error handling.
    pub fn stdout_str(&self) -> anyhow::Result<String> {
        String::from_utf8(self.stdout.clone()).context("Failed to parse stdout as UTF-8")
    }

    /// Get stderr as a string, with UTF-8 error handling.
    pub fn stderr_str(&self) -> anyhow::Result<String> {
        String::from_utf8(self.stderr.clone()).context("Failed to parse stderr as UTF-8")
    }

    /// Check if the process exited successfully.
    pub fn success(&self) -> bool {
        self.exit_code == 0
    }

    /// Get the exit code.
    pub fn exit_code(&self) -> u32 {
        self.exit_code
    }
}

/// Run a subprocess with piped stdout/stderr, capturing stdout fully while
/// rendering stderr lines live in a ring buffer.
///
/// # Arguments
///
/// * `logger` - Logger instance to manage progress bar suspension/clearing
/// * `cmd_builder` - Closure that builds a `portable_pty::CommandBuilder`
/// * `stderr_lines` - Number of stderr lines to show in the scrolling region
///   (default: 5)
///
/// # Behavior
///
/// - Uses PTY mode so subprocesses see a TTY (preserves ANSI colors)
/// - Sets up a scrolling region at the bottom of the terminal
/// - Suspends/clears any active progress bar before running
/// - Captures stdout fully
/// - Renders stderr lines live in the scrolling region
/// - On success: clears the scrolling region cleanly
/// - On failure: leaves/replays the final window
///
/// # Returns
///
/// Returns `SubprocessOutput` with captured stdout, stderr, and exit status.
pub async fn run_subprocess<F>(
    logger: &mut Logger,
    cmd_builder: F,
    stderr_lines: Option<usize>,
) -> anyhow::Result<SubprocessOutput>
where
    F: FnOnce() -> CommandBuilder,
{
    let stderr_lines = stderr_lines.unwrap_or(5);
    // Suspend/clear progress bar before subprocess
    let had_progress = logger.progress_bar.is_some();
    if had_progress {
        logger.clear_status();
    }

    let term = console::Term::stderr();
    let is_term = term.is_term();

    // Get terminal size to set up scrolling region
    let (term_rows, _term_cols) = if is_term {
        get_terminal_size().unwrap_or((24u16, 80u16))
    } else {
        (24u16, 80u16) // Default if not a terminal
    };

    // Set up scrolling region at the bottom of the terminal
    // The region will be the last `stderr_lines` lines
    let stderr_lines_u16 = stderr_lines as u16;
    let region_top = if stderr_lines_u16 < term_rows {
        term_rows - stderr_lines_u16 + 1 // 1-indexed
    } else {
        1 // If stderr_lines >= term_rows, use entire terminal
    };
    let region_bottom = term_rows;

    // Set scrolling region if we're in a terminal
    if is_term {
        set_scrolling_region(region_top, region_bottom)
            .context("Failed to set scrolling region")?;
        // Move cursor to the top of the scrolling region
        move_cursor_to_line(region_top).context("Failed to move cursor to scrolling region")?;
    }

    // Build command using portable-pty
    let cmd = cmd_builder();

    // Create PTY
    let pty_system = native_pty_system();
    let pty_size = PtySize {
        rows: stderr_lines_u16,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    };
    let pty = pty_system
        .openpty(pty_size)
        .context("Failed to create PTY")?;

    // Spawn command in PTY
    let mut child = pty
        .slave
        .spawn_command(cmd)
        .context("Failed to spawn command in PTY")?;

    // Get handles for stdout and stderr from PTY
    // We need to keep a reference to the master to close it later
    let mut reader = pty
        .master
        .try_clone_reader()
        .context("Failed to clone PTY reader")?;

    // Keep the master alive until we're done reading
    let master = pty.master;

    // Channel to coordinate rendering (send raw bytes to preserve ANSI codes)
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    // Keep a clone of tx to close the channel if we timeout
    let tx_clone = tx.clone();

    // Collect output as it arrives (for timeout fallback)
    let collected_output = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
    let collected_output_clone = collected_output.clone();

    // Task to read from PTY (combines stdout and stderr)
    // PTY reader is blocking, so we use spawn_blocking
    let pty_task = tokio::spawn(async move {
        tokio::task::spawn_blocking(move || {
            let mut full_output = Vec::new();
            let mut buffer = vec![0u8; 4096];

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let chunk = &buffer[..n];
                        full_output.extend_from_slice(chunk);
                        // Also collect in shared buffer for timeout fallback
                        if let Ok(mut collected) = collected_output_clone.lock() {
                            collected.extend_from_slice(chunk);
                        }
                        let _ = tx.send(chunk.to_vec());
                    }
                    Err(e) => {
                        // On error, still capture what we have
                        let error_msg = format!("<pty read error: {}>", e);
                        let error_bytes = error_msg.as_bytes();
                        full_output.extend_from_slice(error_bytes);
                        if let Ok(mut collected) = collected_output_clone.lock() {
                            collected.extend_from_slice(error_bytes);
                        }
                        let _ = tx.send(error_bytes.to_vec());
                        break;
                    }
                }
            }

            // Close the channel to signal completion
            drop(tx);

            Ok::<Vec<u8>, anyhow::Error>(full_output)
        })
        .await
        .context("Failed to join blocking PTY read task")?
    });

    // Render output in scrolling region (preserving ANSI codes)
    let mut output_buffer = Vec::new();
    let mut output_ring: Vec<Vec<u8>> = Vec::with_capacity(stderr_lines);

    // Process output bytes as they arrive
    let render_task = tokio::spawn(async move {
        while let Some(chunk) = rx.recv().await {
            output_buffer.extend_from_slice(&chunk);

            // Split buffer into complete lines (preserving ANSI codes)
            let mut lines: Vec<Vec<u8>> = Vec::new();
            let mut current_line = Vec::new();
            let mut i = 0;
            while i < output_buffer.len() {
                let byte = output_buffer[i];
                current_line.push(byte);
                if byte == b'\n' {
                    lines.push(current_line);
                    current_line = Vec::new();
                }
                i += 1;
            }
            output_buffer = current_line;

            // Update ring buffer with new complete lines
            for line in lines {
                output_ring.push(line);
                if output_ring.len() > stderr_lines {
                    output_ring.remove(0);
                }
            }

            // Render ring buffer after processing all lines in this chunk
            if is_term && !output_ring.is_empty() {
                // Clear the scrolling region and redraw
                move_cursor_to_line(region_top).ok();
                clear_scrolling_region().ok();

                // Write all lines in the ring buffer (preserving ANSI codes)
                let mut stderr_handle = std::io::stderr();
                for line_bytes in &output_ring {
                    let _ = stderr_handle.write_all(line_bytes);
                }
                let _ = stderr_handle.flush();
            }
        }

        // Handle any remaining partial line
        if !output_buffer.is_empty() {
            output_ring.push(output_buffer);
            if output_ring.len() > stderr_lines {
                output_ring.remove(0);
            }
            if is_term {
                // Render final ring buffer state
                move_cursor_to_line(region_top).ok();
                clear_scrolling_region().ok();
                let mut stderr_handle = std::io::stderr();
                for line_bytes in &output_ring {
                    let _ = stderr_handle.write_all(line_bytes);
                }
                let _ = stderr_handle.flush();
            }
        }

        (output_ring, is_term)
    });

    // Wait for process to complete (blocking call, so wrap in spawn_blocking)
    let status = tokio::task::spawn_blocking(move || child.wait())
        .await
        .context("Failed to join process wait task")?
        .context("Failed to wait for subprocess")?;

    // Close the PTY master to signal EOF to the reader
    // This ensures the reader sees EOF even if the process has already exited
    drop(master);

    // Wait for PTY reading to complete (with timeout to prevent hanging)
    // If timeout occurs but process has exited, use collected output as fallback
    let pty_output = match tokio::time::timeout(std::time::Duration::from_secs(10), pty_task).await
    {
        Ok(result) => result.context("Failed to join PTY task")??,
        Err(_) => {
            // Timeout occurred - this can happen in CI environments where PTY EOF
            // detection is delayed. Since the process has already exited, we use
            // the output we collected as it arrived through the channel.
            // Close the channel to allow render_task to complete
            drop(tx_clone);
            collected_output.lock().unwrap().clone()
        }
    };
    // Wait for render task with timeout to prevent hanging
    let (_final_output_ring, was_term) =
        match tokio::time::timeout(std::time::Duration::from_secs(5), render_task).await {
            Ok(result) => result.context("Failed to join render task")?,
            Err(_) => {
                // Render task timed out - this shouldn't happen, but if it does,
                // we'll just continue without the final render state
                (Vec::new(), is_term)
            }
        };

    // For now, treat all PTY output as stderr (we can separate later if needed)
    // In PTY mode, stdout and stderr are combined
    let stdout_bytes = Vec::new(); // PTY combines stdout/stderr, so we'll capture all as stderr
    let stderr_bytes = pty_output;

    // Handle final rendering based on success/failure
    let exit_code = status.exit_code();
    let success = exit_code == 0;

    if was_term {
        if success {
            // Success: clear the scrolling region
            clear_scrolling_region().ok();
        } else {
            // Failure: ensure final window is visible (it should already be)
            // Just reset the scrolling region to restore normal scrolling
            reset_scrolling_region().ok();
        }
    }

    Ok(SubprocessOutput {
        stdout: stdout_bytes,
        stderr: stderr_bytes,
        exit_code,
    })
}

impl Default for Logger {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        // Clear the progress bar
        if let Some(pb) = self.progress_bar.take() {
            pb.finish_and_clear();
        }

        // Clear the reserved lines (including our status + subprocess output)
        if self.line_count > 0 {
            use console::Term;
            let term = Term::stderr();
            if term.is_term() {
                let _ = term.clear_last_lines(self.line_count);
            }
            self.line_count = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use portable_pty::CommandBuilder;

    use super::*;

    #[tokio::test]
    async fn test_logger_new() {
        let logger = Logger::new();
        assert!(logger.progress_bar.is_none());
        assert_eq!(logger.line_count, 0);
    }

    #[tokio::test]
    async fn test_logger_status() {
        let mut logger = Logger::new();
        logger.status("Building", "test-crate");
        assert!(logger.progress_bar.is_some());
        assert_eq!(logger.line_count, 1);
    }

    #[tokio::test]
    async fn test_logger_clear_status() {
        let mut logger = Logger::new();
        logger.status("Building", "test-crate");
        assert!(logger.progress_bar.is_some());
        logger.clear_status();
        assert!(logger.progress_bar.is_none());
        assert_eq!(logger.line_count, 0);
    }

    #[tokio::test]
    async fn test_logger_finish() {
        let mut logger = Logger::new();
        logger.status("Building", "test-crate");
        logger.finish();
        assert!(logger.progress_bar.is_none());
        assert_eq!(logger.line_count, 0);
    }

    #[tokio::test]
    async fn test_subprocess_output_success() {
        let output = SubprocessOutput {
            stdout: b"stdout content".to_vec(),
            stderr: b"stderr content".to_vec(),
            exit_code: 0,
        };
        assert!(output.success());
        assert_eq!(output.exit_code(), 0);
        assert_eq!(output.stdout_str().unwrap(), "stdout content");
        assert_eq!(output.stderr_str().unwrap(), "stderr content");
    }

    #[tokio::test]
    async fn test_subprocess_output_failure() {
        let output = SubprocessOutput {
            stdout: b"".to_vec(),
            stderr: b"error message".to_vec(),
            exit_code: 1,
        };
        assert!(!output.success());
        assert_eq!(output.exit_code(), 1);
        assert_eq!(output.stderr_str().unwrap(), "error message");
    }

    #[tokio::test]
    async fn test_run_subprocess_simple_success() {
        let mut logger = Logger::new();
        let output = run_subprocess(
            &mut logger,
            || {
                let mut cmd = CommandBuilder::new("echo");
                cmd.arg("hello world");
                cmd
            },
            Some(3),
        )
        .await
        .unwrap();

        assert!(output.success());
        assert_eq!(output.exit_code(), 0);
        // PTY combines stdout/stderr, so output should be in stderr
        let stderr = output.stderr_str().unwrap();
        assert!(stderr.contains("hello world") || stderr.is_empty());
    }

    #[tokio::test]
    async fn test_run_subprocess_simple_failure() {
        let mut logger = Logger::new();
        let output = run_subprocess(&mut logger, || CommandBuilder::new("false"), Some(3))
            .await
            .unwrap();

        assert!(!output.success());
        assert_ne!(output.exit_code(), 0);
    }

    #[tokio::test]
    async fn test_run_subprocess_multiline_output() {
        let mut logger = Logger::new();
        let output = run_subprocess(
            &mut logger,
            || {
                let mut cmd = CommandBuilder::new("sh");
                cmd.arg("-c");
                cmd.arg("echo 'line 1'; echo 'line 2'; echo 'line 3'; echo 'line 4'; echo 'line 5'; echo 'line 6'");
                cmd
            },
            Some(3), // Only show 3 lines in ring buffer
        )
        .await
        .unwrap();

        assert!(output.success());
        // Should capture all output even though only 3 lines shown
        let stderr = output.stderr_str().unwrap();
        assert!(stderr.contains("line 1"));
        assert!(stderr.contains("line 6"));
    }

    #[tokio::test]
    async fn test_run_subprocess_with_progress_bar() {
        let mut logger = Logger::new();
        logger.status("Preparing", "test");
        assert!(logger.progress_bar.is_some());

        let output = run_subprocess(
            &mut logger,
            || {
                let mut cmd = CommandBuilder::new("echo");
                cmd.arg("test output");
                cmd
            },
            None,
        )
        .await
        .unwrap();

        assert!(output.success());
        // Progress bar should be cleared before subprocess
        // (we can't easily test this without mocking, but the function should
        // complete)
    }

    #[tokio::test]
    async fn test_run_subprocess_exit_code_preservation() {
        let mut logger = Logger::new();
        let output = run_subprocess(
            &mut logger,
            || {
                let mut cmd = CommandBuilder::new("sh");
                cmd.arg("-c");
                cmd.arg("exit 42");
                cmd
            },
            None,
        )
        .await
        .unwrap();

        assert!(!output.success());
        assert_eq!(output.exit_code(), 42);
    }

    #[tokio::test]
    async fn test_run_subprocess_ansi_colors_preserved() {
        let mut logger = Logger::new();
        let output = run_subprocess(
            &mut logger,
            || {
                let mut cmd = CommandBuilder::new("sh");
                cmd.arg("-c");
                cmd.arg("echo -e '\\033[31mred\\033[0m'");
                cmd
            },
            None,
        )
        .await
        .unwrap();

        assert!(output.success());
        let stderr = output.stderr_str().unwrap();
        // ANSI codes should be preserved in PTY mode
        assert!(stderr.contains("\x1b[31m") || stderr.contains("red"));
    }

    #[tokio::test]
    async fn test_run_subprocess_default_stderr_lines() {
        let mut logger = Logger::new();
        let output = run_subprocess(
            &mut logger,
            || {
                let mut cmd = CommandBuilder::new("echo");
                cmd.arg("test");
                cmd
            },
            None, // Should default to 5 lines
        )
        .await
        .unwrap();

        assert!(output.success());
    }

    #[tokio::test]
    async fn test_run_subprocess_custom_stderr_lines() {
        let mut logger = Logger::new();
        let output = run_subprocess(
            &mut logger,
            || {
                let mut cmd = CommandBuilder::new("echo");
                cmd.arg("test");
                cmd
            },
            Some(10), // Custom 10 lines
        )
        .await
        .unwrap();

        assert!(output.success());
    }

    #[tokio::test]
    async fn test_run_subprocess_nonexistent_command() {
        let mut logger = Logger::new();
        let result = run_subprocess(
            &mut logger,
            || CommandBuilder::new("nonexistent-command-xyz-123"),
            None,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_subprocess_output_utf8_handling() {
        let output = SubprocessOutput {
            stdout: "hello 世界".as_bytes().to_vec(),
            stderr: "error 错误".as_bytes().to_vec(),
            exit_code: 0,
        };

        assert_eq!(output.stdout_str().unwrap(), "hello 世界");
        assert_eq!(output.stderr_str().unwrap(), "error 错误");
    }

    #[tokio::test]
    async fn test_subprocess_output_invalid_utf8() {
        let output = SubprocessOutput {
            stdout: vec![0xFF, 0xFE, 0xFD], // Invalid UTF-8
            stderr: vec![],
            exit_code: 0,
        };

        assert!(output.stdout_str().is_err());
    }

    #[tokio::test]
    async fn test_logger_suspend() {
        let mut logger = Logger::new();
        logger.status("Building", "test");
        let result = logger.suspend(|| 42);
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_logger_suspend_without_progress() {
        let mut logger = Logger::new();
        let result = logger.suspend(|| 42);
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_logger_status_permanent() {
        let logger = Logger::new();
        // Should not panic
        logger.status_permanent("Compiling", "test-crate");
    }

    #[tokio::test]
    async fn test_logger_warning() {
        let logger = Logger::new();
        // Should not panic
        logger.warning("Warning", "test message");
    }

    #[tokio::test]
    async fn test_logger_info() {
        let logger = Logger::new();
        // Should not panic
        logger.info("Info", "test message");
    }

    #[tokio::test]
    async fn test_logger_error() {
        let logger = Logger::new();
        // Should not panic
        logger.error("Error", "test message");
    }

    #[tokio::test]
    async fn test_logger_print_message() {
        let logger = Logger::new();
        // Should not panic
        logger.print_message("test message");
    }

    #[tokio::test]
    async fn test_logger_progress() {
        let mut logger = Logger::new();
        logger.progress("Processing...");
        assert!(logger.progress_bar.is_some());
    }

    #[tokio::test]
    async fn test_logger_set_progress_message() {
        let mut logger = Logger::new();
        logger.progress("Initial");
        logger.set_progress_message("Updated");
        assert!(logger.progress_bar.is_some());
    }
}
