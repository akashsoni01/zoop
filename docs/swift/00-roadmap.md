# Embedded Swift Roadmap: Beginner → Advanced

This document maps the full learning journey from zero embedded experience to building production firmware in Swift. Each phase has **milestones** — concrete skills you can demonstrate before moving on.

See the [main README](./README.md) for hardware setup and toolchain installation.

---

## Overview Diagram

```
┌─────────────────────────────────────────────────────────────────────────┐
│  PHASE 0: Environment & First Flash                          (8–12 h)  │
│  Swift install · Embedded Swift toolchain · blink · UART logging       │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  PHASE 1: Language & Firmware Foundations                   (20–30 h)  │
│  01 Swift basics · 02 Embedded Swift · 03 memory · 04 SwiftPM            │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  PHASE 2: Architecture & Abstractions                       (20–30 h)  │
│  05 PAC/HAL/BSP · 06 registers · 07 HAL protocols · 08 GPIO            │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  PHASE 3: Peripherals & Timing                            (25–35 h)    │
│  09 interrupts · 10 timers · 11 PWM · 12 ADC · 13 DAC                   │
└──────────────────────────────────┬──────────────────────────────────────┘
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│  PHASE 4: Buses, Wireless & Host IoT                       (ongoing)  │
│  UART/SPI/I²C · Wi-Fi/BLE · Swift Concurrency · iOS companion apps     │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Phase 0: Environment & First Flash (8–12 hours)

**Goal:** Flash a program to real hardware and see output.

### Topics

- Install Swift, Embedded Swift nightly if required, VS Code / Cursor / Xcode
- Understand what a **target triple** is (e.g., `arm-none-eabi`, `xtensa-esp32s3-none-elf`)
- Clone a community Embedded Swift template or swift-embedded-examples
- Flash LED blink; open serial monitor

### Milestones

| # | Milestone | Verification |
|---|-----------|--------------|
| M0.1 | Toolchain installed | `swift --version` succeeds |
| M0.2 | Board detected | USB serial/JTAG port visible in OS |
| M0.3 | Blink works | On-board LED toggles at ~1 Hz |
| M0.4 | Logging works | UART `print` or semihosting message appears in monitor |

### Resources

- [README — Toolchain Setup](./README.md#swift-installation)
- [Swift Embedded Examples](https://github.com/apple/swift-embedded-examples)
- [02-embedded-swift.md](./02-embedded-swift.md)

---

## Phase 1: Language & Firmware Foundations (20–30 hours)

**Goal:** Understand why Embedded Swift looks different from iOS/macOS Swift.

### Lessons

| Lesson | File | Key Outcome |
|--------|------|-------------|
| Swift Basics | [01-swift-basics.md](./01-swift-basics.md) | Read HAL code; use `Result`, protocols, optionals |
| Embedded Swift | [02-embedded-swift.md](./02-embedded-swift.md) | Explain freestanding mode and what you lose |
| Memory Layout | [03-memory-layout.md](./03-memory-layout.md) | Interpret link map / size output |
| SwiftPM | [04-swiftpm.md](./04-swiftpm.md) | Configure `Package.swift` and cross-compile |

### Milestones

| # | Milestone | Verification |
|---|-----------|--------------|
| M1.1 | Explain value vs reference types | Can articulate why structs beat classes on MCU |
| M1.2 | Parse memory map | Identify `.text`, `.data`, `.bss` sizes |
| M1.3 | Custom build config | Add release-with-size-optimization flags |
| M1.4 | Cross-compile | Build for non-host target without errors |

### Checkpoint Project

Write a freestanding program that prints chip info (flash size, CPU frequency) over UART without using any peripheral beyond serial.

---

## Phase 2: Architecture & Abstractions (20–30 hours)

**Goal:** Navigate the PAC → HAL → BSP stack and write portable GPIO code.

### Lessons

| Lesson | File | Key Outcome |
|--------|------|-------------|
| Architecture | [05-embedded-architecture.md](./05-embedded-architecture.md) | Diagram firmware layers |
| Registers | [06-register-programming.md](./06-register-programming.md) | Read/modify GPIO register safely |
| HAL Protocols | [07-hal.md](./07-hal.md) | Implement `DigitalOutputPin` consumer code |
| GPIO | [08-gpio.md](./08-gpio.md) | Blink + button on breadboard |

### Milestones

| # | Milestone | Verification |
|---|-----------|--------------|
| M2.1 | Layer diagram | Draw PAC/HAL/BSP for ESP32-S3 and STM32 |
| M2.2 | Bare-metal blink | Toggle GPIO via register write (no HAL) |
| M2.3 | HAL blink | Same behavior using board HAL package |
| M2.4 | Portable driver | Write a module using only HAL protocols |
| M2.5 | Button input | Debounced button toggles LED (polling) |

### Checkpoint Project

**Multi-board blink:** Write a small module with a `blink(pin:delayMs:)` function that compiles for both ESP32-S3 and RP2040 using protocol generics.

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
| M3.1 | ISR safety | Explain actor + atomic patterns for shared state |
| M3.2 | Timer tick | 1 ms tick drives a software stopwatch |
| M3.3 | PWM fade | LED brightness ramps smoothly |
| M3.4 | ADC reading | Potentiometer position logged as 0–100% |
| M3.5 | Waveform | DAC or PWM-DAC outputs identifiable sine on scope |

### Checkpoint Project

**Mini console:** Potentiometer selects mode (blink rate / LED brightness / "off"); button cycles modes; interrupt-driven debounce; timer-based scheduling.

---

## Phase 4: Buses, Wireless & Host IoT (Ongoing)

**Goal:** Connect devices and ship maintainable firmware. Some topics await dedicated lessons; start with host Swift where MCU tooling is immature.

### Recommended Next Topics

| Topic | Approach |
|-------|----------|
| UART / SPI / I²C | Freestanding drivers + HAL protocols |
| Wi-Fi / BLE on ESP32 | C SDK bridge or host Swift companion |
| Swift Concurrency on MCU | Lightweight executor + async tasks |
| iOS companion app | CoreBluetooth, Network framework |
| OTA updates | Bootloader + signed image transfer |
| CI/CD | GitHub Actions building firmware artifacts |

### Milestones

| # | Milestone | Verification |
|---|-----------|--------------|
| M4.1 | I²C sensor read | Temperature logged every second |
| M4.2 | Host companion | iOS app displays BLE sensor data |
| M4.3 | Low power | Device sleeps; wakes on button |
| M4.4 | Host tests | `swift test` passes driver logic on macOS |

### Checkpoint Project

**Environmental monitor:** Sensor on MCU logs via UART; iOS app reads over BLE. Split firmware (Embedded Swift) and app (full Swift) responsibilities clearly.

---

## Skill Matrix by Experience Level

| Skill | Beginner (Phase 1–2) | Intermediate (Phase 3–4) | Advanced (Phase 4+) |
|-------|---------------------|--------------------------|---------------------|
| Value / reference types | Read HAL signatures | Design struct-based drivers | Audit unsafe MMIO |
| Memory | Read link map sizes | Optimize section placement | Custom linker scripts |
| GPIO | Digital in/out | Interrupt-driven | Matrix scanning |
| Timing | Busy-loop delay | Hardware timers | Actors + async executor |
| Analog | — | ADC read, PWM | DSP, filtering |
| Buses | — | I²C, SPI | DMA transfers |
| Debugging | UART print | Logic analyzer | LLDB breakpoints |
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
| 1–2 | Phase 0 + [01-swift-basics.md](./01-swift-basics.md) |
| 3–4 | [02-embedded-swift.md](./02-embedded-swift.md) + [03-memory-layout.md](./03-memory-layout.md) |
| 5–6 | [04-swiftpm.md](./04-swiftpm.md) + [05-embedded-architecture.md](./05-embedded-architecture.md) |
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
| Optionals / Result | [01-swift-basics.md](./01-swift-basics.md) | [07-hal.md](./07-hal.md) |
| Embedded Swift / freestanding | [02-embedded-swift.md](./02-embedded-swift.md) | All lessons |
| Linker scripts | [03-memory-layout.md](./03-memory-layout.md) | [04-swiftpm.md](./04-swiftpm.md) |
| PAC | [05-embedded-architecture.md](./05-embedded-architecture.md) | [06-register-programming.md](./06-register-programming.md) |
| HAL protocols | [07-hal.md](./07-hal.md) | [08-gpio.md](./08-gpio.md) – [13-dac.md](./13-dac.md) |
| Critical sections | [09-interrupts.md](./09-interrupts.md) | [10-timers.md](./10-timers.md) |
| Volatile / MMIO | [06-register-programming.md](./06-register-programming.md) | [glossary.md](./glossary.md) |
| Swift Concurrency | [09-interrupts.md](./09-interrupts.md) | [10-timers.md](./10-timers.md) |

---

## Embedded Swift Maturity Notes (2025–2026)

| Area | Status | Recommendation |
|------|--------|----------------|
| Language subset | Apple actively developing | Track Swift forums |
| ESP32-S3 | Community experiments | Use for learning; expect breakage |
| STM32 / RP2040 | Early tooling | Good for bare-metal lessons |
| Wi-Fi / BLE stack | Often C-based | Host Swift or C bridge |
| Swift Concurrency on MCU | Experimental | Start with interrupts + actors |
| Package ecosystem | Small vs Rust/C | Write your own HAL protocols |

Do not let immaturity discourage you — the **concepts** in this curriculum transfer across languages and will remain valid as tooling matures.

---

*Start learning: [01-swift-basics.md](./01-swift-basics.md)*
