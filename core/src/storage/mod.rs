pub mod index;
pub mod meta;
pub mod mock;
pub mod tags;

use crate::error::{CoreError, CoreResult};

pub use index::{
    add_to_index, delete_note, load_index, next_note_number, save_index, update_index_has_text,
    IndexStore, NoteEntry,
};
pub use meta::{read_note_meta_value, save_tag, write_note_meta};
pub use mock::MockStorage;
pub use tags::{add_custom_tag, delete_tag, load_tags, save_tags, TagStore};

/// Trait for filesystem I/O — implemented by `MockStorage` (host) and SD adapter (firmware).
pub trait FileStorage {
    fn read_to_string(&self, path: &str) -> CoreResult<Option<String>>;
    fn read_bytes(&self, path: &str) -> CoreResult<Option<Vec<u8>>>;
    fn write_bytes(&self, path: &str, data: &[u8]) -> CoreResult<()>;
    fn exists(&self, path: &str) -> CoreResult<bool>;
    fn remove(&self, path: &str) -> CoreResult<()>;
    fn rename(&self, from: &str, to: &str) -> CoreResult<()>;

    /// Atomic write: write to `tmp_path`, remove `path`, rename tmp → path.
    fn atomic_write(&self, path: &str, tmp_path: &str, data: &[u8]) -> CoreResult<()> {
        if self.exists(tmp_path)? {
            self.remove(tmp_path)?;
        }
        self.write_bytes(tmp_path, data)?;
        if self.exists(path)? {
            self.remove(path)?;
        }
        self.rename(tmp_path, path)
    }

    fn write_string(&self, path: &str, content: &str) -> CoreResult<()> {
        self.write_bytes(path, content.as_bytes())
    }

    fn atomic_write_string(&self, path: &str, tmp_path: &str, content: &str) -> CoreResult<()> {
        self.atomic_write(path, tmp_path, content.as_bytes())
    }
}

pub(crate) fn storage_err(msg: impl Into<String>) -> CoreError {
    CoreError::Storage(msg.into())
}

pub(crate) fn io_to_storage(e: std::io::Error) -> CoreError {
    storage_err(e.to_string())
}
