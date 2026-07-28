# I²C — Inter-Integrated Circuit

**I²C** (I-squared-C, **I2C**) is a **two-wire**, **multi-master**, **multi-slave** bus: **SDA** (data) and **SCL** (clock). Each device has a 7-bit (or 10-bit) **address**.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [gpio.md](./gpio.md)

---

## Theory

Both lines are **open-drain** with **pull-up resistors** (typically 2.2 kΩ–10 kΩ to 3.3 V). The master generates SCL; either party can pull SDA LOW.

Transaction sequence:

1. **START** — SDA falls while SCL high
2. **Address + R/W bit** — 7 bits address, 1 bit direction
3. **ACK** — receiver pulls SDA LOW
4. **Data bytes** — MSB first, ACK per byte
5. **STOP** — SDA rises while SCL high

**Clock stretching:** a slow slave holds SCL LOW until ready — ESP32-S3 hardware handles this.

Common speeds: **100 kHz** (Standard), **400 kHz** (Fast), **1 MHz** (Fast+).

---

## Timing Diagram (ASCII)

Write transaction — address 0x48, write byte 0x3C:

```
SCL  ──┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐
       └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘

SDA  ──┐ S │A6│A5│A4│A3│A2│A1│A0│W│ACK│D7│..│D0│ACK│ P
       └───────────────────────────────────────────────

S = START, P = STOP, W = Write (0), shaded = slave pulls LOW (ACK)
Address byte = 0x48 << 1 | 0 = 0x90 on wire
```

Repeated START (Sr) allows combined write-then-read (common in sensors):

```
SDA  ── S │ addr W │ data │ Sr │ addr R │ data │ P
```

---

## Packet Format

| Field | Bits | Description |
|-------|------|-------------|
| START | — | Bus claim |
| Slave address | 7 | 0x03–0x77 valid range |
| R/W | 1 | 0 = write, 1 = read |
| Data | 8 × N | Payload |
| ACK/NACK | 1 per byte | LOW = ACK |
| STOP | — | Release bus |

**Register map pattern** (most sensors):

```
START | Addr+W | RegIndex | Sr | Addr+R | Data... | STOP
```

---

## Electrical Characteristics

| Parameter | Typical |
|-----------|---------|
| VDD | 3.3 V (ESP32-S3) |
| Pull-ups | 4.7 kΩ @ 100 kHz, 2.2 kΩ @ 400 kHz |
| Max bus capacitance | 400 pF (Standard mode) |
| 5 V devices | Use level shifter (TXS0102, PCA9306) |

Long cables increase capacitance — reduce pull-up value or lower clock speed.

---

## Rust HAL Sketch

```rust
use embedded_hal::i2c::I2c;

const BME280_ADDR: u8 = 0x76;

pub fn read_chip_id<I: I2c>(i2c: &mut I) -> Result<u8, I::Error> {
    let mut id = [0u8];
    // write register pointer, then read — HAL may provide write_read
    i2c.write_read(BME280_ADDR, &[0xD0], &mut id)?;
    Ok(id[0])
}

pub fn scan_bus<I: I2c>(i2c: &mut I) -> heapless::Vec<u8, 16> {
    let mut found = heapless::Vec::new();
    for addr in 0x08u8..=0x77 {
        if i2c.write(addr, &[]).is_ok() {
            let _ = found.push(addr);
        }
    }
    found
}
```

**Ownership:** `&mut I` exclusive borrow ensures no other task uses the bus concurrently. With Embassy, wrap in `Mutex<CriticalSectionRawMutex, I2cDriver>`.

**Compile (ESP32-S3):**

```toml
esp-hal = { features = ["esp32s3", "i2c"] }
heapless = "0.8"
```

---

## Bare-Metal Sketch (Concept)

```rust
// START condition via HAL preferred; bit-bang for learning:
// 1. SDA high, SCL high, SDA low (START)
// 2. Shift 8 bits, sample SDA on SCL rising
// 3. Release SDA for ACK slot, clock once
// 4. STOP: SCL high, SDA low→high
```

See [06-register-programming.md](../06-register-programming.md) for register access patterns.

---

## Example Projects

| Link | Sensor/Device |
|------|---------------|
| [examples/i2c-scanner.md](../examples/i2c-scanner.md) | Bus discovery |
| [examples/sensor-driver.md](../examples/sensor-driver.md) | Generic driver pattern |
| [projects/environmental-monitor.md](../projects/environmental-monitor.md) | BME280 + SHT40 |
| [projects/weather-station.md](../projects/weather-station.md) | Multi-sensor node |

---

## Common Mistakes

1. **Missing pull-ups** — SDA/SCL never reach HIGH.
2. **Wrong address** — ADDR pin changes LSB (0x76 vs 0x77).
3. **Confusing 8-bit vs 7-bit address** in datasheets.
4. **No repeated START** — read fails after write pointer set.
5. **Bus lock after crash** — power-cycle or clock 9 pulses on SCL.

---

## Exercises

1. Run [examples/i2c-scanner.md](../examples/i2c-scanner.md); document found addresses.
2. Read WHO_AM_I from an IMU (e.g., LSM6DSO @ 0x6A).
3. Implement `embedded-hal` driver for a sensor with `write_read`.
4. Measure rise time with 4.7 kΩ vs 2.2 kΩ pull-ups.

---

## References

- [I2C specification (NXP UM10204)](https://www.nxp.com/docs/en/user-guide/UM10204.pdf)
- [embedded-hal I2c trait](https://docs.rs/embedded-hal/latest/embedded_hal/i2c/trait.I2c.html)
- [esp-hal I2C](https://docs.espressif.com/projects/rust/esp-hal/latest/esp32s3/esp_hal/i2c/index.html)
- Lesson: [07-hal.md](../07-hal.md)

---

*Prev: [spi.md](./spi.md) | Next: [can.md](./can.md)*
