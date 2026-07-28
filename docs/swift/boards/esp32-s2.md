# ESP32-S2 Board Guide

Single-core **Xtensa LX7** with **native USB OTG** — no Wi-Fi or Bluetooth. Good for USB device lessons without radio complexity.

---

## Overview

| Spec | Typical DevKit |
|------|----------------|
| **CPU** | Xtensa LX7 @ 240 MHz |
| **Flash** | 4 MB typical |
| **SRAM** | 320 KB |
| **Wi-Fi / BLE** | None |
| **USB** | Full-speed OTG (GPIO19/20) |

---

## Embedded Swift Maturity

| Path | Status |
|------|--------|
| ESP-IDF + Swift | Supported via ESP-IDF (adapt C6 examples) |
| Bare-metal Swift | Community experiments only |
| USB in Swift | ESP-IDF TinyUSB + Swift app layer — [19-usb.md](../19-usb.md) |

---

## Memory Map Overview

Similar to ESP32-S3 but smaller RAM, no PSRAM on many modules:

```
Internal SRAM 320 KB
Flash XIP
Peripheral registers @ 0x6000_0000
```

---

## GPIO Notes

| GPIO | Notes |
|------|-------|
| GPIO19/20 | USB D-/D+ |
| GPIO0 | Strapping |
| Onboard LED | Board-specific — check schematic |

---

## Peripherals

UART×2, SPI, I²C, USB OTG, TWAI. No radio lessons [20-wifi.md](../20-wifi.md) / [21-bluetooth.md](../21-bluetooth.md) on-chip.

---

## Swift Toolchain

`idf.py set-target esp32s2`. USB debug via serial/JTAG on some boards.

---

*Compare: [esp32-s3.md](./esp32-s3.md) for Wi-Fi + BLE + USB*
