# storage/mock.rs

- **Path:** `core/src/storage/mock.rs`
- **Purpose:** Temp-directory-backed `FileStorage` for host tests (`tempfile`).
- **Key types / functions:**
  - `MockStorage::new`, `with_root`, `root`, `dump_files`
  - Implements `FileStorage`
- **Dependencies:** `std::fs`, `tempfile`, `FileStorage`
- **Tests:** `cargo test -p zoop-core storage::mock::tests`
- **Status:** Host-verified
- **Related:** [mod.md](mod.md), [../../firmware/storage/sd.md](../../firmware/storage/sd.md)
