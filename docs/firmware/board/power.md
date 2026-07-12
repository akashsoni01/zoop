# board/power.rs

- **Path:** `firmware/src/board/power.rs`
- **Purpose:** Board power rails implementing `PowerRails` — logs GPIO targets until wired.
- **Key types / functions:**
  - `BoardPower::new`, `init` (runs `zoop_core::power_on_sequence`)
  - `PowerRails` methods: `battery_hold_on`, `epd_power_on/off`, `audio_power_on/off`, `logged_sequence`
- **Dependencies:** `zoop_core::{io::PowerRails, power_on_sequence}`, `board::config` pins
- **Tests:** Sequence order host-verified in `core/power.rs`
- **Status:** HIL stub
- **Related:** [../../core/power.md](../../core/power.md), [config.md](config.md)
