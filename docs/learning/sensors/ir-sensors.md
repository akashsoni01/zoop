# Infrared (IR) Sensors

Guide to **reflective IR**, **IR break-beam**, and **IR remote receiver** modules for Embedded Rust.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [12-adc.md](../12-adc.md)

---

## Working Principle

### Reflective (TCRT5000, IR LED + phototransistor)

IR LED emits; phototransistor receives reflected light. Distance to reflective surface modulates current — analog or comparator output.

### Break-Beam

LED and receiver face each other; object blocking beam drops signal.

### IR Remote (TSOP38238, etc.)

38 kHz **modulated** IR; demodulator IC outputs digital pulses encoding NEC/RC5 protocols.

---

## Datasheet Key Points (TCRT5000)

| Parameter | Value |
|-----------|-------|
| Peak wavelength | 950 nm |
| Detector | Phototransistor |
| Output | Analog (variable) or digital (with pot) |
| Range | ~1–12 mm (reflective) |

---

## Protocol

| Type | Interface |
|------|-----------|
| Reflective analog | ADC ([12-adc.md](../12-adc.md)) |
| Reflective digital | GPIO input |
| Break-beam | GPIO input (often open-collector) |
| Remote receiver | GPIO + **interrupt on edge** + protocol decode |

No register map.

---

## ESP32-S3 Wiring

### TCRT5000 Reflective

```
TCRT5000          ESP32-S3
────────          ────────
VCC  ───────────► 3V3
GND  ───────────► GND
DO   ───────────► GPIO4  (digital)
AO   ───────────► GPIO1  (ADC, optional)
```

### TSOP38238 IR Receiver

```
TSOP38238         ESP32-S3
─────────         ────────
VCC  ───────────► 3V3
GND  ───────────► GND
OUT  ───────────► GPIO5  (interrupt pin)
```

Add 100 µF + 100 nF on VCC — IR bursts draw pulse current.

---

## Rust Driver Sketch

### Digital Line Sensor

```rust
use embedded_hal::digital::InputPin;

pub fn line_detected<P: InputPin>(sensor: &mut P) -> Result<bool, P::Error> {
    Ok(sensor.is_low()?) // often low = more reflection (module dependent)
}
```

### NEC Remote Decode (simplified)

```rust
pub struct IrNecDecoder {
    last_edge_us: u32,
    bits: u32,
    bit_count: u8,
}

// On GPIO interrupt: measure pulse widths
// Leader: 9 ms low + 4.5 ms high
// Bit 0: 562.5 µs low + 562.5 µs high
// Bit 1: 562.5 µs low + 1687.5 µs high
```

**Crates:** `infrared`, `rpi-pal`-style IR crates — search `nec` + `embedded-hal`.

---

## Bare-Metal Notes

- **Ambient IR** (sunlight) saturates reflective sensors — shield or use modulated schemes.
- TSOP38xx requires **38 kHz carrier** — raw IR LED on/off won't trigger remote receiver.
- Debounce break-beam sensors (mechanical vibration).

---

## HAL / embedded-hal Notes

- `InputPin` for digital modules
- `OneShot` ADC for analog distance/proximity
- `embedded_hal::digital::InputPin` + timer for pulse measurement on IR remote

Use [09-interrupts.md](../09-interrupts.md) for edge-triggered NEC decode.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always triggered | Pot too sensitive | Adjust module potentiometer |
| No remote decode | Wrong carrier freq | Use 38 kHz TSOP; verify with camera (LED visible) |
| Short range | Low LED current | Check current-limit resistor on emitter |
| ADC noisy | PWM nearby | Filter in software; shield sensor |

---

## Example Project: Line-Following Robot

1. Three TCRT5000 digital outputs → GPIO.
2. Read at 200 Hz; PID on motor PWM ([11-pwm.md](../11-pwm.md)).
3. Display line state on SSD1306.

---

## Exercises

1. Calibrate analog TCRT5000 vs distance (mm) curve.
2. Decode NEC remote button codes; map to NeoPixel colors.
3. Measure break-beam response time with logic analyzer.

---

## References

- Vishay TCRT5000 datasheet
- Vishay TSOP38238 datasheet
- NEC infrared protocol timing diagrams
- [12-adc.md](../12-adc.md), [09-interrupts.md](../09-interrupts.md)
