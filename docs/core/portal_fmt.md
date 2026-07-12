# portal_fmt.rs

- **Path:** `core/src/portal_fmt.rs`
- **Purpose:** Portal HTML helpers — HTML escape, URL decode, CSS, and bulk `.txt` export formatting (ports `network.cpp`).
- **Key types / functions:**
  - `EXPORT_MAX_BYTES` (55_000)
  - `ExportNote`, `html_escape`, `format_export_text`, `format_note_heading`, `export_filename`
  - `url_decode_simple`, `portal_css`
- **Dependencies:** None (pure string helpers)
- **Tests:** `cargo test -p zoop-core portal_fmt::tests`
- **Status:** Host-verified
- **Related:** [network/portal.md](network/portal.md)
