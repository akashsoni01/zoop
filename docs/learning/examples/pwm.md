# Example: PWM — LED Dimming & Servo

**Goal:** Generate PWM on LEDC peripheral — fade LED or drive SG90 servo.

**Prerequisites:** [timers.md](./timers.md), [11-pwm.md](../11-pwm.md) if present

**Protocol:** [gpio.md](../communication/gpio.md)

---

## Wiring (Servo SG90)

```
GPIO5 (LEDC) ─── signal (orange)
5V ──────────── power (red)    — use external 5V if servo draws >500 mA
GND ─────────── common (brown)
```

---

## Rust Sketch

```rust
use esp_hal::ledc::{Ledc, LowSpeed, channel, timer};

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let ledc = Ledc::new(peripherals.LEDC);

    let timer = ledc.timer0;
    timer.configure(timer::Config {
        duty: timer::Duty::Duty5Bit,
        frequency: 50.Hz(),  // servo: 50 Hz
        ..Default::default()
    }).unwrap();

    let mut channel = ledc.channel0.configure(
        peripherals.GPIO5,
        channel::Config { timer: timer, ..Default::default() },
    ).unwrap();

    // Servo: 1 ms = 0°, 2 ms = 180° (pulse width in duty ticks)
    channel.set_duty(75).unwrap(); // mid position — tune for your timer config

    loop {}
}
```

### Ownership

`Ledc` splits into timer + channel; pin consumed by `configure`.

---

## Compile Notes

Frequency/duty resolution trade-off on LEDC. See [servo.md](./servo.md) for angles.

*Next: [adc.md](./adc.md)*
