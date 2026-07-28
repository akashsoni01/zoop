# Example: OLED Display (SSD1306)

**Goal:** Render text on 128×64 I²C OLED.

**Prerequisites:** [i2c-scanner.md](./i2c-scanner.md), [displays/README.md](../displays/README.md)

---

## Wiring

```
GPIO8 SDA, GPIO9 SCL → OLED module (I²C 0x3C)
3V3, GND
```

---

## Rust Sketch

```rust
use ssd1306::{prelude::*, I2CDisplayInterface, Ssd1306};
use embedded_graphics::{mono_font::ascii::FONT_6X10, text::Text, prelude::*};

fn main() -> ! {
    let mut display = Ssd1306::new(
        I2CDisplayInterface::new(i2c),
        DisplaySize128x64,
        DisplayRotation::Rotate0,
    );
    display.init().unwrap();

    Text::new("Hello ESP32-S3", Point::new(0, 10), FONT_6X10)
        .draw(&mut display).unwrap();
    display.flush().unwrap();
    loop {}
}
```

### Ownership

Display struct owns I²C interface until dropped.

---

## Compile Notes

```toml
ssd1306 = "0.9"
embedded-graphics = "0.8"
```

See [projects/oled-dashboard.md](../projects/oled-dashboard.md).

*Next: [lcd.md](./lcd.md)*
