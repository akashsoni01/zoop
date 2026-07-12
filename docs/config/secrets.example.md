# secrets.example.toml

- **Path:** `firmware/secrets.example.toml`
- **Purpose:** Template for gitignored `secrets.toml` — WiFi SSID/pass, transcription provider keys, optional base host, local time offset.
- **Key types / functions:** N/A (TOML keys)
  - `wifi_ssid`, `wifi_pass`
  - `transcription_provider` (`cursor` | `openai`)
  - `cursor_api_key`, `openai_key`
  - Optional `transcription_base_host`
  - `local_time_offset_min`
- **Dependencies:** Consumed by `firmware/build.rs`
- **Tests:** Parsed at firmware build
- **Status:** Safe to commit (placeholders only). **Do not commit `secrets.toml`.**
- **Related:** [build.md](build.md), [../core/transcribe.md](../core/transcribe.md)
