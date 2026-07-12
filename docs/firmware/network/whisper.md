# network/whisper.rs

- **Path:** `firmware/src/network/whisper.rs`
- **Purpose:** Whisper client holding optional `TranscriptionConfig`; TLS upload pending HIL.
- **Key types / functions:**
  - `WhisperClient { config }`
  - `new`, `log_config`
- **Dependencies:** `zoop_core::transcribe::TranscriptionConfig`
- **Tests:** Upload flow host-verified via mock `HttpClient` in core
- **Status:** HIL stub
- **Related:** [../../core/network/whisper.md](../../core/network/whisper.md), [../board/secrets.md](../board/secrets.md)
