# Lesson 01: Rust Basics for Embedded

Rust on embedded systems uses the same core language as desktop Rust, but the **constraints** differ: no heap by default, no threads in the usual sense, and every byte of RAM matters. This lesson covers the language features you will encounter daily in firmware.

**Prerequisites:** Basic programming experience in any language.  
**Next:** [02-no-std.md](./02-no-std.md)  
**See also:** [glossary.md](./glossary.md), [07-hal.md](./07-hal.md)

---

## Theory

### Why Rust for Embedded?

| Advantage | Explanation |
|-----------|-------------|
| **Memory safety without GC** | No garbage collector pauses — critical for real-time systems |
| **Zero-cost abstractions** | Traits and generics compile to the same code you'd write in C |
| **Fearless concurrency** | The type system catches data races at compile time |
| **Strong ecosystem** | `embedded-hal`, `defmt`, `probe-rs`, board-specific HAL crates |

Rust does **not** eliminate all firmware bugs — logic errors and hardware mistakes still happen — but it eliminates entire classes of memory corruption bugs common in C firmware.

### Embedded vs Desktop Rust

| Feature | Desktop (`std`) | Embedded (`no_std`) |
|---------|-----------------|----------------------|
| Heap allocation (`Box`, `Vec`) | Always available | Optional (`alloc` crate) |
| Threads | `std::thread` | Executor (Embassy) or bare ISRs |
| I/O | Files, sockets | Registers, GPIO, UART |
| Panic | Unwind or abort | Usually `panic = "abort"` |
| Error handling | `anyhow`, `eyre` | `Result` + custom error enums |

---

## Hardware Overview

This is a language lesson — no wiring required. All code examples are written so they **compile conceptually** for an ESP32-S3, but run on the host where noted for practice.

The ESP32-S3 has limited RAM (~512 KB SRAM). Accidentally cloning a large `String` on every loop iteration can cause a stack overflow or heap exhaustion — ownership rules prevent this at compile time.

---

## Wiring Diagram

Not applicable for this lesson. For your first hardware exercise, see [08-gpio.md](./08-gpio.md).

---

## Memory & Register Explanation

Embedded Rust interacts with memory in three primary locations:

```
┌──────────────────────────────────────────────┐
│  Flash (ROM) — program code, constants         │
│  .text, .rodata                                │
├──────────────────────────────────────────────┤
│  RAM — stack (function locals), .data, .bss  │
│  Stack grows ↓    Heap grows ↑ (if alloc)     │
├──────────────────────────────────────────────┤
│  MMIO Registers — peripheral control         │
│  Fixed addresses (e.g., 0x6000_4000 GPIO)      │
└──────────────────────────────────────────────┘
```

Ownership determines **who may read or write** each region and for how long. See [03-memory-layout.md](./03-memory-layout.md) for linker-level detail.

---

## HAL Implementation

HAL code relies heavily on ownership and traits. Here is a typical pattern:

```rust
use esp_hal::gpio::{Output, OutputPin, Level};

/// Owns the pin — when `led` is dropped, the pin is returned to a safe state.
fn blink_once<P: OutputPin>(mut led: Output<'static, P>) {
    // `led` is mutably borrowed here — we have exclusive access.
    led.set_high().unwrap();
    // Ownership of `led` remains with this function until it returns.
}
```

Key observations:

- `Output<'static, P>` — the `'static` lifetime means the pin lives for the entire program (required for ISRs).
- `mut led` — mutable binding allows calling methods that change pin state.
- Generics (`P: OutputPin`) — compile-time polymorphism with zero runtime cost.

---

## Bare-Metal Implementation

Bare-metal code still uses ownership — Rust does not let you bypass it even when touching registers:

```rust
use core::ptr::{read_volatile, write_volatile};

const GPIO_OUT_REG: *mut u32 = 0x6000_4004 as *mut u32;

/// Toggle a GPIO bit. `pin_mask` is owned by the caller (Copy type — no move issues).
fn toggle_pin(pin_mask: u32) {
    // SAFETY: GPIO_OUT_REG is a valid MMIO address on ESP32-S3.
    // We hold exclusive access through raw pointers — no other Rust reference exists.
    unsafe {
        let current = read_volatile(GPIO_OUT_REG);
        write_volatile(GPIO_OUT_REG, current ^ pin_mask);
    }
}
```

`pin_mask: u32` is `Copy` — passed by value, no borrowing complexity. Register access requires `unsafe` because the compiler cannot verify hardware side effects.

---

## Step-by-Step Explanation

### 1. Ownership

Every value in Rust has exactly **one owner**. When the owner goes out of scope, the value is dropped.

```rust
let buffer = [0u8; 64];   // `buffer` owns the array on the stack
process(buffer);           // Ownership MOVES into `process` — caller can't use buffer after
// buffer;  // COMPILE ERROR: value moved
```

For embedded, prefer stack arrays and static buffers over heap allocation:

```rust
// Good: fixed-size stack buffer, no heap
fn read_sensor() -> [u8; 4] {
    [0x01, 0x02, 0x03, 0x04]
}
```

### 2. Borrowing

Instead of moving, lend a reference:

```rust
fn print_bytes(data: &[u8]) {  // Immutable borrow — read-only access
    for b in data {
        defmt::info!("{}", b);
    }
}

let buf = [1u8, 2, 3];
print_bytes(&buf);   // Borrow — `buf` still owned by caller
print_bytes(&buf);   // Can borrow again
```

Mutable borrow — exclusive write access:

```rust
fn fill_zeros(data: &mut [u8]) {
    for b in data.iter_mut() {
        *b = 0;
    }
}

let mut buf = [1u8; 8];
fill_zeros(&mut buf);  // Only ONE mutable borrow at a time
```

**Embedded rule:** Inside an ISR (Interrupt Service Routine), avoid `&mut` to shared state without synchronization. See [09-interrupts.md](./09-interrupts.md).

### 3. Lifetimes

Lifetimes annotate **how long references are valid**:

```rust
struct Config<'a> {
    name: &'a str,   // Reference must outlive the struct
}

// 'static = lives for entire program — common in embedded
static DEVICE_NAME: &str = "sensor-node";
let cfg = Config { name: DEVICE_NAME };
```

When you see `'static` in HAL signatures, it usually means "this pin/buffer must live forever" — typically because an ISR holds a reference to it.

### 4. Result and Option

Embedded code rarely panics on expected errors. Use `Result`:

```rust
use embedded_hal::digital::OutputPin;

fn set_led_high<P: OutputPin>(pin: &mut P) -> Result<(), P::Error> {
    pin.set_high()   // Returns Result — hardware can fail (e.g., pin not configured)
}

// Handling errors explicitly:
match set_led_high(&mut led) {
    Ok(()) => defmt::info!("LED on"),
    Err(e) => defmt::error!("GPIO error: {:?}", defmt::Debug2Format(&e)),
}
```

`Option` replaces null pointers:

```rust
fn find_config(id: u8) -> Option<&'static Config> {
    CONFIGS.iter().find(|c| c.id == id)
}

if let Some(cfg) = find_config(3) {
    defmt::info!("Found: {}", cfg.name);
}
```

### 5. Traits

Traits define shared behavior — the foundation of `embedded-hal`:

```rust
use embedded_hal::delay::DelayNs;

/// Generic over any delay provider — portable across boards.
fn wait<D: DelayNs>(delay: &mut D, ms: u32) {
    delay.delay_ms(ms);  // Trait method — resolved at compile time
}
```

Common embedded traits (detailed in [07-hal.md](./07-hal.md)):

| Trait | Purpose |
|-------|---------|
| `OutputPin` | Set pin high/low |
| `InputPin` | Read pin state |
| `DelayNs` | Microsecond/millisecond delays |
| `I2c` | I²C bus transactions |

### 6. Modules and Crates

Organize firmware into modules:

```rust
mod sensors {
    pub mod bme280 {
        pub fn read_temp() -> f32 { 22.5 }
    }
}

mod board {
    pub fn init() { /* clock, GPIO setup */ }
}

fn main() -> ! {
    board::init();
    let temp = sensors::bme280::read_temp();
    defmt::info!("Temp: {}", temp);
    loop {}
}
```

Each module controls visibility with `pub`. Crate boundaries (separate `Cargo.toml`) enable reusable drivers.

---

## Common Mistakes

| Mistake | Problem | Fix |
|---------|---------|-----|
| Cloning in hot loops | RAM/flash bloat, timing jitter | Use references or `Copy` types |
| Ignoring `Result` with `.unwrap()` | Panic on hardware failure | Match or `?` with proper error handling |
| `'static` confusion | Compiler rejects ISR setup | Use `StaticCell`, `OnceCell`, or heapless queues |
| Fighting the borrow checker with `unsafe` | Undefined behavior | Restructure code; use `Mutex<RefCell<T>>` |
| Using `String` everywhere | Requires `alloc`; fragments heap | Use `&str`, `heapless::String<N>`, or fixed arrays |
| Mutable static without protection | Data races between ISR and main | Critical sections or atomic types |

---

## Debugging Tips

1. **Read the full compiler error** — borrow checker messages suggest fixes ("consider using `RefCell`").
2. **Enable Clippy:** `cargo clippy -- -W clippy::all` catches embedded anti-patterns.
3. **Use `defmt::trace!`** to log without formatting overhead.
4. **Reduce generics temporarily** — if trait bounds confuse you, monomorphize to one concrete type first, then generalize.
5. **`cargo expand`** — see what the macro/trait generates.

---

## Performance Tips

| Tip | Rationale |
|-----|-----------|
| Prefer `Copy` types (`u32`, `[T; N]`) | No move overhead |
| Use `const` and `static` for lookup tables | Stored in flash, not computed at runtime |
| Avoid `format!` and `String` | Heap allocation is slow and may fail |
| Mark hot-path functions `#[inline]` sparingly | Compiler usually knows best |
| Use `-C opt-level=s` or `z` for release | Smaller flash footprint |

---

## Exercises

### Exercise 1: Ownership Drill

Write a function `accumulate(samples: [u16; 8]) -> u32` that sums samples. Call it twice with the same array (you'll need to use borrowing or `Copy` — `[u16; 8]` is `Copy`).

### Exercise 2: Result Handler

Write `fn parse_command(cmd: &str) -> Result<u8, &'static str>` that parses "LED:0"–"LED:3" into pin numbers. Return an error for invalid input.

### Exercise 3: Trait Bounds

Write a function `toggle_three<P: OutputPin>(pins: &mut [P])` that toggles each pin. (Hint: loop with mutable references.)

### Exercise 4: Module Layout

Split a project into `board`, `drivers/led`, and `app` modules. The `app` module calls `board::init()` and `drivers::led::blink()`.

### Exercise 5: Lifetime Exploration

Create a struct `PinRef<'a, P: OutputPin>` that holds `&'a mut P`. Explain why this cannot be stored in a static ISR context.

---

## References

- [The Rust Book — Ownership](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html)
- [The Rust Book — Traits](https://doc.rust-lang.org/book/ch10-02-traits.html)
- [The Embedded Rust Book — Type Conversion](https://docs.rust-embedded.org/book/interoperability/index.html)
- [Rust by Example — Error Handling](https://doc.rust-lang.org/rust-by-example/error.html)
- [embedded-hal traits documentation](https://docs.rs/embedded-hal/latest/embedded_hal/)

---

*Previous: [README.md](./README.md) | Next: [02-no-std.md](./02-no-std.md)*
