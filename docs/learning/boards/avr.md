# AVR Board Guide (ATmega328P / Arduino)

8-bit **AVR** microcontrollers — classic Arduino Uno/Nano. Rust support exists via **avr-hal** but ecosystem is smaller than ARM/ESP32.

---

## Overview

| Spec | Arduino Uno (ATmega328P) |
|------|--------------------------|
| **PCB size (Uno R3)** | **68.6 × 53.4 mm** |
| **PCB size (Nano)** | **~45 × 18 mm** |
| **CPU** | 8-bit AVR @ 16 MHz |
| **Flash** | 32 KB |
| **SRAM** | 2 KB |
| **EEPROM** | 1 KB |
| **Debug** | No native SWD — ISP/UPDI only |
| **Headers** | Arduino R3 / Nano 2.54 mm |

### Board dimensions

```
           68.6 mm (Uno R3)
    ┌────────────────────────┐
    │ USB │ ATmega328P       │  53.4 mm
    └────────────────────────┘
```

Uno R3 is the shield ecosystem standard. Nano / Pro Mini are better for breadboards and small cases.

---

## Memory Map Overview

Harvard architecture — separate address spaces:

```
Flash 0x0000 – 0x7FFF (32 KB) — program memory (.text, .rodata via PROGMEM)
SRAM  0x0100 – 0x08FF (2 KB) — .data, .bss, stack (grows down from 0x08FF)
EEPROM — non-volatile byte storage
I/O   0x0000 – 0x00FF — registers (GPIO as PORTx, DDRx, PINx)
```

**No heap** recommended — 2 KB RAM fills quickly. Avoid `alloc` — [02-no-std.md](../02-no-std.md).

PROGMEM for strings:

```rust
use avr_device::prelude::*;

static MSG: &str = "Hello"; // often placed in flash via avr-gcc patterns
```

---

## GPIO Layout Notes

Arduino Uno pin mapping:

| Arduino | AVR pin | Notes |
|---------|---------|-------|
| D13 | PB5 | Built-in LED |
| D0/D1 | PD0/PD1 | UART |
| D11–13 | PB3–5 | SPI |
| A4/A5 | PC4/PC5 | I²C (TWCR) |
| D2/D3 | PD2/PD3 | External interrupt INT0/INT1 |

**5 V logic** on Uno — level shift for 3.3 V sensors.

Each pin: **DDRx** (direction), **PORTx** (output/l pull-up), **PINx** (read).

---

## Clock Tree

External 16 MHz crystal or ceramic resonator → CPU clock = 16 MHz (no PLL on 328P). Timer prescalers derive 1 MHz / 62.5 kHz for PWM — [10-timers.md](../10-timers.md), [11-pwm.md](../11-pwm.md).

---

## Boot Sequence

1. Reset vector at flash 0x0000.
2. **Bootloader** (Optiboot) in upper flash — UART programming via `/dev/ttyUSB0`.
3. Application starts after bootloader timeout.

Flash Rust:

```bash
avrdude -c arduino -p atmega328p -P /dev/ttyUSB0 -b 115200 -U flash:w:target/avr-atmega328p-unknown/debug/app.hex
```

Or **ravedude** wrapper in avr-hal templates.

---

## Flash & RAM

| | ATmega328P |
|---|------------|
| Flash | 32 KB (~2 KB bootloader) |
| SRAM | 2 KB |
| EEPROM | 1 KB |

Extremely constrained — suitable for [08-gpio.md](../08-gpio.md), simple [15-uart.md](../15-uart.md), not Wi-Fi/TLS.

---

## Interrupt Vectors Overview

Fixed vector table at flash start — 26 vectors on 328P:

| Vector | Source |
|--------|--------|
| `RESET` | Power-on |
| `INT0`, `INT1` | Pin change |
| `PCINT0–2` | Pin change interrupt groups |
| `USART_RX` | UART — [15-uart.md](../15-uart.md) |
| `TIMER0_COMPA` | Timer compare |
| `ADC` | Conversion complete — [12-adc.md](../12-adc.md) |

**Global interrupt enable:** `sei()` / `cli()` — no NVIC priority levels (nested IRQs limited).

---

## Peripherals

| Peripheral | Notes |
|------------|-------|
| USART0 | 1 UART |
| SPI | Hardware SPI |
| TWI | I²C — [17-i2c.md](../17-i2c.md) |
| Timers 0/1/2 | PWM, timing |
| ADC | 6 channels 10-bit |
| No DMA | CPU must move every byte — [14-dma.md](../14-dma.md) N/A |
| No USB native | UART bootloader only |
| No BLE/Wi-Fi | |

---

## HAL & PAC Crates

| Crate | Role |
|-------|------|
| **avr-device** | PAC for ATmega328P, etc. |
| **avr-hal** | HAL — `atmega328p-hal` board support |
| **arduino-hal** | Uno/Nano convenience |
| **panic-halt** | Panic handler |

Target: **`avr-unknown-gnu-atmega328p`** or **`avr-atmega328p-unknown`** (check rust-avr toolchain status)

Requires **nightly Rust** with `-Z build-std=core` historically — follow [avr-rust book](https://book.avr-rust.com/).

```toml
[dependencies]
atmega328p-hal = { version = "0.5", features = ["arduino_uno"] }
```

**No probe-rs** standard flow — serial bootloader or Atmel-ICE ISP.

---

## When to Use AVR in 2026

- Teaching **bare metal** without ARM complexity
- **5 V** legacy shields
- **Tiny** power/cost when 2 KB RAM suffices

For new networked projects, prefer [esp32-c3.md](./esp32-c3.md) or [rp2040.md](./rp2040.md).

---

*Index: [boards/README.md](./README.md)*
