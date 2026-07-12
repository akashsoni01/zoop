# app/engine.rs

- **Path:** `firmware/src/app/engine.rs`
- **Purpose:** Wires BSP stubs to `zoop_core::App` — boot loads stores/UI; tick advances clock, polls buttons, may enter ultra-sleep.
- **Key types / functions:**
  - `EspClock` (`Clock`) — monotonic ms stub
  - `FirmwareTime` (`TimeSource`) — UTC + RTC write stub
  - `FirmwareEngine` — holds storage/display/audio/buttons/battery/sleep/wifi/portal/whisper/index/tags
  - `FirmwareEngine::new`, `boot`, `tick`
- **Dependencies:** All firmware BSP modules + `zoop_core::{app, buttons, io, storage}`
- **Tests:** Host logic covered in `zoop-core`; firmware engine not host-tested
- **Status:** HIL stub (SD mount required for full app; sleep/GPIO pending)
- **Related:** [../../core/app.md](../../core/app.md), [../main.md](../main.md)
