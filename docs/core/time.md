# time.rs

- **Path:** `core/src/time.rs`
- **Purpose:** NTP sync policy — server list and `time_ready` gating for note timestamps.
- **Key types / functions:**
  - `NTP_SERVERS` — pool.ntp.org, time.google.com, time.cloudflare.com
  - `TimeSyncState` — `current_server`, `on_sync_failed`, `on_sync_success`, `can_stamp_notes`
- **Dependencies:** None
- **Tests:** `cargo test -p zoop-core time::tests`
- **Status:** Host-verified; ESP-IDF SNTP HIL pending
- **Related:** [../firmware/network/ntp.md](../firmware/network/ntp.md)
