# app/engine.rs

- **Path:** `firmware/src/app/engine.rs`
- **Purpose:** Device-side application wrapper. Owns BSP stubs, `EspClock`, `FirmwareTime` (RTC+NTP), index/tag stores, and a `zoop_core::App`. `boot`/`tick` forward into core; full peripheral wiring awaits HIL.

## Component in architecture

```mermaid
flowchart TB
  ENG["FirmwareEngine"]
  APP["zoop_core::App"]
  SD["SdStorage"]
  EPD["EpaperDisplay"]
  AUD["Es8311Audio"]
  BTN["DeviceButtons"]
  NET["Wifi/Ntp/Portal/Whisper"]
  ENG --> APP
  ENG --> SD & EPD & AUD & BTN & NET
  style ENG fill:#f96,stroke:#333,stroke-width:3px
```

## Responsibilities

- **Does:** construct core `App`, expose `boot`/`tick`, provide `Clock`/`TimeSource` stubs
- **Does not:** raw GPIO (other modules)

## Key types / functions

| Item | Role |
|------|------|
| `EspClock` | Monotonic `now_ms` stub |
| `FirmwareTime` | UTC cache + `NtpClient` + `RtcChip`; implements `TimeSource` |
| `FirmwareEngine::new` | Wire storage/display/audio/rtc/whisper |
| `boot` / `tick` | Delegate to core |

## Data / control flow

```mermaid
sequenceDiagram
  participant Main
  participant Eng as FirmwareEngine
  participant App as zoop_core::App
  Main->>Eng: boot
  Eng->>App: boot(clock)
  Main->>Eng: tick
  Eng->>App: tick(buttons, clock)
```

## Dependencies

Outbound: nearly all firmware BSP modules + `zoop_core`. Inbound: `main`.

## Tests

Firmware excluded from host CI. Build-only.

## Status

HIL stub.

## Related

[../../core/app.md](../../core/app.md), [../main.md](../main.md), [../../architecture.md](../../architecture.md)
