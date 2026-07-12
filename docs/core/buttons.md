# buttons.rs

- **Path:** `core/src/buttons.rs`
- **Purpose:** Debounce, long-press, double-tap, and idle REC-hold detection without blocking delays (ports `buttons.cpp`).
- **Key types / functions:**
  - Timing consts: `DEFAULT_DEBOUNCE_MS`, `DEFAULT_REC_HOLD_MS`, `DEFAULT_LONG_MS`, `DEFAULT_DOUBLE_MS`
  - `ButtonEngine` — `poll_rec_raw`, `poll_rec_release`, `poll_rec_double`, `poll_pwr_raw`, `poll_idle_rec_hold`
  - `ButtonPoller<B>` — adapts `io::Buttons` → `ButtonEvents`
- **Dependencies:** `io::ButtonEvents`, `io::Buttons`, `state::ButtonEvent`
- **Tests:** `cargo test -p zoop-core buttons::tests`
- **Status:** Host-verified
- **Related:** [state.md](state.md), [io.md](io.md), [../firmware/input/buttons.md](../firmware/input/buttons.md)
