# display/ui.rs

- **Path:** `core/src/display/ui.rs`
- **Purpose:** E-Ink screen render functions — immediate-mode `show*()` screens keyed by `AppState` (ports `ui.cpp`).
- **Key types / functions:**
  - `HEADER_H`, `HINTS_Y`, `clear_screen`
  - `ScreenId` — Idle, Recording, Menu, Transfer, BatteryLow, UltraSleep, WifiConnecting, Transcribing, …
  - `UiContext` — `render`, `show_error_screen`, `show_battery_low`, `show_ultra_sleep`, `show_wifi_connecting`, `show_transcribing`
- **Dependencies:** `display::draw`, `state::AppState`
- **Tests:** `cargo test -p zoop-core display::ui::tests`
- **Status:** Host-verified
- **Related:** [draw.md](draw.md), [../app.md](../app.md), [../state.md](../state.md)
