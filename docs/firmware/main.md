# main.rs

- **Path:** `firmware/src/main.rs`
- **Purpose:** Firmware entry — ESP-IDF patches/logger, bring up BSP stubs, construct `FirmwareEngine`, boot + tick loop.
- **Key types / functions:**
  - Private modules: `app`, `audio`, `board`, `display`, `input`, `network`, `power`, `storage`
  - `main()` — `BoardPower::init`, `EpaperDisplay`, `SdStorage::mount`, `Es8311Audio`, `RtcChip`, `WhisperClient`, `FirmwareEngine::{boot,tick}`
- **Dependencies:** `esp-idf-svc`, `zoop_core::transcribe`, crate BSP modules
- **Tests:** Not unit-tested on host (excluded from CI workspace tests)
- **Status:** HIL stub (100 ms sleep loop; real timer pending)
- **Related:** [app/engine.md](app/engine.md), [README.md](README.md)
