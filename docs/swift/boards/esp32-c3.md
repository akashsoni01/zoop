# ESP32-C3 Board Guide

Low-cost **RISC-V** Espressif SoC with Wi-Fi + BLE. Smaller memory than S3 — good for cost-sensitive IoT.

---

## Overview

| Spec | Typical DevKitM-1 |
|------|-------------------|
| **CPU** | RISC-V single-core @ 160 MHz |
| **Flash** | 4 MB |
| **SRAM** | 400 KB |
| **Wi-Fi** | 802.11 b/g/n |
| **Bluetooth** | BLE 5.0 |
| **USB** | Serial/JTAG built-in |

---

## Embedded Swift Maturity

| Path | Status |
|------|--------|
| ESP-IDF + Swift | **Good** — same ESP-IDF path as C6 |
| Bare-metal Swift | Early RISC-V experiments |
| Official examples | C6 preferred; C3 ports straightforward |

Target triple: RISC-V via ESP-IDF (`riscv32-esp-elf` toolchain).

---

## Memory Map Overview

```
400 KB internal SRAM
4 MB flash typical
No PSRAM on most C3 modules
```

Tighter RAM than S3 — monitor Wi-Fi heap carefully.

---

## GPIO Notes

| GPIO | Notes |
|------|-------|
| GPIO8 | Often LED on DevKitM-1 |
| GPIO9 | Often button |
| USB | Internal Serial/JTAG |

3.3 V logic; check strapping pins GPIO2, GPIO8, GPIO9.

---

## Peripherals

UART×2, SPI×3, I²C, TWAI, Wi-Fi, BLE. Full lesson coverage except large PSRAM display buffers.

---

## Swift Toolchain

```bash
idf.py set-target esp32c3
```

Adapt [swift-embedded-examples](https://github.com/swiftlang/swift-embedded-examples) C6 projects by changing target.

---

*Upgrade path: [esp32-s3.md](./esp32-s3.md) for PSRAM + dual-core*
