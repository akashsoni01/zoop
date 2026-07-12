# Firmware crate (`zoop-firmware`)

ESP32-S3 firmware for the Waveshare ESP32-S3-ePaper-1.54 Zoop voice notepad. BSP modules implement `zoop-core` traits; most hardware paths are **stubs until HIL**.

## Layout

| Area | Docs |
|------|------|
| Entry / build | [main.md](main.md), [../config/build.md](../config/build.md) |
| App | [app/](app/) |
| Board | [board/](board/) |
| Audio | [audio/](audio/) |
| Display | [display/](display/) |
| Input | [input/](input/) |
| Network | [network/](network/) |
| Power | [power/](power/) |
| Storage | [storage/](storage/) |

## Build

Requires ESP toolchain (`espup install`, source `export-esp.sh`):

```bash
cd firmware && cargo build
```

Host CI excludes this crate: `cargo test --workspace --exclude zoop-firmware`.

## Status

**HIL stub** — compiles for `xtensa-esp32s3-espidf`; peripherals log and return safe defaults until board bring-up.
