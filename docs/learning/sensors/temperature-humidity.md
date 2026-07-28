# Temperature & Humidity Sensors

Guide to common temperature/humidity sensors for Embedded Rust: **DHT11**, **DHT22 (AM2302)**, **AHT20**, **SHT31**, and **DS18B20**.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [10-timers.md](../10-timers.md), [17-i2c.md](../17-i2c.md)

---

## Overview Comparison

| Sensor | Temp Range | Humidity | Interface | Accuracy (typ.) | Notes |
|--------|------------|----------|-----------|-----------------|-------|
| DHT11 | 0–50 °C | 20–90 %RH | Single-wire | ±2 °C, ±5 %RH | Slow, cheap |
| DHT22 | -40–80 °C | 0–100 %RH | Single-wire | ±0.5 °C, ±2 %RH | Better than DHT11 |
| AHT20 | -40–85 °C | 0–100 %RH | I²C | ±0.3 °C, ±2 %RH | Modern, no OTP |
| SHT31 | -40–125 °C | 0–100 %RH | I²C | ±0.2 °C, ±2 %RH | High-end, heater |
| DS18B20 | -55–125 °C | — | 1-Wire | ±0.5 °C | Waterproof probes, no RH |

---

## DHT11 / DHT22 — Working Principle

Both use a ** capacitive humidity sensor** paired with a **NTC thermistor** on one custom ASIC. The MCU sends a start pulse; the sensor responds with a 40-bit frame: humidity (16), temperature (16), checksum (8).

**Timing is everything.** The protocol is not I²C — it is a proprietary single-open-drain bus requiring microsecond-accurate edges. See [08-gpio.md](../08-gpio.md) for open-drain output patterns.

### DHT Frame Format (40 bits)

```
[ RH_int | RH_dec | T_int | T_dec | checksum ]
         checksum = (RH_int + RH_dec + T_int + T_dec) & 0xFF
```

DHT22 sends decimal tenths in the dec bytes; DHT11 often sends zeros in dec bytes.

### ESP32-S3 Wiring (DHT22)

```
ESP32-S3          DHT22 Module
────────          ────────────
GPIO4  ─────────► DATA
3V3    ─────────► VCC
GND    ─────────► GND

(DATA line: 4.7 kΩ – 10 kΩ pull-up to 3V3 — often on module)
```

---

## Deep Dive: DHT22 Protocol Timing

From the Aosong datasheet:

| Phase | Duration | Direction |
|-------|----------|-----------|
| Host start LOW | ≥1 ms | Host → sensor |
| Host release HIGH | 20–40 µs | Host releases |
| Sensor response LOW | ~80 µs | Sensor |
| Sensor response HIGH | ~80 µs | Sensor |
| 40 data bits | ~50 µs LOW + variable HIGH | Sensor |

Each bit: **50 µs LOW**, then **26–28 µs HIGH = 0**, **~70 µs HIGH = 1**.

Minimum interval between reads: **2 seconds** (sensor needs recovery time).

### Rust Driver Sketch (DHT22)

```rust
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};

pub struct Dht22<PIN, DELAY> {
    pin: PIN,
    delay: DELAY,
}

impl<PIN, DELAY, E> Dht22<PIN, DELAY>
where
    PIN: InputPin<Error = E> + OutputPin<Error = E>,
    DELAY: DelayNs,
{
    pub fn read(&mut self) -> Result<(i16, i16), DhtError> {
        // 1. Start pulse
        self.pin.set_low().map_err(DhtError::Pin)?;
        self.delay.delay_ms(2);
        self.pin.set_high().map_err(DhtError::Pin)?;
        self.delay.delay_us(30);

        // 2. Switch to input, wait for sensor ACK edges
        // 3. Sample 40 bits with timeout per bit
        // 4. Verify checksum
        todo!("bit-bang read loop with timeout")
    }
}
```

Use `esp-hal::delay::Delay` or a cycle-counting loop calibrated for your CPU frequency. On ESP32-S3 at 240 MHz, prefer a hardware timer for µs delays rather than busy loops in production.

### Bare-Metal Notes

- Disable interrupts during the 40-bit window if ISR latency exceeds ~10 µs.
- Configure GPIO as open-drain output for start, then input with pull-up for read.
- Implement **timeouts** on every wait-for-edge — a disconnected sensor will hang forever otherwise.

### HAL / embedded-hal Notes

There is no standard `embedded-hal` trait for DHT — drivers take `InputPin + OutputPin + DelayNs`. Crates: `dht-sensor`, `dht-sensor-embedded` (verify `no_std` compatibility for your target).

### Troubleshooting (DHT)

| Symptom | Cause | Fix |
|---------|-------|-----|
| Checksum error | Noise, long wires | Shorten DATA; add 100 nF on VCC |
| Timeout | Wrong pin, no pull-up | Verify wiring; external 10 kΩ |
| NaN / zero humidity | Reading too fast | Wait ≥2 s between reads |
| Works on Arduino, not Rust | Interrupt latency | Short critical section; raise priority |

---

## Deep Dive: AHT20 (I²C)

The **AHT20** (ASair) replaces older DHT-style modules with a proper I²C interface — much easier in Rust.

### Key Datasheet Points

| Parameter | Value |
|-----------|-------|
| I²C address | `0x38` (fixed) |
| VDD | 2.2–5.5 V |
| Conversion time | ~80 ms typical |
| Interface speed | ≤400 kHz |

### Register / Command Map

AHT20 uses **commands**, not a traditional register file:

| Command | Bytes | Purpose |
|---------|-------|---------|
| `0xBE 0x08 0x00` | Init | Calibration enable |
| `0xAC 0x33 0x00` | Trigger | Start measurement |
| `0xBA` | Soft reset | Reset device |
| Read 7 bytes | Status + data | After trigger, when status bit 3 set |

Status byte (first read): bit 3 (`0x08`) = **busy** when set; wait until clear.

### Conversion Formulas (from datasheet)

```text
RH [%] = (raw_humidity / 2^20) × 100
T [°C] = (raw_temp / 2^20) × 200 - 50
```

Raw values extracted from 20-bit fields in the 6 data bytes.

### ESP32-S3 Wiring (AHT20)

```
ESP32-S3          AHT20 Breakout
────────          ───────────────
GPIO8 (SDA) ─────► SDA
GPIO9 (SCL) ─────► SCL
3V3         ─────► VIN
GND         ─────► GND
```

See [17-i2c.md](../17-i2c.md) for bus setup.

### Rust Driver Sketch (AHT20)

```rust
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x38;

pub struct Aht20<I2C, DELAY> {
    i2c: I2C,
    delay: DELAY,
}

impl<I2C, DELAY, E> Aht20<I2C, DELAY>
where
    I2C: I2c<Error = E>,
    DELAY: DelayNs,
{
    pub fn init(&mut self) -> Result<(), E> {
        self.i2c.write(ADDR, &[0xBE, 0x08, 0x00])?;
        self.delay.delay_ms(10);
        Ok(())
    }

    pub fn read(&mut self) -> Result<(f32, f32), E> {
        self.i2c.write(ADDR, &[0xAC, 0x33, 0x00])?;
        self.delay.delay_ms(80);
        let mut buf = [0u8; 7];
        self.i2c.read(ADDR, &mut buf)?;
        while buf[0] & 0x08 != 0 {
            self.delay.delay_ms(10);
            self.i2c.read(ADDR, &mut buf)?;
        }
        let raw_h = ((buf[1] as u32) << 12) | ((buf[2] as u32) << 4) | ((buf[3] as u32) >> 4);
        let raw_t = (((buf[3] as u32) & 0x0F) << 16) | ((buf[4] as u32) << 8) | (buf[5] as u32);
        let rh = (raw_h as f32 / 1_048_576.0) * 100.0;
        let temp = (raw_t as f32 / 1_048_576.0) * 200.0 - 50.0;
        Ok((temp, rh))
    }
}
```

Crates: `aht20` on crates.io (check ESP compatibility).

---

## SHT31 (Brief)

Sensirion **SHT31** offers superior accuracy and an optional **heater** to defeat condensation.

| Item | Value |
|------|-------|
| I²C address | `0x44` (ADDR low) or `0x45` (ADDR high) |
| Measurement cmd | `0x2400` (single shot, high repeatability) |
| CRC | CRC-8 per word (Sensirion polynomial) |

Always verify CRC — Sensirion documents the algorithm in the datasheet application note.

---

## DS18B20 (1-Wire, Temperature Only)

**Maxim/Dallas 1-Wire** bus: single data line, parasitic or external power.

| Item | Value |
|------|-------|
| Resolution | 9–12 bit (configurable) |
| ROM command | `0x55` skip ROM (single device) |
| Convert | `0x44` then wait 750 ms (12-bit) |
| Read scratchpad | `0xBE` → 9 bytes |

Temperature (12-bit): `T = count / 16.0` °C (two's complement).

1-Wire requires precise timing or a dedicated 1-Wire peripheral. On ESP32, bit-banging with critical sections works; consider `one-wire-bus` crate ecosystem.

---

## Example Project: Climate Node

Combine **AHT20** (I²C) with **SSD1306** display ([ssd1306.md](../displays/ssd1306.md)):

1. Sample every 5 s.
2. Show temp/humidity on OLED.
3. If RH > 70 %, log warning via `defmt`.
4. Optional: deep sleep between samples ([ESP32 power docs](https://esp-rs.github.io/book/)).

---

## Exercises

1. **DHT22 bare read** — Implement bit sampling with timeouts; print raw 40 bits over UART.
2. **AHT20 driver** — Full init + read; compare against a known-good sensor.
3. **Multi-sensor** — DS18B20 (outdoor probe) + AHT20 (indoor) on same firmware; different buses.
4. **Dew point** — Compute Magnus formula from AHT20 readings using `micromath`.

---

## References

- Aosong DHT22 datasheet (AM2302)
- ASair AHT20 datasheet
- Sensirion SHT31 datasheet + CRC application note
- Maxim DS18B20 datasheet
- [17-i2c.md](../17-i2c.md) — I²C on ESP32-S3
- [08-gpio.md](../08-gpio.md) — GPIO modes and pull-ups
