# Matrix Keypads — 4×4 and Variants

Guide to **membrane matrix keypads** scanned via GPIO for Embedded Swift.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

Keys arranged in **row/column matrix**. No key pressed: rows and columns isolated. Key press connects one row to one column. MCU drives rows low one at a time, reads columns to detect intersection.

4×4 keypad: 4 rows + 4 columns = 8 GPIO pins (16 keys).

---

## Datasheet Notes

| Parameter | Typical 4×4 |
|-----------|-------------|
| Pins | 8 (4 row, 4 col) |
| Contact | Momentary short |
| Debounce | 5–20 ms software |
| No diodes | Ghosting on multi-press without diodes |

---

## Protocol

**GPIO matrix scan** — no registers. Algorithm:

1. Set all rows inputs with pull-up
2. For each row: drive low, read all columns
3. Map (row, col) to key code

See [08-gpio.md](../08-gpio.md).

---

## Register Map

Not applicable.

---

## ESP32-S3 Wiring

```
ESP32-S3          4×4 Keypad
────────          ──────────
GPIO4  ─────────► R1
GPIO5  ─────────► R2
GPIO6  ─────────► R3
GPIO7  ─────────► R4
GPIO15 ◄───────── C1
GPIO16 ◄───────── C2
GPIO17 ◄───────── C3
GPIO18 ◄───────── C4
```

Pin mapping is flexible — match your scan code.

---

## Swift Driver Sketch

```swift
protocol DigitalPin {
    mutating func setOutput() throws
    mutating func setInputPullUp() throws
    mutating func setHigh() throws
    mutating func setLow() throws
    mutating func read() throws -> Bool
}

struct Keypad4x4<R: DigitalPin, C: DigitalPin> {
    var rows: [R]   // 4 row pins
    var cols: [C]   // 4 col pins
    let keymap: [[Character]] = [
        ["1","2","3","A"],
        ["4","5","6","B"],
        ["7","8","9","C"],
        ["*","0","#","D"]
    ]

    mutating func scan() throws -> Character? {
        for r in 0..<4 {
            for i in 0..<4 where i != r { try rows[i].setInputPullUp() }
            try rows[r].setOutput()
            try rows[r].setLow()
            for c in 0..<4 {
                if try !cols[c].read() { return keymap[r][c] }
            }
        }
        return nil
    }
}
```

---

## Bare-Metal Notes

- Debounce: require stable reading for 10 ms before accepting.
- Without diodes, only one key at a time is reliable.
- Optional: use column interrupts + row scan for lower CPU use.

---

## HAL / Protocol-Oriented Driver Notes

`Keypad` protocol returns optional key with timestamp. Matrix dimensions and keymap injected at init.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Multiple keys | Ghosting | Scan one key; add diodes |
| Missed presses | No debounce | Add 10 ms debounce |
| Wrong key | Row/col swap | Verify pin map |

---

## Example Project

**PIN entry lock:** 4×4 keypad + SSD1306 masked PIN. Compare hash; unlock GPIO relay on match.

---

## References

- [08-gpio.md](../08-gpio.md)
