# storage/mod.rs

- **Path:** `core/src/storage/mod.rs`
- **Purpose:** Storage module root — `FileStorage` trait plus re-exports of index/tags/meta/mock APIs.
- **Key types / functions:**
  - Trait `FileStorage` — `read_to_string`, `read_bytes`, `write_bytes`, `exists`, `remove`, `rename`, `atomic_write`, `write_string`, `atomic_write_string`
  - Re-exports: `IndexStore`, `NoteEntry`, `TagStore`, `MockStorage`, index/tag/meta helpers
- **Dependencies:** Submodules, `error`
- **Tests:** Trait defaults exercised via mock tests
- **Status:** Host-verified
- **Related:** [index.md](index.md), [tags.md](tags.md), [mock.md](mock.md)
