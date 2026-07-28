# ESP32-S3 Board Guide

**Recommended primary board** for this curriculum. Pairs with lessons [08-gpio.md](../08-gpio.md) through [21-bluetooth.md](../21-bluetooth.md).

---

## Overview

| Spec | Typical DevKitC-1 (N8R8) |
|------|--------------------------|
| **PCB size** | **62.74 × 25.40 mm** (L × W); ~1.6 mm thick FR4 |
| **CPU** | Dual-core Xtensa LX7 up to 240 MHz |
| **Flash** | 8 MB external (QIO) |
| **SRAM** | 512 KB internal + 8 MB PSRAM (octal) |
| **Wi-Fi** | 802.11 b/g/n 2.4 GHz |
| **Bluetooth** | BLE 5.0 |
| **USB** | USB-OTG (GPIO19/20) + USB Serial/JTAG |
| **Headers** | Dual 2.54 mm pitch; breadboard-friendly width |

### Board dimensions

```
                62.74 mm
    ┌──────────────────────────────┐
    │ USB-C │ WROOM-1 │ PCB ant.   │  25.40 mm
    └──────────────────────────────┘
         dual 2.54 mm pin headers
```

Official drawing: Espressif [ESP32-S3-DevKitC-1 Dimensions (PDF/DXF)](https://docs.espressif.com/projects/esp-dev-kits/en/latest/esp32s3/esp32-s3-devkitc-1/user_guide_v1.1.html). Module alone (WROOM-1) is smaller (~**25.5 × 18.0 mm**); keep antenna clearance free of metal in enclosures.

---

## Memory Map Overview

```
0x3FF0_0000 ─┬─ Internal SRAM (512 KB total, multiple regions)
             │   ├── DRAM for data/BSS/heap
             │   ├── IRAM for code (optional)
             │   └── RTC FAST/SLOW (8 KB each) — survives light sleep
0x3C00_0000 ─┴─ External flash mapped (XIP) — code & rodata
0x6000_0000 ─── Peripheral registers (UART, SPI, GPIO, GDMA, ...)
0x3F40_0000 ─── PSRAM (when enabled) — large buffers, framebuffers
```

| Region | Use in Rust |
|--------|-------------|
| **Flash** | `.text`, `.rodata`, `const` |
| **Internal SRAM** | `.data`, `.bss`, stack, DMA buffers |
| **PSRAM** | `esp-alloc` heap extension, display buffers |
| **RTC memory** | `#[link_section = ".rtc.data"]` for deep sleep persistence |

See [03-memory-layout.md](../03-memory-layout.md) and [14-dma.md](../14-dma.md) for DMA-capable regions.

---

## GPIO Layout Notes

DevKitC-1 commonly breaks out:

| Function | GPIO | Notes |
|----------|------|-------|
| **Onboard LED** | GPIO48 | Active high on many revisions — verify schematic |
| **UART0 TX/RX** | GPIO43/44 | Default console |
| **USB D-/D+** | GPIO19/20 | USB-OTG — strap sensitive |
| **Strapping** | GPIO0, 3, 45, 46 | Boot mode — avoid heavy loads at reset |
| **JTAG** | Internal USB | No external wiring |

**Rules:**

- **3.3 V logic** — not 5 V tolerant on most pins.
- **ADC1** pins GPIO1–10; **ADC2** conflicts with Wi-Fi — avoid ADC2 when radio active.
- Use **GPIO matrix** for peripheral pin mux — not fixed like STM32 AF tables.

---

## Clock Tree

```
External crystal (40 MHz typical)
        │
        ▼
    PLL (CPU, APB, Wi-Fi, USB)
        ├── CPU clock: up to 240 MHz
        ├── APB clock: 80 MHz typical
        ├── Wi-Fi/BT clock domain
        └── USB 48 MHz requirement
```

Configure via `esp_hal::clock::ClockControl` or ESP-IDF `sdkconfig`. After changing clocks, **recompute UART/SPI baud** dividers — [15-uart.md](../15-uart.md).

---

## Boot Sequence

1. **ROM bootloader** reads strapping pins → SPI flash or UART download mode.
2. **2nd stage bootloader** (in flash) loads partition table.
3. **Application** entry — Rust `entry` attribute or ESP-IDF `app_main`.
4. **USB Serial/JTAG** available early for flash/debug.

Flash with:

```bash
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/app
```

Partition table (ESP-IDF): `factory`, `ota_0`, `nvs`, `phy_init` — Rust `esp-idf` projects; bare `esp-hal` may use simpler layout.

---

## Flash & RAM

| Resource | DevKit N8R8 | Tips |
|----------|-------------|------|
| Flash | 8 MB | Monitor with `cargo size` |
| Internal RAM | 512 KB | Wi-Fi/BLE stacks consume large share |
| PSRAM | 8 MB | Enable in HAL; not all DMA paths support PSRAM |
| RTC RAM | 16 KB total | Split FAST/SLOW |

Enable PSRAM in project config for graphics — [16-spi.md](../16-spi.md) display buffers.

---

## Interrupt Vectors Overview

ESP32-S3 uses a **proprietary interrupt matrix** mapping peripheral sources to CPU exceptions (not ARM NVIC):

| Category | Examples |
|----------|----------|
| **GPIO** | Pin edge/level |
| **UART** | RX FIFO, TX done |
| **GDMA** | Transfer complete |
| **Wi-Fi/BT** | MAC events (handled by binary stack) |
| **Timer** | SYSTIMER, TG0/TG1 |

Rust ISRs via `esp-hal` interrupt handlers or `#[interrupt]` where supported. Wi-Fi interrupts mostly opaque in ESP-IDF — [20-wifi.md](../20-wifi.md).

Priority: configurable per source; keep ISRs short — [09-interrupts.md](../09-interrupts.md).

---

## Peripherals

| Peripheral | Count | Lesson |
|------------|-------|--------|
| UART | 3 | [15-uart.md](../15-uart.md) |
| SPI | 4 (SPI0/1 flash) | [16-spi.md](../16-spi.md) |
| I²C | 2 | [17-i2c.md](../17-i2c.md) |
| GDMA | Multiple channels | [14-dma.md](../14-dma.md) |
| TWAI (CAN) | 1 + external PHY | [18-can.md](../18-can.md) |
| USB OTG | 1 | [19-usb.md](../19-usb.md) |
| Wi-Fi / BLE | Radio | [20-wifi.md](../20-wifi.md), [21-bluetooth.md](../21-bluetooth.md) |
| ADC | 2 units | [12-adc.md](../12-adc.md) |
| RMT, LEDC | Many channels | [11-pwm.md](../11-pwm.md) |

---

## HAL & PAC Crates

| Crate | Role | Cargo example |
|-------|------|---------------|
| **esp32s3** | PAC — raw registers | `esp32s3 = "0.28"` |
| **esp-hal** | HAL — `no_std` drivers | `esp-hal = { features = ["esp32s3"] }` |
| **esp-alloc** | Heap, PSRAM allocator | Optional |
| **esp-wifi** | Wi-Fi for esp-hal | [20-wifi.md](../20-wifi.md) |
| **esp-idf-hal** | HAL on ESP-IDF (`std`) | Alternative path |
| **esp-idf-svc** | Wi-Fi, NVS, HTTP services | Higher level |

**Templates:**

- [esp-template](https://github.com/esp-rs/esp-template) — esp-hal bare metal
- [esp-idf-template](https://github.com/esp-rs/esp-idf-template) — ESP-IDF + Rust

**Target triples:**

- `xtensa-esp32s3-none-elf` — bare metal esp-hal
- `xtensa-esp32s3-espidf` — ESP-IDF

Install via **espup** — [README.md](../README.md).

---

## Related Boards

- [esp32.md](./esp32.md) — original dual-core, no native USB
- [esp32-s2.md](./esp32-s2.md) — single core, USB, no radio
- [esp32-c3.md](./esp32-c3.md) — RISC-V, smaller, no PSRAM on many modules

---

*Index: [boards/README.md](./README.md)*
