# Embedded Rust Learning Path

Welcome to a structured, hands-on curriculum for learning **Embedded Rust** — writing firmware for microcontrollers (MCUs) using the Rust programming language without a full operating system (`no_std`).

This guide assumes you know basic programming concepts but **not** necessarily Rust or electronics. Every acronym is explained on first use.

---

## What You Will Learn

By the end of this curriculum you will be able to:

- Write safe, idiomatic Rust for resource-constrained hardware
- Understand memory layout, linker scripts, and register-level programming
- Use **embedded-hal** traits to write portable driver code
- Blink LEDs, read buttons, configure timers, PWM, ADC, and DAC peripherals
- Debug firmware with `defmt`, probes, and logic analyzers
- Move from beginner examples to production-quality firmware architecture

---

## Estimated Study Time

| Phase | Topics | Hours |
|-------|--------|-------|
| Phase 0 — Setup | Toolchain, board, first flash | 8–12 |
| Phase 1 — Rust Foundations | Ownership, `no_std`, memory, Cargo | 20–30 |
| Phase 2 — Architecture | PAC/HAL/BSP, registers, GPIO | 20–30 |
| Phase 3 — Peripherals | Interrupts, timers, PWM, ADC, DAC | 25–35 |
| Phase 4 — Projects & Advanced | RTOS, async, wireless, OTA | 15–25 |
| **Total** | | **~80–120 hours** |

Pace yourself: one lesson per week is perfectly fine. Embedded skills compound — consistency beats speed.

---

## Recommended Order

Follow the numbered lessons sequentially. Each builds on the previous:

```
README (you are here)
  └── 00-roadmap.md … 07-hal.md     ← Foundations
  └── 08-gpio.md … 13-dac.md        ← Core peripherals
  └── 14-dma.md … 21-bluetooth.md   ← Buses & wireless
  └── 22-embassy.md … 27-design-patterns.md
  └── sensors/ · displays/ · communication/
  └── examples/ · projects/ · boards/
  └── glossary.md                   ← A–Z reference (use anytime)
```

Cross-links inside each lesson point to related topics. Keep [glossary.md](./glossary.md) open in a second tab.

---

## Hardware Requirements

### Primary Development Board (Recommended)

**ESP32-S3 DevKitC-1** (Espressif) — dual-core Xtensa LX7, Wi-Fi/BLE, USB-JTAG built in, excellent Rust support via `esp-hal`.

| Spec | Value |
|------|-------|
| Flash | 8–16 MB |
| RAM | 512 KB SRAM + optional PSRAM |
| USB | Native USB (CDC + JTAG) |
| Price | ~$10–15 USD |

Alternative boards (lessons note differences where relevant):

| Board | MCU | Why Consider It |
|-------|-----|-----------------|
| **STM32 Nucleo-F411RE** | ARM Cortex-M4 | Industry standard, `probe-rs` + `embassy` ecosystem |
| **Raspberry Pi Pico (RP2040)** | Dual Cortex-M0+ | Cheap, great `rp-hal`, PIO peripheral |
| **nRF52840 DK** | ARM Cortex-M4 | Bluetooth Low Energy (BLE) focus |

### Basic Components Kit

| Component | Purpose | Notes |
|-----------|---------|-------|
| Breadboard + jumper wires | Prototyping | 830-point breadboard is fine |
| 3× LEDs (any color) | GPIO, PWM lessons | Include 220 Ω–1 kΩ resistors |
| 3× Tactile push buttons | Input, interrupts | Active-low with pull-ups |
| 10× 220 Ω resistors | Current limiting | |
| Potentiometer (10 kΩ) | ADC lesson | Linear taper |
| USB cable | Power + programming | Match your board (USB-C or Micro-B) |
| Multimeter (optional) | Voltage checks | Helps debug wiring |

### Optional (Intermediate+)

- Logic analyzer (e.g., Saleae clone, ~$10) — inspect PWM, I²C, SPI waveforms
- Oscilloscope — analog signal verification
- Servo motor (SG90) — PWM lesson extension
- Second board — test portable HAL code

---

## Rust Installation

### 1. Install Rust via rustup

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
rustup component add rustfmt clippy
```

Verify:

```bash
rustc --version   # e.g., rustc 1.85.0
cargo --version
```

### 2. ESP32-S3 Toolchain (espup + espflash)

For Espressif targets you need the Xtensa/RISC-V Rust toolchain managed by **espup**:

```bash
cargo install espup espflash
espup install
# Follow printed instructions to source the export script, e.g.:
source ~/export-esp.sh
```

Flash firmware:

```bash
espflash flash --monitor target/xtensa-esp32s3-espidf/debug/blink
```

The `--monitor` flag opens a serial console for `defmt` or `println!` output.

### 3. ARM Toolchain (STM32 / RP2040) — probe-rs

For ARM Cortex-M boards, the standard host `rustc` supports cross-compilation out of the box:

```bash
# Example: RP2040
rustup target add thumbv6m-none-eabi

# Example: STM32 (Cortex-M4)
rustup target add thumbv7em-none-eabihf

cargo install probe-rs-tools
```

Flash with probe-rs:

```bash
probe-rs run --chip RP2040 target/thumbv6m-none-eabi/debug/blink
```

### 4. Useful Cargo Tools

```bash
cargo install cargo-binutils    # objdump, size, nm for firmware
cargo install flip-link           # stack overflow protection (ARM)
cargo install cargo-embed         # alternative runner (ARM)
```

---

## Probe Setup

A **debug probe** lets you flash firmware and set breakpoints without a bootloader.

| Board | Probe Needed | Connection |
|-------|-------------|------------|
| ESP32-S3 DevKitC-1 | **None** (USB-JTAG built in) | USB-C cable |
| STM32 Nucleo | **None** (ST-Link built in) | USB cable |
| RP2040 Pico | **None** (BOOTSEL USB drag-and-drop) or external probe | USB or SWD pins |
| Custom STM32 PCB | ST-Link v2, J-Link, or DAPLink | SWDIO, SWCLK, GND, 3V3 |

### probe-rs Configuration

Create `Embed.toml` in your project root (ARM projects):

```toml
[default.probe]
protocol = "Swd"
speed = 4000

[default.flashing]
enabled = true

[default.general]
chip = "STM32F411RETx"   # change for your chip
```

For ESP32-S3, prefer `espflash` unless you configure OpenOCD with the built-in USB-JTAG.

---

## Debugging Tools

| Tool | Platform | Use Case |
|------|----------|----------|
| **defmt** | All | Structured, low-overhead logging over RTT or UART |
| **probe-rs** | ARM | Flash, GDB, RTT |
| **espflash** | ESP32 | Flash + monitor |
| **cargo-embed** | ARM | Integrated RTT + GDB |
| **OpenOCD** | ARM, ESP (via JTAG) | Industry-standard debug server |
| **Logic analyzer** | All | Digital timing (I²C, SPI, PWM) |
| **`cargo size`** | All | Flash/RAM usage per section |

Enable `defmt` in `Cargo.toml`:

```toml
[dependencies]
defmt = "0.3"
defmt-rtt = "0.4"

[defmt]
# espflash / probe-rs reads this automatically in many templates
```

Log from firmware:

```rust
use defmt::info;

info!("Temperature: {} °C", temp);  // No heap allocation!
```

---

## Recommended Resources

### Books

| Title | Author | Notes |
|-------|--------|-------|
| *The Embedded Rust Book* | Rust Embedded WG | Free online — [docs.rust-embedded.org/book](https://docs.rust-embedded.org/book/) |
| *Embedded Systems with ARM Cortex-M* | Yiu | Hardware background (C-focused but excellent) |
| *Programming Rust* (2nd ed.) | Blandy, Orendorff | Deep Rust language reference |
| *Hands-On RTOS with Microcontrollers* | Barry | FreeRTOS concepts (compare with Embassy) |

### YouTube & Video

- **Ferrous Systems — Embedded Rust** (training previews)
- **Espressif Developer Portal** — ESP32-S3 hardware overview
- **Phil's Lab** — STM32 hardware design (electronics context)
- **James Munns** — `no_std`, `defmt`, `knurling` tooling talks

### Blogs & Communities

- [Rust Embedded WG Matrix/Discord](https://matrix.to/#/#rust-embedded:matrix.org)
- [Knurling Blog](https://knurling.org/blog/) — testing & tooling for embedded Rust
- [Espressif Rust Book](https://esp-rs.github.io/book/)
- [/r/embedded](https://reddit.com/r/embedded) and [/r/rust](https://reddit.com/r/rust)

### Datasheets & Reference Manuals

| Document | Link / Search |
|----------|---------------|
| ESP32-S3 Technical Reference | [espressif.com](https://www.espressif.com/en/products/socs/esp32-s3) |
| ESP32-S3 Datasheet | Espressif document center |
| STM32F411 Reference Manual | ST RM0383 |
| RP2040 Datasheet | [raspberrypi.com/documentation/microcontrollers](https://www.raspberrypi.com/documentation/microcontrollers/) |
| ARM Cortex-M4 Generic User Guide | ARM DUI 0553 |

---

## Folder & Lesson Index

| File | Title | Prerequisites |
|------|-------|---------------|
| [00-roadmap.md](./00-roadmap.md) | Full phased roadmap | None |
| [01-rust-basics.md](./01-rust-basics.md) | Rust fundamentals for embedded | Basic programming |
| [02-no-std.md](./02-no-std.md) | `no_std` firmware | [01](./01-rust-basics.md) |
| [03-memory-layout.md](./03-memory-layout.md) | Flash, RAM, linker scripts | [02](./02-no-std.md) |
| [04-cargo.md](./04-cargo.md) | Cargo for embedded | [02](./02-no-std.md) |
| [05-embedded-architecture.md](./05-embedded-architecture.md) | PAC / HAL / BSP | [04](./04-cargo.md) |
| [06-register-programming.md](./06-register-programming.md) | MMIO & registers | [05](./05-embedded-architecture.md) |
| [07-hal.md](./07-hal.md) | embedded-hal traits | [06](./06-register-programming.md) |
| [08-gpio.md](./08-gpio.md) | GPIO blink & button | [07](./07-hal.md) |
| [09-interrupts.md](./09-interrupts.md) | IRQs & ISRs | [08](./08-gpio.md) |
| [10-timers.md](./10-timers.md) | Timers & delays | [09](./09-interrupts.md) |
| [11-pwm.md](./11-pwm.md) | PWM & dimming | [10](./10-timers.md) |
| [12-adc.md](./12-adc.md) | Analog input | [10](./10-timers.md) |
| [13-dac.md](./13-dac.md) | Analog output | [12](./12-adc.md) |
| [14-dma.md](./14-dma.md) | Direct Memory Access | [10](./10-timers.md) |
| [15-uart.md](./15-uart.md) | UART serial | [08](./08-gpio.md) |
| [16-spi.md](./16-spi.md) | SPI bus | [15](./15-uart.md) |
| [17-i2c.md](./17-i2c.md) | I²C bus | [15](./15-uart.md) |
| [18-can.md](./18-can.md) | CAN / TWAI | [15](./15-uart.md) |
| [19-usb.md](./19-usb.md) | USB device (HID/CDC) | [05](./05-embedded-architecture.md) |
| [20-wifi.md](./20-wifi.md) | Wi-Fi station & TLS | [15](./15-uart.md) |
| [21-bluetooth.md](./21-bluetooth.md) | BLE GATT | [20](./20-wifi.md) |
| [22-embassy.md](./22-embassy.md) | Async embedded (Embassy) | [09](./09-interrupts.md) |
| [23-rtic.md](./23-rtic.md) | RTIC concurrency | [09](./09-interrupts.md) |
| [24-low-power.md](./24-low-power.md) | Sleep & current budgets | [10](./10-timers.md) |
| [25-debugging.md](./25-debugging.md) | probe-rs, defmt, GDB | [04](./04-cargo.md) |
| [26-testing.md](./26-testing.md) | Host tests, HIL, CI | [07](./07-hal.md) |
| [27-design-patterns.md](./27-design-patterns.md) | State machines, BSP, drivers | [07](./07-hal.md) |
| [glossary.md](./glossary.md) | Term glossary A–Z | Anytime |

### Extended Sections

| Section | Description |
|---------|-------------|
| [boards/](./boards/README.md) | MCU family guides (ESP32-Sx/Cx, STM32, RP2040, nRF52, AVR, MSP430) |
| [communication/](./communication/README.md) | Protocol deep-dives (GPIO, UART, SPI, I²C, CAN, USB, Wi-Fi, BLE, LoRa, MQTT, HTTP, …) |
| [examples/](./examples/README.md) | Hands-on Rust sketches with ownership notes |
| [projects/](./projects/README.md) | Full tutorial builds (weather station, robot, CAN analyzer, …) |
| [sensors/](./sensors/README.md) | Sensor-specific guides (BME280, IMU, GPS, …) |
| [displays/](./displays/README.md) | Display modules (OLED, TFT, NeoPixel, character LCD) |

---

## Quick Start Checklist

- [ ] Install Rust via `rustup`
- [ ] Install board-specific tools (`espup` or `probe-rs`)
- [ ] Connect dev board via USB; confirm serial port appears
- [ ] Clone or create a project from [esp-template](https://github.com/esp-rs/esp-template) or [cargo-generate embedded template](https://github.com/rust-embedded/cortex-m-quickstart)
- [ ] Flash the built-in LED blink example
- [ ] Read [01-rust-basics.md](./01-rust-basics.md) and proceed in order

---

## How to Use This Curriculum

1. **Read the theory** section of each lesson before wiring anything.
2. **Build the HAL example first** — it is easier to debug.
3. **Then try the bare-metal version** to understand what the HAL hides.
4. **Complete the exercises** — they reinforce portable patterns.
5. **Consult the glossary** when you encounter unfamiliar terms.

Good luck — embedded Rust is challenging but deeply rewarding. The compiler is your ally on constrained hardware where bugs are expensive.

---

*Next step: [00-roadmap.md](./00-roadmap.md)*
