# Character LCD — HD44780 16×2 / 20×4

Guide to **HD44780-compatible** character LCDs via **4-bit parallel GPIO** or **I²C backpack (PCF8574)**.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [17-i2c.md](../17-i2c.md)

---

## Working Principle

**HD44780 controller** drives dot-matrix character cells (5×8 font in CGROM). MCU writes commands/data in **4-bit or 8-bit parallel** nibbles. I²C backpacks bit-bang same protocol via PCF8574 expander.

Display: 16×2 = 32 character positions; 20×4 = 80 positions across 4 rows.

---

## Datasheet Notes

| Parameter | Value |
|-----------|-------|
| Controller | HD44780 compatible |
| Supply | 5 V (contrast pot on V0) |
| Interface | Parallel 4-bit or I²C backpack |
| Backlight | LED on/off via backpack P3 |

I²C backpack address typically `0x27` or `0x3F`.

---

## Protocol

### 4-bit Parallel

Pins: RS, EN, D4–D7 (+ RW tied GND). Write nibble high then low with EN strobe.

### I²C Backpack

PCF8574 mapping: P7 RS, P6 RW (read unused), P5 EN, P4 BL, P3–P0 data.

See [08-gpio.md](../08-gpio.md) and [17-i2c.md](../17-i2c.md).

---

## Register Map (HD44780 Commands)

| Cmd | Code | Description |
|-----|------|-------------|
| Clear display | `0x01` | Clear, home |
| Return home | `0x02` | Cursor home |
| Entry mode | `0x06` | Increment cursor |
| Display ON/OFF | `0x0C` | Display on, cursor off |
| Function set | `0x28` | 4-bit, 2 line, 5×8 font |
| Set DDRAM | `0x80` + addr | Cursor position |
| CGRAM | `0x40` + addr | Custom characters |

DDRAM row offsets: line0=0x00, line1=0x40 (16×2); 20×4 uses 0x00, 0x40, 0x14, 0x54.

---

## ESP32-S3 Wiring (I²C Backpack)

```
ESP32-S3          LCD Backpack
────────          ────────────
GPIO8  ─────────► SDA
GPIO9  ─────────► SCL
5V     ─────────► VCC
GND    ─────────► GND
```

Contrast pot on module — adjust for readable blocks.

### 4-bit Direct GPIO

```
RS → GPIO14, EN → GPIO15, D4–D7 → GPIO4–7
```

---

## Swift Driver Sketch

```swift
struct HD44780_I2C<B: I2CBus> {
    var bus: B
    let address: UInt8 = 0x27

    mutating func initDisplay() throws {
        delayMs(50)
        try write4bits(0x03); delayMs(5)
        try write4bits(0x03); delayUs(150)
        try write4bits(0x03)
        try write4bits(0x02) // 4-bit mode
        try command(0x28)    // 2 lines
        try command(0x0C)    // display on
        try command(0x06)    // entry mode
        try clear()
    }

    mutating func print(_ text: String, row: Int, col: Int) throws {
        let base: UInt8 = row == 0 ? 0x80 : 0xC0
        try command(base + UInt8(col))
        for c in text.utf8 { try writeData(c) }
    }

    private mutating func command(_ cmd: UInt8) throws {
        try send(cmd, rs: false)
    }

    private mutating func writeData(_ data: UInt8) throws {
        try send(data, rs: true)
    }

    private mutating func send(_ value: UInt8, rs: Bool) throws {
        let hi = expand(value >> 4, rs: rs, bl: true)
        let lo = expand(value & 0x0F, rs: rs, bl: true)
        try pulse(hi); try pulse(lo)
    }
}
```

---

## Bare-Metal Notes

- Init sequence timing critical — delays after first three `0x03` writes.
- 5 V module with 3.3 V I²C often works (backpack has pull-ups).
- Custom characters: 8 patterns × 8 bytes in CGRAM.

---

## HAL / Protocol-Oriented Driver Notes

`TextDisplay` protocol: `clear()`, `print(row:col:text:)`. Parallel and I²C backends share command layer.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Solid blocks row 0 | Contrast | Adjust V0 pot |
| Blank | Init timing | Lengthen startup delays |
| Wrong address | Backpack variant | Scan 0x27, 0x3F |
| Garbage chars | 8-bit vs 4-bit | Send 0x28 function set |

---

## Example Project

**Debug console:** Redirect log lines to LCD row 0; show BME280 temp on row 1 ([sensors/bme280.md](../sensors/bme280.md)).

---

## References

- [HD44780 Datasheet (Hitachi)](https://www.sparkfun.com/datasheets/LCD/HD44780.pdf)
- [08-gpio.md](../08-gpio.md), [17-i2c.md](../17-i2c.md)
