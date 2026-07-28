# nRF52840 Board Guide (Nordic)

**ARM Cortex-M4F** with best-in-class **BLE** stack ecosystem — reference for [21-bluetooth.md](../21-bluetooth.md) and ultra-low-power [24-low-power.md](../24-low-power.md).

Example board: **nRF52840 DK**.

---

## Overview

| Spec | nRF52840 DK |
|------|-------------|
| **CPU** | Cortex-M4F @ 64 MHz |
| **Flash** | 1 MB |
| **SRAM** | 256 KB |
| **Bluetooth** | BLE 5, Mesh capable |
| **USB** | Full-speed device |
| **802.15.4** | Optional multiprotocol with SoftDevice |

---

## Memory Map Overview

```
0x0000_0000 ─── Flash 1 MB (SoftDevice reserves lower region if used)
0x2000_0000 ─── RAM 256 KB
0x4000_0000 ─── Peripherals
0x1000_0000 ─── FICR / UICR (factory info, user config)
0x4000_0000 ─── PPI — Programmable Peripheral Interconnect
```

**SoftDevice** (Nordic BLE stack binary) occupies fixed flash/RAM when enabled — check allocation map for S140 version.

Modern **trouble-host** + **embassy-nrf** path reduces SoftDevice dependency — verify current Embassy book.

---

## GPIO Layout Notes

nRF52840 DK:

| Pin | Function |
|-----|----------|
| **P0.13** | LED1 |
| **P0.14–16** | LEDs 2–4 |
| **Button 1–4** | GPIO with pull-up |
| **P0.06/P0.08** | UARTE default (via edge connector) |

**3.3 V** — no 5 V tolerance. **GPIO absolute max** 3.6 V.

**Pin mapping:** `P0.xx` and `P1.xx` (port 1 on 52840). `nrf52840-pac` uses `P0`, `P1` registers.

---

## Clock Tree

```
HFCLK ──► 64 MHz CPU (from crystal or HFINT)
LFCLK ──► 32.768 kHz RTC / BLE sleep timing
HFXO 32 MHz ──► radio accuracy
```

BLE connection scheduling requires stable **LFCLK** — use crystal, not RC, for production — [21-bluetooth.md](../21-bluetooth.md).

---

## Boot Sequence

1. M0 boots from flash vector table.
2. Optional **MBR** (Master Boot Record) with SoftDevice.
3. Rust `entry` via `cortex-m-rt`.

Flash:

```bash
probe-rs run --chip nRF52840_xxAA target/thumbv7em-none-eabihf/debug/app
```

On-board **J-Link OB** debugger — SWD + virtual UART.

---

## Flash & RAM

| | nRF52840 |
|---|----------|
| Flash | 1 MB |
| RAM | 256 KB |

SoftDevice S140 ~152 KB flash + ~12 KB RAM (version dependent) — plan layout in linker script.

---

## Interrupt Vectors Overview

NVIC with Nordic-specific priorities:

| IRQ | Source |
|-----|--------|
| `POWER_CLOCK` | Power events |
| `RADIO` | BLE 2.4 GHz |
| `UARTE0` | UART with EasyDMA — [14-dma.md](../14-dma.md) |
| `SPIM0` | SPI master |
| `TWIM0` | I²C master — [17-i2c.md](../17-i2c.md) |
| `GPIOTE` | Pin tasks/events |
| `SAADC` | ADC — [12-adc.md](../12-adc.md) |
| `USBD` | USB — [19-usb.md](../19-usb.md) |

**PPI** wires events to tasks without CPU — e.g., timer → ADC sample → RAM.

---

## Peripherals

| Peripheral | Nordic name | Lesson |
|------------|-------------|--------|
| UART | UARTE (EasyDMA) | [15-uart.md](../15-uart.md) |
| SPI | SPIM | [16-spi.md](../16-spi.md) |
| I²C | TWIM | [17-i2c.md](../17-i2c.md) |
| BLE radio | RADIO + controller | [21-bluetooth.md](../21-bluetooth.md) |
| USBD | Device | [19-usb.md](../19-usb.md) |
| PWM | PWM | [11-pwm.md](../11-pwm.md) |
| QSPI | External flash on some modules | |

**No Wi-Fi** native — use companion ESP or cellular module — [20-wifi.md](../20-wifi.md) on ESP32 instead.

---

## HAL & PAC Crates

| Crate | Role |
|-------|------|
| **nrf52840-pac** | PAC |
| **nrf-hal-common** | Shared HAL |
| **embassy-nrf** | Async BLE/USB/time |
| **trouble-host** | Rust BLE host |
| **softdevice** | SoftDevice bindings (legacy path) |

Target: **`thumbv7em-none-eabihf`**

probe-rs chip: **`nRF52840_xxAA`**

```toml
[dependencies]
embassy-nrf = { version = "0.3", features = ["nrf52840", "time-driver-rtc1"] }
```

---

## Low Power Strength

**System ON** idle ~few µA with RAM retention; **System OFF** ~300 nA with GPIO wake — benchmark with PPK2 — [24-low-power.md](../24-low-power.md).

---

*Index: [boards/README.md](./README.md)*
