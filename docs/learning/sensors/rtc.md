# Real-Time Clock (RTC) — DS3231, PCF8563

**RTC modules** keep **calendar time** with battery backup when MCU is powered off — essential for timestamps, alarms, and scheduling.

**Prerequisites:** [17-i2c.md](../17-i2c.md)

---

## Working Principle

**RTC IC** contains oscillator (32.768 kHz crystal), clock/calendar counters, and **BCD registers** for year/month/day/hour/minute/second. **DS3231** adds temperature-compensated crystal (TCXO) for ±2 ppm accuracy.

---

## Datasheet Key Points (DS3231)

| Parameter | Value |
|-----------|-------|
| I²C address | `0x68` |
| Battery backup | CR2032 via module |
| Accuracy | ±2 ppm (0–40 °C) |
| Alarms | Two programmable |
| Temperature | On-chip sensor (0x11–0x12) |

### PCF8563 (Alternative)

| Address | `0x51` |
| Accuracy | Depends on crystal |
| Lower cost, no TCXO |

---

## Protocol

Standard I²C register read/write. Time stored as **BCD** (Binary-Coded Decimal).

---

## Register Map (DS3231)

| Addr | Name | Description |
|------|------|-------------|
| `0x00` | Seconds | CH bit = oscillator stop |
| `0x01` | Minutes | |
| `0x02` | Hours | 24h mode recommended |
| `0x03` | Day of week | 1–7 |
| `0x04` | Date | |
| `0x05` | Month / century | |
| `0x06` | Year | 00–99 offset from 2000 |
| `0x0E` | Control | EOSC, alarm flags |
| `0x0F` | Status | A1F, A2F, OSF (oscillator fail) |
| `0x11` | Temperature MSB | Signed integer °C |
| `0x12` | Temperature LSB | 0.25 °C resolution |

### BCD Conversion

```rust
fn bcd_to_dec(b: u8) -> u8 { (b & 0x0F) + ((b >> 4) & 0x0F) * 10 }
fn dec_to_bcd(d: u8) -> u8 { ((d / 10) << 4) | (d % 10) }
```

---

## ESP32-S3 Wiring

```
DS3231           ESP32-S3
──────           ────────
VCC  ──────────► 3V3
GND  ──────────► GND
SDA  ──────────► GPIO8
SCL  ──────────► GPIO9
SQW  ──────────► GPIO7 (optional 1 Hz square wave)
```

Install CR2032 on module for backup. **Conflict note:** DS3231 address `0x68` matches MPU-6050 — do not share address on same bus without one device re-addressed (not possible on DS3231) — use separate I²C bus or different RTC.

---

## Rust Driver Sketch

```rust
use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x68;

pub struct DateTime {
    pub sec: u8, pub min: u8, pub hour: u8,
    pub day: u8, pub date: u8, pub month: u8, pub year: u8,
}

pub struct Ds3231<I2C> {
    i2c: I2C,
}

impl<I2C, E> Ds3231<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn read_datetime(&mut self) -> Result<DateTime, E> {
        let mut buf = [0u8; 7];
        self.i2c.write_read(ADDR, &[0x00], &mut buf)?;
        Ok(DateTime {
            sec: bcd_to_dec(buf[0] & 0x7F),
            min: bcd_to_dec(buf[1]),
            hour: bcd_to_dec(buf[2] & 0x3F),
            day: bcd_to_dec(buf[3]),
            date: bcd_to_dec(buf[4]),
            month: bcd_to_dec(buf[5] & 0x1F),
            year: bcd_to_dec(buf[6]),
        })
    }

    pub fn set_datetime(&mut self, dt: &DateTime) -> Result<(), E> {
        let data = [
            0x00,
            dec_to_bcd(dt.sec),
            dec_to_bcd(dt.min),
            dec_to_bcd(dt.hour),
            dec_to_bcd(dt.day),
            dec_to_bcd(dt.date),
            dec_to_bcd(dt.month),
            dec_to_bcd(dt.year),
        ];
        self.i2c.write(ADDR, &data)?;
        Ok(())
    }
}
```

**Crates:** `ds323x`, `pcf8563`.

---

## Bare-Metal Notes

- Clear **OSF** (oscillator stop flag) after power loss before trusting time.
- Set **24-hour mode** (bit 6 of hour reg = 0 in 24h convention per datasheet).
- Sync from **GPS** ([gps.md](./gps.md)) or **NTP** (ESP32 Wi-Fi) periodically.
- SQW pin can provide **1 Hz interrupt** for tick scheduling.

---

## HAL / embedded-hal Notes

```rust
pub trait RealTimeClock {
    type Error;
    fn now(&mut self) -> Result<DateTime, Self::Error>;
    fn set(&mut self, t: &DateTime) -> Result<(), Self::Error>;
}
```

Compose with GPS for automatic set-on-fix.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Year 165 | Oscillator stopped (OSF) | Replace battery; set time |
| Wrong weekday | Not updated | Set day register when setting date |
| I²C conflict @ 0x68 | MPU-6050 on bus | Separate buses or remove conflict |
| Drift (PCF8563) | Cheap crystal | Use DS3231 for accuracy |

---

## Example Project: Data Logger Timestamps

1. DS3231 provides time for BMP280 samples.
2. CSV lines: `2026-07-28 12:00:00,1013.25 hPa`.
3. Store on SPI flash ([flash-memory.md](./flash-memory.md)).

---

## Exercises

1. Implement alarm IRQ on SQW — wake ESP32 from light sleep.
2. Read DS3231 internal temperature; compare with BME280.
3. Sync RTC from GPS `$GPRMC` once fix acquired.

---

## References

- Maxim DS3231 datasheet
- NXP PCF8563 datasheet
- [gps.md](./gps.md), [17-i2c.md](../17-i2c.md)
