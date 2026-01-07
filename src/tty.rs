//! TTY detection utilities for respecting cargo's progress settings.

/// Check if progress should be shown based on cargo's term.progress.when
/// setting (respects CARGO_TERM_PROGRESS_WHEN environment variable).
///
/// Returns `true` if progress should be shown, `false` otherwise.
///
/// # Values
///
/// - `"never"` - Never show progress
/// - `"always"` - Always show progress
/// - `"auto"` (default) - Show if stdout is a TTY (interactive terminal)
///
/// # Examples
///
/// ```no_run
/// use cargo_plugin_utils::should_show_progress;
///
/// if should_show_progress() {
///     // Show progress bar
/// }
/// ```
#[allow(clippy::disallowed_methods)] // CLI tool needs direct env access
pub fn should_show_progress() -> bool {
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
            atty::is(atty::Stream::Stdout)
        }
        _ => {
            // Default to auto behavior for unknown values
            atty::is(atty::Stream::Stdout)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_show_progress() {
        // Should not panic
        let _ = should_show_progress();
    }
}
