# Example: Rotary Encoder

**Goal:** Read quadrature encoder (KY-040) for position/count.

**Prerequisites:** [interrupts.md](./interrupts.md)

---

## Wiring

```
GPIO18 CLK ── encoder A
GPIO19 DT  ── encoder B
GPIO4  SW  ── button (optional)
GND, 3V3
```

---

## Rust Sketch

```rust
use core::sync::atomic::{AtomicI32, Ordering};

static COUNT: AtomicI32 = AtomicI32::new(0);

// In GPIO ISR: read A/B, update COUNT (Gray code decode)
fn decode_quadrature(a: bool, b: bool, last: (bool, bool)) -> i32 {
    // Standard state table — increment/decrement on transitions
    0 // simplified
}

fn main() -> ! {
    loop {
        defmt::info!("Count: {}", COUNT.load(Ordering::Relaxed));
    }
}
```

Prefer **PCNT** hardware peripheral on ESP32-S3 for high-rate encoders.

*Next: [rtc.md](./rtc.md)*
