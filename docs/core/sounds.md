# sounds.rs

- **Path:** `core/src/sounds.rs`
- **Purpose:** UI sound policy with enable/disable toggle (ports `sounds.h`).
- **Key types / functions:**
  - `SoundsPolicy` — `new`, `set_enabled`, `play`, `last_event`, `clear_log`
- **Dependencies:** `io::{Audio, SoundKind}`
- **Tests:** `cargo test -p zoop-core sounds::tests`
- **Status:** Host-verified
- **Related:** [io.md](io.md), [app.md](app.md)
