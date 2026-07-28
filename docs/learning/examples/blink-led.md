# Example: Blink LED

**Goal:** Toggle the onboard or external LED — your first embedded Rust program.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [07-hal.md](../07-hal.md)

**Protocol:** [gpio.md](../communication/gpio.md)

---

## Wiring (ASCII)

```
ESP32-S3 DevKitC-1 (external LED option):

    GPIO2 ────[ 330Ω ]────|>|──── 3V3
                          LED
    GND ──────────────────────── (cathode to GPIO via resistor if active-low)
```

Onboard RGB LED on many DevKitC-1 boards: **GPIO48** (check silkscreen).

---

## Rust Sketch

```rust
#![no_std]
#![no_main]

use esp_hal::clock::Clocks;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output};
use esp_hal::prelude::*;

#[esp_hal::main]
fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let clocks = Clocks::get();

    // Ownership: `led` takes ownership of GPIO48 pin config.
    // It cannot be copied — Rust prevents double-use of the peripheral.
    let mut led = Output::new(peripherals.GPIO48, Level::Low);

    let mut delay = Delay::new(&clocks);

    loop {
        led.set_high();              // embedded-hal OutputPin
        delay.delay_millis(500);
        led.set_low();
        delay.delay_millis(500);
    }
}
```

### Ownership Notes

- `peripherals.GPIO48` is **moved** into `Output::new` — you cannot also bind that pin to I²C.
- `delay` borrows `clocks` immutably; `led` is independent.
- Infinite `loop` returns `!` (never type) — valid for `main` in `no_std`.

---

## Compile & Flash

```bash
# From esp-template project
cargo build --release
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/blink
```

**Common build errors:**

| Error | Fix |
|-------|-----|
| `GPIO48` not found | Enable correct chip feature in `esp-hal` |
| Linker error | Run `espup install` and source export script |
| LED doesn't blink | Wrong pin — try GPIO2 or check schematic |

---

## Extensions

- Replace delay with [timers.md](./timers.md) interrupt.
- Add [button.md](./button.md) to toggle blink rate.

*Next: [button.md](./button.md)*
