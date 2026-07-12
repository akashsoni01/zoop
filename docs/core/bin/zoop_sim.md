# bin/zoop_sim.rs

- **Path:** `core/src/bin/zoop_sim.rs`
- **Purpose:** Offline host harness — scripted PWR→menu and REC→record→tag demo against mock BSP (binary `zoop-sim`).
- **Key types / functions:**
  - `main` — boots `App` with mocks, advances clock/buttons, prints state transitions
  - Helper `advance(...)`
- **Dependencies:** `zoop_core::{app, buttons, mock, state, storage}`
- **Tests:** Flow covered by `tests/zoop_sim_flow.rs`; run binary: `cargo run -p zoop-core --bin zoop-sim`
- **Status:** Host-verified
- **Related:** [../tests/zoop_sim_flow.md](../tests/zoop_sim_flow.md), [../app.md](../app.md)
