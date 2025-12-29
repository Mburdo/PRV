//! Link persistence to `.prv/links/` directory.
//!
//! Links are stored as JSON files organized by commit SHA prefix:
//! `.prv/links/ab/abcdef123.json`

use crate::link::Link;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Storage for commit-session links.
pub struct LinkStorage {
    base_path: PathBuf,
}

impl LinkStorage {
    /// Create a new LinkStorage rooted at the given repository path.
    pub fn new(repo_root: &Path) -> Self {
        Self {
            base_path: repo_root.join(".prv").join("links"),
        }
    }

    /// Get the file path for a given commit SHA.
    /// Uses first 2 characters as directory prefix for filesystem efficiency.
    fn link_path(&self, commit_sha: &str) -> PathBuf {
        let prefix = &commit_sha[..2.min(commit_sha.len())];
        self.base_path
            .join(prefix)
            .join(format!("{}.json", commit_sha))
    }

    /// Save a link to disk.
    pub fn save(&self, link: &Link) -> Result<()> {
        let path = self.link_path(&link.commit_sha);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(link)?;
        std::fs::write(&path, json)?;
        Ok(())
    }

    /// Load a link by commit SHA. Returns None if not found.
    pub fn load(&self, commit_sha: &str) -> Result<Option<Link>> {
        let path = self.link_path(commit_sha);
        if !path.exists() {
            return Ok(None);
        }
        let json = std::fs::read_to_string(&path)?;
        let link: Link = serde_json::from_str(&json)?;
        Ok(Some(link))
    }

    /// Check if a link exists for the given commit SHA.
    pub fn exists(&self, commit_sha: &str) -> bool {
        self.link_path(commit_sha).exists()
    }

    /// Get the base path for link storage.
    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let storage = LinkStorage::new(tmp.path());
        let link = Link::new("abc123def", 42, 0.9, 0);

        storage.save(&link).unwrap();
        let loaded = storage.load("abc123def").unwrap();

        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.commit_sha, "abc123def");
        assert_eq!(loaded.session_id, 42);
        assert_eq!(loaded.confidence, 0.9);
        assert_eq!(loaded.match_step, 0);
    }

    #[test]
    fn test_load_nonexistent() {
        let tmp = TempDir::new().unwrap();
        let storage = LinkStorage::new(tmp.path());
        let result = storage.load("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_exists() {
        let tmp = TempDir::new().unwrap();
        let storage = LinkStorage::new(tmp.path());
        let link = Link::new("abc123def", 42, 0.9, 0);

        assert!(!storage.exists("abc123def"));
        storage.save(&link).unwrap();
        assert!(storage.exists("abc123def"));
    }

    #[test]
    fn test_path_structure() {
        let tmp = TempDir::new().unwrap();
        let storage = LinkStorage::new(tmp.path());
        let path = storage.link_path("abcdef123");

        let path_str = path.to_string_lossy();
        assert!(path_str.contains(".prv"));
        assert!(path_str.contains("links"));
        assert!(path_str.contains("ab")); // prefix directory
        assert!(path_str.ends_with("abcdef123.json"));
    }

    #[test]
    fn test_directory_created_on_save() {
        let tmp = TempDir::new().unwrap();
        let storage = LinkStorage::new(tmp.path());

        // Directory shouldn't exist yet
        assert!(!storage.base_path().exists());

        let link = Link::new("xyz789", 1, 1.0, 0);
        storage.save(&link).unwrap();

        // Directory should now exist
        assert!(storage.base_path().exists());
        assert!(tmp.path().join(".prv/links/xy/xyz789.json").exists());
    }

    #[test]
    fn test_pretty_json_format() {
        let tmp = TempDir::new().unwrap();
        let storage = LinkStorage::new(tmp.path());
        let link = Link::new("abc123", 42, 0.9, 0);

        storage.save(&link).unwrap();

        let path = storage.link_path("abc123");
        let content = std::fs::read_to_string(path).unwrap();

        // Pretty-printed JSON should have newlines
        assert!(content.contains('\n'));
        assert!(content.contains("commit_sha"));
        assert!(content.contains("session_id"));
    }

    #[test]
    fn test_multiple_links() {
        let tmp = TempDir::new().unwrap();
        let storage = LinkStorage::new(tmp.path());

        let link1 = Link::new("aaa111", 1, 0.8, 0);
        let link2 = Link::new("bbb222", 2, 0.9, 1);
        let link3 = Link::new("aab333", 3, 1.0, 2); // Same prefix as link1

        storage.save(&link1).unwrap();
        storage.save(&link2).unwrap();
        storage.save(&link3).unwrap();

        assert_eq!(storage.load("aaa111").unwrap().unwrap().session_id, 1);
        assert_eq!(storage.load("bbb222").unwrap().unwrap().session_id, 2);
        assert_eq!(storage.load("aab333").unwrap().unwrap().session_id, 3);
    }
}
