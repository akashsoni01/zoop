# Example: Stepper Motor

**Goal:** Drive 28BYJ-48 (ULN2003 driver) or A4988 step/dir stepper.

**Prerequisites:** [motor-driver.md](./motor-driver.md), [timers.md](./timers.md)

---

## Wiring (A4988)

```
GPIO15 STEP ── pulse
GPIO16 DIR  ── direction
GPIO17 EN   ── enable (active low)
VMOT 12V, GND common
```

---

## Rust Sketch

```rust
fn step_motor(step_pin: &mut Output, steps: u32, delay_us: u32) {
    for _ in 0..steps {
        step_pin.set_high().ok();
        esp_hal::delay::Delay::new().delay_micros(2);
        step_pin.set_low().ok();
        esp_hal::delay::Delay::new().delay_micros(delay_us);
    }
}
```

Use timer ISR for smooth acceleration ramps in [projects/robot.md](../projects/robot.md).

*Next: [encoder.md](./encoder.md)*
