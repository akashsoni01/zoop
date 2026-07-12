# board/secrets.rs

- **Path:** `firmware/src/board/secrets.rs`
- **Purpose:** Includes build-generated `secrets_config.rs` from `OUT_DIR` (WiFi + transcription constants).
- **Key types / functions:**
  - Generated consts (via `build.rs`): `WIFI_SSID`, `WIFI_PASS`, `LOCAL_TIME_OFFSET_MIN`, `TRANSCRIPTION_PROVIDER`, `TRANSCRIPTION_HOST`, `TRANSCRIPTION_PATH`, `TRANSCRIPTION_API_KEY`, `TRANSCRIPTION_MODEL`, `TRANSCRIPTION_BOUNDARY`
- **Dependencies:** `include!(concat!(env!("OUT_DIR"), "/secrets_config.rs"))`
- **Tests:** Validated at build via `TranscriptionConfig::from_secrets`
- **Status:** Build-time config (do not commit `secrets.toml`)
- **Related:** [../../config/build.md](../../config/build.md), [../../config/secrets.example.md](../../config/secrets.example.md)
