# transcribe.rs

- **Path:** `core/src/transcribe.rs`
- **Purpose:** Whisper-style transcription HTTP helpers — provider selection, multipart body layout, request head (ports `transcribeOnce`).
- **Key types / functions:**
  - Consts: `TRANSCRIPTION_PATH`, `DEFAULT_MODEL`, `DEFAULT_BOUNDARY`, `OPENAI_HOST`, `CURSOR_HOST`
  - `TranscriptionProvider`, `TranscriptionConfig`, `TranscribeError`
  - `TranscriptionConfig::from_secrets`, `multipart_preamble`/`epilogue`, `http_request_head`
  - `parse_transcription_response`
- **Dependencies:** `whisper_parse`
- **Tests:** `cargo test -p zoop-core transcribe::tests`
- **Status:** Host-verified
- **Related:** [whisper_parse.md](whisper_parse.md), [network/whisper.md](network/whisper.md), [../config/secrets.example.md](../config/secrets.example.md)
