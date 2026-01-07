//! Scrolling region helpers for terminal output.

use std::io::Write;

use anyhow::Context;
use console::Term;

/// Get terminal size (rows, cols).
pub fn get_terminal_size() -> anyhow::Result<(u16, u16)> {
    let term = Term::stdout();
    term.size_checked().context("Failed to get terminal size")
}

/// Set scrolling region using DECSTBM (Set Top and Bottom Margins).
///
/// Sets the scrolling region to lines `top` through `bottom` (1-indexed).
/// All scrolling operations will be confined to this region.
pub fn set_scrolling_region(top: u16, bottom: u16) -> anyhow::Result<()> {
    // DECSTBM: ESC [ top ; bottom r
    // top and bottom are 1-indexed
    let mut stderr = std::io::stderr();
    write!(stderr, "\x1b[{};{}r", top, bottom).context("Failed to set scrolling region")?;
    stderr.flush().context("Failed to flush stdout")?;
    Ok(())
}

/// Reset scrolling region (restore full terminal scrolling).
///
/// Resets the scrolling region to the entire terminal.
pub fn reset_scrolling_region() -> anyhow::Result<()> {
    // Reset scrolling region: ESC [ r (no parameters means full terminal)
    let mut stderr = std::io::stderr();
    write!(stderr, "\x1b[r").context("Failed to reset scrolling region")?;
    stderr.flush().context("Failed to flush stdout")?;
    Ok(())
}

/// Clear the scrolling region.
///
/// Clears all lines within the current scrolling region.
pub fn clear_scrolling_region() -> anyhow::Result<()> {
    // Move to top of region and clear to bottom
    // ESC [ 1 J clears from cursor to bottom of screen
    // But we want to clear the region, so we need to:
    // 1. Move to top of region
    // 2. Clear lines in region
    let mut stderr = std::io::stderr();
    // For now, just clear from cursor to end of screen
    // The actual region clearing will be handled by the caller
    // who knows the exact region bounds
    write!(stderr, "\x1b[J").context("Failed to clear scrolling region")?;
    stderr.flush().context("Failed to flush stdout")?;
    Ok(())
}

/// Move cursor to a specific line (1-indexed).
pub fn move_cursor_to_line(line: u16) -> anyhow::Result<()> {
    // CUP (Cursor Position): ESC [ row ; col H
    // line is 1-indexed
    let mut stderr = std::io::stderr();
    write!(stderr, "\x1b[{};1H", line).context("Failed to move cursor to line")?;
    stderr.flush().context("Failed to flush stdout")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_terminal_size() {
        // Should return Some on a real terminal, None otherwise
        // We can't easily test the exact values, but we can test it doesn't panic
        let _size = get_terminal_size();
    }

    #[test]
    fn test_set_scrolling_region() {
        // Test that it doesn't panic
        // In a real terminal, this would set the scrolling region
        let _ = set_scrolling_region(1u16, 10u16);
    }

    #[test]
    fn test_reset_scrolling_region() {
        // Test that it doesn't panic
        let _ = reset_scrolling_region();
    }

    #[test]
    fn test_clear_scrolling_region() {
        // Test that it doesn't panic
        let _ = clear_scrolling_region();
    }

    #[test]
    fn test_move_cursor_to_line() {
        // Test that it doesn't panic
        let _ = move_cursor_to_line(5u16);
    }

    #[test]
    fn test_scrolling_region_sequence() {
        // Test a sequence of operations
        let _ = set_scrolling_region(1u16, 5u16);
        let _ = move_cursor_to_line(1u16);
        let _ = clear_scrolling_region();
        let _ = reset_scrolling_region();
    }
}
