# ESP32-S2 Board Guide

Single-core **Xtensa LX7** with **native USB OTG** — no Wi-Fi/BLE. Good for USB device learning ([19-usb.md](../19-usb.md)) on a budget.

---

## Overview

| Spec | Typical |
|------|---------|
| **CPU** | Xtensa LX7 @ 240 MHz |
| **Flash** | 4 MB+ |
| **SRAM** | 320 KB |
| **Wi-Fi / BLE** | None |
| **USB** | Full-speed OTG |

---

## Memory Map Overview

```
0x3FF0_0000 ─── SRAM 320 KB
0x3F00_0000 ─── Flash XIP
0x6000_0000 ─── Peripherals
```

No PSRAM on many S2 modules — large framebuffers limited.

---

## GPIO Layout Notes

| GPIO | Notes |
|------|-------|
| GPIO18 | Often onboard LED |
| GPIO0 | Boot strap |
| 43+ pins | Check module — fewer than ESP32 classic |

**Touch sensor** channels on many pins — capacitive sensing without external hardware.

All GPIO **3.3 V**; limited 5 V tolerance — read TRM pin list.

---

## Clock Tree

40 MHz XTAL → CPU/APB PLLs. **48 MHz USB clock** required for USB peripheral — enable before enumeration — [19-usb.md](../19-usb.md).

---

## Boot Sequence

ROM → 2nd stage → app. USB CDC download supported. No Wi-Fi coprocessor — simpler power profile than ESP32-S3.

Target: `xtensa-esp32s2-none-elf`

---

## Flash & RAM

320 KB RAM is tighter than S3 — avoid large `std` stacks. Suitable for USB HID, sensors, display via SPI.

---

## Interrupt Vectors Overview

Standard Espressif matrix: GPIO, SPI, I²C, USB, DMA, timers. Single core — no dual-core affinity concerns.

---

## Peripherals

| Peripheral | Lesson |
|------------|--------|
| USB OTG | [19-usb.md](../19-usb.md) |
| SPI, I²C, UART | [16-spi.md](../16-spi.md), [17-i2c.md](../17-i2c.md), [15-uart.md](../15-uart.md) |
| DMA | [14-dma.md](../14-dma.md) |
| AES, SHA, RSA accel | Crypto workloads |
| No TWAI on all variants | Check chip revision |

---

## HAL & PAC Crates

| Crate | Features |
|-------|----------|
| **esp32s2** | PAC |
| **esp-hal** | `features = ["esp32s2"]` |
| **embassy-usb** | USB device stacks |

---

*For Wi-Fi + USB, choose [esp32-s3.md](./esp32-s3.md) or [esp32-c3.md](./esp32-c3.md).*
