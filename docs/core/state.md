# state.rs

- **Path:** `core/src/state.rs`
- **Purpose:** Typed app state machine — screens, button events, and explicit transitions (ports `types.h` / `pala_note.ino`).
- **Key types / functions:**
  - `AppState`, `ButtonEvent`, `Transition`
  - `StateMachine` — `new`, `state`, `apply`, `set_last_rec_num`, `take_activity_reset`
- **Dependencies:** `error::{CoreError, CoreResult}`
- **Tests:** `cargo test -p zoop-core state::tests`
- **Status:** Host-verified
- **Related:** [app.md](app.md), [buttons.md](buttons.md), [display/ui.md](display/ui.md)
