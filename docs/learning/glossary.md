# Embedded Rust Glossary (A–Z)

Quick reference for terms used throughout the [Embedded Rust Learning Path](./README.md). Terms link to the lesson where they are introduced in depth.

---

## A

**ADC (Analog-to-Digital Converter)**  
Hardware that samples an analog voltage and outputs a digital value. See [12-adc.md](./12-adc.md).

**AHB (Advanced High-performance Bus)**  
ARM system bus connecting CPU, memory, and high-speed peripherals. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**APB (Advanced Peripheral Bus)**  
Lower-speed ARM bus for peripherals; timers and UART often clock from APB. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**API (Application Programming Interface)**  
Functions and types exposed by a crate for use by other code.

**Async (Asynchronous)**  
Programming model where operations yield instead of blocking; common in Embassy. See [07-hal.md](./07-hal.md).

**Atomic**  
Indivisible CPU operations (`AtomicU32`, etc.) safe for ISR/main communication without mutexes on single-core. See [09-interrupts.md](./09-interrupts.md).

**Attenuation (ADC)**  
ESP32 ADC setting extending measurable voltage range (e.g., 12 dB → 0–3.3 V). See [12-adc.md](./12-adc.md).

---

## B

**Bare-metal**  
Firmware running directly on hardware without an OS. See [02-no-std.md](./02-no-std.md).

**Bitfield**  
Group of bits within a register representing a configuration field. See [06-register-programming.md](./06-register-programming.md).

**Bootloader**  
Code in ROM or flash that loads your application after reset. See [02-no-std.md](./02-no-std.md).

**Borrowing**  
Temporary access to data via `&T` or `&mut T` without transferring ownership. See [01-rust-basics.md](./01-rust-basics.md).

**BSS**  
Block Started by Symbol — uninitialized static variables zeroed at boot. See [03-memory-layout.md](./03-memory-layout.md).

**BSP (Board Support Package)**  
Board-specific pin mappings and init code. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**Bus (I²C, SPI, AHB, APB)**  
Shared communication pathway; capitalized "Bus" often means on-chip interconnect.

---

## C

**Cargo**  
Rust build tool and package manager. See [04-cargo.md](./04-cargo.md).

**Clock Tree**  
Network of oscillators, PLLs, and dividers producing CPU/peripheral clocks. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**CMSIS (Cortex Microcontroller Software Interface Standard)**  
ARM standard headers and startup conventions used by PAC crates.

**Critical Section**  
Code region where interrupts are masked for atomic access. See [09-interrupts.md](./09-interrupts.md).

**Cross-compilation**  
Building on host PC for a different target architecture. See [04-cargo.md](./04-cargo.md).

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

**defmt**  
Deferred logging framework for embedded — low overhead structured logs. See [02-no-std.md](./02-no-std.md).

**DMA (Direct Memory Access)**  
Hardware that moves data between peripherals and RAM without CPU. See [00-roadmap.md](./00-roadmap.md).

**Duty Cycle**  
Fraction of PWM period the signal is HIGH. See [11-pwm.md](./11-pwm.md).

---

## E

**ELF (Executable and Linkable Format)**  
Standard file format for compiled firmware binaries.

**embedded-hal**  
Trait definitions for portable embedded drivers. See [07-hal.md](./07-hal.md).

**Embassy**  
Async embedded framework for Rust. See [00-roadmap.md](./00-roadmap.md).

**Entry Point**  
First function executed after reset (`main`, `Reset`, or `#[entry]`). See [02-no-std.md](./02-no-std.md).

**Errata**  
Silicon errata sheet documenting chip bugs and workarounds.

**ESP32-S3**  
Espressif dual-core Xtensa MCU with Wi-Fi/BLE — primary example in this curriculum. See [README.md](./README.md).

**Exception**  
CPU-internal event (HardFault, NMI) distinct from peripheral interrupts. See [09-interrupts.md](./09-interrupts.md).

---

## F

**Feature Flag**  
Cargo.toml option enabling conditional compilation (`features = ["esp32s3"]`). See [04-cargo.md](./04-cargo.md).

**Flash**  
Non-volatile memory storing program code. See [03-memory-layout.md](./03-memory-layout.md).

**FPU (Floating Point Unit)**  
Hardware for fast float math (Cortex-M4F, etc.).

**Frequency**  
Cycles per second (Hz); PWM and timer configuration parameter. See [11-pwm.md](./11-pwm.md).

---

## G

**GPIO (General Purpose Input/Output)**  
Digital pins configurable as input or output. See [08-gpio.md](./08-gpio.md).

**GPTimer**  
General-purpose timer on ESP32. See [10-timers.md](./10-timers.md).

**Ground (GND)**  
Common reference voltage; all circuits must share ground.

---

## H

**HAL (Hardware Abstraction Layer)**  
Safe wrappers over PAC providing idiomatic APIs. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**HardFault**  
ARM exception for serious CPU errors (invalid instruction, bad memory access).

**Heap**  
Dynamic memory region for `alloc`; optional in embedded. See [03-memory-layout.md](./03-memory-layout.md).

**HSE (High-Speed External)**  
External crystal oscillator for accurate clocks. See [05-embedded-architecture.md](./05-embedded-architecture.md).

---

## I

**I²C (Inter-Integrated Circuit)**  
Two-wire serial bus (SDA, SCL) for sensors and EEPROM. See [00-roadmap.md](./00-roadmap.md).

**IDE (Integrated Development Environment)**  
Editor like VS Code / Cursor with rust-analyzer.

**INL/DNL (Integral/Differential Non-Linearity)**  
ADC linearity specifications. See [12-adc.md](./12-adc.md).

**Interrupt**  
Hardware signal causing CPU to run an ISR. See [09-interrupts.md](./09-interrupts.md).

**I/O MUX (Input/Output Multiplexer)**  
ESP32 pin function selector routing pin to GPIO or peripheral. See [06-register-programming.md](./06-register-programming.md).

**IRQ (Interrupt Request)**  
Signal from peripheral requesting CPU attention. See [09-interrupts.md](./09-interrupts.md).

**ISR (Interrupt Service Routine)**  
Function handling an interrupt — must be short and safe. See [09-interrupts.md](./09-interrupts.md).

**IRAM (Instruction RAM)**  
Fast internal RAM on ESP32 for time-critical code. See [03-memory-layout.md](./03-memory-layout.md).

---

## J

**JTAG (Joint Test Action Group)**  
Debug interface standard; ESP32-S3 supports USB-JTAG.

---

## K

**Knurling**  
Ferrous Systems embedded tooling project (`defmt`, `probe-rs`, `flip-link`).

---

## L

**LEDC (LED Control)**  
ESP32 PWM peripheral. See [11-pwm.md](./11-pwm.md).

**Lifetime (`'a`, `'static`)**  
Rust annotation for reference validity duration. See [01-rust-basics.md](./01-rust-basics.md).

**Linker Script**  
`.ld` file defining memory regions and section placement. See [03-memory-layout.md](./03-memory-layout.md).

**LTO (Link-Time Optimization)**  
Cross-crate optimization at link stage. See [04-cargo.md](./04-cargo.md).

---

## M

**MMIO (Memory-Mapped I/O)**  
Peripheral registers accessed as memory addresses. See [06-register-programming.md](./06-register-programming.md).

**MCU (Microcontroller Unit)**  
Single-chip computer with CPU, memory, and peripherals. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**Monomorphization**  
Compiler generating specialized code for each generic type instance. See [07-hal.md](./07-hal.md).

**MPU (Microprocessor Unit)**  
CPU requiring external memory — contrast with MCU.

**Mutex**  
Synchronization primitive ensuring exclusive access; `critical_section::Mutex` in `no_std`. See [09-interrupts.md](./09-interrupts.md).

---

## N

**NVIC (Nested Vectored Interrupt Controller)**  
ARM interrupt controller with priorities. See [09-interrupts.md](./09-interrupts.md).

**NVS (Non-Volatile Storage)**  
ESP-IDF flash partition for key-value config storage.

**no_std**  
Attribute excluding Rust standard library. See [02-no-std.md](./02-no-std.md).

**NMI (Non-Maskable Interrupt)**  
Highest-priority interrupt that cannot be disabled.

---

## O

**Open-drain**  
Output mode pulling LOW or floating HIGH (external pull-up). See [08-gpio.md](./08-gpio.md).

**Option**  
Rust enum `Some(T)` | `None` replacing null pointers. See [01-rust-basics.md](./01-rust-basics.md).

**OTA (Over-The-Air Update)**  
Remote firmware update. See [00-roadmap.md](./00-roadmap.md).

**Ownership**  
Rust rule: each value has one owner. See [01-rust-basics.md](./01-rust-basics.md).

---

## P

**PAC (Peripheral Access Crate)**  
Auto-generated register definitions from SVD. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**Panic**  
Unrecoverable error path; calls panic handler in `no_std`. See [02-no-std.md](./02-no-std.md).

**Peripheral**  
On-chip hardware block (UART, SPI, timer, etc.).

**PIO (Programmable I/O)**  
RP2040 custom state machines for bit-banging protocols.

**PLL (Phase-Locked Loop)**  
Clock multiplier generating high-frequency CPU clock. See [05-embedded-architecture.md](./05-embedded-architecture.md).

**Polling**  
Repeatedly checking hardware state in a loop. Contrast with interrupts. See [08-gpio.md](./08-gpio.md).

**probe-rs**  
ARM debug and flash tool. See [README.md](./README.md).

**Push-pull**  
GPIO output actively drives HIGH and LOW. See [08-gpio.md](./08-gpio.md).

**PWM (Pulse Width Modulation)**  
Digital signal with variable duty cycle. See [11-pwm.md](./11-pwm.md).

---

## Q

**Quantization**  
ADC process mapping continuous voltage to discrete digital steps. See [12-adc.md](./12-adc.md).

---

## R

**RAM (Random Access Memory)**  
Volatile memory for stack, heap, and variables. See [03-memory-layout.md](./03-memory-layout.md).

**RefCell**  
Interior mutability container; used with Mutex in ISRs. See [09-interrupts.md](./09-interrupts.md).

**Register**  
Memory-mapped hardware control word. See [06-register-programming.md](./06-register-programming.md).

**Resolution (ADC/DAC/PWM)**  
Number of discrete steps (bits). See [12-adc.md](./12-adc.md), [13-dac.md](./13-dac.md).

**Result**  
Rust enum `Ok(T)` | `Err(E)` for fallible operations. See [01-rust-basics.md](./01-rust-basics.md).

**ROM**  
Read-only memory containing boot code. See [03-memory-layout.md](./03-memory-layout.md).

**RP2040**  
Raspberry Pi dual Cortex-M0+ microcontroller. See [README.md](./README.md).

**RTIC (Real-Time Interrupt-driven Concurrency)**  
Framework for priority-based preemptive scheduling. See [00-roadmap.md](./00-roadmap.md).

**RTT (Real-Time Transfer)**  
Debug transport for `defmt` via probe. See [README.md](./README.md).

**Rust-analyzer**  
IDE language server for Rust code completion and diagnostics.

---

## S

**SAR ADC (Successive Approximation Register)**  
Common ADC architecture on ESP32 and STM32. See [12-adc.md](./12-adc.md).

**Semihosting**  
Debug mechanism routing I/O through probe to host.

**Slice (`&[u8]`)**  
Borrowed view into contiguous memory — common for buffers. See [01-rust-basics.md](./01-rust-basics.md).

**SPI (Serial Peripheral Interface)**  
Four-wire bus (MOSI, MISO, SCK, CS). See [00-roadmap.md](./00-roadmap.md).

**Stack**  
Memory for locals and call frames; grows downward. See [03-memory-layout.md](./03-memory-layout.md).

**Static**  
Variable with `'static` lifetime stored in `.data` or `.bss`. See [03-memory-layout.md](./03-memory-layout.md).

**STM32**  
STMicroelectronics ARM MCU family. See [README.md](./README.md).

**SVCall / PendSV**  
ARM exceptions used by RTOS context switching.

**SVD (System View Description)**  
XML describing registers for PAC generation. See [06-register-programming.md](./06-register-programming.md).

**Swd (Serial Wire Debug)**  
Two-wire ARM debug interface (SWDIO, SWCLK). See [README.md](./README.md).

**SysTick**  
ARM system timer for delays and ticks. See [10-timers.md](./10-timers.md).

---

## T

**Target Triple**  
Rust platform identifier (e.g., `thumbv7em-none-eabihf`). See [04-cargo.md](./04-cargo.md).

**Technical Reference Manual (TRM)**  
Detailed peripheral documentation from chip vendor.

**Timer**  
Hardware counter for delays, PWM timebase, and scheduling. See [10-timers.md](./10-timers.md).

**Trait**  
Rust interface definition enabling generics and embedded-hal. See [01-rust-basics.md](./01-rust-basics.md), [07-hal.md](./07-hal.md).

---

## U

**UART (Universal Asynchronous Receiver-Transmitter)**  
Serial communication peripheral. See [00-roadmap.md](./00-roadmap.md).

**USB-JTAG**  
Built-in debug on ESP32-S3 via USB — no external probe needed. See [README.md](./README.md).

---

## V

**Vector Table**  
Table of ISR addresses at start of flash (ARM). See [09-interrupts.md](./09-interrupts.md).

**Vref (Reference Voltage)**  
Voltage reference for ADC/DAC conversion. See [12-adc.md](./12-adc.md).

**Volatile**  
Keyword/attribute ensuring compiler emits every MMIO access. See [06-register-programming.md](./06-register-programming.md).

---

## W

**W1TS / W1TC (Write 1 to Set / Clear)**  
ESP32 GPIO registers for atomic bit set/clear. See [06-register-programming.md](./06-register-programming.md).

**Watchdog (WDT)**  
Timer that resets MCU if not periodically fed — safety mechanism.

**WFI (Wait For Interrupt)**  
ARM sleep instruction waking on interrupt. See [10-timers.md](./10-timers.md).

**Workspace**  
Cargo multi-crate project sharing build directory. See [04-cargo.md](./04-cargo.md).

---

## X

**XIP (Execute In Place)**  
Running code directly from external flash via cache. See [03-memory-layout.md](./03-memory-layout.md).

**Xtensa**  
CPU architecture used in ESP32-S3 (custom Tensilica core).

---

## Y

**Yield**  
In async contexts, surrender control to executor without blocking.

---

## Z

**Zero-cost Abstraction**  
Rust trait/generics compiling to same code as manual implementation. See [07-hal.md](./07-hal.md).

---

## Abbreviation Quick Table

| Abbr | Expansion |
|------|-----------|
| ADC | Analog-to-Digital Converter |
| BSP | Board Support Package |
| DAC | Digital-to-Analog Converter |
| DMA | Direct Memory Access |
| GPIO | General Purpose Input/Output |
| HAL | Hardware Abstraction Layer |
| I²C | Inter-Integrated Circuit |
| IRQ | Interrupt Request |
| ISR | Interrupt Service Routine |
| MCU | Microcontroller Unit |
| MMIO | Memory-Mapped I/O |
| NVIC | Nested Vectored Interrupt Controller |
| PAC | Peripheral Access Crate |
| PLL | Phase-Locked Loop |
| PWM | Pulse Width Modulation |
| SPI | Serial Peripheral Interface |
| SWD | Serial Wire Debug |
| TRM | Technical Reference Manual |
| UART | Universal Asynchronous Receiver-Transmitter |

---

*Return to [README.md](./README.md) | Browse [00-roadmap.md](./00-roadmap.md)*
