# network/ntp.rs

- **Path:** `firmware/src/network/ntp.rs`
- **Purpose:** NTP sync stub using `TimeSyncState` server rotation.
- **Key types / functions:**
  - `NtpClient { state: TimeSyncState }`
  - `sync_step` (returns `Ok(false)`), `on_success`, `on_failure`
- **Dependencies:** `zoop_core::{time::TimeSyncState, io::TimeSource}`
- **Tests:** Server policy host-verified in `core/time.rs`
- **Status:** HIL stub (ESP-IDF SNTP pending)
- **Related:** [../../core/time.md](../../core/time.md), [../board/rtc.md](../board/rtc.md)
