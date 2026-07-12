# network/whisper.rs

- **Path:** `core/src/network/whisper.rs`
- **Purpose:** Whisper upload with pluggable `HttpClient` — retries, writes `.txt`, updates index.
- **Key types / functions:**
  - Consts: `CHUNK_SIZE`, `MAX_RETRIES`, `RETRY_DELAY_MS`, `REQUEST_TIMEOUT_MS`
  - Trait `HttpClient::post_multipart_wav`
  - `transcribe_note`, `pending_transcription`, `transcribe_all`
- **Dependencies:** `paths`, `storage`, `transcribe`, `whisper_parse`
- **Tests:** `cargo test -p zoop-core network::whisper::tests`
- **Status:** Host-verified (mock HTTP); ESP-TLS upload HIL pending
- **Related:** [../transcribe.md](../transcribe.md), [../../firmware/network/whisper.md](../../firmware/network/whisper.md)
