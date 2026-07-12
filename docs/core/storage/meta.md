# storage/meta.rs

- **Path:** `core/src/storage/meta.rs`
- **Purpose:** Per-note `.meta` key=value files and UTC→local device label conversion.
- **Key types / functions:**
  - `note_meta_path`, `read_note_meta_value`, `write_note_meta`, `save_tag`
  - `utc_to_local_device_label(utc_iso, offset_min)`
- **Dependencies:** `paths`, `index`, `FileStorage`
- **Tests:** `cargo test -p zoop-core storage::meta::tests`
- **Status:** Host-verified
- **Related:** [index.md](index.md), [../record.md](../record.md)
