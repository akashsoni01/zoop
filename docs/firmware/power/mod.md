# power/mod.rs

- **Path:** `firmware/src/power/mod.rs`
- **Purpose:** Battery ADC stub implementing `BatteryAdc` + sleep re-export.
- **Key types / functions:**
  - `BatteryMonitor` — `read_percent`, `read_mv_samples` (returns fixed 2100 mV × 16)
  - `pub use sleep::SleepManager`
- **Dependencies:** `zoop_core::{battery, io::BatteryAdc}`, `board::config::BAT_ADC_PIN`
- **Tests:** Curve host-verified in `core/battery.rs`
- **Status:** HIL stub
- **Related:** [sleep.md](sleep.md), [../../core/battery.md](../../core/battery.md)
