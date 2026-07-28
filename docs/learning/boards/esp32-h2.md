# ESP32-H2 Board Guide

**RISC-V** SoC focused on **802.15.4** (Thread/Zigbee/Matter) and **BLE** — **no Wi-Fi**. Ideal for mesh sensor networks and low-power radio learning.

---

## Overview

| Spec | Typical DevKitM-1 |
|------|-------------------|
| **PCB size** | **~48 × 20 mm** (L × W; mini DevKitM form factor) |
| **CPU** | RISC-V @ 96 MHz |
| **Flash** | 4 MB |
| **SRAM** | 320 KB |
| **Wi-Fi** | None |
| **802.15.4** | Yes — Thread/Zigbee |
| **Bluetooth** | BLE 5 |
| **Headers** | Dual 2.54 mm pitch |

### Board dimensions

Similar compact footprint to [ESP32-C3-DevKitM-1](./esp32-c3.md). Verify Espressif’s DXF for your H2 DevKit revision — RF layout and antenna keep-out dominate enclosure rules more than raw mm.

---

## Memory Map Overview

```
0x4080_0000 ─── SRAM 320 KB
0x4200_0000 ─── Flash XIP
0x6000_0000 ─── Peripherals
```

Smaller RAM — suitable for border router / sleepy end device roles with careful stack sizing.

---

## GPIO Layout Notes

Fewer pins than S3/C6 — prioritize radio front-end routing on module. DevKit provides LED, button, USB Serial/JTAG.

**3.3 V** IO; consult module pinout for ADC-capable pins.

---

## Clock Tree

40 MHz reference; lower max CPU than C6/S3 — sufficient for 802.15.4 MAC and BLE stack. LP clock domain for sleep timers — [24-low-power.md](../24-low-power.md).

---

## Boot Sequence

Espressif standard ROM bootloader. Flash with espflash once RISC-V H2 target installed via espup.

---

## Flash & RAM

320 KB RAM — plan for BLE + OpenThread concurrent use via ESP-IDF heap configuration. No PSRAM.

---

## Interrupt Vectors Overview

RISC-V interrupt controller; 802.15.4 and BLE radio interrupts serviced by Espressif binary stack. Application callbacks for packet RX/TX.

---

## Peripherals

| Peripheral | Notes |
|------------|-------|
| BLE | [21-bluetooth.md](../21-bluetooth.md) |
| IEEE 802.15.4 | Thread/Zigbee — ESP-IDF / Matter SDK |
| UART, SPI, I²C | Standard |
| USB Serial/JTAG | Debug |
| No Wi-Fi | Use [esp32-c6.md](./esp32-c6.md) for Wi-Fi 6 + 802.15.4 |

---

## HAL & PAC Crates

| Crate | Notes |
|-------|-------|
| **esp32h2** | PAC |
| **esp-hal** | `features = ["esp32h2"]` |
| **esp-idf-svc** | OpenThread/BLE integration |

Pure Rust mesh stacks less mature than ESP-IDF — expect C library interop for Matter production.

---

*Wi-Fi needed? [esp32-s3.md](./esp32-s3.md) or [esp32-c6.md](./esp32-c6.md).*
