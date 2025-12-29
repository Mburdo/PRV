//! In-memory index for fast commit→session lookup.
//!
//! Provides O(1) lookup of session IDs by commit SHA, with disk persistence
//! to `.prv/index.json`.

use crate::link::Link;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// In-memory index mapping commit SHAs to session IDs.
pub struct LinkIndex {
    index: HashMap<String, i64>,
}

impl LinkIndex {
    /// Create a new empty index.
    pub fn new() -> Self {
        Self {
            index: HashMap::new(),
        }
    }

    /// Load index from disk. Returns empty index if file doesn't exist.
    pub fn load(repo_root: &Path) -> Result<Self> {
        let index_path = repo_root.join(".prv").join("index.json");
        if !index_path.exists() {
            return Ok(Self::new());
        }
        let json = std::fs::read_to_string(&index_path)?;
        let index: HashMap<String, i64> = serde_json::from_str(&json)?;
        Ok(Self { index })
    }

    /// Save index to disk.
    pub fn save(&self, repo_root: &Path) -> Result<()> {
        let index_path = repo_root.join(".prv").join("index.json");
        if let Some(parent) = index_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string(&self.index)?;
        std::fs::write(&index_path, json)?;
        Ok(())
    }

    /// Insert a link into the index.
    pub fn insert(&mut self, link: &Link) {
        self.index.insert(link.commit_sha.clone(), link.session_id);
    }

    /// Get session ID for a commit SHA.
    pub fn get(&self, commit_sha: &str) -> Option<i64> {
        self.index.get(commit_sha).copied()
    }

    /// Check if a commit SHA is in the index.
    pub fn contains(&self, commit_sha: &str) -> bool {
        self.index.contains_key(commit_sha)
    }

    /// Get the number of entries in the index.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Check if the index is empty.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Remove an entry from the index.
    pub fn remove(&mut self, commit_sha: &str) -> Option<i64> {
        self.index.remove(commit_sha)
    }

    /// Iterate over all entries.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &i64)> {
        self.index.iter()
    }
}

impl Default for LinkIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_is_empty() {
        let index = LinkIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let mut index = LinkIndex::new();
        let link = Link::new("abc123", 42, 0.9, 0);
        index.insert(&link);

        assert_eq!(index.get("abc123"), Some(42));
        assert_eq!(index.len(), 1);
        assert!(!index.is_empty());
    }

    #[test]
    fn test_get_nonexistent() {
        let index = LinkIndex::new();
        assert_eq!(index.get("nonexistent"), None);
    }

    #[test]
    fn test_contains() {
        let mut index = LinkIndex::new();
        let link = Link::new("abc123", 42, 0.9, 0);

        assert!(!index.contains("abc123"));
        index.insert(&link);
        assert!(index.contains("abc123"));
    }

    #[test]
    fn test_remove() {
        let mut index = LinkIndex::new();
        let link = Link::new("abc123", 42, 0.9, 0);
        index.insert(&link);

        assert_eq!(index.remove("abc123"), Some(42));
        assert!(!index.contains("abc123"));
        assert_eq!(index.remove("abc123"), None);
    }

    #[test]
    fn test_save_and_load() {
        let tmp = TempDir::new().unwrap();
        let mut index = LinkIndex::new();
        index.insert(&Link::new("abc", 1, 0.9, 0));
        index.insert(&Link::new("def", 2, 0.8, 1));
        index.save(tmp.path()).unwrap();

        let loaded = LinkIndex::load(tmp.path()).unwrap();
        assert_eq!(loaded.get("abc"), Some(1));
        assert_eq!(loaded.get("def"), Some(2));
        assert_eq!(loaded.len(), 2);
    }

    #[test]
    fn test_load_nonexistent_returns_empty() {
        let tmp = TempDir::new().unwrap();
        let index = LinkIndex::load(tmp.path()).unwrap();
        assert!(index.is_empty());
    }

    #[test]
    fn test_save_creates_directory() {
        let tmp = TempDir::new().unwrap();
        let index = LinkIndex::new();

        // Directory shouldn't exist yet
        assert!(!tmp.path().join(".prv").exists());

        index.save(tmp.path()).unwrap();

        // Directory should now exist
        assert!(tmp.path().join(".prv").exists());
        assert!(tmp.path().join(".prv/index.json").exists());
    }

    #[test]
    fn test_multiple_inserts_same_key() {
        let mut index = LinkIndex::new();
        index.insert(&Link::new("abc", 1, 0.9, 0));
        index.insert(&Link::new("abc", 2, 0.8, 1)); // Same SHA, different session

        // Should overwrite
        assert_eq!(index.get("abc"), Some(2));
        assert_eq!(index.len(), 1);
    }

    #[test]
    fn test_iter() {
        let mut index = LinkIndex::new();
        index.insert(&Link::new("aaa", 1, 0.9, 0));
        index.insert(&Link::new("bbb", 2, 0.9, 0));
        index.insert(&Link::new("ccc", 3, 0.9, 0));

        let entries: Vec<_> = index.iter().collect();
        assert_eq!(entries.len(), 3);
    }

    #[test]
    fn test_default() {
        let index = LinkIndex::default();
        assert!(index.is_empty());
    }
}
