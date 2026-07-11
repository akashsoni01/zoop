use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use super::{io_to_storage, FileStorage};
use crate::error::CoreResult;

/// In-memory filesystem backed by a temp directory for host tests.
#[derive(Debug)]
pub struct MockStorage {
    root: PathBuf,
}

impl MockStorage {
    pub fn new() -> CoreResult<Self> {
        let root = tempfile::tempdir()
            .map_err(|e| super::storage_err(e.to_string()))?
            .keep();
        Ok(Self { root })
    }

    pub fn with_root(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn full_path(&self, path: &str) -> PathBuf {
        let rel = path.strip_prefix('/').unwrap_or(path);
        self.root.join(rel)
    }

    fn ensure_parent(&self, path: &Path) -> CoreResult<()> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(io_to_storage)?;
            }
        }
        Ok(())
    }
}

impl FileStorage for MockStorage {
    fn read_to_string(&self, path: &str) -> CoreResult<Option<String>> {
        let full = self.full_path(path);
        if !full.exists() {
            return Ok(None);
        }
        let data = fs::read_to_string(&full).map_err(io_to_storage)?;
        Ok(Some(data))
    }

    fn read_bytes(&self, path: &str) -> CoreResult<Option<Vec<u8>>> {
        let full = self.full_path(path);
        if !full.exists() {
            return Ok(None);
        }
        let data = fs::read(&full).map_err(io_to_storage)?;
        Ok(Some(data))
    }

    fn write_bytes(&self, path: &str, data: &[u8]) -> CoreResult<()> {
        let full = self.full_path(path);
        self.ensure_parent(&full)?;
        fs::write(&full, data).map_err(io_to_storage)
    }

    fn exists(&self, path: &str) -> CoreResult<bool> {
        Ok(self.full_path(path).exists())
    }

    fn remove(&self, path: &str) -> CoreResult<()> {
        let full = self.full_path(path);
        if full.exists() {
            if full.is_dir() {
                fs::remove_dir_all(&full).map_err(io_to_storage)?;
            } else {
                fs::remove_file(&full).map_err(io_to_storage)?;
            }
        }
        Ok(())
    }

    fn rename(&self, from: &str, to: &str) -> CoreResult<()> {
        let from_path = self.full_path(from);
        let to_path = self.full_path(to);
        self.ensure_parent(&to_path)?;
        fs::rename(&from_path, &to_path).map_err(io_to_storage)
    }
}

/// Snapshot of written files for assertions in tests.
impl MockStorage {
    pub fn dump_files(&self) -> CoreResult<HashMap<String, String>> {
        let mut out = HashMap::new();
        walk_dir(&self.root, &self.root, &mut out)?;
        Ok(out)
    }
}

fn walk_dir(root: &Path, dir: &Path, out: &mut HashMap<String, String>) -> CoreResult<()> {
    for entry in fs::read_dir(dir).map_err(io_to_storage)? {
        let entry = entry.map_err(io_to_storage)?;
        let path = entry.path();
        if path.is_dir() {
            walk_dir(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .map_err(|e| super::storage_err(e.to_string()))?;
            let key = rel.to_string_lossy().replace('\\', "/");
            let content = fs::read_to_string(&path).map_err(io_to_storage)?;
            out.insert(key, content);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_atomic_write_leaves_no_tmp() {
        let store = MockStorage::new().expect("temp dir");
        store
            .atomic_write_string("notes/index.csv", "notes/index.tmp", "1,Work,0\n")
            .expect("atomic write");
        assert!(store.exists("notes/index.csv").expect("exists"));
        assert!(!store.exists("notes/index.tmp").expect("exists"));
    }
}
