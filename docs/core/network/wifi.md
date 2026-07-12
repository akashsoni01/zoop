# network/wifi.rs

- **Path:** `core/src/network/wifi.rs`
- **Purpose:** Pure WiFi connect/retry policy (ports sync/transfer retry UI from `pala_note.ino`).
- **Key types / functions:**
  - `WifiMode` — `Off`, `Sta`
  - `WifiConnectPhase` — `Idle`, `Connecting`, `Connected`, `Failed`
  - `SYNC_MAX_ATTEMPTS` (20), `SYNC_RETRY_MS` (500), `TRANSFER_MAX_ATTEMPTS` (24)
  - `advance_wifi_connect`, `post_sync_policy`
- **Dependencies:** None
- **Tests:** `cargo test -p zoop-core network::wifi::tests`
- **Status:** Host-verified; ESP-IDF STA HIL pending
- **Related:** [../../firmware/network/wifi.md](../../firmware/network/wifi.md)
