//! Command normalization for loop detection.
//!
//! Pure functions that normalize test runner commands into canonical families,
//! enabling detection of repeated command patterns. Inspired by cc-top's
//! command normalization for loop detection.

/// Command families for normalization.
///
/// Each family groups equivalent test runner invocations so that minor
/// variations (e.g., different test names or flags) are treated as the
/// same logical command for loop detection.
const COMMAND_FAMILIES: &[(&str, &[&str])] = &[
    ("cargo-test", &["cargo test", "cargo nextest run"]),
    ("make-test", &["make test", "make check"]),
    (
        "npm-test",
        &["npm test", "npm run test", "npx jest", "npx vitest"],
    ),
    ("pytest", &["pytest", "python -m pytest"]),
    ("go-test", &["go test"]),
];

/// Check if a command string starts with a given prefix.
///
/// Matching is case-insensitive on the prefix. The command must either
/// equal the prefix exactly or have the prefix followed by whitespace.
///
/// # Examples
/// ```
/// use ragentop_core::normalize::matches_prefix;
///
/// assert!(matches_prefix("cargo test --lib", "cargo test"));
/// assert!(matches_prefix("cargo test", "cargo test"));
/// assert!(!matches_prefix("cargo testing", "cargo test"));
/// ```
#[must_use]
pub fn matches_prefix(cmd: &str, prefix: &str) -> bool {
    let cmd_lower = cmd.trim().to_lowercase();
    let prefix_lower = prefix.to_lowercase();

    if cmd_lower == prefix_lower {
        return true;
    }

    cmd_lower.starts_with(&prefix_lower)
        && cmd_lower
            .as_bytes()
            .get(prefix_lower.len())
            .is_some_and(|&b| b == b' ')
}

/// Normalize a command string for loop detection.
///
/// If the command matches a known family, returns a BLAKE3 hash of the
/// family name. This allows different invocations of the same test runner
/// to be detected as repeated commands.
///
/// Unknown commands return `None`.
///
/// # Examples
/// ```
/// use ragentop_core::normalize::normalize_command;
///
/// let h1 = normalize_command("cargo test --lib");
/// let h2 = normalize_command("cargo test integration");
/// assert_eq!(h1, h2); // Same family -> same hash
///
/// assert!(normalize_command("unknown-tool run").is_none());
/// ```
#[must_use]
pub fn normalize_command(command: &str) -> Option<String> {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return None;
    }

    for (family, prefixes) in COMMAND_FAMILIES {
        for prefix in *prefixes {
            if matches_prefix(trimmed, prefix) {
                let hash = blake3::hash(family.as_bytes());
                return Some(hash.to_hex().to_string());
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- matches_prefix --

    #[test]
    fn matches_exact_prefix() {
        assert!(matches_prefix("cargo test", "cargo test"));
    }

    #[test]
    fn matches_prefix_with_args() {
        assert!(matches_prefix("cargo test --lib", "cargo test"));
    }

    #[test]
    fn no_match_partial_word() {
        assert!(!matches_prefix("cargo testing", "cargo test"));
    }

    #[test]
    fn no_match_different_command() {
        assert!(!matches_prefix("cargo build", "cargo test"));
    }

    #[test]
    fn matches_prefix_case_insensitive() {
        assert!(matches_prefix("Cargo Test --lib", "cargo test"));
    }

    #[test]
    fn matches_prefix_with_leading_whitespace() {
        assert!(matches_prefix("  cargo test --lib", "cargo test"));
    }

    // -- normalize_command: cargo-test family --

    #[test]
    fn normalize_cargo_test() {
        let hash = normalize_command("cargo test --lib");
        assert!(hash.is_some());
    }

    #[test]
    fn normalize_cargo_test_variants_same_hash() {
        let h1 = normalize_command("cargo test --lib");
        let h2 = normalize_command("cargo test integration");
        let h3 = normalize_command("cargo nextest run --lib");
        assert_eq!(h1, h2);
        assert_eq!(h2, h3);
    }

    // -- normalize_command: make-test family --

    #[test]
    fn normalize_make_test() {
        assert!(normalize_command("make test").is_some());
    }

    #[test]
    fn normalize_make_test_variants_same_hash() {
        let h1 = normalize_command("make test");
        let h2 = normalize_command("make check");
        assert_eq!(h1, h2);
    }

    // -- normalize_command: npm-test family --

    #[test]
    fn normalize_npm_test() {
        assert!(normalize_command("npm test").is_some());
    }

    #[test]
    fn normalize_npm_variants_same_hash() {
        let h1 = normalize_command("npm test");
        let h2 = normalize_command("npm run test");
        let h3 = normalize_command("npx jest --coverage");
        let h4 = normalize_command("npx vitest");
        assert_eq!(h1, h2);
        assert_eq!(h2, h3);
        assert_eq!(h3, h4);
    }

    // -- normalize_command: pytest family --

    #[test]
    fn normalize_pytest() {
        assert!(normalize_command("pytest tests/").is_some());
    }

    #[test]
    fn normalize_pytest_variants_same_hash() {
        let h1 = normalize_command("pytest tests/");
        let h2 = normalize_command("python -m pytest");
        assert_eq!(h1, h2);
    }

    // -- normalize_command: go-test family --

    #[test]
    fn normalize_go_test() {
        assert!(normalize_command("go test ./...").is_some());
    }

    // -- normalize_command: cross-family differences --

    #[test]
    fn different_families_different_hashes() {
        let cargo = normalize_command("cargo test");
        let npm = normalize_command("npm test");
        let pytest = normalize_command("pytest");
        assert_ne!(cargo, npm);
        assert_ne!(npm, pytest);
        assert_ne!(cargo, pytest);
    }

    // -- normalize_command: unknown / edge cases --

    #[test]
    fn normalize_unknown_command() {
        assert!(normalize_command("unknown-tool run").is_none());
    }

    #[test]
    fn normalize_empty_string() {
        assert!(normalize_command("").is_none());
    }

    #[test]
    fn normalize_whitespace_only() {
        assert!(normalize_command("   ").is_none());
    }

    #[test]
    fn normalize_command_returns_consistent_hash() {
        let h1 = normalize_command("cargo test");
        let h2 = normalize_command("cargo test");
        assert_eq!(h1, h2);
    }
}
