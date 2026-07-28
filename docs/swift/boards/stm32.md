# STM32 Board Guide

**ARM Cortex-M** family — industry standard. Example: **Nucleo-F411RE** (Cortex-M4F).

Embedded Swift support: **blink-level examples** in [swift-embedded-examples/stm32-blink](https://github.com/swiftlang/swift-embedded-examples/tree/main/stm32-blink). No full HAL ecosystem yet — use **swift-mmio** + reference manual or C HAL via interop.

---

## Overview

| Spec | Nucleo-F411RE |
|------|---------------|
| **CPU** | ARM Cortex-M4F @ 100 MHz |
| **Flash** | 512 KB |
| **SRAM** | 128 KB |
| **FPU** | Single precision |
| **Debug** | ST-Link built-in (SWD) |

---

## Embedded Swift Maturity

| Path | Status |
|------|--------|
| Bare-metal blink | **Working** (STM32F746G-DISCO in examples) |
| Full HAL in Swift | **Not available** — use swift-mmio or C drivers |
| Real-time patterns | **Best fit** for [23-realtime-patterns.md](../23-realtime-patterns.md) (NVIC) |
| Host companion | macOS Swift for HIL — [26-testing.md](../26-testing.md) |

Community: [swift-stm32c011-examples](https://github.com/xtremekforever/swift-stm32c011-examples) (tiny MCU).

---

## Memory Map Overview

```
0x0800_0000 ─── Flash (512 KB)
0x2000_0000 ─── SRAM (128 KB)
0x4000_0000 ─── Peripherals (GPIO, USART, SPI, ...)
0xE000_0000 ─── NVIC, SysTick, MPU
```

---

## GPIO Layout Notes

Nucleo-F411RE:

| Pin | Function |
|-----|----------|
| PA5 | User LED |
| PC13 | User button |
| PA2/PA3 | USART2 → ST-Link VCP |

**Alternate Function** mapping in `GPIOx_AFR` — swift-mmio or generated SVD types.

---

## Clock Tree

HSE 8 MHz → PLL → SYSCLK 100 MHz. APB1/APB2 dividers affect UART/SPI clocks.

---

## Boot Sequence

Flash at `0x0800_0000`; boot from reset vector. Flash via ST-Link (`openocd` + `probe-rs`).

---

## Interrupt Vectors

**NVIC** — ideal for priority-based patterns — [23-realtime-patterns.md](../23-realtime-patterns.md):

```swift
@_cdecl("USART2_IRQHandler")
func usart2Handler() { /* ... */ }
```

---

## Peripherals

USART×3, SPI×3, I²C×3, bxCAN, USB FS (some parts), ADC, timers. Lessons [14-dma.md](../14-dma.md)–[19-usb.md](../19-usb.md) apply with STM32 register names.

---

## Swift Toolchain

- Swift embedded snapshot with `-enable-experimental-feature Embedded`
- **swift-mmio** + STM32 SVD from [cmsis-svd](https://github.com/posborne/cmsis-svd)
- **probe-rs** / OpenOCD for debug — [25-debugging.md](../25-debugging.md)
- Linker script for flash/RAM regions

No Wi-Fi on-chip — external module or host gateway for [20-wifi.md](../20-wifi.md).

---

*Index: [boards/README.md](./README.md)*
