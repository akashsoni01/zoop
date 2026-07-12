# storage/tags.rs

- **Path:** `core/src/storage/tags.rs`
- **Purpose:** Tag list persistence (`notes/tags.txt`) with defaults, add/delete, and Untagged remapping.
- **Key types / functions:**
  - `TagStore` — `tags`, `contains_ignore_case`, `index_of`
  - `load_tags`, `save_tags`, `add_custom_tag`, `delete_tag`, `tag_has_notes`, `replace_tag_on_notes`
- **Dependencies:** `paths`, `index`, `FileStorage`, `error`
- **Tests:** `cargo test -p zoop-core storage::tags::tests`
- **Status:** Host-verified
- **Related:** [index.md](index.md), [../network/portal.md](../network/portal.md)
