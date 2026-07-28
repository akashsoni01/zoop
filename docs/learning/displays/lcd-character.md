# Character LCD (HD44780-Style)

**16×2** and **20×4** alphanumeric LCDs use **HD44780** controller (or compatible) — parallel **4-bit** mode or **I²C backpack** (PCF8574).

**Prerequisites:** [08-gpio.md](../08-gpio.md), [17-i2c.md](../17-i2c.md)

---

## Theory

**Character LCD** displays fixed **5×8 dot font** characters in a grid (not arbitrary graphics like OLED). HD44780 **DDRAM** stores character codes; **CGRAM** allows 8 custom glyphs.

Controller handles character generation — host sends ASCII + commands.

Common sizes:

| Columns × Rows | DDRAM addresses |
|----------------|-----------------|
| 16×2 | 2 lines × 16 |
| 20×4 | 4 lines (non-contiguous addr map) |

---

## Wiring

### I²C Backpack (Recommended)

```
LCD I²C Backpack    ESP32-S3
────────────────    ────────
VCC  ─────────────► 5V (LCD needs 5V contrast circuit)
GND  ─────────────► GND
SDA  ─────────────► GPIO8
SCL  ─────────────► GPIO9
```

PCF8574 backpack address typically **`0x27`** or **`0x3F`**.

Pin mapping on backpack:

| PCF8574 bit | LCD pin |
|-------------|---------|
| P0 | RS |
| P1 | RW (often tied low) |
| P2 | E (enable) |
| P3 | Backlight |
| P4–P7 | D4–D7 (4-bit mode) |

### Direct 4-Bit Parallel (GPIO Heavy)

```
LCD Pin    ESP32-S3
───────    ────────
RS    ───► GPIO4
E     ───► GPIO5
D4-D7 ───► GPIO12-15
VSS   ───► GND
VDD   ───► 5V
VO    ───► contrast pot wiper
```

See [08-gpio.md](../08-gpio.md) — needs 6+ GPIO vs 2 for I²C.

```
                    ┌─────────────┐
              VDD ──┤ VDD     VSS ├── GND
         contrast ──┤ VO          │
                    │  RS  E  D4-D7
                    └──┬───┬───┬──
                       │   │   └── GPIO12-15
                    GPIO4 GPIO5
```

---

## Protocol

### 4-Bit Nibble Transfer

1. Set RS (0=command, 1=data)
2. Put high nibble on D4–D7
3. Pulse E HIGH ≥450 ns
4. Put low nibble on D4–D7
5. Pulse E again

### Init Sequence (4-bit)

```text
Power wait 50 ms
0x03 ×3 (8-bit wake) with delays
0x02 (switch 4-bit)
Function set: 4-bit, 2 lines, 5x8 font
Display on, cursor off
Entry mode: increment
Clear display
```

### I²C Backpack

Same nibbles, but written via PCF8574 byte with backlight bit — **slow** (~100 kHz effective) but simple.

---

## Framebuffer Concepts

No pixel framebuffer — **character buffer** abstraction:

```rust
struct LcdBuffer {
    lines: [heapless::String<21>; 4],
}
```

Track dirty lines; rewrite only changed line on update (reduces flicker).

Custom CGRAM: upload 8 glyphs (degree symbol, icons) once at init.

---

## Rust + embedded-graphics

`embedded-graphics` targets pixel displays — HD44780 use **`hd44780`** or **`lcd-display-driver`** crates instead:

```toml
[dependencies]
hd44780 = "0.8"
embedded-hal = "1.0"
```

I²C backpack via `pcf8574` expander:

```rust
use hd44780::Hd44780;
use pcf8574::OutputPinExpander;

let mut lcd = Hd44780::new(expander, DelayMs);
lcd.reset()?;
lcd.display(Hd44780Mode::On, Hd44780Cursor::Off, Hd44780Blink::Off)?;
lcd.write_str("Hello, Rust!")?;
```

For `embedded-graphics` integration, use **`embedded-graphics-hd44780`** (community) for minimal pixel-style API on custom CGRAM tiles — limited resolution.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Wrong I²C address | No text | Scan 0x27, 0x3F |
| 3.3 V on 5V LCD VDD | Dim/no display | 5V supply (I²C backpack OK at 5V) |
| Contrast not adjusted | Blank blue | Tune VO potentiometer |
| 4-bit init skipped | Garbage blocks | Follow 0x03/0x02 sequence |
| Line 3 wrap wrong on 20×4 | Text split wrong | Use correct DDRAM line offsets |
| Backlight always on | High current | Bit 3 on PCF8574 control |

### 20×4 Line Addresses

| Line | DDRAM start |
|------|-------------|
| 0 | 0x00 |
| 1 | 0x40 |
| 2 | 0x14 |
| 3 | 0x54 |

---

## Exercises

1. **Scrolling marquee** — Shift string on 16×2 without full clear.
2. **Custom glyph** — Upload °C symbol in CGRAM; show temperature from BMP280.
3. **Parallel vs I²C** — Compare update latency for full screen refresh.
4. **Menu UI** — Rotary encoder ([encoders.md](../sensors/encoders.md)) navigates LCD menu with `>` cursor.

---

## Example Project: GPS Status Line

[gps.md](../sensors/gps.md) on 20×4 LCD:

- Line 1: UTC time
- Line 2: Lat/lon
- Line 3: Satellites + HDOP
- Line 4: Fix status

Minimal RAM; runs alongside heavy TFT on same firmware.

---

## References

- Hitachi HD44780 datasheet
- PCF8574 I/O expander datasheet
- [17-i2c.md](../17-i2c.md), [08-gpio.md](../08-gpio.md)
- `hd44780` crate documentation
