# board/rtc.rs

- **Path:** `firmware/src/board/rtc.rs`
- **Purpose:** PCF85063 RTC stub on I2C. Logs address/pins; `read_utc_iso` returns `None` until HIL. Used by `FirmwareTime` for persistence of NTP results.

## Component in architecture

```mermaid
flowchart LR
  NTP["NtpClient"]
  TIME["FirmwareTime"]
  RTC["RtcChip HIL stub"]
  NTP --> TIME --> RTC
  style RTC fill:#f96,stroke:#333,stroke-width:3px
```

## Responsibilities

Stub init/read/write/sync helpers.

## Key types / functions

| Item | Role |
|------|------|
| `RtcChip::init` | Log I2C `0x51` |
| `read_utc_iso` | `None` (stub) |
| `write_utc_iso` | Log only |
| `sync_system_from_chip` / `sync_chip_from_system` | Log stubs |

## Dependencies

`board::config` I2C constants. Inbound: `main`, `FirmwareTime`.

## Status

HIL stub.

## Related

[../../core/time.md](../../core/time.md), [config.md](config.md), [../../architecture.md](../../architecture.md)
