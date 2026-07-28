# Lesson 02: no_std — Firmware Without the Standard Library

Desktop Rust programs use the **standard library** (`std`) for files, networking, threads, and heap allocation. Microcontroller firmware typically has none of these — you write `#![no_std]` Rust.

**Prerequisites:** [01-rust-basics.md](./01-rust-basics.md)  
**Next:** [03-memory-layout.md](./03-memory-layout.md)  
**See also:** [04-cargo.md](./04-cargo.md), [glossary.md](./glossary.md)

---

## Theory

### What is `no_std`?

`#![no_std]` is a crate-level attribute that tells the compiler: **do not link `std`**. You still have access to **`core`** — the subset of Rust that needs no operating system:

| Available in `core` | NOT available without `std`/`alloc` |
|---------------------|-------------------------------------|
| `Option`, `Result`, iterators | `File`, `TcpStream`, `println!` (host) |
| Slices, arrays, tuples | Thread spawning |
| Traits (`Copy`, `Send`, `Sync`) | Dynamic dispatch with `Box` (needs alloc) |
| Atomic types, `mem` utilities | `format!` macro (needs alloc) |

### The `#![no_main]` Attribute

Firmware has no operating system to call `main()`. Instead:

1. **Reset vector** points to startup code (from linker script + runtime crate)
2. Startup initializes `.data`, zeroes `.bss`, sets up stack pointer
3. Your `#[entry]` function or `main()` runs

```rust
#![no_std]   // No standard library
#![no_main]  // We provide our own entry point (not hosted main)

use panic_halt as _;  // Panic handler (see below)
```

### Optional: `alloc` Crate

If your chip has enough RAM and you configure a global allocator, you can enable **`alloc`** for `Box`, `Vec`, `String`:

```rust
#![no_std]
extern crate alloc;

use alloc::vec::Vec;  // Heap-backed vector — use sparingly on MCUs
```

Most beginner embedded projects avoid `alloc` entirely. ESP32-S3 projects sometimes use it for Wi-Fi buffers; STM32F411 (128 KB RAM) often does not.

---

## Hardware Overview

### ESP32-S3 Boot Flow

```
Power-on
   │
   ▼
ROM Bootloader (in chip ROM)
   │  — checks GPIO strap pins, flash config
   ▼
2nd Stage Bootloader (in flash)
   │  — loads application partition
   ▼
Your firmware `_start` / `main`
   │  — `.data` copy, `.bss` zero, static init
   ▼
Application `main()` or `#[entry] fn main()`
```

### STM32 Boot Flow (Comparison)

ARM Cortex-M chips vector directly from flash address `0x0000_0000`:

```
Reset Vector → Stack Pointer value
Offset +4    → Reset_Handler (startup assembly)
             → __initialize_data, __zero_bss
             → main()
```

RP2040 is similar to STM32 — both use ARM Cortex-M boot conventions.

---

## Wiring Diagram

Not applicable. Ensure your dev board is connected via USB for flashing — see [README.md](./README.md#hardware-requirements).

---

## Memory & Register Explanation

### What Runs Where in `no_std`

| Section | Content | Initialized By |
|---------|---------|----------------|
| `.text` | Machine code | Flashed |
| `.rodata` | String literals, const arrays | Flashed |
| `.data` | Mutable statics with initial values | Copied from flash at boot |
| `.bss` | Zero-initialized statics | Zeroed at boot |
| Stack | Local variables, call frames | Grows at runtime |

Without `std`, there is **no default heap** unless you provide `_sbrk` or use `esp-alloc`:

```rust
// ESP32-S3 example — global allocator setup
use esp_alloc as _;
```

---

## HAL Implementation

Modern HAL crates hide startup details. ESP32-S3 with `esp-hal`:

```rust
#![no_std]
#![no_main]

use defmt::info;
use esp_hal::{
    clock::ClockControl,
    gpio::{Io, Level, Output},
    peripherals::Peripherals,
};
use panic_halt as _;

#[esp_hal::macros::entry]  // Macro generates proper entry point + runtime init
fn main() -> ! {
    // `Peripherals::take()` — ownership: only one caller may take peripherals.
    // Returns a struct owning all hardware blocks; fields are consumed as used.
    let peripherals = Peripherals::take();

    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
    let mut led = Output::new(io.pins.gpio2, Level::Low);

    loop {
        led.toggle().ok();
        // Delay via busy loop or timer — see [10-timers.md](./10-timers.md)
        for _ in 0..500_000 { core::hint::spin_loop(); }
    }
}
```

**Ownership notes:**

- `Peripherals::take()` consumes the singleton — prevents double-init of hardware.
- `led` is owned by `main` and mutably borrowed in the loop.
- Return type `!` (never type) — firmware runs forever, never returns to an OS.

### STM32 with `cortex-m-rt` (Comparison)

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f4xx_hal as hal;

#[entry]  // Attribute marks entry; cortex-m-rt provides startup
fn main() -> ! {
    // HAL init...
    loop {}
}
```

---

## Bare-Metal Implementation

Minimal `no_std` skeleton showing explicit pieces:

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

/// Called on panic — required in every `no_std` binary.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // `&PanicInfo` is borrowed — we don't own the panic message.
    loop {
        core::hint::spin_loop();
    }
}

/// Reset handler — on ARM this replaces `_start`.
/// On ESP32, esp-hal generates equivalent code.
#[no_mangle]
pub extern "C" fn Reset() -> ! {
    // Startup would copy .data, zero .bss here (usually in assembly).
    main();
}

fn main() -> ! {
    loop {
        core::hint::spin_loop();
    }
}
```

In practice, **never write this from scratch** — use `cortex-m-rt`, `esp-hal`, or `rp2040-hal` runtimes. Understanding the pieces helps when debugging boot failures.

### Panic Handler Variants

| Crate | Behavior |
|-------|----------|
| `panic-halt` | Infinite loop — smallest code size |
| `panic-probe` | Breakpoint for probe-rs debugging |
| `panic-semihosting` | Print panic message via debug probe |
| Custom | Blink LED pattern, log via `defmt`, then halt |

```rust
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    defmt::panic!("{}", defmt::Display2Format(info));  // Logs then halts
}
```

---

## Step-by-Step Explanation

### Step 1: Create a `no_std` Project

```bash
cargo generate esp-rs/esp-template  # ESP32-S3
# OR
cargo generate rust-embedded/cortex-m-quickstart  # ARM
```

### Step 2: Inspect `Cargo.toml`

```toml
[package]
name = "my-firmware"
version = "0.1.0"
edition = "2021"

[dependencies]
esp-hal = { version = "0.21", features = ["esp32s3"] }
defmt = "0.3"
defmt-rtt = "0.4"
panic-halt = "0.2"

[profile.release]
panic = "abort"       # No stack unwinding — saves flash
codegen-units = 1
lto = true
opt-level = "s"
```

`panic = "abort"` — on panic, stop immediately. Unwinding requires `std` infrastructure.

### Step 3: Understand `#![no_main]`

The `#[entry]` macro expands to:

- A vector table (ARM) or ROM entry symbol (ESP)
- Initialization of memory sections
- Call to your function

Your function **must** return `!` (never) — there is nowhere to return to.

### Step 4: Add `defmt` Logging

```toml
[dependencies.defmt]
version = "0.3"

[dependencies.defmt-rtt]
version = "0.4"

[features]
default = ["defmt"]
```

```rust
defmt::info!("Boot complete");  // No std::io — uses RTT buffer
```

### Step 5: Build and Flash

```bash
cargo build --release
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/my-firmware
```

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Forgetting `#![no_std]` | Links `std`; binary won't fit | Add attribute at crate root |
| Missing panic handler | Link error: `rust_begin_unwind` | Add `panic-halt` or custom handler |
| Using `println!` | Compile error | Use `defmt::info!` or UART write |
| Using `std` types in dependencies | Transitive `std` pull-in | Enable `default-features = false` on deps |
| `main()` returns `()` instead of `!` | Compile error on embedded | Return `!`; infinite loop at end |
| Calling `Peripherals::take()` twice | Panic at runtime | Take once; pass references |

---

## Debugging Tips

1. **Link errors mentioning `std`** — run `cargo tree` and look for crates without `default-features = false`.
2. **Immediate hang after flash** — startup or panic before logging; use `panic-probe` crate.
3. **`cargo nm` / `cargo size`** — verify `_start` or `Reset` symbol exists.
4. **Compare with known-good template** — diff your `Cargo.toml` and `.cargo/config.toml`.
5. **Enable `defmt` timestamps** — helps correlate events.

---

## Performance Tips

| Tip | Impact |
|-----|--------|
| `panic = "abort"` | Smaller binary, faster fail |
| Avoid `alloc` | Deterministic RAM usage |
| `lto = true` in release | Cross-crate inlining |
| `-C link-arg=-nostartfiles` (only if you know why) | Custom startup — advanced |
| Use `core::hint::spin_loop()` in idle loops | Lower power than empty loop on some CPUs |

---

## Exercises

### Exercise 1: Minimal Binary

Create a `no_std` project that does nothing but loop. Measure flash usage with `cargo size --release`.

### Exercise 2: Custom Panic Handler

Write a panic handler that toggles an GPIO pin five times fast, then halts. (Requires board init before panic — test with deliberate `panic!()`.)

### Exercise 3: Dependency Audit

Add a desktop-only crate (e.g., `serde_json`) and observe the error. Fix with appropriate `no_std`-compatible alternatives.

### Exercise 4: Boot Message

Print three `defmt` messages: "Starting", "Clocks OK", "Entering main loop". Verify order in serial monitor.

### Exercise 5: Compare Entry Points

Document the differences between `#[esp_hal::macros::entry]`, `#[entry]` (cortex-m-rt), and RP2040's `entry` for your target board.

---

## References

- [The Embedded Rust Book — no_std](https://docs.rust-embedded.org/book/intro/no-std.html)
- [RFC 1184 — no_std](https://rust-lang.github.io/rfcs/1184-no-std.html)
- [esp-hal documentation](https://docs.espressif.com/projects/rust/esp-hal/latest/)
- [cortex-m-rt crate](https://docs.rs/cortex-m-rt/)
- [defmt book](https://defmt.ferrous-systems.com/)

---

*Previous: [01-rust-basics.md](./01-rust-basics.md) | Next: [03-memory-layout.md](./03-memory-layout.md)*
