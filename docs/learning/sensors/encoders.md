# Rotary Encoders

**Quadrature encoders** output **A** and **B** phase signals 90° apart — direction and position from edge sequence.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [09-interrupts.md](../09-interrupts.md)

---

## Working Principle

As shaft rotates, channels A and B toggle. **Gray code** sequence indicates direction:

| Edge | A | B | Direction (typ.) |
|------|---|---|------------------|
| CW step 1 | ↑ | 0 | ... |
| CW step 2 | 1 | ↑ | ... |

**Detents** on mechanical encoders give tactile clicks (often 20 steps/revolution).

Types:

| Type | Output |
|------|--------|
| Mechanical | Quadrature GPIO |
| Optical | Quadrature GPIO |
| Magnetic (AS5600) | I²C angle |

---

## Datasheet Key Points (Mechanical)

| Parameter | Typical |
|-----------|---------|
| Pulses per revolution | 20 (with detents) |
| Bounce time | 1–5 ms |
| Operating voltage | 3.3–5 V |

No register map for basic mechanical encoders.

### AS5600 Magnetic (I²C)

| Addr | `0x36` |
| Reg `0x0C`–`0x0D` | Raw angle 12-bit |

---

## Protocol

**GPIO quadrature:** poll or interrupt on both channels.

**Encoder state machine:** 4-bit state `(A,B,prev)` lookup table for robust decoding (Ben Buxton algorithm).

---

## ESP32-S3 Wiring (Mechanical)

```
Encoder          ESP32-S3
───────          ────────
VCC  ──────────► 3V3
GND  ──────────► GND
CLK (A) ───────► GPIO4  (interrupt)
DT  (B) ───────► GPIO5
SW  (btn) ─────► GPIO6  (optional push)
```

Add **hardware debounce** (1 nF + 10 kΩ) for noisy encoders.

---

## Rust Driver Sketch

```rust
use core::sync::atomic::{AtomicI32, Ordering};

static COUNT: AtomicI32 = AtomicI32::new(0);

// Lookup table: 16 entries for prev<<2 | curr
const TABLE: [i8; 16] = [0, -1, 1, 0, 1, 0, 0, -1, -1, 0, 0, 1, 0, 1, -1, 0];

pub struct QuadratureDecoder {
    state: u8,
}

impl QuadratureDecoder {
    pub fn update(&mut self, a: bool, b: bool) -> i32 {
        let curr = ((a as u8) << 1) | (b as u8);
        let idx = (self.state << 2) | curr;
        let delta = TABLE[idx as usize] as i32;
        self.state = curr;
        COUNT.fetch_add(delta, Ordering::Relaxed);
        delta
    }
}
```

Call from GPIO ISR on **both** A and B edges ([09-interrupts.md](../09-interrupts.md)).

**Crates:** `rotary-encoder-embedded`, `encoders`.

---

## Bare-Metal Notes

- **Interrupt both channels** for full resolution (4× PPR).
- ISR must be **short** — increment counter only; no `defmt` in ISR.
- Mechanical bounce causes false counts — combine table decoder with minimum edge interval.

---

## HAL / embedded-hal Notes

```rust
pub trait PositionSource {
    type Error;
    fn position(&mut self) -> Result<i32, Self::Error>;
}
```

For AS5600:

```rust
use embedded_hal::i2c::I2c;
// read 0x0C angle register, unwrap 0-4095 to degrees
```

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Count skips | Missing interrupt | Enable both pins |
| Wrong direction | A/B swapped | Swap wires or invert delta |
| Jitter at rest | Bounce | Capacitor debounce; ignore sub-threshold |
| No counts | Open drain module | Enable pull-ups |

---

## Example Project: Menu Navigator

1. Encoder adjusts setting on SSD1306 ([ssd1306.md](../displays/ssd1306.md)).
2. Press SW to confirm.
3. Wrap count for brightness 0–255 ([11-pwm.md](../11-pwm.md)).

---

## Exercises

1. Implement Buxton decoder; verify 20 detents = 80 counts (4× mode).
2. Add velocity: larger steps when spinning fast.
3. Read AS5600 absolute angle over I²C for knob with no mechanical limit.

---

## References

- Ben Buxton — "How to Build a Rotary Encoder Interface"
- AMS AS5600 datasheet
- [09-interrupts.md](../09-interrupts.md), [08-gpio.md](../08-gpio.md)
