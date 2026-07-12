# wav.rs

- **Path:** `core/src/wav.rs`
- **Purpose:** 16 kHz mono 16-bit PCM WAV header build/parse (ports `record.cpp` header rewrite).
- **Key types / functions:**
  - Consts: `SAMPLE_RATE` (16000), `CHANNELS`, `BITS_PER_SAMPLE`, `HEADER_LEN` (44)
  - `WavHeader`, `build_wav_header`, `parse_wav_header`
- **Dependencies:** `error::{CoreError, CoreResult}`
- **Tests:** `cargo test -p zoop-core wav::tests`
- **Status:** Host-verified
- **Related:** [record.md](record.md)
