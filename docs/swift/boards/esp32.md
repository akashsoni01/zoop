# ESP32 (Original) Board Guide

Dual-core **Xtensa LX6** SoC — mature ESP-IDF ecosystem. For new Embedded Swift projects, prefer **ESP32-S3** unless you already own hardware.

---

## Overview

| Spec | Typical module |
|------|----------------|
| **CPU** | Dual Xtensa LX6 @ 240 MHz |
| **Flash** | 4–16 MB |
| **SRAM** | 520 KB |
| **Wi-Fi** | 802.11 b/g/n |
| **Bluetooth** | Classic + BLE |
| **USB** | External UART chip (CP2102/CH340) |

---

## Embedded Swift Maturity

| Path | Status |
|------|--------|
| ESP-IDF + Swift | Feasible — same pattern as S3/C6 |
| Bare-metal Swift | Minimal community work |
| Host companion | iOS/macOS CoreBluetooth + Network |

No dedicated examples in [swift-embedded-examples](https://github.com/swiftlang/swift-embedded-examples) for original ESP32 — port from C6/S3 templates.

---

## Memory Map Overview

```
0x3FF0_0000 ─── Internal SRAM (~520 KB)
0x3F40_0000 ─── External PSRAM (optional)
0x3F40_0000 ─── Flash XIP mapping
0x3FF0_0000 ─── Peripheral registers
```

RTC memory: 8 KB slow + 8 KB fast — [24-low-power.md](../24-low-power.md).

---

## GPIO Layout Notes

| Pin | Common use |
|-----|------------|
| GPIO2 | Onboard LED (strap pin) |
| GPIO0 | Boot strap |
| GPIO34–39 | Input only |

ADC2 conflicts with Wi-Fi — use ADC1 when radio active.

---

## Clock Tree

40 MHz crystal → PLLs for CPU (240 MHz), APB (80 MHz), Wi-Fi/BT. Same concepts as [esp32-s3.md](./esp32-s3.md).

---

## Boot Sequence

ROM → 2nd stage → app. Flash via external USB-UART (not native JTAG). Use `idf.py set-target esp32`.

---

## Peripherals

UART×3, SPI, I²C, TWAI, no native USB device. Lessons [15-uart.md](../15-uart.md) through [18-can.md](../18-can.md) apply; [19-usb.md](../19-usb.md) requires external chip.

---

## Swift Toolchain

ESP-IDF component integration; target `esp32`. External UART for serial debug — [25-debugging.md](../25-debugging.md).

---

*See also: [esp32-s3.md](./esp32-s3.md)*
