# Real-Time Clock — DS3231, PCF8563

Guide to **I²C RTC modules** for battery-backed date/time on Embedded Swift projects.

**Prerequisites:** [17-i2c.md](../17-i2c.md)

---

## Working Principle

**32.768 kHz crystal** drives calendar counter with BCD registers for seconds through year. DS3231 includes **temperature-compensated crystal (TCXO)** for ±2 ppm accuracy. Coin cell (CR2032) maintains time when main power off.

---

## Datasheet Notes

| Parameter | DS3231 |
|-----------|--------|
| I²C address | `0x68` |
| Accuracy | ±2 ppm (0–40 °C) |
| Battery | 2.3–3.6 V backup |
| Alarms | Two programmable |

PCF8563: `0x51`, lower cost, no TCXO.

---

## Protocol

**I²C** register read/write. Time registers in **BCD format**.

See [17-i2c.md](../17-i2c.md).

---

## Register Map (DS3231)

| Addr | Name | Description |
|------|------|-------------|
| `0x00` | Seconds | 0–59 BCD |
| `0x01` | Minutes | 0–59 |
| `0x02` | Hours | 24h mode bit 6 |
| `0x03` | Day of week | 1–7 |
| `0x04` | Date | 1–31 |
| `0x05` | Month / century | |
| `0x06` | Year | 00–99 |
| `0x0F` | Status | OSF (oscillator stop) flag |
| `0x10`–`0x12` | Temperature | On-chip sensor (DS3231) |

---

## ESP32-S3 Wiring

```
ESP32-S3          DS3231 Module
────────          ─────────────
GPIO8  ─────────► SDA
GPIO9  ─────────► SCL
3V3    ─────────► VCC
GND    ─────────► GND
```

Battery holder on module maintains RTC when ESP32 powered off.

---

## Swift Driver Sketch

```swift
struct DateTime: Sendable {
    var year: UInt8, month: UInt8, day: UInt8
    var hour: UInt8, minute: UInt8, second: UInt8
}

struct DS3231<B: I2CBus> {
    var bus: B
    let address: UInt8 = 0x68

    mutating func read() throws -> DateTime {
        let d = try bus.read(from: address, register: 0x00, count: 7)
        return DateTime(
            year: bcdToDec(d[6]),
            month: bcdToDec(d[5] & 0x1F),
            day: bcdToDec(d[4]),
            hour: bcdToDec(d[2] & 0x3F),
            minute: bcdToDec(d[1]),
            second: bcdToDec(d[0] & 0x7F)
        )
    }

    mutating func write(_ dt: DateTime) throws {
        let data: [UInt8] = [
            decToBcd(dt.second), decToBcd(dt.minute), decToBcd(dt.hour),
            1, decToBcd(dt.day), decToBcd(dt.month), decToBcd(dt.year)
        ]
        try bus.write(to: address, register: 0x00, data: data)
    }

    func bcdToDec(_ b: UInt8) -> UInt8 { (b >> 4) * 10 + (b & 0x0F) }
    func decToBcd(_ d: UInt8) -> UInt8 { ((d / 10) << 4) | (d % 10) }
}
```

---

## Bare-Metal Notes

- Check OSF flag on boot — indicates power loss; prompt user to set time.
- DS3231 SQW pin can output 1 Hz square wave for tick interrupt.
- Avoid writing time during read (use burst read).

---

## HAL / Protocol-Oriented Driver Notes

`RealTimeClock` protocol: `now() -> DateTime`, `set(_:)`. PCF8563 implements same protocol with different register layout.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Wrong time after boot | OSF set | Clear OSF; set time |
| I²C NACK | Dead battery | Replace CR2032 |
| Drift (PCF8563) | No TCXO | Use NTP sync periodically |

---

## Example Project

**Data logger timestamps:** DS3231 provides ISO-like timestamps for BME280 samples written to flash.

---

## References

- [DS3231 Datasheet (Maxim)](https://datasheets.maximintegrated.com/en/ds/DS3231.pdf)
- [17-i2c.md](../17-i2c.md)
