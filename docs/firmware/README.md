# Firmware crate (`zoop-firmware`)

ESP32-S3 board support package and thin engine over `zoop-core`. Most drivers are **HIL stubs** that log and compile for `xtensa-esp32s3-espidf` until Waveshare bring-up.

## Component in architecture

```mermaid
flowchart TB
  subgraph Device["ESP32-S3 firmware"]
    MAIN["main.rs"]
    ENGINE["FirmwareEngine"]
    BSP["BSP · SD · E-Ink · ES8311 · GPIO · WiFi"]
    MAIN --> ENGINE --> BSP
  end
  CORE["zoop-core"]
  ENGINE --> CORE
  style MAIN fill:#f96,stroke:#333,stroke-width:2px
  style ENGINE fill:#f96,stroke:#333,stroke-width:2px
  style BSP fill:#f96,stroke:#333,stroke-width:2px
```

Full system map: [architecture.md](../architecture.md).

## Child docs

| Area | Docs |
|------|------|
| Entry | [main.md](main.md) |
| App | [app/](app/) |
| Board | [board/](board/) |
| Audio | [audio/](audio/) |
| Display | [display/](display/) |
| Input | [input/](input/) |
| Network | [network/](network/) |
| Power | [power/](power/) |
| Storage | [storage/](storage/) |

## How components interact

`main` inits power → display → SD → audio → RTC → whisper config → `FirmwareEngine::boot`/`tick` loop. Engine wraps `zoop_core::App` with BSP stubs (`SdStorage`, `EpaperDisplay`, `Es8311Audio`, `GpioButtons`, …).

## Build

```bash
cd firmware && cargo build
```

## Status

**Build-time / HIL stub** — compiles; hardware paths pending. Host logic verified in `zoop-core`.
