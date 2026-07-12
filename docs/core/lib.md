# lib.rs

- **Path:** `core/src/lib.rs`
- **Purpose:** Crate root — declares modules and re-exports the public API of `zoop-core`.
- **Key types / functions:**
  - Module tree: `app`, `battery`, `buttons`, `display`, `error`, `io`, `mock`, `network`, `paths`, `portal_fmt`, `power`, `record`, `sleep`, `sounds`, `state`, `storage`, `time`, `transcribe`, `wav`, `whisper_parse`
  - Re-exports: `App`, `BatteryCurve`, `ButtonEngine`, `CoreError`/`CoreResult`, I/O traits, portal/whisper/wifi helpers, storage API, `TranscriptionConfig`, `WavHeader`, etc.
- **Dependencies:** All `core/src` modules
- **Tests:** N/A (re-export only); covered by module/integration tests
- **Status:** Host-verified
- **Related:** [README](README.md), [app.md](app.md), [error.md](error.md)
