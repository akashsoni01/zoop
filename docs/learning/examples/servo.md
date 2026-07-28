# Example: Servo Motor (SG90)

**Goal:** Position servo 0–180° with 50 Hz PWM pulse width.

**Prerequisites:** [pwm.md](./pwm.md)

---

## Rust Sketch

```rust
fn angle_to_duty(angle: u8) -> u32 {
    // Map 0–180° → pulse 1–2 ms @ 50 Hz (depends on LEDC resolution)
    let min_duty = 26;  // tune empirically
    let max_duty = 128;
    min_duty + (angle as u32 * (max_duty - min_duty) / 180)
}

fn set_servo(channel: &mut LedcChannel, angle: u8) {
    channel.set_duty(angle_to_duty(angle)).ok();
}
```

Power servos from **external 5V** — do not back-feed USB.

*Next: [stepper.md](./stepper.md)*
