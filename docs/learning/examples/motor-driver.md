# Example: DC Motor Driver (H-Bridge)

**Goal:** Control direction and speed of brushed DC motor via L298N or DRV8833.

**Prerequisites:** [pwm.md](./pwm.md), [08-gpio.md](../08-gpio.md)

---

## Wiring (DRV8833)

```
GPIO12 IN1 ── direction A
GPIO13 IN2 ── direction B
GPIO14 PWM ── speed (LEDC)
Motor terminals ── OUT1, OUT2
VMotor 6–12V, common GND with ESP32
```

---

## Rust Sketch

```rust
fn motor_forward(speed: u8, in1: &mut Output, in2: &mut Output, pwm: &mut LedcChannel) {
    in1.set_high().ok();
    in2.set_low().ok();
    pwm.set_duty(speed).ok();
}

fn motor_stop(in1: &mut Output, in2: &mut Output) {
    in1.set_low().ok();
    in2.set_low().ok();
}
```

### Ownership

Each GPIO consumed once; PWM channel separate handle.

---

## Safety

Add flyback diodes if not on module. Never stall motor at full duty without current limit.

See [projects/motor-controller.md](../projects/motor-controller.md).

*Next: [servo.md](./servo.md)*
