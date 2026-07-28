# Example: Button Input

**Goal:** Read a tactile button with internal pull-up, toggle LED on press.

**Prerequisites:** [blink-led.md](./blink-led.md), [08-gpio.md](../08-gpio.md)

---

## Wiring

```
         3V3 (internal pull-up enabled in software)
          │
    GPIO4 ├─── [Button] ─── GND
```

Active-**low**: pressed = `is_low() == true`.

---

## Rust Sketch

```rust
use esp_hal::gpio::{Input, Pull, Level, Output};

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut led = Output::new(peripherals.GPIO48, Level::Low);
    let button = Input::new(peripherals.GPIO4, Pull::Up);

    let mut led_on = false;
    let mut last_stable = button.is_high();

    loop {
        let raw = button.is_high();
        // Simple debounce: require 20 ms stable (use timer in production)
        if raw == last_stable {
            if button.is_low() && !led_on {
                led_on = true;
                led.set_high().ok();
            } else if button.is_high() && led_on {
                // toggle on release — adjust logic as needed
            }
        }
        last_stable = raw;
        // See interrupts.md for edge-triggered approach
    }
}
```

### Ownership

- `button` is `Input` (immutable borrow for reads); no `mut` needed unless reconfiguring.
- Share state (`led_on`) on stack; for ISR use `StaticCell` or atomic.

---

## Compile Notes

Same target as [blink-led.md](./blink-led.md). Add `embassy-time` for proper debounce timing.

---

## Extensions

- Move debounce to [interrupts.md](./interrupts.md).
- Build [projects/smart-switch.md](../projects/smart-switch.md).

*Prev: [blink-led.md](./blink-led.md) | Next: [interrupts.md](./interrupts.md)*
