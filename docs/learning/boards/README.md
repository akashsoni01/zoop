# Board Guides — Index

This directory contains **MCU-specific reference guides** for the Embedded Rust learning path. Each document covers memory maps, clocks, boot flow, interrupts, peripherals, and the Rust **PAC** (Peripheral Access Crate) / **HAL** (Hardware Abstraction Layer) ecosystem.

**Primary learning board:** [ESP32-S3](./esp32-s3.md) — start here if you own one DevKit.

---

## How to Choose a Board

| Goal | Recommended board | Approx. PCB size (L × W) | Guide |
|------|-------------------|--------------------------|-------|
| Wi-Fi + BLE + USB + Rust | ESP32-S3 DevKitC-1 | **62.7 × 25.4 mm** | [esp32-s3.md](./esp32-s3.md) |
| Lowest cost Espressif RISC-V | ESP32-C3 DevKitM-1 | **~48 × 20 mm** | [esp32-c3.md](./esp32-c3.md) |
| Wi-Fi 6 + Thread/Zigbee | ESP32-C6 / H2 | **~55 × 25 mm** / mini | [esp32-c6.md](./esp32-c6.md), [esp32-h2.md](./esp32-h2.md) |
| Industry ARM, RTIC/Embassy | STM32 Nucleo-64 | **82.5 × 70 mm** | [stm32.md](./stm32.md) |
| Education, PIO, `$4` | Raspberry Pi Pico | **51 × 21 mm** | [rp2040.md](./rp2040.md) |
| BLE specialist | nRF52840 DK | **102 × 64 mm** | [nrf52.md](./nrf52.md) |
| Legacy 8-bit AVR | Arduino Uno R3 | **68.6 × 53.4 mm** | [avr.md](./avr.md) |
| Ultra-low-power MSP430 | LaunchPad MSP430FR2xx | **~70 × 51 mm** | [msp430.md](./msp430.md) |

### Board dimensions (reference DevKits)

PCB outline sizes help with breadboard fit, 3D-printed cases, and enclosure design. Values are **length × width** for the common official DevKit (not the bare module). Thickness is typically **1.6 mm** FR4 plus connectors. Always verify the revision DXF/PDF from the vendor.

| Guide | Reference board | PCB (L × W) | Header pitch | Notes |
|-------|-----------------|-------------|--------------|-------|
| [esp32-s3.md](./esp32-s3.md) | ESP32-S3-DevKitC-1 | **62.74 × 25.40 mm** | 2.54 mm | Official Espressif PCB drawing |
| [esp32.md](./esp32.md) | ESP32-DevKitC-V4 | **~54.9 × 27.9 mm** | 2.54 mm | Classic dual-row DevKitC |
| [esp32-s2.md](./esp32-s2.md) | ESP32-S2-DevKitC-1 | **~54.4 × 25.4 mm** | 2.54 mm | Similar slim DevKitC family |
| [esp32-c3.md](./esp32-c3.md) | ESP32-C3-DevKitM-1 | **~48.3 × 20.3 mm** | 2.54 mm | Compact MINI-1 module board |
| [esp32-c6.md](./esp32-c6.md) | ESP32-C6-DevKitC-1 | **~51.6 × 25.4 mm** | 2.54 mm | Confirm revision DXF |
| [esp32-h2.md](./esp32-h2.md) | ESP32-H2-DevKitM-1 | **~48 × 20 mm** | 2.54 mm | Mini form like C3 DevKitM |
| [rp2040.md](./rp2040.md) | Raspberry Pi Pico | **51 × 21 mm** | 2.54 mm | Official Pico datasheet |
| [stm32.md](./stm32.md) | Nucleo-64 (e.g. F411RE) | **82.5 × 70 mm** | Arduino + Morpho | ST Nucleo-64 form factor |
| [nrf52.md](./nrf52.md) | nRF52840 DK | **102 × 64 mm** | Arduino-ish | Nordic PCA10056 |
| [avr.md](./avr.md) | Arduino Uno R3 | **68.6 × 53.4 mm** | Arduino | Classic Uno footprint |
| [msp430.md](./msp430.md) | MSP430 LaunchPad | **~70 × 51 mm** | BoosterPack | TI LaunchPad family |

```
Breadboard tip (ESP DevKitC / Pico style):

  ┌────────────────────────────┐  ← length (USB end → antenna)
  │  USB   MCU/module   ANT    │
  └────────────────────────────┘
         ↑ width (pin header to pin header ≈ 0.9–1.0")
```

Clone / third-party boards (OceanLabz camera kits, N16R8 modules, etc.) may differ — measure your PCB before designing a case.

### Decision checklist

1. **Required radios?** Wi-Fi → ESP32-S3/C6; BLE only → nRF52 or ESP32-H2.
2. **Debug probe?** ESP32-S3 and STM32 Nucleo include USB debug; Pico uses UF2 or SWD.
3. **Rust maturity?** ESP32-S3, STM32, RP2040, nRF52 have strong HAL ecosystems; AVR/MSP430 are narrower.
4. **RAM/flash budget?** Large apps (TLS + UI) → ESP32-S3 with PSRAM; tiny sensor → MSP430 or C3.
5. **Team familiarity?** Match existing hardware inventory when possible.

---

## Espressif Family Comparison

| Chip | CPU | Wi-Fi | BLE | USB | Rust target |
|------|-----|-------|-----|-----|-------------|
| [ESP32](./esp32.md) | Xtensa LX6 dual | 2.4 GHz | Classic+LE | External | `xtensa-esp32-none-elf` |
| [ESP32-S2](./esp32-s2.md) | Xtensa LX7 | — | — | OTG | `xtensa-esp32s2-none-elf` |
| [ESP32-S3](./esp32-s3.md) | Xtensa LX7 dual | 2.4 GHz | LE | OTG + JTAG | `xtensa-esp32s3-none-elf` |
| [ESP32-C3](./esp32-c3.md) | RISC-V | 2.4 GHz | LE | Serial/JTAG | `riscv32imc-esp-espidf` / esp-hal |
| [ESP32-C6](./esp32-c6.md) | RISC-V dual | 2.4 Wi-Fi 6 | LE + 802.15.4 | Serial/JTAG | RISC-V esp-hal |
| [ESP32-H2](./esp32-h2.md) | RISC-V | — | LE + 802.15.4 | Serial/JTAG | RISC-V esp-hal |

All Espressif guides share tooling: [espup](https://github.com/esp-rs/espup), [espflash](https://github.com/esp-rs/espflash), [esp-hal](https://github.com/esp-rs/esp-hal).

---

## Non-Espressif Guides

| Guide | Ecosystem highlights |
|-------|---------------------|
| [stm32.md](./stm32.md) | `embassy-stm32`, `probe-rs`, vast chip catalog |
| [rp2040.md](./rp2040.md) | `embassy-rp`, `rp2040-hal`, PIO |
| [nrf52.md](./nrf52.md) | `embassy-nrf`, SoftDevice/trouble BLE |
| [avr.md](./avr.md) | `avr-hal`, Arduino form factor |
| [msp430.md](./msp430.md) | `msp430-rt`, FRAM devices |

---

## What Each Guide Contains

Every board document follows the same outline:

1. **Board dimensions** — PCB length × width for the reference DevKit
2. **Memory map** — flash, RAM, special regions (RTC, PSRAM, FRAM)
3. **GPIO** — strapping, LED/button pins, 5 V tolerance warnings
4. **Clock tree** — sources, PLLs, CPU/bus frequencies
5. **Boot sequence** — ROM bootloader, flash offset, UF2/DFU
6. **Flash & RAM sizes** — typical dev kit parts
7. **Interrupt vectors** — NVIC or platform equivalent
8. **Peripherals** — UART/SPI/I²C instance counts
9. **HAL & PAC crates** — Cargo names, features, links

---

## Cross-Links to Lessons

| Topic | Lessons |
|-------|---------|
| GPIO / LED | [08-gpio.md](../08-gpio.md) |
| UART | [15-uart.md](../15-uart.md) |
| SPI / I²C | [16-spi.md](../16-spi.md), [17-i2c.md](../17-i2c.md) |
| Wi-Fi / BLE | [20-wifi.md](../20-wifi.md), [21-bluetooth.md](../21-bluetooth.md) |
| Async / RTIC | [22-embassy.md](../22-embassy.md), [23-rtic.md](../23-rtic.md) |
| Debug | [25-debugging.md](../25-debugging.md) |
| Power | [24-low-power.md](../24-low-power.md) |

---

## Suggested Learning Paths by Board

**ESP32-S3 path:** [esp32-s3.md](./esp32-s3.md) → lessons 08–21 → Embassy → Wi-Fi.

**STM32 path:** [stm32.md](./stm32.md) → RTIC blink → Embassy → probe-rs debug.

**RP2040 path:** [rp2040.md](./rp2040.md) → HAL GPIO → PIO lesson (external resources) → USB.

---

*Back to [learning README](../README.md)*
