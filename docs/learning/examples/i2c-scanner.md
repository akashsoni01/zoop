# Example: I²C Scanner

**Goal:** Detect all 7-bit addresses on the I²C bus.

**Prerequisites:** [17-i2c.md](../17-i2c.md), [communication/i2c.md](../communication/i2c.md)

---

## Wiring

```
GPIO8  ── SDA ──┬── sensors/displays
GPIO9  ── SCL ──┤
3V3, GND ───────┘
External 4.7 kΩ pull-ups if modules lack them
```

---

## Rust Sketch

```rust
use embedded_hal::i2c::I2c;

fn scan<I: I2c>(i2c: &mut I) {
    for addr in 0x08u8..=0x77 {
        match i2c.write(addr, &[]) {
            Ok(()) => defmt::info!("Found 0x{:02X}", addr),
            Err(_) => {}
        }
    }
}

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut i2c = /* I2C::new(peripherals.I2C0, ...) */;
    loop {
        scan(&mut i2c);
        delay.delay_secs(5);
    }
}
```

### Ownership

`&mut I` — exclusive bus access during scan.

---

## Expected Addresses

| Device | Address |
|--------|---------|
| BME280 | 0x76 / 0x77 |
| SSD1306 OLED | 0x3C |
| MPU6050 | 0x68 |

See [sensors/README.md](../sensors/README.md).

*Next: [sensor-driver.md](./sensor-driver.md)*
