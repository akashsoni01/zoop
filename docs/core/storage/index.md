# storage/index.rs

- **Path:** `core/src/storage/index.rs`
- **Purpose:** `notes/index.csv` load/save and note CRUD helpers (ports `notes.cpp` index functions).
- **Key types / functions:**
  - `NoteEntry { num, tag, has_text }`, `IndexStore`
  - `load_index`, `save_index`, `add_to_index`, `update_index_has_text`, `delete_note`, `next_note_number`
- **Dependencies:** `paths`, `FileStorage`, `error`
- **Tests:** `cargo test -p zoop-core storage::index::tests`
- **Status:** Host-verified
- **Related:** [tags.md](tags.md), [meta.md](meta.md), [../paths.md](../paths.md)
