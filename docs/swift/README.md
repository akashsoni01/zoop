# Embedded Swift Learning Path

Welcome to a structured, hands-on curriculum for learning **Embedded Swift** — writing firmware for microcontrollers (MCUs) using Apple's **Embedded Swift** subset and the full Swift language where appropriate.

This guide assumes you know basic programming concepts but **not** necessarily Swift or electronics. Every acronym is explained on first use.

> **Honest note on maturity:** Embedded Swift is **actively evolving** (2024–2026). Tooling for ESP32-S3, STM32, and RP2040 exists in community and Apple research projects, but ecosystems are less mature than Embedded Rust or C. This curriculum teaches portable concepts and notes where you may need host Swift (iOS/macOS) with CoreBluetooth or Network for phone-side IoT while MCU firmware catches up.

---

## What You Will Learn

By the end of this curriculum you will be able to:

- Write safe, idiomatic Swift for resource-constrained hardware
- Understand **ARC (Automatic Reference Counting)**, value vs reference types, and when Embedded Swift omits them
- Use **protocol-oriented HAL** traits to write portable driver code
- Blink LEDs, read buttons, configure timers, PWM, ADC, and DAC peripherals
- Debug firmware with UART logging, semihosting, probes, and `os_log` on Apple hosts
- Apply **Swift Concurrency** (async/await, actors) and interrupt-driven patterns for real-time code
- Move from beginner examples to production-quality firmware architecture

---

## Estimated Study Time

| Phase | Topics | Hours |
|-------|--------|-------|
| Phase 0 — Setup | Toolchain, board, first flash | 8–12 |
| Phase 1 — Swift Foundations | Types, Embedded Swift, memory, SwiftPM | 20–30 |
| Phase 2 — Architecture | PAC/HAL/BSP, registers, GPIO | 20–30 |
| Phase 3 — Peripherals | Interrupts, timers, PWM, ADC, DAC | 25–35 |
| Phase 4 — Buses & sensors | DMA, UART/SPI/I²C, sensors, displays | 20–30 |
| Phase 5 — Wireless & production | Wi-Fi/BLE, concurrency, projects | 15–25 |
| **Total** | | **~80–120 hours** |

Pace yourself: one lesson per week is perfectly fine. Embedded skills compound — consistency beats speed.

---

## Recommended Order

Follow the numbered lessons sequentially. Each builds on the previous:

```
README (you are here)
  └── 00-roadmap.md … 07-hal.md     ← Foundations
  └── 08-gpio.md … 13-dac.md        ← Core peripherals
  └── 14-dma.md … 27-design-patterns.md  ← Buses, wireless, production
  └── sensors/ · displays/ · communication/
  └── examples/ · projects/ · boards/
  └── glossary.md                   ← A–Z reference (use anytime)
```

Cross-links inside each lesson point to related topics. Keep [glossary.md](./glossary.md) open in a second tab.

For comparison with the Embedded Rust curriculum, see [../learning/README.md](../learning/README.md).

---

## Hardware Requirements

### Primary Development Board (Recommended)

**ESP32-S3 DevKitC-1** (Espressif) — dual-core Xtensa LX7, Wi-Fi/BLE, USB-JTAG built in. Community Embedded Swift experiments target this board.

| Spec | Value |
|------|-------|
| Flash | 8–16 MB |
| RAM | 512 KB SRAM + optional PSRAM |
| USB | Native USB (CDC + JTAG) |
| Price | ~$10–15 USD |

Alternative boards (lessons note differences where relevant):

| Board | MCU | Embedded Swift Notes |
|-------|-----|----------------------|
| **STM32 Nucleo-F411RE** | ARM Cortex-M4 | Strong probe ecosystem; Embedded Swift ARM targets emerging |
| **Raspberry Pi Pico (RP2040)** | Dual Cortex-M0+ | Cheap; community freestanding Swift experiments |
| **Apple Silicon Mac + iOS device** | Host Swift | CoreBluetooth, Network, accessory protocols for IoT gateway apps |

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
- ST-Link or J-Link — external debug for custom STM32 PCBs

---

## Swift Installation

### 1. Install Swift Toolchain

**macOS (recommended for Embedded Swift development):**

```bash
# Option A: Xcode Command Line Tools + Swift from Xcode
xcode-select --install
swift --version   # e.g., Swift 6.0

# Option B: Standalone toolchain from swift.org
# Download from https://www.swift.org/install/macos/
```

**Linux:**

```bash
# Follow instructions at https://www.swift.org/install/linux/
swift --version
```

Verify Swift Package Manager:

```bash
swift package --version
```

### 2. Embedded Swift / Freestanding Toolchain

Embedded Swift requires a toolchain build with **freestanding** support. As of 2025–2026:

- Watch Apple's Swift Embedded Workgroup announcements and the [swift-embedded-examples](https://github.com/apple/swift-embedded-examples) repository
- Some targets require **Swift nightly** or a custom-built toolchain
- Check your board's community README before assuming `swift build` works out of the box

```bash
# Example: install nightly (macOS) — verify current instructions on swift.org
# swiftly install nightly
# swift +nightly --version
```

**What you lose vs full Swift:** Foundation, most of the standard library, ARC for heap types, dynamic dispatch in many cases, and runtime reflection. See [02-embedded-swift.md](./02-embedded-swift.md).

### 3. Board-Specific Flash Tools

| Platform | Tool | Install |
|----------|------|---------|
| ESP32-S3 | esptool.py / espflash | `pip install esptool` or project-specific flash script |
| STM32 | OpenOCD + probe-rs or ST-Link utilities | `brew install openocd` (macOS) |
| RP2040 | picotool | Build from [raspberrypi/picotool](https://github.com/raspberrypi/picotool) |

Flash example (ESP32-S3, project-dependent):

```bash
swift build -c release --triple xtensa-esp32s3-none-elf   # when supported
python -m esptool write_flash 0x0 .build/release/firmware.bin
```

Always follow your project's README — Embedded Swift build flags change frequently.

### 4. Useful SwiftPM Commands

```bash
swift build                              # Debug build
swift build -c release                   # Optimized build
swift run                                # Host executables only
swift test                               # Run package tests
swift package describe --type json       # Inspect targets
```

See [04-swiftpm.md](./04-swiftpm.md) for cross-compilation and `Package.swift` structure.

---

## Probe Setup

A **debug probe** lets you flash firmware and set breakpoints without a bootloader.

| Board | Probe Needed | Connection |
|-------|-------------|------------|
| ESP32-S3 DevKitC-1 | **None** (USB-JTAG built in) | USB-C cable |
| STM32 Nucleo | **None** (ST-Link built in) | USB cable |
| RP2040 Pico | **None** (BOOTSEL USB drag-and-drop) or external probe | USB or SWD pins |
| Custom STM32 PCB | ST-Link v2, J-Link, or DAPLink | SWDIO, SWCLK, GND, 3V3 |

### Debugging Servers

| Tool | Platform | Use Case |
|------|----------|----------|
| **OpenOCD** | ARM, ESP (JTAG) | GDB server, flash |
| **probe-rs** | ARM | Flash, GDB (works alongside Swift debug when supported) |
| **LLDB** | All | Swift-native debugger; use with `-g` debug symbols |
| **USB-JTAG (ESP32-S3)** | ESP32 | Built-in; no external probe |

Configure LLDB for remote embedded targets per your project's `launch.json` or `.lldbinit` snippets.

---

## Debugging & Logging

| Tool | Platform | Use Case |
|------|----------|----------|
| **print / UART** | MCU | Simple serial logging over USB-CDC or UART pins |
| **Semihosting** | ARM (dev only) | Print via debugger — slow, not for production |
| **os_log** | iOS/macOS host | Structured logging for companion apps |
| **LLDB** | All | Breakpoints, watchpoints, memory inspection |
| **Logic analyzer** | All | Digital timing (I²C, SPI, PWM) |

Embedded Swift logging (UART-style):

```swift
// Freestanding firmware — no Foundation
func log(_ message: StaticString) {
    uartWrite(message.utf8Start, message.utf8CodeUnitCount)
}
```

Host-side companion app:

```swift
import os

let logger = Logger(subsystem: "com.example.iot", category: "sensor")
logger.info("Temperature: \(temp, privacy: .public) °C")
```

---

## Recommended Resources

### Books

| Title | Author | Notes |
|-------|--------|-------|
| *The Swift Programming Language* | Apple | Free — [docs.swift.org](https://docs.swift.org/swift-book/) |
| *Embedded Systems with ARM Cortex-M* | Yiu | Hardware background (C-focused but excellent) |
| *Swift Concurrency by Example* | Hacking with Swift | async/await patterns applicable to actors on MCU |
| *Programming Embedded Systems* | Barr & Massa | Classic firmware concepts (language-agnostic) |

### YouTube & Video

- **WWDC sessions** — Swift language evolution, Embedded Swift announcements
- **Espressif Developer Portal** — ESP32-S3 hardware overview
- **Phil's Lab** — STM32 hardware design (electronics context)
- **Embedded.fm podcast** — firmware engineering culture

### Blogs & Communities

- [Swift Forums — Embedded](https://forums.swift.org/c/development/embedded/43)
- [swift-embedded-examples (GitHub)](https://github.com/apple/swift-embedded-examples)
- [/r/embedded](https://reddit.com/r/embedded) and [/r/swift](https://reddit.com/r/swift)
- [Espressif Documentation](https://docs.espressif.com/)

### Datasheets & Reference Manuals

| Document | Link / Search |
|----------|---------------|
| ESP32-S3 Technical Reference | [espressif.com](https://www.espressif.com/en/products/socs/esp32-s3) |
| STM32F411 Reference Manual | ST RM0383 |
| RP2040 Datasheet | [raspberrypi.com/documentation/microcontrollers](https://www.raspberrypi.com/documentation/microcontrollers/) |
| ARM Cortex-M4 Generic User Guide | ARM DUI 0553 |

---

## Folder & Lesson Index

| File | Title | Prerequisites |
|------|-------|---------------|
| [00-roadmap.md](./00-roadmap.md) | Full phased roadmap | None |
| [01-swift-basics.md](./01-swift-basics.md) | Swift fundamentals for embedded | Basic programming |
| [02-embedded-swift.md](./02-embedded-swift.md) | Freestanding Embedded Swift | [01](./01-swift-basics.md) |
| [03-memory-layout.md](./03-memory-layout.md) | Flash, RAM, linker scripts | [02](./02-embedded-swift.md) |
| [04-swiftpm.md](./04-swiftpm.md) | Swift Package Manager for embedded | [02](./02-embedded-swift.md) |
| [05-embedded-architecture.md](./05-embedded-architecture.md) | PAC / HAL / BSP | [04](./04-swiftpm.md) |
| [06-register-programming.md](./06-register-programming.md) | MMIO & registers | [05](./05-embedded-architecture.md) |
| [07-hal.md](./07-hal.md) | Protocol-oriented HAL | [06](./06-register-programming.md) |
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
| [20-wifi.md](./20-wifi.md) | Wi-Fi station | [15](./15-uart.md) |
| [21-bluetooth.md](./21-bluetooth.md) | BLE GATT | [20](./20-wifi.md) |
| [22-swift-concurrency.md](./22-swift-concurrency.md) | Async embedded | [09](./09-interrupts.md) |
| [23-realtime-patterns.md](./23-realtime-patterns.md) | Interrupts + actors | [09](./09-interrupts.md) |
| [24-low-power.md](./24-low-power.md) | Sleep & current budgets | [10](./10-timers.md) |
| [25-debugging.md](./25-debugging.md) | LLDB, UART, probes | [04](./04-swiftpm.md) |
| [26-testing.md](./26-testing.md) | Host tests, HIL, CI | [07](./07-hal.md) |
| [27-design-patterns.md](./27-design-patterns.md) | State machines, BSP, drivers | [07](./07-hal.md) |
| [glossary.md](./glossary.md) | Term glossary A–Z | Anytime |

### Extended Sections

| Section | Description |
|---------|-------------|
| [boards/](./boards/README.md) | MCU family guides (ESP32-Sx/Cx, STM32, RP2040, nRF52, AVR, MSP430) |
| [communication/](./communication/README.md) | Protocol deep-dives (GPIO, UART, SPI, I²C, CAN, USB, Wi-Fi, BLE, LoRa, MQTT, HTTP, …) |
| [examples/](./examples/README.md) | Hands-on Swift sketches with ownership notes |
| [projects/](./projects/README.md) | Full tutorial builds (weather station, robot, CAN analyzer, …) |
| [sensors/](./sensors/README.md) | Sensor-specific guides (BME280, IMU, GPS, …) |
| [displays/](./displays/README.md) | Display modules (OLED, TFT, NeoPixel, character LCD) |

---

## Language Mapping (Rust → Swift)

If you come from [Embedded Rust](../learning/README.md):

| Embedded Rust | Embedded Swift |
|---------------|----------------|
| Cargo | Swift Package Manager (SwiftPM) |
| `no_std` | Embedded Swift / freestanding |
| Embassy | Swift Concurrency (async/await) |
| RTIC | Interrupts + actors + critical sections |
| embedded-hal | Protocol-oriented HAL traits |
| defmt | print / UART / semihosting / os_log |
| Ownership / borrow checker | ARC + value semantics + `UnsafePointer` |
| `Result<T, E>` | `Result<T, Error>` / typed errors |
| `Option<T>` | `Optional` / `T?` |

---

## Quick Start Checklist

- [ ] Install Swift via Xcode or swift.org
- [ ] Confirm Embedded Swift toolchain availability for your target
- [ ] Install board flash tools (esptool, OpenOCD, or picotool)
- [ ] Connect dev board via USB; confirm serial port appears
- [ ] Clone or create a project from swift-embedded-examples or community template
- [ ] Flash the built-in LED blink example
- [ ] Read [01-swift-basics.md](./01-swift-basics.md) and proceed in order

---

## How to Use This Curriculum

1. **Read the theory** section of each lesson before wiring anything.
2. **Build the HAL example first** — it is easier to debug.
3. **Then try the bare-metal version** to understand what the HAL hides.
4. **Complete the exercises** — they reinforce portable patterns.
5. **Consult the glossary** when you encounter unfamiliar terms.

Good luck — Embedded Swift is young but Swift's type safety and protocol-oriented design translate well to firmware. The compiler is your ally on constrained hardware where bugs are expensive.

---

*Next step: [00-roadmap.md](./00-roadmap.md)*
