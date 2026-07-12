# board/power.rs

- **Path:** `firmware/src/board/power.rs`
- **Purpose:** `PowerRails` impl for battery hold, EPD, and audio rails. Currently logs GPIO intents and records sequence strings; calls `zoop_core::power_on_sequence` from `init`.

## Component in architecture

```mermaid
flowchart LR
  MAIN["main"]
  BP["BoardPower HIL stub"]
  CORE["power_on_sequence"]
  MAIN --> BP --> CORE
  style BP fill:#f96,stroke:#333,stroke-width:3px
```

## Responsibilities

Implement `PowerRails`; boot init sequence.

## Key types / functions

| Item | Role |
|------|------|
| `BoardPower::new` / `init` | Construct + run on-sequence |
| `PowerRails` methods | `battery_hold_on`, `epd_power_on/off`, `audio_power_on/off` |

## Dependencies

Outbound: `board::config` pins, `zoop_core::io::PowerRails`, `zoop_core::power_on_sequence`.

## Status

HIL stub.

## Related

[../../core/power.md](../../core/power.md), [config.md](config.md), [../../architecture.md](../../architecture.md)
