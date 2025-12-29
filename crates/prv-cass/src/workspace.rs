//! Workspace-to-repository mapping
//!
//! Maps git repositories to CASS workspaces using path matching.

use crate::{CassDb, Workspace};
use anyhow::Result;
use std::path::Path;

/// Find CASS workspace matching a git repo path.
///
/// Canonicalizes both paths to handle:
/// - Trailing slashes
/// - Symlinks
/// - Case-insensitivity on macOS
pub fn find_workspace_for_repo(db: &CassDb, repo_path: &Path) -> Result<Option<Workspace>> {
    let canonical = repo_path.canonicalize()?;
    let repo_str = canonical.to_string_lossy();

    for ws in db.workspaces()? {
        let ws_path = Path::new(&ws.path);
        if let Ok(ws_canonical) = ws_path.canonicalize() {
            // Case-insensitive on macOS
            #[cfg(target_os = "macos")]
            let matches = ws_canonical.to_string_lossy().to_lowercase() == repo_str.to_lowercase();

            #[cfg(not(target_os = "macos"))]
            let matches = ws_canonical == canonical;

            if matches {
                return Ok(Some(ws));
            }
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_path_with_trailing_slash() {
        let p1 = PathBuf::from("/tmp");
        let p2 = PathBuf::from("/tmp/");
        // canonicalize strips trailing slash
        let c1 = p1.canonicalize().unwrap();
        let c2 = p2.canonicalize().unwrap();
        assert_eq!(c1, c2);
    }

    #[test]
    #[ignore] // Requires CASS
    fn test_find_workspace_real() {
        let db = CassDb::open().unwrap();
        let cwd = std::env::current_dir().unwrap();
        let result = find_workspace_for_repo(&db, &cwd);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nonexistent_path() {
        let path = Path::new("/nonexistent/path/12345");
        assert!(path.canonicalize().is_err());
    }
}
