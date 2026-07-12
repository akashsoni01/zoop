# app.rs

- **Path:** `core/src/app.rs`
- **Purpose:** High-level application engine wiring the state machine to BSP traits (storage, display, audio, time, battery) for offline UX flows.
- **Key types / functions:**
  - `FIRMWARE_VERSION`, `LOCAL_TIME_OFFSET_MIN`
  - `App<'a, S, D, A, T, ADC>` — fields for stores, UI indices, record session, last screen
  - `App::new`, `boot`, `tick`, `battery_percent`, `redraw`
- **Dependencies:** `battery`, `buttons`, `display::ui`, `error`, `io`, `paths`, `record`, `sleep`, `sounds`, `state`, `storage`
- **Tests:** `cargo test -p zoop-core app::tests` (`boot_loads_stores_and_renders_idle`)
- **Status:** Host-verified
- **Related:** [state.md](state.md), [record.md](record.md), [display/ui.md](display/ui.md), [../firmware/app/engine.md](../firmware/app/engine.md)
