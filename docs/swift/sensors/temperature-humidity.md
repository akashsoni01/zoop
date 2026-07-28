# Temperature & Humidity Sensors

Guide to common temperature/humidity sensors for Embedded Swift: **DHT11**, **DHT22 (AM2302)**, **AHT20**, **SHT31**, and **DS18B20**.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [17-i2c.md](../17-i2c.md)

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

Both use a **capacitive humidity sensor** paired with an **NTC thermistor** on one custom ASIC. The MCU sends a start pulse; the sensor responds with a 40-bit frame: humidity (16), temperature (16), checksum (8).

**Timing is everything.** The protocol is not I²C — it is a proprietary single-open-drain bus requiring microsecond-accurate edges. See [08-gpio.md](../08-gpio.md) for open-drain output patterns.

### DHT Frame Format (40 bits)

```
[ RH_int | RH_dec | T_int | T_dec | checksum ]
         checksum = (RH_int + RH_dec + T_int + T_dec) & 0xFF
```

---

## Datasheet Notes (DHT22)

| Parameter | Value |
|-----------|-------|
| Supply | 3.3–5 V (use 3.3 V on ESP32-S3) |
| Start pulse LOW | ≥1 ms |
| Min interval between reads | 2 s |
| Pull-up | 4.7 kΩ – 10 kΩ on DATA |

Each bit: **50 µs LOW**, then **26–28 µs HIGH = 0**, **~70 µs HIGH = 1**.

---

## Protocol (DHT22)

| Phase | Duration | Direction |
|-------|----------|-----------|
| Host start LOW | ≥1 ms | Host → sensor |
| Host release HIGH | 20–40 µs | Host releases |
| Sensor response LOW | ~80 µs | Sensor |
| Sensor response HIGH | ~80 µs | Sensor |
| 40 data bits | ~50 µs LOW + variable HIGH | Sensor |

---

## Register Map

DHT devices have **no register map** — data arrives as a single 40-bit serial frame. I²C sensors (AHT20, SHT31) use standard register addressing (see below).

### AHT20 Register Map

| Addr | Name | Description |
|------|------|-------------|
| `0x00` | Status | Busy bit 7, calibrated bit 3 |
| `0xBE` | Init | Soft reset / calibration trigger |
| `0xAC` | Trigger | Start measurement command |
| `0x71` | Reset | Soft reset |

### SHT31 Register Map

| Addr | Name | Description |
|------|------|-------------|
| `0x2400` | Measure (high rep) | Single-shot, clock stretch |
| `0x306D` | Status | Alert flags |
| `0x30A2` | Soft reset | Reset device |

### DS18B20 (1-Wire)

| Cmd | Value | Description |
|-----|-------|-------------|
| `0xCC` | Skip ROM | Single device on bus |
| `0x44` | Convert T | Start temperature conversion |
| `0xBE` | Read scratchpad | 9 bytes: temp + config + CRC |

---

## ESP32-S3 Wiring

### DHT22

```
ESP32-S3          DHT22 Module
────────          ────────────
GPIO4  ─────────► DATA
3V3    ─────────► VCC
GND    ─────────► GND
```

### AHT20 / SHT31 (I²C)

```
ESP32-S3          Sensor
────────          ──────
GPIO8  ─────────► SDA
GPIO9  ─────────► SCL
3V3    ─────────► VCC
GND    ─────────► GND
```

See [17-i2c.md](../17-i2c.md) for bus setup.

### DS18B20 (1-Wire on GPIO)

```
ESP32-S3          DS18B20
────────          ───────
GPIO5  ─────────► DQ (4.7 kΩ pull-up to 3V3)
3V3    ─────────► VCC
GND    ─────────► GND
```

---

## Swift Driver Sketch (DHT22)

```swift
protocol DigitalPin {
    mutating func setHigh() throws
    mutating func setLow() throws
    mutating func read() throws -> Bool
    mutating func setInputPullUp() throws
    mutating func setOutputOpenDrain() throws
}

protocol MicroDelay {
    func delayMicroseconds(_ us: UInt32)
    func delayMilliseconds(_ ms: UInt32)
}

enum DHTError: Error { case checksum, timeout, pin }

struct DHT22<P: DigitalPin, D: MicroDelay> {
    var pin: P
    var delay: D

    mutating func read() throws -> (tempC: Float, humidity: Float) {
        try pin.setOutputOpenDrain()
        try pin.setLow()
        delay.delayMilliseconds(2)
        try pin.setHigh()
        delay.delayMicroseconds(30)
        try pin.setInputPullUp()

        // Wait for sensor ACK edges with timeout
        var bits: UInt64 = 0
        for _ in 0..<40 {
            try waitForLow(timeoutUs: 100)
            let highUs = measureHighPulse()
            bits = (bits << 1) | (highUs > 40 ? 1 : 0)
        }

        let rh = UInt16((bits >> 32) & 0xFFFF)
        let temp = UInt16((bits >> 16) & 0xFFFF)
        let cs = UInt8(bits & 0xFF)
        let sum = UInt8((rh >> 8) + (rh & 0xFF) + (temp >> 8) + (temp & 0xFF))
        guard cs == sum else { throw DHTError.checksum }

        return (Float(temp) / 10.0, Float(rh) / 10.0)
    }

    private mutating func waitForLow(timeoutUs: UInt32) throws { /* edge wait */ }
    private mutating func measureHighPulse() -> UInt32 { 0 }
}
```

### Swift Driver Sketch (AHT20)

```swift
protocol I2CBus {
    func write(to address: UInt8, register: UInt8, data: [UInt8]) throws
    func read(from address: UInt8, register: UInt8, count: Int) throws -> [UInt8]
}

struct AHT20<B: I2CBus> {
    let bus: B
    let address: UInt8 = 0x38

    mutating func read() throws -> (tempC: Float, humidity: Float) {
        try bus.write(to: address, register: 0xAC, data: [0x33, 0x00])
        delayMs(80)
        let raw = try bus.read(from: address, register: 0x00, count: 6)
        let h = (UInt32(raw[1]) << 12) | (UInt32(raw[2]) << 4) | (UInt32(raw[3]) >> 4)
        let t = ((UInt32(raw[3]) & 0x0F) << 16) | (UInt32(raw[4]) << 8) | UInt32(raw[5])
        let rh = Float(h) * 100.0 / 1_048_576.0
        let temp = Float(t) * 200.0 / 1_048_576.0 - 50.0
        return (temp, rh)
    }
}
```

---

## Bare-Metal Notes

- Disable interrupts during the DHT 40-bit window if ISR latency exceeds ~10 µs.
- Configure GPIO as open-drain output for DHT start, then input with pull-up for read.
- Implement **timeouts** on every wait-for-edge — a disconnected sensor hangs forever otherwise.
- DS18B20 requires strict 1-Wire timing; bit slots are 60–120 µs.

---

## HAL / Protocol-Oriented Driver Notes

There is no standard trait for DHT — drivers take `DigitalPin + MicroDelay`. I²C sensors depend on `I2CBus` with register-oriented read/write helpers. Keep compensation math in pure Swift structs (no hardware deps) for unit testing on host.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Checksum error (DHT) | Noise, long wires | Shorten DATA; add 100 nF on VCC |
| Timeout (DHT) | Wrong pin, no pull-up | Verify wiring; external 10 kΩ |
| NaN / zero humidity | Reading too fast | Wait ≥2 s between DHT reads |
| AHT20 always busy | Missing init | Send `0xBE 0x08 0x00` once at boot |
| DS18B20 85 °C | Power parasite mode issue | Use external 3.3 V supply |
| I²C NACK | Wrong address | Scan bus per [17-i2c.md](../17-i2c.md) |

---

## Example Project

**Indoor climate node:** AHT20 + SSD1306 ([displays/ssd1306.md](../displays/ssd1306.md)). Sample every 5 s, show temp/humidity, log min/max over 24 h. Use DS18B20 on a second GPIO for outdoor probe comparison.

---

## References

- [Aosong DHT22 datasheet](https://www.sparkfun.com/datasheets/Sensors/Temperature/DHT22.pdf)
- [ASAIR AHT20 datasheet](https://files.seeedstudio.com/wiki/Grove-AHT20_I2C_Industrial_Grade_Temperature_and_Humidity_Sensor/AHT20-datasheet-2020-4-16.pdf)
- [Sensirion SHT31 datasheet](https://wwwSensirion.com/file/datasheet_sht3x.pdf)
- [Maxim DS18B20 datasheet](https://datasheets.maximintegrated.com/en/ds/DS18B20.pdf)
- [08-gpio.md](../08-gpio.md) — bit-banging patterns
- [17-i2c.md](../17-i2c.md) — I²C wiring on ESP32-S3
