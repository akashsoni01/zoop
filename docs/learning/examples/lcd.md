# Example: Character LCD (HD44780)

**Goal:** Print strings on 16×2 parallel or I²C backpack LCD.

**Prerequisites:** [08-gpio.md](../08-gpio.md) or [i2c-scanner.md](./i2c-scanner.md)

---

## Wiring (I²C PCF8574 backpack)

```
GPIO8 SDA, GPIO9 SCL → LCD backpack (often 0x27 or 0x3F)
5V power for LCD contrast (some modules 3V3)
```

---

## Rust Sketch

```rust
use hd44780_driver::{HD44780, CursorMode, DisplayMode};
use embedded_hal::i2c::I2c;

fn show<I: I2c>(lcd: &mut HD44780<I>, msg: &str) {
    lcd.reset().unwrap();
    lcd.display_mode(DisplayMode::Normal).unwrap();
    lcd.write_str(msg).unwrap();
}
```

Parallel mode uses 6–10 GPIO lines — prefer I²C backpack for breadboard simplicity.

---

## Compile Notes

`hd44780-driver` crate with `i2c` feature. Long messages need custom scrolling logic.

*Next: [motor-driver.md](./motor-driver.md)*
