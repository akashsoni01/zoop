# firmware/build.rs

- **Path:** `firmware/build.rs`
- **Purpose:** Embuild ESP-IDF sysenv output + generate `secrets_config.rs` from `secrets.toml` (or example fallback).
- **Key types / functions:**
  - `main` → `embuild::espidf::sysenv::output()` + `generate_secrets_config`
  - Reads WiFi/transcription fields; validates via `zoop_core::TranscriptionConfig::from_secrets`
  - Writes consts into `$OUT_DIR/secrets_config.rs`
- **Dependencies:** `embuild`, `toml`, `zoop_core`
- **Tests:** Panics at build if provider/key invalid
- **Status:** Active (build-time)
- **Related:** [secrets.example.md](secrets.example.md), [../firmware/board/secrets.md](../firmware/board/secrets.md)
