# ESP32-C6 Board Guide

**Primary official target** for Embedded Swift examples from Apple/Espressif. RISC-V dual-core with Wi-Fi 6 and 802.15.4.

---

## Overview

| Spec | ESP32-C6-DevKitC-1 |
|------|---------------------|
| **CPU** | RISC-V dual-core @ 160 MHz |
| **Flash** | 8 MB typical |
| **SRAM** | 512 KB |
| **Wi-Fi** | 802.11ax (Wi-Fi 6) 2.4 GHz |
| **Bluetooth** | BLE 5.0 + IEEE 802.15.4 |
| **USB** | Serial/JTAG |

---

## Embedded Swift Maturity

| Path | Status |
|------|--------|
| ESP-IDF + Swift | **Official examples** — LED, NeoPixel, UART echo, Matter |
| Bare-metal Swift | **Best PoC** — [georgik/esp32-c6-swift-baremetal](https://github.com/georgik/esp32-c6-swift-baremetal) |
| swift-mmio + SVD | UART, GPIO, SPI, display on bare metal |

Start here if following [swift-embedded-examples](https://github.com/swiftlang/swift-embedded-examples) verbatim.

---

## Memory Map Overview

```
512 KB SRAM
8 MB flash typical
Peripherals @ standard Espressif map
802.15.4 radio — Thread/Matter path
```

---

## GPIO Notes

DevKitC-1:

| GPIO | Common use |
|------|------------|
| GPIO8 | RGB LED (NeoPixel example) |
| GPIO9 | Button |
| UART | Default console pins per board doc |

---

## Peripherals

UART, SPI, I²C, GDMA, TWAI, USB Serial/JTAG, Wi-Fi 6, BLE, 802.15.4. Lessons [14-dma.md](../14-dma.md)–[21-bluetooth.md](../21-bluetooth.md) apply; Wi-Fi 6 notes in [20-wifi.md](../20-wifi.md).

---

## Swift Toolchain

```bash
swiftly install 6.2-dev   # or latest embedded snapshot
git clone https://github.com/swiftlang/swift-embedded-examples
cd esp32-led-blink-sdk
idf.py set-target esp32c6
idf.py build flash monitor
```

Bare-metal:

```bash
# georgik/esp32-c6-swift-baremetal
make
espflash flash target/riscv32imac-unknown-none-elf/release/app.bin
```

Target: `riscv32-none-none-elf` / ESP-IDF RISC-V toolchain.

---

## Related

- [esp32-s3.md](./esp32-s3.md) — more RAM/PSRAM, Xtensa
- [esp32-h2.md](./esp32-h2.md) — 802.15.4 without Wi-Fi
- [swift-matter-examples](https://github.com/swiftlang/swift-matter-examples)

---

*Index: [boards/README.md](./README.md)*
