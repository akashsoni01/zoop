# Firmware input

GPIO button adapters for REC (GPIO0) and PWR (GPIO18).

## Component in architecture

```mermaid
flowchart LR
  GPIO["GPIO REC/PWR"]
  GB["GpioButtons HIL stub"]
  POLL["ButtonPoller"]
  APP["App::tick"]
  GPIO -.-> GB --> POLL --> APP
  style GB fill:#f96,stroke:#333,stroke-width:3px
```

[architecture.md](../../architecture.md).

## Child docs

| Doc | Source |
|-----|--------|
| [mod.md](mod.md) | `mod.rs` |
| [buttons.md](buttons.md) | `buttons.rs` |

## Status

HIL stub (reads return false).
