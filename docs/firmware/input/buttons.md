# input/buttons.rs

- **Path:** `firmware/src/input/buttons.rs`
- **Purpose:** `Buttons` impl over board REC/PWR pins. Currently always returns unpressed; wraps as `DeviceButtons = ButtonPoller<GpioButtons>` for the engine.

## Component in architecture

```mermaid
flowchart LR
  PINS["BTN_REC / BTN_PWR"]
  GB["GpioButtons HIL stub"]
  CORE["buttons::ButtonPoller"]
  PINS -.-> GB --> CORE
  style GB fill:#f96,stroke:#333,stroke-width:3px
```

## Responsibilities

- **Does:** stub pin reads; type alias for poller
- **Does not:** debounce (core `ButtonEngine` does)

## Key types / functions

| Item | Role |
|------|------|
| `GpioButtons::new` | Construct |
| `rec_pressed` / `pwr_pressed` | Stub `false` |
| `DeviceButtons` | `ButtonPoller<GpioButtons>` |

## Dependencies

- **Outbound:** `zoop_core::{buttons::ButtonPoller, io::Buttons}`, `board::config`
- **Inbound:** `FirmwareEngine`

## Tests

Build-only.

## Status

HIL stub.

## Related

[../../core/buttons.md](../../core/buttons.md), [../board/config.md](../board/config.md), [../../architecture.md](../../architecture.md)
