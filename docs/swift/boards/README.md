# Board Guides — Index

MCU-specific reference guides for the **Embedded Swift** learning path. Each document covers memory maps, clocks, boot flow, interrupts, peripherals, and the Swift toolchain ecosystem.

**Primary learning board:** [ESP32-S3](./esp32-s3.md) — best Espressif target for PSRAM, USB, Wi-Fi, and BLE with Swift via ESP-IDF.

---

## How to Choose a Board

| Goal | Recommended board | Guide |
|------|-------------------|-------|
| Wi-Fi + BLE + USB + Swift | ESP32-S3 DevKitC-1 | [esp32-s3.md](./esp32-s3.md) |
| Official Embedded Swift examples (ESP-IDF) | ESP32-C6 DevKitC-1 | [esp32-c6.md](./esp32-c6.md) |
| Bare-metal Swift PoC (no OS) | ESP32-C6 | [esp32-c6.md](./esp32-c6.md) |
| Wi-Fi 6 + Thread/Zigbee | ESP32-C6 / H2 | [esp32-c6.md](./esp32-c6.md), [esp32-h2.md](./esp32-h2.md) |
| Industry ARM | STM32 Nucleo-F411/F446 | [stm32.md](./stm32.md) |
| Education, low cost | Raspberry Pi Pico (RP2040) | [rp2040.md](./rp2040.md) |
| BLE specialist | nRF52840 DK | [nrf52.md](./nrf52.md) |
| USB-only Espressif | ESP32-S2 | [esp32-s2.md](./esp32-s2.md) |
| Legacy 8-bit | Arduino Uno (ATmega328P) | [avr.md](./avr.md) |
| Ultra-low-power | LaunchPad MSP430FR2xx | [msp430.md](./msp430.md) |

### Decision checklist

1. **Required radios?** Wi-Fi → ESP32-S3/C6; BLE only → nRF52 or ESP32-H2.
2. **Swift path?** ESP-IDF + Embedded Swift (maturest) vs bare-metal (C6 PoC only).
3. **Debug probe?** ESP32-S3 includes USB-JTAG; STM32 Nucleo includes ST-Link.
4. **RAM/flash budget?** Large apps → ESP32-S3 + PSRAM; tiny sensor → MSP430 or C3.
5. **Host companion?** Pair MCU with iOS/macOS Swift (CoreBluetooth, Network) — [21-bluetooth.md](../21-bluetooth.md), [20-wifi.md](../20-wifi.md).

---

## Embedded Swift Maturity by Board

| Board | ESP-IDF + Swift | Bare-metal Swift | Host Swift companion |
|-------|-----------------|------------------|----------------------|
| ESP32-C6 | **Best** (official examples) | Community PoC | iOS/macOS |
| ESP32-S3 | Good (ESP-IDF) | Early | iOS/macOS |
| ESP32-C3 | Good | Early | iOS/macOS |
| STM32 | Blink examples | Experimental | macOS |
| RP2040 | — | Blink examples | macOS |
| nRF52 | — | Minimal | CoreBluetooth |
| AVR / MSP430 | — | Not viable yet | — |

Honest assessment: **no board has a complete pure-Swift HAL** comparable to Rust `esp-hal` or `embassy-stm32` yet. Plan for **Swift app logic + C HAL (ESP-IDF)** or **swift-mmio bare-metal** on supported targets.

---

## Espressif Family Comparison

| Chip | CPU | Wi-Fi | BLE | USB | Embedded Swift path |
|------|-----|-------|-----|-----|---------------------|
| [ESP32](./esp32.md) | Xtensa LX6 dual | 2.4 GHz | Classic+LE | External UART | ESP-IDF |
| [ESP32-S2](./esp32-s2.md) | Xtensa LX7 | — | — | OTG | ESP-IDF |
| [ESP32-S3](./esp32-s3.md) | Xtensa LX7 dual | 2.4 GHz | LE | OTG + JTAG | ESP-IDF |
| [ESP32-C3](./esp32-c3.md) | RISC-V | 2.4 GHz | LE | Serial/JTAG | ESP-IDF |
| [ESP32-C6](./esp32-c6.md) | RISC-V dual | Wi-Fi 6 | LE + 802.15.4 | Serial/JTAG | ESP-IDF + bare-metal PoC |
| [ESP32-H2](./esp32-h2.md) | RISC-V | — | LE + 802.15.4 | Serial/JTAG | ESP-IDF |

Tooling: [Swiftly](https://www.swift.org/install/), [swift-embedded-examples](https://github.com/swiftlang/swift-embedded-examples), ESP-IDF, [espflash](https://github.com/esp-rs/espflash).

---

## Non-Espressif Guides

| Guide | Embedded Swift highlights |
|-------|---------------------------|
| [stm32.md](./stm32.md) | `swift-embedded-examples/stm32-blink`, swift-mmio |
| [rp2040.md](./rp2040.md) | Community blink; Pico SDK via C interop |
| [nrf52.md](./nrf52.md) | Minimal; CoreBluetooth on host for testing |
| [avr.md](./avr.md) | Not supported — use host Swift or C firmware |
| [msp430.md](./msp430.md) | Not supported — experimental C only |

---

## What Each Guide Contains

1. **Memory map** — flash, RAM, RTC, PSRAM
2. **GPIO** — strapping, LED/button pins
3. **Clock tree** — sources, PLLs
4. **Boot sequence** — ROM bootloader, flash offset
5. **Interrupt vectors** — NVIC or platform equivalent
6. **Peripherals** — instance counts
7. **Swift toolchain** — targets, examples, honesty about gaps

---

## Cross-Links to Lessons

| Topic | Lessons |
|-------|---------|
| DMA | [14-dma.md](../14-dma.md) |
| UART / SPI / I²C | [15-uart.md](../15-uart.md), [16-spi.md](../16-spi.md), [17-i2c.md](../17-i2c.md) |
| Wi-Fi / BLE | [20-wifi.md](../20-wifi.md), [21-bluetooth.md](../21-bluetooth.md) |
| Concurrency / Real-time | [22-swift-concurrency.md](../22-swift-concurrency.md), [23-realtime-patterns.md](../23-realtime-patterns.md) |
| Debug / Test | [25-debugging.md](../25-debugging.md), [26-testing.md](../26-testing.md) |

---

*Back to Embedded Swift curriculum (lessons 01–27)*
