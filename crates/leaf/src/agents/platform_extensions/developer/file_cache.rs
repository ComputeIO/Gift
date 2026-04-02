use std::collections::HashMap;
use std::path::PathBuf;

/// Hash + byte length pair identifying file content at a point in time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileState {
    pub hash: String,
    pub len: u64,
}

/// Tracks file content hashes across tool calls within a session.
///
/// When an agent re-reads a file that hasn't changed since the last read,
/// the cache returns `CacheStatus::Unchanged` so the caller can emit a
/// short stub instead of the full content — saving significant tokens on
/// long sessions where the agent re-reads the same files repeatedly.
pub struct FileStateCache {
    entries: HashMap<PathBuf, FileState>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheStatus {
    /// File content is identical to the last read.
    Unchanged { len: u64 },
    /// File is new or has changed since the last read.
    Changed { hash: String, len: u64 },
}

impl FileStateCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Check whether `path` has changed since the last recorded state.
    ///
    /// Computes a blake3 hash of `content` and compares against the cached
    /// entry. Returns `Unchanged` on a match, `Changed` otherwise (and
    /// updates the cache entry).
    pub fn check(&mut self, path: &PathBuf, content: &[u8]) -> CacheStatus {
        let hash = blake3::hash(content).to_hex().to_string();
        let len = content.len() as u64;

        if let Some(cached) = self.entries.get(path) {
            if cached.hash == hash {
                return CacheStatus::Unchanged { len };
            }
        }

        self.entries.insert(
            path.clone(),
            FileState {
                hash: hash.clone(),
                len,
            },
        );
        CacheStatus::Changed { hash, len }
    }

    /// Invalidate the cached state for `path`.
    ///
    /// Call this after write or edit operations so the next read
    /// will always return `Changed`.
    pub fn invalidate(&mut self, path: &PathBuf) {
        self.entries.remove(path);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for FileStateCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cache_is_empty() {
        let cache = FileStateCache::new();
        assert!(cache.is_empty());
    }

    #[test]
    fn first_read_is_changed() {
        let mut cache = FileStateCache::new();
        let path = PathBuf::from("/tmp/test.rs");
        let status = cache.check(&path, b"hello world");
        assert_eq!(
            status,
            CacheStatus::Changed {
                hash: blake3::hash(b"hello world").to_hex().to_string(),
                len: 11
            }
        );
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn same_content_is_unchanged() {
        let mut cache = FileStateCache::new();
        let path = PathBuf::from("/tmp/test.rs");
        cache.check(&path, b"hello world");
        let status = cache.check(&path, b"hello world");
        assert_eq!(status, CacheStatus::Unchanged { len: 11 });
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn modified_content_is_changed() {
        let mut cache = FileStateCache::new();
        let path = PathBuf::from("/tmp/test.rs");
        cache.check(&path, b"version 1");
        let status = cache.check(&path, b"version 2");
        assert_eq!(
            status,
            CacheStatus::Changed {
                hash: blake3::hash(b"version 2").to_hex().to_string(),
                len: 9
            }
        );
    }

    #[test]
    fn invalidate_forces_reread() {
        let mut cache = FileStateCache::new();
        let path = PathBuf::from("/tmp/test.rs");
        cache.check(&path, b"hello");
        cache.invalidate(&path);
        assert!(cache.is_empty());
        let status = cache.check(&path, b"hello");
        assert!(matches!(status, CacheStatus::Changed { .. }));
    }

    #[test]
    fn different_paths_independent() {
        let mut cache = FileStateCache::new();
        let path_a = PathBuf::from("/tmp/a.rs");
        let path_b = PathBuf::from("/tmp/b.rs");
        cache.check(&path_a, b"same content");
        let status_b = cache.check(&path_b, b"same content");
        assert!(matches!(status_b, CacheStatus::Changed { .. }));
        let status_a = cache.check(&path_a, b"same content");
        assert!(matches!(status_a, CacheStatus::Unchanged { .. }));
    }
}
