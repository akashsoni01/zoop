# battery.rs

- **Path:** `core/src/battery.rs`
- **Purpose:** Piecewise battery voltage → percent curve (ports `battery.cpp`), plus ADC millivolt averaging.
- **Key types / functions:**
  - `BatteryCurve` — `LOW_THRESHOLD` (15), `RECOVER_THRESHOLD` (20)
  - `battery_percent_from_voltage(v) -> Option<u8>`
  - `voltage_from_adc_mv(mv)`, `battery_percent_from_adc_samples(samples)`
- **Dependencies:** None (pure logic)
- **Tests:** `cargo test -p zoop-core battery::tests`
- **Status:** Host-verified
- **Related:** [io.md](io.md) (`BatteryAdc`), [sleep.md](sleep.md), [../firmware/power/mod.md](../firmware/power/mod.md)
