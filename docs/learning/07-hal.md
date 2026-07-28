# Lesson 07: embedded-hal — Portable Hardware Abstraction

**embedded-hal** is a collection of traits defining common embedded operations (GPIO, I²C, SPI, delay). Drivers written against these traits work on any board whose HAL implements them.

**Prerequisites:** [06-register-programming.md](./06-register-programming.md)  
**Next:** [08-gpio.md](./08-gpio.md)  
**See also:** [05-embedded-architecture.md](./05-embedded-architecture.md), [01-rust-basics.md](./01-rust-basics.md)

---

## Theory

### Why HAL Traits Exist

Without shared traits, every sensor driver is tied to one chip's GPIO API:

```
❌ Bad:  bme280_stm32.rs, bme280_esp32.rs, bme280_rp2040.rs
✅ Good: bme280.rs using embedded_hal::i2c::I2c
```

Traits enable **compile-time polymorphism** — monomorphization generates specialized code with zero runtime overhead.

### embedded-hal Versions

| Version | Status | Notes |
|---------|--------|-------|
| **0.2.x** | Legacy | Still common in older drivers |
| **1.0** | Current | Cleaner trait split (`I2c`, `SpiBus`, `DelayNs`) |
| **async 1.0** | Growing | `embedded-hal-async` for Embassy |

Always check driver docs for required version.

### Core Trait Categories

| Category | Traits | Purpose |
|----------|--------|---------|
| Digital I/O | `InputPin`, `OutputPin`, `StatefulOutputPin` | GPIO |
| Timing | `DelayNs`, `DelayUs` | Millisecond/microsecond delays |
| Serial Buses | `I2c`, `SpiBus`, `SpiDevice` | Sensor communication |
| Analog | `Adc`, `Dac` | ADC/DAC channels |
| PWM | `SetDutyCycle` | Pulse width modulation |
| CAN, Random | `Can`, `RngCore` | Specialized |

---

## Hardware Overview

embedded-hal is **hardware-agnostic** — it defines behavior, not registers. Your board's HAL crate implements traits for physical peripherals:

| Board | HAL Crate | embedded-hal Support |
|-------|-----------|---------------------|
| ESP32-S3 | `esp-hal` | Via `esp-hal-embedded-hal` adapter or native impl |
| STM32F411 | `stm32f4xx-hal` | Native 1.0 impl |
| RP2040 | `rp2040-hal` | Native impl |

The **adapter pattern** bridges chip HAL types to embedded-hal traits when needed.

---

## Wiring Diagram

Trait-based code doesn't change wiring — physical connections are identical regardless of abstraction layer. See [08-gpio.md](./08-gpio.md) for LED/button wiring.

Conceptual portability:

```
┌─────────────┐     ┌──────────────────┐     ┌─────────────┐
│  BME280     │────►│  Driver crate    │────►│  Your app   │
│  (I2C)      │     │  (embedded-hal)  │     │             │
└─────────────┘     └────────┬─────────┘     └─────────────┘
                             │ trait bounds
              ┌──────────────┼──────────────┐
              ▼              ▼              ▼
         esp-hal       stm32f4xx-hal    rp2040-hal
```

---

## Memory & Register Explanation

Traits don't add runtime memory — monomorphization inlines calls to HAL methods that ultimately write registers. Generic functions may **duplicate code** for each concrete type (code size trade-off).

```rust
// Compiled twice if called with two different pin types — two copies in flash
fn blink<P: OutputPin>(pin: &mut P) { /* ... */ }
```

Use trait objects (`dyn OutputPin`) only when dynamic dispatch is required — rare in embedded due to `dyn` overhead and `no_std` limitations.

---

## HAL Implementation

### Using embedded-hal 1.0 Traits

```rust
use embedded_hal::digital::OutputPin;
use embedded_hal::delay::DelayNs;

/// Blink `count` times — generic over any output pin and delay provider.
/// `led` is mutably borrowed for duration of function — caller retains ownership.
pub fn blink<P, D>(led: &mut P, delay: &mut D, count: u32) -> Result<(), P::Error>
where
    P: OutputPin,
    D: DelayNs,
{
    for _ in 0..count {
        led.set_high()?;           // Trait method — resolves to board HAL
        delay.delay_ms(200);       // 200 ms on
        led.set_low()?;
        delay.delay_ms(200);       // 200 ms off
    }
    Ok(())
}
```

**Ownership/borrowing notes:**

- `&mut P` — exclusive borrow of pin; caller owns `P`.
- `Result<(), P::Error>` — hardware errors vary by HAL; associated type `P::Error`.
- `?` propagates errors without panic.

### ESP32-S3 Usage

```rust
use esp_hal::delay::Delay;
use esp_hal::gpio::{Io, Level, Output};
use esp_hal::peripherals::Peripherals;

let peripherals = Peripherals::take();
let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);
let mut led = Output::new(io.pins.gpio2, Level::Low);
let mut delay = Delay::new(&clocks);

// esp-hal Output may implement OutputPin via embedded-hal trait
embedded_hal::digital::OutputPin::set_high(&mut led).ok();
blink(&mut led, &mut delay, 5).ok();
```

### STM32 Usage (Comparison)

```rust
let mut led = gpioa.pa5.into_push_pull_output();
let mut delay = cp.Systick.delay(&clocks);
blink(&mut led, &mut delay, 5).ok();
```

Same `blink` function — different concrete types.

---

## Bare-Metal Implementation

You can implement traits manually for custom wrappers:

```rust
use embedded_hal::digital::{ErrorType, OutputPin};

pub struct BitBangPin {
    reg_set: *mut u32,
    reg_clear: *mut u32,
    mask: u32,
}

impl OutputPin for BitBangPin {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        // SAFETY: reg_set valid for program lifetime
        unsafe { core::ptr::write_volatile(self.reg_set, self.mask); }
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        unsafe { core::ptr::write_volatile(self.reg_clear, self.mask); }
        Ok(())
    }
}

impl ErrorType for BitBangPin {
    type Error = core::convert::Infallible;  // Cannot fail
}
```

`Infallible` — error type with no variants; `?` compiles away.

---

## Step-by-Step Explanation

### Step 1: Add embedded-hal Dependency

```toml
embedded-hal = "1.0"
```

### Step 2: Write Generic Driver Function

Use trait bounds on function parameters, not concrete pin types.

### Step 3: Verify HAL Implements Traits

Check docs: `esp-hal` GPIO — look for `impl OutputPin for Output`.

If missing, use adapter crate or implement wrapper.

### Step 4: Handle Errors

```rust
match led.set_high() {
    Ok(()) => {}
    Err(e) => defmt::error!("GPIO: {:?}", defmt::Debug2Format(&e)),
}
```

### Step 5: Test on Second Board (Portability Proof)

Compile for two targets with same driver crate — fix trait version mismatches.

### Step 6: Optional Async Migration

```rust
use embedded_hal_async::i2c::I2c;

async fn read_sensor<I: I2c>(i2c: &mut I) -> Result<[u8; 6], I::Error> {
    let mut buf = [0u8; 6];
    i2c.read(0x76, &mut buf).await?;
    Ok(buf)
}
```

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Mixing embedded-hal 0.2 and 1.0 | Trait not implemented errors | Align versions across deps |
| Concrete pin types in driver API | Won't compile on other boards | Use trait bounds |
| Forgetting `&mut` on pin | Cannot call `set_high` | Pins need mutable access |
| Ignoring associated `Error` type | Unhandled error paths | Use `Result` properly |
| `dyn Trait` in `no_std` | Needs allocator for vtables | Stick to generics |
| Not enabling HAL features | Missing trait impls | Check Cargo features |

---

## Debugging Tips

1. **`cargo tree -i embedded-hal`** — find version conflicts.
2. **IDE hover on trait method** — see which impl is used.
3. **Monomorphization bloat** — `cargo bloat` if flash too large.
4. **Create type alias** during debug: `type Led = Output<'static, GpioPin<'static, 2>>;`
5. **Read driver source** — trait bounds document requirements.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| Generics over `dyn` | Static dispatch, inlinable |
| Batch I²C reads | One transaction vs many |
| `DelayNs` from hardware timer | More accurate than busy loop ([10-timers.md](./10-timers.md)) |
| Share one I2c bus via `&mut` | Exclusive access — no mutex needed in single-core |
| Feature-gate unused drivers | Smaller binary |

---

## Exercises

### Exercise 1: Generic Blink Crate

Create a library crate with `blink`, `set`, `clear` functions using `OutputPin`.

### Exercise 2: Error Mapping

Wrap a fallible pin in a newtype that maps all errors to `()` — when is this safe?

### Exercise 3: Dual-Target Build

Compile the same driver for ESP32-S3 and RP2040; document `Cargo.toml` differences.

### Exercise 4: Custom Delay

Implement `DelayNs` using a busy loop; measure accuracy with logic analyzer.

### Exercise 5: Trait Audit

Pick three crates on crates.io; list which embedded-hal traits they require.

---

## References

- [embedded-hal 1.0 docs](https://docs.rs/embedded-hal/1.0.0/embedded_hal/)
- [embedded-hal-async docs](https://docs.rs/embedded-hal-async/)
- [The Embedded Rust Book — HAL](https://docs.rust-embedded.org/book/start/hal.html)
- [Rust Embedded WG driver guidelines](https://github.com/rust-embedded/embedded-hal/blob/master/docs/driver-implementation-guidelines.md)
- [esp-hal-embedded-hal adapter](https://github.com/esp-rs/esp-hal-embedded-hal)

---

*Previous: [06-register-programming.md](./06-register-programming.md) | Next: [08-gpio.md](./08-gpio.md)*
