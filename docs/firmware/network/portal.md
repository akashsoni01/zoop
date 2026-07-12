# network/portal.rs

- **Path:** `firmware/src/network/portal.rs`
- **Purpose:** Transfer-mode HTTP server shell. `start(ip)` / `stop` toggle `active` and log; will wrap `zoop_core::network::portal` handlers with esp-idf HTTP server.

## Component in architecture

```mermaid
flowchart LR
  TP["TransferPortal HIL stub"]
  CORE["core portal routes"]
  HTTP["esp-idf HTTP :80"]
  TP -.-> HTTP --> CORE
  style TP fill:#f96,stroke:#333,stroke-width:3px
```

## Responsibilities

- **Does:** track transfer-mode active flag; log URL
- **Does not:** bind port yet

## Key types / functions

| Item | Role |
|------|------|
| `TransferPortal` | `active` flag |
| `start` / `stop` | Log transfer mode |

## HTTP routes (core — to be wired)

```mermaid
flowchart TB
  P["TransferPortal"]
  P --> R1["GET /"]
  P --> R2["GET /api/notes"]
  P --> R3["GET /export.txt"]
  P --> R4["GET /tags · /tag/*"]
  P --> R5["GET /wav · /txt · /audio"]
```

## Dependencies

- **Outbound:** logging only today; future `zoop_core::network::portal`
- **Inbound:** engine / network init

## Tests

Core portal tests; firmware build-only.

## Status

HIL stub.

## Related

[../../core/network/portal.md](../../core/network/portal.md), [../../architecture.md](../../architecture.md)
