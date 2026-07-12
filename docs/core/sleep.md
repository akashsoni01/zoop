# sleep.rs

- **Path:** `core/src/sleep.rs`
- **Purpose:** Ultra-sleep idle timer and battery-warning overlay policy (ports `sleep.cpp` without ESP deep sleep).
- **Key types / functions:**
  - `DEFAULT_ULTRA_SLEEP_MS` (120_000), `BAT_WARN_OVERLAY_MS`
  - `ActivityTimer` — `reset_activity`, `should_ultra_sleep`, `update_battery_warning`, `battery_warning_active`
  - `WakeCause`, `wake_cause_from_pins`
- **Dependencies:** `state::AppState`
- **Tests:** `cargo test -p zoop-core sleep::tests`
- **Status:** Host-verified; `esp_sleep` HIL pending
- **Related:** [battery.md](battery.md), [../firmware/power/sleep.md](../firmware/power/sleep.md)
