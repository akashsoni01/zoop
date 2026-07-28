# Matrix Keypads

**Matrix keypads** (4×4, 4×3) scan **rows and columns** to detect key presses with fewer GPIO pins than individual buttons.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [09-interrupts.md](../09-interrupts.md)

---

## Working Principle

Keys connect row lines to column lines at intersections. Firmware drives one row **LOW** at a time, reads all columns — LOW column indicates pressed key at (row, col).

```
        C0   C1   C2   C3
    R0 [ 1 ] [ 2 ] [ 3 ] [ A ]
    R1 [ 4 ] [ 5 ] [ 6 ] [ B ]
    R2 [ 7 ] [ 8 ] [ 9 ] [ C ]
    R3 [ * ] [ 0 ] [ # ] [ D ]
```

Without diodes, **ghosting** occurs when three keys form a rectangle — software debouncing and anti-ghost diodes mitigate this.

---

## Datasheet / Pinout

Keypads are passive — no IC register map. Document your pin labels:

| Pin | Function |
|-----|----------|
| 8 pins on 4×4 | 4 rows + 4 columns (order varies by vendor) |

Verify with multimeter continuity mode.

---

## Protocol

**GPIO matrix scan** — no serial bus. Typical scan rate 100–500 Hz.

Optional: **I²C PCF8574** backpack on some keypad boards (rare).

---

## ESP32-S3 Wiring (4×4 Direct GPIO)

```
Keypad Row0 ──► GPIO4  (output)
Keypad Row1 ──► GPIO5
Keypad Row2 ──► GPIO6
Keypad Row3 ──► GPIO7
Keypad Col0 ──► GPIO15 (input, pull-up)
Keypad Col1 ──► GPIO16
Keypad Col2 ──► GPIO17
Keypad Col3 ──► GPIO18
```

All columns need **internal or external pull-ups** (10 kΩ).

---

## Rust Driver Sketch

```rust
use embedded_hal::digital::{InputPin, OutputPin};

const KEYMAP: [[char; 4]; 4] = [
    ['1','2','3','A'],
    ['4','5','6','B'],
    ['7','8','9','C'],
    ['*','0','#','D'],
];

pub struct Keypad<R, C, const NR: usize, const NC: usize> {
    rows: R,
    cols: C,
}

impl<R, C, E, const NR: usize, const NC: usize> Keypad<R, C, NR, NC>
where
    R: embedded_hal::digital::OutputPin<Error = E>,
    C: embedded_hal::digital::InputPin<Error = E>,
{
    pub fn scan(&mut self) -> Result<Option<char>, E> {
        for r in 0..NR {
            // drive row r low, others high
            for i in 0..NR {
                if i == r { self.rows.set_low(i)?; } else { self.rows.set_high(i)?; }
            }
            for c in 0..NC {
                if self.cols.is_low(c)? {
                    return Ok(Some(KEYMAP[r][c]));
                }
            }
        }
        Ok(None)
    }
}
```

Use trait objects or macro for pin arrays; **crates:** `keypad-matrix`.

---

## Bare-Metal Notes

- **Debounce:** require stable read for 20 ms before accepting key.
- **Key hold / repeat** — separate timer for long-press.
- Drive inactive rows **HIGH** (or Hi-Z with pull-ups on rows — alternative topology).

---

## HAL / embedded-hal Notes

`OutputPin` + `InputPin` per row/column. No dedicated trait — wrap as:

```rust
pub trait KeyScanner {
    type Error;
    fn poll_key(&mut self) -> Result<Option<char>, Self::Error>;
}
```

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Multiple keys | Ghosting | Add diodes; avoid 3-key chords |
| Missed presses | Scan too slow | Increase scan rate |
| Random keys | Floating columns | Enable pull-ups |
| Wrong character | Row/col swapped | Fix KEYMAP wiring table |

---

## Example Project: PIN Entry Lock

1. 4×4 keypad + SSD1306 masked PIN entry.
2. Compare to stored PIN in flash ([flash-memory.md](./flash-memory.md)).
3. Success → green NeoPixel; fail → red flash.

---

## Exercises

1. Implement debounce state machine with press/release events.
2. Add long-press on `*` for backspace.
3. Measure worst-case scan time vs GPIO count.

---

## References

- [08-gpio.md](../08-gpio.md)
- [encoders.md](./encoders.md) — alternative input
- [touch-sensors.md](./touch-sensors.md) — capacitive alternative
