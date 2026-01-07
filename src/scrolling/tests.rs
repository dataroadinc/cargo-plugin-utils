//! Tests for scrolling module.

#[cfg(test)]
mod tests {
    use super::super::*;

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
        let _ = set_scrolling_region(1, 10);
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
        let _ = move_cursor_to_line(5);
    }

    #[test]
    fn test_scrolling_region_sequence() {
        // Test a sequence of operations
        let _ = set_scrolling_region(1, 5);
        let _ = move_cursor_to_line(1);
        let _ = clear_scrolling_region();
        let _ = reset_scrolling_region();
    }
}
