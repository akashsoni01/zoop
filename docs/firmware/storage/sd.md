# storage/sd.rs

- **Path:** `firmware/src/storage/sd.rs`
- **Purpose:** SD card storage adapter implementing `FileStorage` — mount stub returns unmounted store.
- **Key types / functions:**
  - `SdStorage::mount` → `mounted: false`
  - `FileStorage` methods return `None` / errors while unmounted
- **Dependencies:** `zoop_core::storage::FileStorage`, `board::config` SD pins/paths
- **Tests:** Persistence host-verified via `MockStorage` in core
- **Status:** HIL stub (SDIO mount pending)
- **Related:** [../../core/storage/README.md](../../core/storage/README.md)
