# mock.rs

- **Path:** `core/src/mock.rs`
- **Purpose:** Host-testable mock BSP implementations for display, buttons, power, audio, clock, time, and battery ADC.
- **Key types / functions:**
  - `MockDisplay`, `MockButtons`, `MockPowerRails`, `MockAudio`, `MockClock`, `MockTime`, `MockBatteryAdc`
  - `ButtonSchedule`, `MockFileRegistry`
  - Consts: `EPD_WIDTH`, `EPD_HEIGHT`, `FRAMEBUFFER_BYTES`
- **Dependencies:** `io` traits, `error`
- **Tests:** Used heavily by unit/integration tests (no dedicated suite beyond consumers)
- **Status:** Host-verified
- **Related:** [io.md](io.md), [bin/zoop_sim.md](bin/zoop_sim.md), [tests/integration_offline.md](tests/integration_offline.md)
