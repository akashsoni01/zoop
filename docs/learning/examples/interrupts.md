# Example: GPIO Interrupts

**Goal:** Respond to button edges via ISR instead of polling.

**Prerequisites:** [button.md](./button.md), [09-interrupts.md](../09-interrupts.md)

---

## Rust Sketch

```rust
use core::cell::RefCell;
use critical_section::Mutex;
use esp_hal::gpio::{Input, Pull, Event, PinInterrupt};
use esp_hal::interrupt::InterruptHandler;

static FLAG: Mutex<RefCell<bool>> = Mutex::new(RefCell::new(false));

#[interrupt]
fn GPIO() {
    critical_section::with(|cs| {
        *FLAG.borrow_ref_mut(cs) = true;
    });
}

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut button = Input::new(peripherals.GPIO4, Pull::Up);

    button.listen(Event::NegativeEdge); // press = falling edge
    button.enable_interrupt();

    loop {
        let triggered = critical_section::with(|cs| {
            let mut f = FLAG.borrow_ref_mut(cs);
            let v = *f;
            *f = false;
            v
        });
        if triggered {
            // handle press — keep ISR minimal!
        }
    }
}
```

### Ownership

- ISR cannot capture local variables — use `static` + `Mutex<RefCell<_>>` or atomics (`AtomicBool`).
- `button` must stay alive; disabling drop of pin handle.

---

## Compile Notes

Enable interrupt feature in HAL. Keep ISR under ~10 µs — defer work to main loop or Embassy task.

*Next: [timers.md](./timers.md)*
