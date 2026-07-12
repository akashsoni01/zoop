# Firmware app

Wraps `zoop_core::App` with device clocks, buttons, and BSP handles.

## Component in architecture

```mermaid
flowchart LR
  MAIN["main"]
  ENG["FirmwareEngine"]
  APP["zoop_core::App"]
  MAIN --> ENG --> APP
  style ENG fill:#f96,stroke:#333,stroke-width:3px
```

[architecture.md](../../architecture.md).

## Child docs

| Doc | Source |
|-----|--------|
| [mod.md](mod.md) | `app/mod.rs` |
| [engine.md](engine.md) | `app/engine.rs` |

## Status

HIL stub — engine boots when SD stub allows; full loop pending hardware.
