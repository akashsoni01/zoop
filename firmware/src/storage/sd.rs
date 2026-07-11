//! SD card storage adapter — FAT32 via ESP-IDF VFS (stub until mount wired).

use log::warn;
use zoop_core::error::{CoreError, CoreResult};
use zoop_core::storage::FileStorage;

use crate::board::config::SD_MOUNT;

pub struct SdStorage;

impl SdStorage {
    pub fn mount() -> CoreResult<Self> {
        warn!("sd: mount at {SD_MOUNT} (stub — HIL pending)");
        Ok(Self)
    }
}

impl FileStorage for SdStorage {
    fn read_to_string(&self, path: &str) -> CoreResult<Option<String>> {
        let _ = path;
        Err(CoreError::Storage("SD not mounted (stub)".into()))
    }

    fn read_bytes(&self, path: &str) -> CoreResult<Option<Vec<u8>>> {
        let _ = path;
        Err(CoreError::Storage("SD not mounted (stub)".into()))
    }

    fn write_bytes(&self, path: &str, data: &[u8]) -> CoreResult<()> {
        let _ = (path, data);
        Err(CoreError::Storage("SD not mounted (stub)".into()))
    }

    fn exists(&self, path: &str) -> CoreResult<bool> {
        let _ = path;
        Ok(false)
    }

    fn remove(&self, path: &str) -> CoreResult<()> {
        let _ = path;
        Err(CoreError::Storage("SD not mounted (stub)".into()))
    }

    fn rename(&self, from: &str, to: &str) -> CoreResult<()> {
        let _ = (from, to);
        Err(CoreError::Storage("SD not mounted (stub)".into()))
    }
}
