# app/mod.rs

- **Path:** `firmware/src/app/mod.rs`
- **Purpose:** Application layer — re-exports engine and core state helpers.
- **Key types / functions:**
  - `pub mod engine`
  - Re-exports: `FirmwareEngine`, `SoundsPolicy`, `AppState`, `StateMachine`, `Transition`
- **Dependencies:** `engine`, `zoop_core::{sounds, state}`
- **Tests:** N/A
- **Status:** HIL stub (engine wiring)
- **Related:** [engine.md](engine.md), [../../core/app.md](../../core/app.md)
