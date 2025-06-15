/// Removes all whitespace characters from a string
pub fn remove_white_space(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).collect()
}

/// Checks if a string is blank (empty or only whitespace)
pub fn isblank(s: &str) -> bool {
    s.trim().is_empty()
}
