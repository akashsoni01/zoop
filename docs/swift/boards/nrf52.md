# nRF52840 Board Guide

**Nordic Semiconductor** BLE specialist — nRF52840 DK is the reference for [21-bluetooth.md](../21-bluetooth.md) without Wi-Fi coexistence.

Embedded Swift on nRF: **minimal** in official examples. BLE development typically **nRF Connect SDK (C)** or **Rust embassy-nrf** today; use **CoreBluetooth on iOS/macOS** as host companion for Swift-centric workflows.

---

## Overview

| Spec | nRF52840 DK |
|------|-------------|
| **CPU** | ARM Cortex-M4F @ 64 MHz |
| **Flash** | 1 MB |
| **SRAM** | 256 KB |
| **Bluetooth** | BLE 5.0 + Mesh |
| **USB** | Full-speed device |
| **Debug** | Onboard J-Link |

---

## Embedded Swift Maturity

| Path | Status |
|------|--------|
| Bare-metal Swift | Not in official examples |
| nRF Connect SDK + Swift | Theoretical C interop — immature |
| **CoreBluetooth host** | **Production** — pair DK firmware (any language) with Swift iOS app |

Best learning path: flash Nordic **NUS example** (C) on DK; build **Swift central** on iPhone — [21-bluetooth.md](../21-bluetooth.md).

---

## Memory Map Overview

```
0x0000_0000 ─── Flash (1 MB)
0x2000_0000 ─── RAM (256 KB)
0x4000_0000 ─── Peripherals
0xE000_0000 ─── NVIC
```

---

## GPIO Notes

nRF52840 DK:

| Pin | Function |
|-----|----------|
| P0.13 | LED1 |
| P0.14 | LED2 |
| P0.15 | LED3 |
| P0.16 | LED4 |
| P0.11 | Button 1 |
| P0.12 | Button 2 |

EasyDMA on UARTE, SPIM, TWIM — [14-dma.md](../14-dma.md).

---

## Clock Tree

64 MHz core from HFCLK; 32 kHz LFCLK for RTC and BLE sleep timing — [24-low-power.md](../24-low-power.md) excellence.

---

## Peripherals

UARTE, SPIM, TWIM, SAADC, USB device, BLE radio. No Wi-Fi — [20-wifi.md](../20-wifi.md) requires external module.

---

## Swift Toolchain (host-focused)

1. Flash Nordic SDK sample via `west flash`.
2. Build iOS/macOS app with **CoreBluetooth** — scan/connect to DK.
3. Optional: experiment with swift-mmio on M4 for bare-metal blink.

J-Link debug via `nrfjprog` or SEGGER tools — [25-debugging.md](../25-debugging.md).

---

*BLE lesson: [21-bluetooth.md](../21-bluetooth.md)*
