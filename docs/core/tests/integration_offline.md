# tests/integration_offline.rs

- **Path:** `core/tests/integration_offline.rs`
- **Purpose:** Full offline loop — record → tag → list → play → delete (WiFi off).
- **Key types / functions:**
  - Test `offline_record_tag_list_play_delete`
  - Helper `advance` for clock + button polling
- **Dependencies:** `zoop_core` app/mock/storage/state
- **Tests:** `cargo test -p zoop-core --test integration_offline`
- **Status:** Host-verified (M2 sign-off)
- **Related:** [zoop_sim_flow.md](zoop_sim_flow.md), [../app.md](../app.md)
