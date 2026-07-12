# io.rs

- **Path:** `core/src/io.rs`
- **Purpose:** Trait-based I/O boundaries — mockable on host, implemented by firmware BSP.
- **Key types / functions:**
  - Traits: `Display`, `Audio`, `Buttons`, `PowerRails`, `Clock`, `TimeSource`, `BatteryAdc`, `ButtonEvents`
  - `SoundKind` — `Select`, `Next`, `Back`, `Saved`, `Delete`, `Success`, `Error`
- **Dependencies:** `error::CoreResult`, `state::ButtonEvent`
- **Tests:** Covered via `mock` implementations in module tests
- **Status:** Host-verified (trait contracts)
- **Related:** [mock.md](mock.md), [../firmware/README.md](../firmware/README.md)
