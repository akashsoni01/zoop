# network/portal.rs

- **Path:** `firmware/src/network/portal.rs`
- **Purpose:** Transfer-mode portal shell — start/stop logging until `esp-idf-svc` HTTP server is wired.
- **Key types / functions:**
  - `TransferPortal { active }`
  - `start(ip)`, `stop()`
- **Dependencies:** `log` (handlers live in `zoop_core::network::portal`)
- **Tests:** Routes host-verified in core
- **Status:** HIL stub
- **Related:** [../../core/network/portal.md](../../core/network/portal.md)
