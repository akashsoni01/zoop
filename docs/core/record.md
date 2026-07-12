# record.rs

- **Path:** `core/src/record.rs`
- **Purpose:** WAV recording session — stream PCM to storage, rewrite header on stop, finalize note meta/index.
- **Key types / functions:**
  - `MIN_MONO_BYTES`, `MIN_RECORD_MS`
  - `RecordOutcome` — `Success`, `TooShort`, `WriteFailed`
  - `RecordSession` — `start`, `begin`, `pump`, `stop`
  - `finalize_new_note`, `duration_ms_from_mono_bytes`
- **Dependencies:** `io::Audio`, `paths`, `storage`, `wav`
- **Tests:** `cargo test -p zoop-core record::tests`
- **Status:** Host-verified; ES8311 + SD HIL pending
- **Related:** [wav.md](wav.md), [app.md](app.md), [../firmware/audio/es8311.md](../firmware/audio/es8311.md)
