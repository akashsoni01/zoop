# power/sleep.rs

- **Path:** `firmware/src/power/sleep.rs`
- **Purpose:** Ultra-sleep stub wrapping `ActivityTimer`; logs ext1 wake intent.
- **Key types / functions:**
  - `SleepManager { timer: ActivityTimer }`
  - `should_sleep`, `enter_ultra_sleep`
- **Dependencies:** `zoop_core::{sleep::ActivityTimer, state::AppState}`, button pin consts
- **Tests:** Timer policy host-verified in `core/sleep.rs`
- **Status:** HIL stub (`esp_sleep_enable_ext1_wakeup` pending)
- **Related:** [../../core/sleep.md](../../core/sleep.md)
