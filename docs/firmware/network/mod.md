# network/mod.rs

- **Path:** `firmware/src/network/mod.rs`
- **Purpose:** Network submodule — re-exports managers and `init_network_stubs()`.
- **Key types / functions:**
  - Re-exports: `NtpClient`, `TransferPortal`, `WhisperClient`, `WifiManager`
  - `init_network_stubs()` — logs that WiFi/TLS/HTTP are pending
- **Dependencies:** Submodules, `log`
- **Tests:** N/A
- **Status:** HIL stub
- **Related:** [wifi.md](wifi.md), [../../core/network/README.md](../../core/network/README.md)
