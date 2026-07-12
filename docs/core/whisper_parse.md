# whisper_parse.rs

- **Path:** `core/src/whisper_parse.rs`
- **Purpose:** Minimal Whisper JSON `"text"` field extraction (ports `parseWhisperText`).
- **Key types / functions:**
  - `parse_whisper_text(resp) -> Option<String>`
- **Dependencies:** None
- **Tests:** `cargo test -p zoop-core whisper_parse::tests`
- **Status:** Host-verified
- **Related:** [transcribe.md](transcribe.md), [network/whisper.md](network/whisper.md)
