# Hall Effect Sensors

**Hall sensors** detect **magnetic field** presence — used for proximity, rotation speed, and homing switches.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

**Hall effect:** voltage across conductor proportional to magnetic flux density **B**. Integrated sensors output digital or analog signal when **B** exceeds threshold.

| Type | Output | Example |
|------|--------|---------|
| Unipolar digital | LOW when N pole near | A3144 |
| Latching | Toggle on N, clear on S | US1881 |
| Linear analog | Voltage ∝ B | SS49E |
| Programmable | I²C thresholds | DRV5032 |

---

## Datasheet Key Points (A3144)

| Parameter | Value |
|-----------|-------|
| Supply | 4.5–28 V (module often 3.3–5 V) |
| Output | Open-collector digital |
| Trigger | ~30–50 mT typ. |
| Hysteresis | Built-in |

---

## Protocol

**Digital:** GPIO input with pull-up (open-collector output).

**Analog (SS49E):** ADC ([12-adc.md](../12-adc.md)).

**I²C (DRV5032):** Register-based configuration.

### DRV5032 Register Map (Example)

| Reg | Description |
|-----|-------------|
| `0x00` | Device ID |
| `0x01` | Configuration (polarity, pulse) |
| `0x02` | Status (field detected) |

---

## ESP32-S3 Wiring (A3144 Module)

```
A3144 Module      ESP32-S3
────────────      ────────
VCC  ───────────► 3V3
GND  ───────────► GND
OUT  ───────────► GPIO4  (internal pull-up enabled)
```

Place **south pole** of magnet toward branded face (verify module datasheet — polarity matters for unipolar).

---

## Rust Driver Sketch

```rust
use embedded_hal::digital::InputPin;

pub fn magnet_near<P: InputPin>(hall: &mut P) -> Result<bool, P::Error> {
    Ok(hall.is_low()?) // open-collector: low = active (module dependent)
}
```

Debouncing for mechanical vibration:

```rust
pub struct DebouncedHall<P, DELAY> {
    pin: P,
    delay: DELAY,
    stable_count: u8,
}
```

---

## Bare-Metal Notes

- **Open-collector** requires pull-up (internal OK for short wires).
- **Latching** sensors need alternating poles to reset — know your part.
- Motors and Wi-Fi antennas create **stray fields** — test placement.

---

## HAL / embedded-hal Notes

- `InputPin` for digital Hall
- `OneShot` ADC for linear Hall (position sensing)
- `I2c` for DRV5032

Use [09-interrupts.md](../09-interrupts.md) for revolution counting with encoder + Hall index.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always active | Strong ambient field | Move sensor; reduce motor current |
| Never triggers | Wrong pole / distance | Flip magnet; move closer |
| Bounce | Vibration | Software debounce 5–20 ms |
| 5 V module on GPIO | Overvoltage | Use 3.3 V module or divider |

---

## Example Project: Door Alarm

1. Hall on frame, magnet on door.
2. When separated (field lost), buzz speaker ([speakers.md](./speakers.md)).
3. Log events with RTC timestamp ([rtc.md](./rtc.md)).

---

## Exercises

1. Measure trigger/release distance vs magnet size.
2. Count wheel rotations: one magnet per revolution on GPIO interrupt.
3. Compare A3144 vs DRV5032 I²C status register.

---

## References

- Allegro A3144 datasheet
- TI DRV5032 datasheet
- [08-gpio.md](../08-gpio.md), [encoders.md](./encoders.md)
