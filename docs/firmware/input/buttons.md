# input/buttons.rs

- **Path:** `firmware/src/input/buttons.rs`
- **Purpose:** GPIO button input implementing `Buttons` (always released until HIL).
- **Key types / functions:**
  - `GpioButtons::new`
  - `Buttons::{rec_pressed, pwr_pressed}` → currently `false`
  - Type alias `DeviceButtons = ButtonPoller<GpioButtons>`
- **Dependencies:** `zoop_core::{buttons::ButtonPoller, io::Buttons}`, `board::config` pin numbers
- **Tests:** Debounce logic host-verified in `core/buttons.rs`
- **Status:** HIL stub
- **Related:** [../../core/buttons.md](../../core/buttons.md)
