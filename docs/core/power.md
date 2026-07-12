# power.rs

- **Path:** `core/src/power.rs`
- **Purpose:** Power-rail bring-up / sleep prep ordering (ports `board_power_bsp`).
- **Key types / functions:**
  - `POWER_ON_SEQUENCE`, `POWER_SLEEP_SEQUENCE`
  - `power_on_sequence`, `power_sleep_sequence`
  - `sequence_matches` (host test helper)
- **Dependencies:** `io::PowerRails`
- **Tests:** `cargo test -p zoop-core power::tests`
- **Status:** Host-verified (order); GPIO timing HIL on firmware
- **Related:** [io.md](io.md), [../firmware/board/power.md](../firmware/board/power.md)
