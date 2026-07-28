# Embedded Rust Roadmap: Beginner → Advanced

This document maps the full learning journey from zero embedded experience to building production firmware in Rust. Each phase has **milestones** — concrete skills you can demonstrate before moving on.

See the [main README](./README.md) for hardware setup and toolchain installation.

---

## Overview Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│  PHASE 0: Environment & First Flash                          (8–12 h)  │
│  Rust install · espup/probe-rs · blink example · defmt                 │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  PHASE 1: Language & Firmware Foundations                   (20–30 h)  │
│  01 Rust basics · 02 no_std · 03 memory · 04 Cargo                       │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  PHASE 2: Architecture & Abstractions                       (20–30 h)  │
│  05 PAC/HAL/BSP · 06 registers · 07 embedded-hal · 08 GPIO             │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  PHASE 3: Peripherals & Timing                            (25–35 h)    │
│  09 interrupts · 10 timers · 11 PWM · 12 ADC · 13 DAC                   │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  PHASE 4: Buses, Sensors & Displays                        (20–30 h)    │
│  14–19 DMA/UART/SPI/I²C/CAN/USB · sensors/ · displays/ · examples/     │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  PHASE 5: Wireless, Async & Production                       (ongoing)  │
│  20–27 Wi-Fi/BLE/Embassy/RTIC · communication/ · projects/ · boards/   │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 0: Environment & First Flash (8–12 hours)

**Goal:** Flash a program to real hardware and see output.

### Topics

- Install Rust, board tools, VS Code / Cursor with `rust-analyzer`
- Understand what a **target triple** is (e.g., `xtensa-esp32s3-none-elf`)
- Clone `esp-template` or `cortex-m-quickstart`
- Flash LED blink; open serial monitor

### Milestones

| # | Milestone | Verification |
|---|-----------|--------------|
| M0.1 | Toolchain installed | `rustc --version` and `espflash --version` or `probe-rs --version` succeed |
| M0.2 | Board detected | USB serial/JTAG port visible in OS |
| M0.3 | Blink works | On-board LED toggles at ~1 Hz |
| M0.4 | Logging works | `defmt` or `println!` message appears in monitor |

### Resources

- [README — Toolchain Setup](./README.md#rust-installation)
- [Espressif Rust Book — Getting Started](https://esp-rs.github.io/book/installation/index.html)

---

## Phase 1: Language & Firmware Foundations (20–30 hours)

**Goal:** Understand why embedded Rust looks different from desktop Rust.

### Lessons

| Lesson | File | Key Outcome |
|--------|------|-------------|
| Rust Basics | [01-rust-basics.md](./01-rust-basics.md) | Read HAL code; use `Result`, traits, modules |
| no_std | [02-no-std.md](./02-no-std.md) | Explain `#![no_std]` and panic handler |
| Memory Layout | [03-memory-layout.md](./03-memory-layout.md) | Interpret `cargo size` output |
| Cargo | [04-cargo.md](./04-cargo.md) | Configure `Cargo.toml` and `.cargo/config.toml` |

### Milestones

| # | Milestone | Verification |
|---|-----------|--------------|
| M1.1 | Explain ownership in ISR context | Can articulate why `'static` bounds exist |
| M1.2 | Parse memory map | Identify `.text`, `.data`, `.bss` sizes |
| M1.3 | Custom profile | Add a `release-opt-size` profile in `Cargo.toml` |
| M1.4 | Cross-compile | Build for non-host target without errors |

### Checkpoint Project

Write a `no_std` program that prints chip info (flash size, CPU frequency) over `defmt` without using any peripheral beyond UART/USB.

---

## Phase 2: Architecture & Abstractions (20–30 hours)

**Goal:** Navigate the PAC → HAL → BSP stack and write portable GPIO code.

### Lessons

| Lesson | File | Key Outcome |
|--------|------|-------------|
| Architecture | [05-embedded-architecture.md](./05-embedded-architecture.md) | Diagram firmware layers |
| Registers | [06-register-programming.md](./06-register-programming.md) | Read/modify GPIO register safely |
| HAL Traits | [07-hal.md](./07-hal.md) | Implement `OutputPin` consumer code |
| GPIO | [08-gpio.md](./08-gpio.md) | Blink + button on breadboard |

### Milestones

| # | Milestone | Verification |
|---|-----------|--------------|
| M2.1 | Layer diagram | Draw PAC/HAL/BSP for ESP32-S3 and STM32 |
| M2.2 | Bare-metal blink | Toggle GPIO via register write (no HAL) |
| M2.3 | HAL blink | Same behavior using `esp-hal` or `stm32f4xx-hal` |
| M2.4 | Portable driver | Write a crate using only `embedded-hal` traits |
| M2.5 | Button input | Debounced button toggles LED (polling) |

### Checkpoint Project

**Multi-board blink:** Write a small crate with a `blink(pin, delay_ms)` function that compiles for both ESP32-S3 and RP2040 using trait generics.

---

## Phase 3: Peripherals & Timing (25–35 hours)

**Goal:** Master time-sensitive operations — interrupts, timers, and analog I/O.

### Lessons

| Lesson | File | Key Outcome |
|--------|------|-------------|
| Interrupts | [09-interrupts.md](./09-interrupts.md) | GPIO ISR with debounce |
| Timers | [10-timers.md](./10-timers.md) | Hardware timer periodic tick |
| PWM | [11-pwm.md](./11-pwm.md) | LED fade + servo angle |
| ADC | [12-adc.md](./12-adc.md) | Read potentiometer voltage |
| DAC | [13-dac.md](./13-dac.md) | Generate sine wave (where hardware supports) |

### Milestones

| # | Milestone | Verification |
|---|-----------|--------------|
| M3.1 | ISR safety | Explain `Mutex<RefCell<T>>` pattern |
| M3.2 | Timer tick | 1 ms tick drives a software stopwatch |
| M3.3 | PWM fade | LED brightness ramps smoothly |
| M3.4 | ADC reading | Potentiometer position logged as 0–100% |
| M3.5 | Waveform | DAC or PWM-DAC outputs identifiable sine on scope |

### Checkpoint Project

**Mini console:** Potentiometer selects mode (blink rate / LED brightness / "off"); button cycles modes; interrupt-driven debounce; timer-based scheduling.

---

## Phase 4: Buses, Sensors & Displays (20–30 hours)

**Goal:** Talk to real sensors and screens; combine peripherals into applications.

### Lessons

| Lesson | File | Key Outcome |
|--------|------|-------------|
| DMA | [14-dma.md](./14-dma.md) | Buffer ownership for peripheral transfers |
| UART | [15-uart.md](./15-uart.md) | Serial echo + framing |
| SPI | [16-spi.md](./16-spi.md) | Transaction with a display/sensor |
| I²C | [17-i2c.md](./17-i2c.md) | Bus scanner + sensor read |
| CAN | [18-can.md](./18-can.md) | Frame TX/RX filters |
| USB | [19-usb.md](./19-usb.md) | CDC or HID device |

### Apply With

- [sensors/](./sensors/README.md) — BME280, MPU6050, VL53L0X, …
- [displays/](./displays/README.md) — SSD1306, ST7789, WS2812, …
- [examples/](./examples/README.md) — blink → MQTT client ladder
- Capstone: [projects/weather-station.md](./projects/weather-station.md)

### Milestones

| # | Milestone | Verification |
|---|-----------|--------------|
| M4.1 | I²C sensor read | Temperature logged every second |
| M4.2 | Display output | Sensor data rendered on OLED |
| M4.3 | Low power | Device sleeps; wakes on button — [24-low-power.md](./24-low-power.md) |
| M4.4 | Host tests | `cargo test` passes driver logic on PC — [26-testing.md](./26-testing.md) |

### Checkpoint Project

**Environmental monitor:** BME280 + OLED + deep sleep between readings; data logged via `defmt`. See [projects/environmental-monitor.md](./projects/environmental-monitor.md).

---

## Phase 5: Wireless, Async & Production (Ongoing)

**Goal:** Ship maintainable connected firmware with CI, updates, and concurrency models.

### Lessons

| Lesson | File | Key Outcome |
|--------|------|-------------|
| Wi-Fi | [20-wifi.md](./20-wifi.md) | Station mode + TLS notes |
| Bluetooth | [21-bluetooth.md](./21-bluetooth.md) | BLE GATT advertise/notify |
| Embassy | [22-embassy.md](./22-embassy.md) | Async tasks on MCU |
| RTIC | [23-rtic.md](./23-rtic.md) | Priority-based concurrency |
| Low power | [24-low-power.md](./24-low-power.md) | Current budget & wake sources |
| Debugging | [25-debugging.md](./25-debugging.md) | probe-rs / defmt / GDB |
| Testing | [26-testing.md](./26-testing.md) | Host + HIL + CI |
| Patterns | [27-design-patterns.md](./27-design-patterns.md) | Type-state, BSP, driver crates |

### Also Study

| Area | Location |
|------|----------|
| Protocol deep-dives | [communication/](./communication/README.md) |
| Board family details | [boards/](./boards/README.md) |
| End-to-end builds | [projects/](./projects/README.md) |

### Milestones

| # | Milestone | Verification |
|---|-----------|--------------|
| M5.1 | Async blink | Embassy executor runs two tasks |
| M5.2 | CI pipeline | GitHub Action builds firmware artifact |
| M5.3 | OTA update | Device flashes new binary over Wi-Fi — [examples/ota-update.md](./examples/ota-update.md) |
| M5.4 | Custom PCB | Firmware runs on non-dev-kit hardware |

---

## Skill Matrix by Experience Level

| Skill | Beginner (Phase 1–2) | Intermediate (Phase 3–4) | Advanced (Phase 5) |
|-------|---------------------|--------------------------|-------------------|
| Ownership / lifetimes | Read HAL signatures | Design `'static` ISR shared state | Audit unsafe blocks |
| Memory | Read `cargo size` | Optimize section placement | Custom linker scripts |
| GPIO | Digital in/out | Interrupt-driven | Matrix scanning |
| Timing | Busy-loop delay | Hardware timers | RTIC / Embassy |
| Analog | — | ADC read, PWM | DSP, filtering |
| Buses | — | I²C, SPI | DMA transfers |
| Debugging | `defmt` println | Logic analyzer | GDB breakpoints, ITM |
| Architecture | Use HAL | Write portable drivers | Design BSP layer |

---

## Weekly Study Plans

### Intensive (20 h/week → ~5 weeks)

| Week | Focus | Deliverable |
|------|-------|-------------|
| 1 | Phase 0 + Phase 1 | Blink + memory map understood |
| 2 | Phase 2 (architecture + GPIO) | Breadboard blink + button |
| 3 | Phase 3a (interrupts + timers) | Debounced interrupt input |
| 4 | Phase 3b (PWM + ADC) | Pot-controlled LED fade |
| 5 | Phase 3c (DAC) + review | Checkpoint mini-console |

### Relaxed (5 h/week → ~20 weeks)

| Weeks | Focus |
|-------|-------|
| 1–2 | Phase 0 + [01-rust-basics.md](./01-rust-basics.md) |
| 3–4 | [02-no-std.md](./02-no-std.md) + [03-memory-layout.md](./03-memory-layout.md) |
| 5–6 | [04-cargo.md](./04-cargo.md) + [05-embedded-architecture.md](./05-embedded-architecture.md) |
| 7–8 | [06-register-programming.md](./06-register-programming.md) + [07-hal.md](./07-hal.md) |
| 9–10 | [08-gpio.md](./08-gpio.md) hands-on |
| 11–12 | [09-interrupts.md](./09-interrupts.md) |
| 13–14 | [10-timers.md](./10-timers.md) |
| 15–16 | [11-pwm.md](./11-pwm.md) |
| 17–18 | [12-adc.md](./12-adc.md) |
| 19–20 | [13-dac.md](./13-dac.md) + checkpoint project |

---

## How to Assess Readiness for the Next Phase

Before advancing, you should be able to **explain aloud** (rubber-duck style):

1. **Phase 0 → 1:** "What happens from power-on to `main`?"
2. **Phase 1 → 2:** "Where does my variable live — stack, heap, or static?"
3. **Phase 2 → 3:** "What would I change to port this GPIO code to a different chip?"
4. **Phase 3 → 4:** "Why is busy-loop delay bad, and what replaces it?"
5. **Phase 4 → 5:** "How would I test this driver without hardware?"

If any answer is shaky, revisit the linked lesson rather than rushing forward.

---

## Cross-Reference Index

| Concept | Primary Lesson | Also Covered In |
|---------|---------------|-----------------|
| Ownership | [01-rust-basics.md](./01-rust-basics.md) | [09-interrupts.md](./09-interrupts.md) |
| `no_std` | [02-no-std.md](./02-no-std.md) | All lessons |
| Linker scripts | [03-memory-layout.md](./03-memory-layout.md) | [04-cargo.md](./04-cargo.md) |
| PAC | [05-embedded-architecture.md](./05-embedded-architecture.md) | [06-register-programming.md](./06-register-programming.md) |
| embedded-hal | [07-hal.md](./07-hal.md) | [08-gpio.md](./08-gpio.md) – [13-dac.md](./13-dac.md) |
| Critical sections | [09-interrupts.md](./09-interrupts.md) | [10-timers.md](./10-timers.md) |
| Volatile / MMIO | [06-register-programming.md](./06-register-programming.md) | [glossary.md](./glossary.md) |

---

*Start learning: [01-rust-basics.md](./01-rust-basics.md)*
