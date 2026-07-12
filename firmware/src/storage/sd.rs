//! SD card storage adapter — FAT32 via ESP-IDF VFS (stub until mount wired).

use log::{info, warn};
use zoop_core::error::{CoreError, CoreResult};
use zoop_core::storage::FileStorage;

use crate::board::config::{INDEX_FILE, NOTES_DIR, SD_CLK, SD_CMD, SD_D0, SD_MOUNT, TAG_FILE};

pub struct SdStorage {
    mounted: bool,
}

impl SdStorage {
    pub fn mount() -> CoreResult<Self> {
        info!(
            "sd: mount stub at {SD_MOUNT}{NOTES_DIR} (CLK={SD_CLK} CMD={SD_CMD} D0={SD_D0} — HIL pending)"
        );
        Ok(Self { mounted: false })
    }
}

impl FileStorage for SdStorage {
    fn read_to_string(&self, path: &str) -> CoreResult<Option<String>> {
        if !self.mounted {
            let _ = path;
            return Ok(None);
        }
        Err(CoreError::Storage("SD read not implemented (stub)".into()))
    }

    fn read_bytes(&self, path: &str) -> CoreResult<Option<Vec<u8>>> {
        if !self.mounted {
            let _ = path;
            return Ok(None);
        }
        Err(CoreError::Storage("SD read not implemented (stub)".into()))
    }

    fn write_bytes(&self, path: &str, data: &[u8]) -> CoreResult<()> {
        warn!("sd: write stub {path} ({} bytes)", data.len());
        let _ = (INDEX_FILE, TAG_FILE);
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
