# paths.rs

- **Path:** `core/src/paths.rs`
- **Purpose:** Storage path constants and limits mirroring `config.h`.
- **Key types / functions:**
  - Paths: `NOTES_DIR`, `INDEX_FILE`, `INDEX_TMP`, `TAG_FILE`, `TAG_TMP`
  - Limits: `MAX_TAGS`, `MAX_TAG_LEN`, `DEFAULT_TAGS`
  - `note_path(num, ext) -> String` → `notes/note_NNN.ext`
- **Dependencies:** None
- **Tests:** Used by storage/record/portal tests
- **Status:** Host-verified
- **Related:** [storage/README.md](storage/README.md), [../firmware/board/config.md](../firmware/board/config.md)
