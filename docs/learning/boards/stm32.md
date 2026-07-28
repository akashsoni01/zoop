# STM32 Board Guide

**ARM Cortex-M** family — industry standard for [23-rtic.md](../23-rtic.md), [22-embassy.md](../22-embassy.md), and **probe-rs** debugging. Example: **Nucleo-F411RE** (Cortex-M4F).

---

## Overview

| Spec | Nucleo-F411RE (Nucleo-64) |
|------|---------------------------|
| **PCB size** | **82.5 × 70 mm** (Nucleo-64 form factor) |
| **CPU** | ARM Cortex-M4F @ 100 MHz |
| **Flash** | 512 KB |
| **SRAM** | 128 KB |
| **FPU** | Single precision |
| **Debug** | ST-Link built-in (SWD) |
| **Headers** | Arduino Uno R3 + ST Morpho |

### Board dimensions

```
              82.5 mm
    ┌────────────────────────────┐
    │ ST-Link │ MCU / Arduino   │  70 mm
    └────────────────────────────┘
```

**Nucleo-64** boards share this outline (UM1724). **Nucleo-32** is smaller (~50 × 40 mm class); **Nucleo-144** is larger. Morpho headers sit outside the Arduino footprint — leave clearance in enclosures.

STM32 spans **M0+ to M7** — this guide’s patterns apply broadly; verify your exact reference manual.

---

## Memory Map Overview

Cortex-M standard layout:

```
0x0000_0000 ─── Flash (512 KB) — alias at 0x0800_0000 on boot
0x2000_0000 ─── SRAM (128 KB)
0x4000_0000 ─── AHB/APB peripherals (GPIO, USART, SPI, ...)
0xE000_0000 ─── Cortex-M private peripheral bus (NVIC, SysTick, MPU)
```

Optional **CCM** (Core Coupled Memory) on some F4 parts — fast RAM for time-critical data.

See [03-memory-layout.md](../03-memory-layout.md).

---

## GPIO Layout Notes

Nucleo-F411RE:

| Pin | Function |
|-----|----------|
| **PA5** | User LED (green) |
| **PC13** | User button (blue pill) |
| **PA2/PA3** | USART2 → ST-Link virtual COM |
| **Arduino D0–D15** | Morpho headers — 3.3 V |

**Alternate Function (AF)** mapping — each peripheral pin selects AF number in `GPIOx_AFR` registers. HAL/Embassy abstracts this:

```rust
let tx = gpioa.pa2.into_alternate::<7>(); // USART2 TX AF7
```

5 V tolerant pins marked in datasheet — Nucleo Arduino pins generally **not** 5 V on all ST boards.

---

## Clock Tree

```
HSE 8 MHz (ST-Link MCO or crystal)
    │
    ▼
  PLL ──► SYSCLK 100 MHz
            ├── AHB 100 MHz
            ├── APB1 50 MHz (USART2, I²C1, timers)
            └── APB2 100 MHz (USART1, SPI1, ADC)
```

Configure in `embassy-stm32` `config.rs` or `stm32f4xx-hal` RCC. Timer clocks on APB may **double** when prescaler > 1 — timer math in [10-timers.md](../10-timers.md).

---

## Boot Sequence

1. Reset vector from flash `0x0800_0000`.
2. Startup code copies `.data`, zeroes `.bss`.
3. `main()` — Rust `cortex-m-rt` entry.

Flash via **probe-rs**:

```bash
probe-rs run --chip STM32F411RETx target/thumbv7em-none-eabihf/debug/app
```

**Boot0** pin + serial bootloader (USART1) for factory programming without probe.

---

## Flash & RAM

| Resource | F411RE |
|----------|--------|
| Flash | 512 KB |
| RAM | 128 KB |

Use `cargo size` and `#![no_main]` — no heap unless allocator added. **flip-link** for stack overflow guard — [README.md](../README.md).

---

## Interrupt Vectors Overview

**NVIC** (Nested Vectored Interrupt Controller) — up to 82 IRQs on F411:

| IRQ | Source |
|-----|--------|
| `EXTI15_10` | GPIO lines 10–15 (button) |
| `USART2` | Serial |
| `DMA1_Streamx` | DMA — [14-dma.md](../14-dma.md) |
| `TIM2` | Timer — [10-timers.md](../10-timers.md) |
| `OTG_FS` | USB — [19-usb.md](../19-usb.md) |

Priority: 0 = highest (4 bits implemented on F4). RTIC assigns task priorities — [23-rtic.md](../23-rtic.md).

---

## Peripherals

| Bus | Examples |
|-----|----------|
| APB1 | USART2/3, I²C1/2/3, TIM2–5 |
| APB2 | USART1, SPI1, ADC1, TIM1/8 |
| AHB1 | GPIO, DMA, CRC |

**bxCAN** on many STM32 (not F411 — no CAN on F411; F446 has CAN). **FDCAN** on G4/H7 — [18-can.md](../18-can.md).

---

## HAL & PAC Crates

| Crate | Role |
|-------|------|
| **stm32f411** | PAC — `stm32f4 = "0.15"` feature |
| **stm32f4xx-hal** | Traditional HAL |
| **embassy-stm32** | Async HAL + executor integration |
| **cortex-m-rt** | Startup, vectors |
| **cortex-m** | NVIC, WFI |

Target: `thumbv7em-none-eabihf` (M4F with FPU)

```toml
[dependencies]
embassy-stm32 = { version = "0.17", features = ["stm32f411re"] }
```

Chip string for probe-rs: **`STM32F411RETx`**

---

## Why Choose STM32 for Learning

- **probe-rs + GDB** first-class — [25-debugging.md](../25-debugging.md)
- **RTIC** reference platform — [23-rtic.md](../23-rtic.md)
- **Embassy** broad chip support — [22-embassy.md](../22-embassy.md)
- Transferable to automotive/industrial job skills

---

*Index: [boards/README.md](./README.md)*
