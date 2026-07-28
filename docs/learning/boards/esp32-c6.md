# ESP32-C6 Board Guide

**RISC-V** with **Wi-Fi 6 (802.11ax)**, **BLE 5**, and **802.15.4** (Thread/Zigbee/Matter radio) — Espressif's multi-protocol IoT chip generation.

---

## Overview

| Spec | Typical |
|------|---------|
| **CPU** | RISC-V (HP + LP cores) @ 160 MHz |
| **Flash** | 4–8 MB |
| **SRAM** | 512 KB |
| **Wi-Fi** | 2.4 GHz 802.11ax |
| **802.15.4** | Thread / Zigbee / Matter |
| **Bluetooth** | BLE 5 |

---

## Memory Map Overview

```
0x4080_0000 ─── SRAM regions (HP system)
0x4200_0000 ─── Flash XIP
0x6000_0000 ─── Peripherals
```

**LP (Low Power) domain** — separate RAM and CPU for sleep-side tasks — advanced [24-low-power.md](../24-low-power.md).

---

## GPIO Layout Notes

Pin count varies by module (ESP32-C6-WROOM). Check dev kit schematic:

- **ADC** channels on selected pins
- **Strapping** pins for boot — GPIO4, 5, 8, 9, 15 common
- **3.3 V logic**

802.15.4 RF — keep antenna area clear; same layout concerns as Wi-Fi modules.

---

## Clock Tree

PLL from 40 MHz crystal; separate clocks for Wi-Fi 6 MAC, 802.15.4, BLE. Coexistence handled in radio controller — similar to ESP32-S3 Wi-Fi + BLE — [20-wifi.md](../20-wifi.md).

---

## Boot Sequence

Standard Espressif ROM → flash bootloader → app. Tooling via **espflash** / **espup** with C6 RISC-V target.

Target evolving — check [esp-rs book](https://esp-rs.github.io/book/) for current triple name.

---

## Flash & RAM

512 KB SRAM — Matter/Thread stacks may require careful heap tuning. Flash 8 MB recommended for OTA + Matter credentials.

---

## Interrupt Vectors Overview

RISC-V PLIC + Espressif matrix. Wi-Fi 6 and 802.15.4 events largely handled by binary libraries — application sees callback APIs.

---

## Peripherals

| Peripheral | Lesson relevance |
|------------|------------------|
| Wi-Fi 6 | [20-wifi.md](../20-wifi.md) |
| BLE | [21-bluetooth.md](../21-bluetooth.md) |
| 802.15.4 | Matter/Thread (ESP-IDF/Zigbee SDK) |
| UART, SPI, I²C, GDMA | Standard lessons |
| LP UART/I²C | Ultra-low-power side core |
| USB Serial/JTAG | Debug |

---

## HAL & PAC Crates

| Crate | Status |
|-------|--------|
| **esp32c6** | PAC |
| **esp-hal** | `features = ["esp32c6"]` — active development |
| **esp-idf-svc** | Full radio stacks via ESP-IDF |

Rust Matter/Thread support follows ESP-IDF maturity — verify crate README before production.

---

*802.15.4-only without Wi-Fi? Consider [esp32-h2.md](./esp32-h2.md).*
