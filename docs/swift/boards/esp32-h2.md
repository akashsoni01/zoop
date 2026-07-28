# ESP32-H2 Board Guide

RISC-V SoC focused on **BLE 5.0** and **IEEE 802.15.4** (Thread/Zigbee) — **no Wi-Fi**.

---

## Overview

| Spec | ESP32-H2-DevKitM-1 |
|------|---------------------|
| **CPU** | RISC-V @ 96 MHz |
| **Flash** | 4 MB |
| **SRAM** | 320 KB |
| **Wi-Fi** | None |
| **Bluetooth** | BLE 5.0 |
| **802.15.4** | Thread / Zigbee / Matter RCP |
| **USB** | Serial/JTAG |

---

## Embedded Swift Maturity

| Path | Status |
|------|--------|
| ESP-IDF + Swift | Expected to follow C6 pattern — check latest examples |
| Bare-metal Swift | Not primary focus yet |
| Matter / Thread | [swift-matter-examples](https://github.com/swiftlang/swift-matter-examples) ecosystem |

Ideal for **BLE + mesh** without Wi-Fi coexistence issues — [21-bluetooth.md](../21-bluetooth.md).

---

## Memory Map Overview

```
320 KB SRAM — tighter than C6/S3
4 MB flash
802.15.4 radio domain
```

Plan smaller heaps than ESP32-S3 Wi-Fi projects.

---

## GPIO Notes

Check DevKitM-1 schematic for LED and button pins. USB Serial/JTAG for flash/debug — [25-debugging.md](../25-debugging.md).

---

## Peripherals

UART, SPI, I²C, BLE, 802.15.4. No [20-wifi.md](../20-wifi.md) on-chip. TWAI for CAN with external transceiver.

---

## Swift Toolchain

`idf.py set-target esp32h2`. Port C6 Embedded Swift projects; verify ESP-IDF 5.x+ H2 support.

---

*Compare: [nrf52.md](./nrf52.md) for Nordic BLE specialist*
