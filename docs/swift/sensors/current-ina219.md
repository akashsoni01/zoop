# INA219 — I²C Current/Power Monitor

The **Texas Instruments INA219** measures **high-side current**, **bus voltage**, and **power** over I²C.

**Prerequisites:** [17-i2c.md](../17-i2c.md)

---

## Working Principle

Shunt voltage (±320 mV range) and bus voltage (0–26 V) measured by 12-bit ADC. Internal math computes current from programmed calibration register matching shunt resistor value.

---

## Datasheet Notes

| Parameter | Value |
|-----------|-------|
| I²C address | `0x40`–`0x4F` (A0/A1 pins) |
| Shunt voltage LSB | 10 µV |
| Bus voltage LSB | 4 mV |
| Max bus voltage | 26 V |
| Common-mode | 0–26 V (high-side) |

---

## Protocol

Standard **I²C** register file, 16-bit registers big-endian. See [17-i2c.md](../17-i2c.md).

---

## Register Map

| Addr | Name | Description |
|------|------|-------------|
| `0x00` | CONFIG | Reset, PGA, bus ADC, shunt ADC |
| `0x01` | SHUNT_VOLTAGE | Signed shunt reading |
| `0x02` | BUS_VOLTAGE | Bus V, conversion ready bit |
| `0x03` | POWER | Power (requires calibration) |
| `0x04` | CURRENT | Signed current |
| `0x05` | CALIBRATION | Sets current LSB |

### Calibration

`Cal = 0.04096 / (Current_LSB × R_shunt)` where Current_LSB = max expected / 32768.

---

## ESP32-S3 Wiring

```
ESP32-S3          INA219 Breakout
────────          ───────────────
GPIO8  ─────────► SDA
GPIO9  ─────────► SCL
3V3    ─────────► VCC
GND    ─────────► GND

Inline shunt: VIN+ → load+ ; shunt → load-
```

---

## Swift Driver Sketch

```swift
struct INA219<B: I2CBus> {
    var bus: B
    let address: UInt8 = 0x40
    var currentLSB: Float = 0.0001  // 100 µA/bit

    mutating func initDevice(shuntOhms: Float, maxCurrentA: Float) throws {
        currentLSB = maxCurrentA / 32768.0
        let cal = UInt16(0.04096 / (currentLSB * shuntOhms))
        try writeReg(0x05, cal)
        try writeReg(0x00, 0x019F) // 32V, 320mV, 12-bit, continuous
    }

    mutating func readBusVoltage() throws -> Float {
        let raw = try readReg(0x02)
        let shifted = raw >> 3
        return Float(shifted) * 0.004
    }

    mutating func readCurrentMa() throws -> Float {
        let raw = Int16(bitPattern: try readReg(0x04))
        return Float(raw) * currentLSB * 1000.0
    }

    private mutating func readReg(_ reg: UInt8) throws -> UInt16 {
        let d = try bus.read(from: address, register: reg, count: 2)
        return UInt16(d[0]) << 8 | UInt16(d[1])
    }
}
```

---

## Bare-Metal Notes

- Shunt resistor value printed on breakout (often 0.1 Ω).
- High-side sensing: INA219 GND references ESP32 ground; load may be at higher potential on VIN+.
- Reset via CONFIG bit 15 (`0x8000`).

---

## HAL / Protocol-Oriented Driver Notes

Implement `PowerMonitor` protocol: `busVoltage()`, `currentMa()`, `powerMw()`. Calibration computed once at init from shunt value.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Zero current | CAL not written | Recompute calibration |
| Negative current | Reverse flow | Expected for bi-directional |
| Wrong address | A0/A1 straps | Scan 0x40–0x4F |

---

## Example Project

**Solar monitor:** INA219 on panel feed. Log mW to EEPROM; display charge current on ST7789.

---

## References

- [INA219 Datasheet (TI)](https://www.ti.com/lit/ds/symlink/ina219.pdf)
- [17-i2c.md](../17-i2c.md)
