# Example: Hardware Timers

**Goal:** Blink LED at 1 Hz using a hardware timer interrupt (no busy-wait).

**Prerequisites:** [interrupts.md](./interrupts.md), [10-timers.md](../10-timers.md) if present, [09-interrupts.md](../09-interrupts.md)

---

## Rust Sketch

```rust
use esp_hal::timer::timg::TimerGroup;
use esp_hal::timer::Timer;
use esp_hal::gpio::{Level, Output};

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut led = Output::new(peripherals.GPIO48, Level::Low);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    let mut timer0 = timg0.timer0;
    timer0.set_interrupt_handler(timer_isr);
    timer0.load_value(1_000_000u64).unwrap(); // 1 s @ 1 MHz tick — calibrate
    timer0.start();

    loop {
        // LED toggled in ISR or via flag
    }
}

#[handler]
fn timer_isr() {
    // toggle LED via atomic or flag
}
```

### Ownership

Timer peripheral moved into `timer0`; only one owner configures reload value.

---

## Compile Notes

Calibrate tick rate against `Clocks`. Prefer `embassy-time` with `Timer` for async delays.

*Next: [pwm.md](./pwm.md)*
