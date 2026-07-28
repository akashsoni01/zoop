# ESP32-C3 Board Guide

**RISC-V** single-core Espressif SoC — low cost, Wi-Fi + BLE, excellent for learning RISC-V Rust toolchain.

---

## Overview

| Spec | Typical DevKitM-1 |
|------|-------------------|
| **PCB size** | **~48.3 × 20.3 mm** (L × W) — compact MINI module board |
| **CPU** | RISC-V 32-bit @ 160 MHz |
| **Flash** | 4 MB |
| **SRAM** | 400 KB |
| **Wi-Fi** | 802.11 b/g/n |
| **Bluetooth** | BLE 5.0 |
| **USB** | USB Serial/JTAG (built-in) |
| **Headers** | Dual 2.54 mm pitch |

### Board dimensions

```
           ~48.3 mm
    ┌────────────────────┐
    │ USB │ MINI-1 │ ant │  ~20.3 mm
    └────────────────────┘
```

Smaller than DevKitC — fits tighter breadboard / wearable prototypes. Official: Espressif [ESP32-C3-DevKitM-1 Dimensions](https://docs.espressif.com/projects/esp-idf/en/latest/esp32c3/hw-reference/esp32c3/user-guide-devkitm-1.html).

---

## Memory Map Overview

```
0x3FC8_0000 ─── SRAM (400 KB)
0x4200_0000 ─── Flash XIP (cached)
0x6000_0000 ─── Peripherals
```

RISC-V **no Xtensa** — different target triple and espup components.

---

## GPIO Layout Notes

| GPIO | DevKitM-1 common |
|------|------------------|
| GPIO8 | Onboard LED (active low on some boards) |
| GPIO9 | Boot button |
| GPIO18/19 | USB D-/D+ internal |

**Total ~22 GPIO** — fewer than S3. Strapping on GPIO2, 8, 9 at reset.

---

## Clock Tree

External 40 MHz → PLL for CPU, Wi-Fi, USB Serial/JTAG. Simpler than dual-core ESP32.

---

## Boot Sequence

ROM USB download or flash boot. **espflash** over built-in USB — similar workflow to S3 — [25-debugging.md](../25-debugging.md).

Targets:

- `riscv32imc-unknown-unknown-elf` (esp-hal bare metal)
- `riscv32imc-esp-espidf` (ESP-IDF)

---

## Flash & RAM

400 KB RAM — sufficient for BLE sensor nodes; tight for TLS + heavy heap. No PSRAM on standard C3 modules.

---

## Interrupt Vectors Overview

RISC-V **CLINT/PLIC** style via Espressif wrapper — GPIO, UART, Wi-Fi, timers. Single hart — straightforward concurrency — [22-embassy.md](../22-embassy.md).

---

## Peripherals

| Peripheral | Notes |
|------------|-------|
| UART × 2 | |
| SPI × 3 | |
| I²C × 1 | |
| GDMA | [14-dma.md](../14-dma.md) |
| TWAI | CAN |
| BLE + Wi-Fi | [20-wifi.md](../20-wifi.md), [21-bluetooth.md](../21-bluetooth.md) |
| No USB OTG | Serial/JTAG only — not full device stack on all configs |

---

## HAL & PAC Crates

| Crate | Notes |
|-------|-------|
| **esp32c3** | PAC |
| **esp-hal** | `features = ["esp32c3"]` |
| **esp-wifi** | Community Wi-Fi for esp-hal |

Install RISC-V Rust via **espup** — same tool as Xtensa, different toolchain slice.

---

*Need more GPIO/PSRAM? [esp32-s3.md](./esp32-s3.md). Wi-Fi 6? [esp32-c6.md](./esp32-c6.md).*
