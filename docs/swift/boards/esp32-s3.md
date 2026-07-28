# ESP32-S3 Board Guide

**Recommended primary board** for this Embedded Swift curriculum. Pairs with lessons [08-gpio.md](../08-gpio.md) through [21-bluetooth.md](../21-bluetooth.md).

---

## Overview

| Spec | Typical DevKitC-1 (N8R8) |
|------|--------------------------|
| **CPU** | Dual-core Xtensa LX7 up to 240 MHz |
| **Flash** | 8 MB external (QIO) |
| **SRAM** | 512 KB internal + 8 MB PSRAM (octal) |
| **Wi-Fi** | 802.11 b/g/n 2.4 GHz |
| **Bluetooth** | BLE 5.0 |
| **USB** | USB-OTG (GPIO19/20) + USB Serial/JTAG |

---

## Embedded Swift Maturity

| Path | Status |
|------|--------|
| **Embedded Swift + ESP-IDF** | **Best on S3** for Wi-Fi/BLE/USB — Swift app layer, C drivers |
| **Bare-metal Swift** | **Early** — fewer examples than C6; use swift-mmio experimentally |
| **Host Swift (iOS/macOS)** | **Production** — CoreBluetooth + Network companion apps |

Official [swift-embedded-examples](https://github.com/swiftlang/swift-embedded-examples) focus on **ESP32-C6**; S3 projects follow the same ESP-IDF integration pattern with `esp32s3` target.

---

## Memory Map Overview

```
0x3FF0_0000 ─┬─ Internal SRAM (512 KB)
             │   ├── DRAM for data/BSS/heap
             │   ├── IRAM for code (optional)
             │   └── RTC FAST/SLOW (8 KB each)
0x3C00_0000 ─┴─ External flash (XIP)
0x6000_0000 ─── Peripheral registers
0x3F40_0000 ─── PSRAM (when enabled)
```

| Region | Use in Embedded Swift |
|--------|----------------------|
| **Flash** | Code, constants, CA certs |
| **Internal SRAM** | Stack, DMA buffers, Wi-Fi heap |
| **PSRAM** | Display buffers, large allocations via ESP-IDF |
| **RTC memory** | `@Section(".rtc.data")` for deep sleep — [24-low-power.md](../24-low-power.md) |

---

## GPIO Layout Notes

DevKitC-1 commonly breaks out:

| Function | GPIO | Notes |
|----------|------|-------|
| **Onboard LED** | GPIO48 | Active high — verify schematic |
| **UART0 TX/RX** | GPIO43/44 | Default console |
| **USB D-/D+** | GPIO19/20 | USB-OTG — strap sensitive |
| **Strapping** | GPIO0, 3, 45, 46 | Boot mode |
| **JTAG** | Internal USB | No external wiring |

- **3.3 V logic** — not 5 V tolerant.
- **ADC1** GPIO1–10; **ADC2** conflicts with Wi-Fi.

---

## Clock Tree

```
40 MHz crystal → PLL
        ├── CPU: up to 240 MHz
        ├── APB: 80 MHz typical
        ├── Wi-Fi/BT domain
        └── USB 48 MHz
```

Recompute UART/SPI baud after clock changes — [15-uart.md](../15-uart.md).

---

## Boot Sequence

1. ROM bootloader reads strapping → SPI flash or UART download.
2. 2nd stage bootloader loads partition table.
3. Application entry — Embedded Swift `@main` via ESP-IDF.
4. USB Serial/JTAG available early.

Flash (ESP-IDF project):

```bash
idf.py set-target esp32s3
idf.py build flash monitor
```

---

## Flash & RAM

| Resource | DevKit N8R8 | Tips |
|----------|-------------|------|
| Flash | 8 MB | Wi-Fi/BLE stack consumes significant flash |
| Internal RAM | 512 KB | Monitor `esp_get_free_heap_size()` |
| PSRAM | 8 MB | Enable for graphics — not all DMA paths support PSRAM |
| RTC RAM | 16 KB | Deep sleep persistence |

---

## Interrupt Vectors Overview

ESP32-S3 **interrupt matrix** (not ARM NVIC):

| Category | Examples |
|----------|----------|
| GPIO | Pin edge/level |
| UART | RX FIFO, TX done |
| GDMA | Transfer complete — [14-dma.md](../14-dma.md) |
| Wi-Fi/BT | Opaque in binary stack |
| Timer | SYSTIMER, TG0/TG1 |

Keep ISRs short — [23-realtime-patterns.md](../23-realtime-patterns.md).

---

## Peripherals

| Peripheral | Count | Lesson |
|------------|-------|--------|
| UART | 3 | [15-uart.md](../15-uart.md) |
| SPI | 4 (SPI0/1 flash) | [16-spi.md](../16-spi.md) |
| I²C | 2 | [17-i2c.md](../17-i2c.md) |
| GDMA | Multiple | [14-dma.md](../14-dma.md) |
| TWAI (CAN) | 1 + external PHY | [18-can.md](../18-can.md) |
| USB OTG | 1 | [19-usb.md](../19-usb.md) |
| Wi-Fi / BLE | Radio | [20-wifi.md](../20-wifi.md), [21-bluetooth.md](../21-bluetooth.md) |

---

## Swift Toolchain

| Component | Notes |
|-----------|-------|
| **Swift 6.2+ dev snapshot** | Embedded Swift feature flag |
| **Swiftly** | Toolchain management |
| **ESP-IDF 5.x / 6.x** | C HAL, FreeRTOS, Wi-Fi/BLE |
| **swift-mmio** | Bare-metal register access (experimental on S3) |
| **Xcode Embedded Swift template** | Optional IDE integration |

Target: build Embedded Swift as ESP-IDF component; see Espressif [Embedded Swift blog](https://developer.espressif.com/blog/build-embedded-swift-application-for-esp32c6/) (adapt target to `esp32s3`).

---

## Related Boards

- [esp32-c6.md](./esp32-c6.md) — official Embedded Swift example target
- [esp32-s2.md](./esp32-s2.md) — USB without radio
- [esp32-c3.md](./esp32-c3.md) — smaller RISC-V

---

*Index: [boards/README.md](./README.md)*
