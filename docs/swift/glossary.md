# Embedded Swift Glossary (A–Z)

Quick reference for terms used throughout the [Embedded Swift Learning Path](./README.md). Terms link to the lesson where they are introduced in depth.

---

## A

**ADC (Analog-to-Digital Converter)**  
Hardware that samples an analog voltage and outputs a digital value. See [12-adc.md](./12-adc.md).

**Actor**  
Swift concurrency type that serializes access to mutable state — useful for debouncing and sensor fusion on host; ISRs must not call actors directly. See [09-interrupts.md](./09-interrupts.md).

**AHB (Advanced High-performance Bus)**  
ARM system bus connecting CPU, memory, and high-speed peripherals. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**APB (Advanced Peripheral Bus)**  
Lower-speed ARM bus for peripherals; timers and UART often clock from APB. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**API (Application Programming Interface)**  
Functions and types exposed by a module for use by other code.

**ARC (Automatic Reference Counting)**  
Swift memory management for **class** instances — retain/release at compile-inserted points. Limited or absent in freestanding Embedded Swift; prefer structs. See [01-swift-basics.md](./01-swift-basics.md).

**Async / Await**  
Swift Concurrency keywords for asynchronous operations — maps conceptually to Embassy in Rust; full runtime may be limited on MCU. See [09-interrupts.md](./09-interrupts.md).

**Atomic**  
Indivisible operations (`ManagedAtomic`, C atomics) safe for ISR/main communication without locks on single-core. See [09-interrupts.md](./09-interrupts.md).

**Attenuation (ADC)**  
ESP32 ADC setting extending measurable voltage range (e.g., 12 dB → 0–3.3 V). See [12-adc.md](./12-adc.md).

---

## B

**Bare-metal**  
Firmware running directly on hardware without an OS. See [02-embedded-swift.md](./02-embedded-swift.md).

**Bitfield**  
Group of bits within a register representing a configuration field. See [06-register-programming.md](./06-register-programming.md).

**Bootloader**  
Code in ROM or flash that loads your application after reset. See [02-embedded-swift.md](./02-embedded-swift.md).

**BSP (Board Support Package)**  
Board-specific pin mappings and init code. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**BSS**  
Block Started by Symbol — uninitialized static variables zeroed at boot. See [03-memory-layout.md](./03-memory-layout.md).

**Bus (I²C, SPI, AHB, APB)**  
Shared communication pathway; capitalized "Bus" often means on-chip interconnect.

---

## C

**Class**  
Swift reference type with ARC — avoid on MCU firmware; use structs instead. See [01-swift-basics.md](./01-swift-basics.md).

**Clock Tree**  
Network of oscillators, PLLs, and dividers producing CPU/peripheral clocks. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**CMSIS (Cortex Microcontroller Software Interface Standard)**  
ARM standard headers and startup conventions used by PAC layers.

**CoreBluetooth**  
Apple framework for BLE on iOS/macOS — valid host-side IoT path when MCU lacks Swift BLE stack. See [README.md](./README.md).

**Critical Section**  
Code region where interrupts are masked for atomic access. See [09-interrupts.md](./09-interrupts.md).

**Cross-compilation**  
Building on host Mac for a different target architecture. See [04-swiftpm.md](./04-swiftpm.md).

**Cortex-M**  
ARM microcontroller core family (M0+, M4, M7, etc.) used in STM32, RP2040.

---

## D

**DAC (Digital-to-Analog Converter)**  
Converts digital values to analog voltage. See [13-dac.md](./13-dac.md).

**Datasheet**  
Manufacturer document with electrical specs and pin descriptions.

**Debug Probe**  
Hardware (ST-Link, J-Link, DAPLink) for flash and debug over SWD/JTAG. See [README.md](./README.md).

**DelayMs (protocol)**  
HAL trait for millisecond delays. See [07-hal.md](./07-hal.md).

**DigitalInputPin / DigitalOutputPin**  
HAL protocols for GPIO. See [07-hal.md](./07-hal.md), [08-gpio.md](./08-gpio.md).

**DMA (Direct Memory Access)**  
Hardware that moves data between peripherals and RAM without CPU. Future advanced topic.

**Duty Cycle**  
Fraction of PWM period the signal is HIGH. See [11-pwm.md](./11-pwm.md).

---

## E

**Embedded Swift**  
Apple's freestanding Swift subset for microcontrollers — no Foundation, minimal runtime. See [02-embedded-swift.md](./02-embedded-swift.md).

**ELF (Executable and Linkable Format)**  
Standard firmware binary format produced by Swift/LLVM toolchain.

**Enum**  
Swift sum type — ideal for errors, pin modes, and state machines. See [01-swift-basics.md](./01-swift-basics.md).

**Error (protocol)**  
Swift protocol for typed error handling; prefer small enums over Foundation errors. See [01-swift-basics.md](./01-swift-basics.md).

**EXTI (External Interrupt)**  
STM32 external interrupt lines connected to GPIO pins. See [09-interrupts.md](./09-interrupts.md).

**Existential (`any Protocol`)**  
Runtime-polymorphic protocol type — avoid on MCU hot paths; prefer generics. See [07-hal.md](./07-hal.md).

---

## F

**Flash**  
Non-volatile memory storing program code and constants. See [03-memory-layout.md](./03-memory-layout.md).

**Foundation**  
Apple's core framework (String, Date, FileManager) — **not** available in freestanding Embedded Swift.

**Freestanding**  
Swift compilation mode without OS assumptions — synonymous with Embedded Swift context. See [02-embedded-swift.md](./02-embedded-swift.md).

**FPU (Floating Point Unit)**  
Hardware floating-point — STM32F411 has one; M0+ (RP2040) does not — prefer integer math.

---

## G

**Generic**  
Swift type or function parameterized by types — enables zero-cost HAL abstraction. See [07-hal.md](./07-hal.md).

**GPIO (General Purpose Input/Output)**  
Digital pins configurable as input or output. See [08-gpio.md](./08-gpio.md).

**Guard**  
Swift statement for early exit when optional unwrap fails. See [01-swift-basics.md](./01-swift-basics.md).

---

## H

**HAL (Hardware Abstraction Layer)**  
Protocol-oriented peripheral APIs hiding register details. See [05-embedded-architecture.md](./05-embedded-architecture.md), [07-hal.md](./07-hal.md).

**Heap**  
Dynamic memory region — disable or avoid in Embedded Swift firmware. See [03-memory-layout.md](./03-memory-layout.md).

---

## I

**IDE (Integrated Development Environment)**  
Xcode, Cursor, or VS Code for editing and debugging.

**I²C (Inter-Integrated Circuit)**  
Two-wire serial bus — future lesson topic.

**IRQ (Interrupt Request)**  
Hardware signal requesting CPU attention. See [09-interrupts.md](./09-interrupts.md).

**ISR (Interrupt Service Routine)**  
Handler function invoked on interrupt — must be short. See [09-interrupts.md](./09-interrupts.md).

**IRAM (Instruction RAM)**  
Fast RAM on ESP32 for time-critical code — linker section placement.

---

## J

**JTAG (Joint Test Action Group)**  
Debug interface standard; ESP32-S3 includes USB-JTAG.

---

## K

**Knurling**  
(Rust ecosystem) — for Swift, see swift-embedded-examples and community tooling.

---

## L

**LEDC (LED Control)**  
ESP32 PWM peripheral for LED dimming and servo signals. See [11-pwm.md](./11-pwm.md).

**Linker Script**  
File defining memory regions and section placement. See [03-memory-layout.md](./03-memory-layout.md).

**LLDB**  
LLVM debugger used with Swift — breakpoints and memory inspection. See [README.md](./README.md).

**LUT (Look-Up Table)**  
Precomputed values in flash — sine waves for DAC. See [13-dac.md](./13-dac.md).

---

## M

**ManagedAtomic**  
Swift Atomics type for lock-free ISR/main communication. See [09-interrupts.md](./09-interrupts.md).

**MMIO (Memory-Mapped I/O)**  
Peripheral registers accessed as memory addresses. See [06-register-programming.md](./06-register-programming.md).

**MCU (Microcontroller Unit)**  
Single-chip computer with CPU, flash, RAM, and peripherals.

**Mutating**  
Keyword marking struct methods that modify `self`. See [01-swift-basics.md](./01-swift-basics.md).

---

## N

**NVIC (Nested Vectored Interrupt Controller)**  
ARM interrupt controller — priorities and enable bits. See [09-interrupts.md](./09-interrupts.md).

**no_std (Rust term)**  
Rust equivalent: **Embedded Swift / freestanding**. See [02-embedded-swift.md](./02-embedded-swift.md).

---

## O

**Optional (`T?`)**  
Swift type representing presence or absence of a value. See [01-swift-basics.md](./01-swift-basics.md).

**OpenOCD**  
Open On-Chip Debugger — GDB server for ARM targets. See [README.md](./README.md).

**os_log**  
Apple structured logging API for iOS/macOS host apps. See [README.md](./README.md).

**OTA (Over-The-Air)**  
Firmware update via network — advanced topic in Phase 4.

---

## P

**PAC (Peripheral Access Crate)**  
Low-level typed register definitions — Swift struct/enum register blocks. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**Package.swift**  
SwiftPM manifest — equivalent to Cargo.toml. See [04-swiftpm.md](./04-swiftpm.md).

**PinMode**  
Configuration: input, output, analog, alternate function. See [08-gpio.md](./08-gpio.md).

**PLL (Phase-Locked Loop)**  
Clock multiplier generating high-speed CPU clock from crystal.

**Protocol**  
Swift interface definition — HAL trait equivalent. See [07-hal.md](./07-hal.md).

**PWM (Pulse Width Modulation)**  
Digital waveform with variable duty cycle. See [11-pwm.md](./11-pwm.md).

**Prescaler**  
Timer clock divider. See [10-timers.md](./10-timers.md).

---

## R

**Reference Type**  
Classes and actors — heap allocated with ARC on host. See [01-swift-basics.md](./01-swift-basics.md).

**Result (`Result<Success, Failure>`)**  
Enum representing success or typed failure. See [01-swift-basics.md](./01-swift-basics.md).

**RP2040**  
Raspberry Pi Pico microcontroller — dual Cortex-M0+.

**RTIC (Rust term)**  
Real-Time Interrupt-driven Concurrency — Swift equivalent: interrupts + actors. See [09-interrupts.md](./09-interrupts.md).

**Reset Vector**  
Address CPU jumps to on power-on reset.

---

## S

**Semihosting**  
Debug mechanism routing print to host via debugger — not for production. See [README.md](./README.md).

**SPI (Serial Peripheral Interface)**  
Four-wire synchronous bus — future lesson.

**SRAM**  
Static RAM — volatile working memory. See [03-memory-layout.md](./03-memory-layout.md).

**Stack**  
Memory for function locals and call frames — overflow causes crashes. See [03-memory-layout.md](./03-memory-layout.md).

**StaticString**  
Compile-time string literal type — no heap allocation. See [02-embedded-swift.md](./02-embedded-swift.md).

**Struct**  
Swift value type — preferred for embedded drivers. See [01-swift-basics.md](./01-swift-basics.md).

**Swift Concurrency**  
async/await, Task, Actor — maps to Embassy conceptually. See [09-interrupts.md](./09-interrupts.md).

**SwiftPM (Swift Package Manager)**  
Build and dependency tool. See [04-swiftpm.md](./04-swiftpm.md).

**SWD (Serial Wire Debug)**  
Two-wire ARM debug interface.

**SysTick**  
ARM system timer — common 1 ms tick source. See [10-timers.md](./10-timers.md).

---

## T

**Target Triple**  
Architecture string (e.g., `armv7em-none-eabihf`). See [04-swiftpm.md](./04-swiftpm.md).

**Timer**  
Hardware peripheral for periodic events and PWM. See [10-timers.md](./10-timers.md).

**TRM (Technical Reference Manual)**  
Detailed peripheral documentation from chip vendor.

**Tuple**  
Fixed-size heterogeneous collection — useful for fixed buffers without heap.

---

## U

**UART (Universal Asynchronous Receiver-Transmitter)**  
Serial communication peripheral — primary logging transport. See [README.md](./README.md).

**UnsafeMutablePointer**  
Swift pointer type for MMIO and C interop. See [06-register-programming.md](./06-register-programming.md).

**USB-CDC**  
USB Communication Device Class — virtual serial port over USB.

---

## V

**Value Type**  
Structs and enums — copied on assignment, no ARC. See [01-swift-basics.md](./01-swift-basics.md).

**Vector Table**  
Table of ISR addresses at boot. See [09-interrupts.md](./09-interrupts.md).

**Volatile**  
C keyword for MMIO; Swift uses unsafe pointers with documented side effects. See [06-register-programming.md](./06-register-programming.md).

**Vref**  
ADC/DAC reference voltage — typically 3.3 V. See [12-adc.md](./12-adc.md).

---

## W

**W1TS / W1TC**  
Write-1-to-set / write-1-to-clear GPIO registers on ESP32. See [06-register-programming.md](./06-register-programming.md).

**WFI (Wait For Interrupt)**  
CPU sleep instruction until next interrupt — saves power.

**Whole Module Optimization (WMO)**  
Swift compiler mode `-wmo` — optimizes across files. See [04-swiftpm.md](./04-swiftpm.md).

---

## X

**Xcode**  
Apple IDE including Swift toolchain, debugger, and iOS simulator.

**Xtensa**  
CPU architecture in ESP32-S3 (LX7 cores).

---

## Z

**Zero-init (.bss)**  
Static variables default to zero at boot without flash storage cost. See [03-memory-layout.md](./03-memory-layout.md).

---

## Cross-Reference: Rust → Swift Terms

| Embedded Rust | Embedded Swift |
|---------------|----------------|
| Cargo | SwiftPM |
| `no_std` | Embedded Swift / freestanding |
| `embedded-hal` | Protocol-oriented HAL |
| Embassy | Swift Concurrency |
| RTIC | Interrupts + actors |
| defmt | UART print / os_log |
| Ownership | ARC + value types + unsafe pointers |
| `&'static` | Static storage / global lifetime |
| Trait | Protocol |
| Crate | Package / module target |

See [README.md — Language Mapping](./README.md#language-mapping-rust--swift).

---

*Return to [README.md](./README.md)*
