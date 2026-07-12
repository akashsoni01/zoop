# Firmware power

Battery ADC monitor and ultra-sleep manager stubs.

## Component in architecture

```mermaid
flowchart LR
  BAT["BatteryMonitor HIL stub"]
  SLP["SleepManager HIL stub"]
  CORE["battery / sleep policy"]
  BAT & SLP --> CORE
  style BAT fill:#f96,stroke:#333,stroke-width:2px
  style SLP fill:#f96,stroke:#333,stroke-width:2px
```

[architecture.md](../../architecture.md).

## Child docs

| Doc | Source |
|-----|--------|
| [mod.md](mod.md) | `power/mod.rs` |
| [sleep.md](sleep.md) | `power/sleep.rs` |

## Status

HIL stub (ADC returns fixed samples; sleep logs only).
