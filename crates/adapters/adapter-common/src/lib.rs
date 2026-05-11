use std::path::Path;
use std::time::{Duration, SystemTime};

use sysinfo::{ProcessRefreshKind, RefreshKind, System};

/// Threshold for considering an agent "active" based on file modification time.
pub const ACTIVE_THRESHOLD: Duration = Duration::from_secs(300);

/// Check if a process with the given name is currently running.
///
/// Uses `sysinfo` with minimal refresh to avoid unnecessary overhead.
#[must_use]
pub fn is_process_running(name: &str) -> bool {
    let s =
        System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
    s.processes()
        .values()
        .any(|p| p.name().to_string_lossy().contains(name))
}

/// Check if a file at `path` was modified within the given `threshold` duration.
///
/// On every supported platform (Linux, macOS, Windows), creating a file sets
/// its `mtime` to the creation timestamp. As a result, a freshly-created
/// session file — e.g. a new `.jsonl` opened when an agent starts — is
/// classified as "recently modified" on the very first poll. Subsequent
/// writes update `mtime`, so this function covers both new and updated
/// files without a separate file-descriptor scan.
///
/// Returns `false` if the file does not exist or its modification time cannot
/// be read.
#[must_use]
pub fn is_recently_modified(path: &Path, threshold: Duration) -> bool {
    path.metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| SystemTime::now().duration_since(t).ok())
        .is_some_and(|age| age < threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    use tempfile::TempDir;

    #[test]
    fn recently_modified_true_for_new_file() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let dir = TempDir::new()?;
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "hello")?;

        assert!(is_recently_modified(&file_path, ACTIVE_THRESHOLD));
        Ok(())
    }

    #[test]
    fn recently_modified_false_for_nonexistent() {
        let path = Path::new("/tmp/ragentop-nonexistent-file-abc123");
        assert!(!is_recently_modified(path, ACTIVE_THRESHOLD));
    }
}
