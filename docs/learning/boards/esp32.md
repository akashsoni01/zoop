# ESP32 (Original) Board Guide

Dual-core **Xtensa LX6** SoC — mature ecosystem but superseded by **ESP32-S3** for new Rust projects unless you already own hardware.

---

## Overview

| Spec | Typical module |
|------|----------------|
| **CPU** | Dual Xtensa LX6 @ 240 MHz |
| **Flash** | 4–16 MB |
| **SRAM** | 520 KB |
| **Wi-Fi** | 802.11 b/g/n |
| **Bluetooth** | Classic + BLE |
| **USB** | External UART chip (CP2102/CH340) — no native USB device |

See [esp32-s3.md](./esp32-s3.md) for modern equivalent with USB-JTAG.

---

## Memory Map Overview

Similar to ESP32-S3 but smaller PSRAM adoption on older modules:

```
0x3FF0_0000 ─── Internal SRAM (~520 KB)
0x3F40_0000 ─── External PSRAM (optional, module-dependent)
0x3F40_0000 ─── Flash XIP mapping
0x3FF0_0000 ─── Peripheral registers
```

**RTC memory:** 8 KB slow + 8 KB fast for deep sleep — [24-low-power.md](../24-low-power.md).

---

## GPIO Layout Notes

Classic DevKit (e.g., ESP32-WROOM-32):

| Pin | Common use |
|-----|------------|
| GPIO2 | Onboard LED (strap pin — caution) |
| GPIO0 | Boot strap |
| GPIO34–39 | Input only (no pull-up) |

**Strapping pins:** GPIO0, 2, 4, 5, 12, 15 affect boot — read TRM before heavy pull-ups.

ADC2 **conflicts with Wi-Fi** — use ADC1 only when radio active.

---

## Clock Tree

40 MHz crystal → PLLs for CPU (240 MHz), APB (80 MHz), Wi-Fi/BT baseband. Same conceptual tree as S3 — [esp32-s3.md](./esp32-s3.md).

---

## Boot Sequence

1. ROM loader from SPI flash or UART download.
2. Bootloader + app partition.
3. External USB-UART for serial — not integrated JTAG.

Flash: `espflash flash --monitor target/xtensa-esp32-none-elf/debug/app`

---

## Flash & RAM

| | Typical |
|---|---------|
| Flash | 4 MB |
| RAM | 520 KB |
| PSRAM | 0–4 MB (module option) |

Wi-Fi stack RAM footprint similar to S3 — plan ~150 KB for network apps — [20-wifi.md](../20-wifi.md).

---

## Interrupt Vectors Overview

Proprietary interrupt matrix — GPIO, UART, SPI, Wi-Fi, timers. Two CPU cores — pin tasks to core 0 for Wi-Fi compatibility (ESP-IDF convention).

---

## Peripherals

| Peripheral | Notes |
|------------|-------|
| UART × 3 | [15-uart.md](../15-uart.md) |
| SPI × 4 | SPI0/1 flash-bound |
| I²C × 2 | [17-i2c.md](../17-i2c.md) |
| I²S, RMT, LEDC | Audio/PWM |
| TWAI | CAN with PHY — [18-can.md](../18-can.md) |
| DAC × 2 | Real DAC — [13-dac.md](../13-dac.md) |
| No native USB OTG | Use external chip for USB |

---

## HAL & PAC Crates

| Crate | Target |
|-------|--------|
| **esp32** | PAC |
| **esp-hal** | `features = ["esp32"]` |
| **esp-idf-hal/svc** | `xtensa-esp32-espidf` |

Target: `xtensa-esp32-none-elf`

---

*Prefer [esp32-s3.md](./esp32-s3.md) for new purchases.*
