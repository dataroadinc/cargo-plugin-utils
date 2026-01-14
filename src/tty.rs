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
    use std::env;

    use super::*;

    /// Helper to run a test with a specific env var value, then restore
    /// original
    fn with_env_var<F, R>(key: &str, value: Option<&str>, test_fn: F) -> R
    where
        F: FnOnce() -> R,
    {
        let original = env::var(key).ok();
        match value {
            Some(val) => unsafe { env::set_var(key, val) },
            None => unsafe { env::remove_var(key) },
        }
        let result = test_fn();
        match original {
            Some(val) => unsafe { env::set_var(key, &val) },
            None => unsafe { env::remove_var(key) },
        }
        result
    }

    #[test]
    fn test_should_show_progress_default() {
        // Without env var set, should use "auto" behavior
        with_env_var("CARGO_TERM_PROGRESS_WHEN", None, || {
            // Result depends on whether we're in a TTY, but should not panic
            let _ = should_show_progress();
        });
    }

    #[test]
    fn test_should_show_progress_never() {
        with_env_var("CARGO_TERM_PROGRESS_WHEN", Some("never"), || {
            assert!(
                !should_show_progress(),
                "should return false when set to 'never'"
            );
        });
    }

    #[test]
    fn test_should_show_progress_always() {
        with_env_var("CARGO_TERM_PROGRESS_WHEN", Some("always"), || {
            assert!(
                should_show_progress(),
                "should return true when set to 'always'"
            );
        });
    }

    #[test]
    fn test_should_show_progress_auto() {
        with_env_var("CARGO_TERM_PROGRESS_WHEN", Some("auto"), || {
            // Result depends on TTY, but should not panic
            let _ = should_show_progress();
        });
    }

    #[test]
    fn test_should_show_progress_unknown_value() {
        // Unknown values should fall back to auto behavior
        with_env_var("CARGO_TERM_PROGRESS_WHEN", Some("unknown_value"), || {
            // Result depends on TTY, but should not panic
            let _ = should_show_progress();
        });
    }

    #[test]
    fn test_should_show_progress_empty_string() {
        // Empty string is an unknown value, should fall back to auto
        with_env_var("CARGO_TERM_PROGRESS_WHEN", Some(""), || {
            let _ = should_show_progress();
        });
    }
}
