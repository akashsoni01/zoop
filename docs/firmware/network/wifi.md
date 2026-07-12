# network/wifi.rs

- **Path:** `firmware/src/network/wifi.rs`
- **Purpose:** WiFi STA policy wrapper — advances `WifiConnectPhase` via core pure functions.
- **Key types / functions:**
  - `WifiManager { phase }`
  - `new`, `connect_step(connected, elapsed_ms)`
- **Dependencies:** `zoop_core::network::wifi::{advance_wifi_connect, WifiConnectPhase, SYNC_MAX_ATTEMPTS}`
- **Tests:** Policy host-verified in `core/network/wifi.rs`
- **Status:** HIL stub (no ESP-IDF STA yet)
- **Related:** [../../core/network/wifi.md](../../core/network/wifi.md)
