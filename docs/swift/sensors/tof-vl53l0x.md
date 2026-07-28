# VL53L0X — Time-of-Flight Distance Sensor

The **STMicro VL53L0X** is an **IR laser ToF** sensor measuring distance up to ~2 m over I²C.

**Prerequisites:** [17-i2c.md](../17-i2c.md)

---

## Working Principle

VCSEL laser emits IR pulses; SPAD array measures return time. ST firmware on-chip computes distance in mm. Supports single-shot and continuous ranging modes.

---

## Datasheet Notes

| Parameter | Value |
|-----------|-------|
| I²C address | `0x29` (default) |
| Range | Up to 1200 mm (mode dependent) |
| Accuracy | ±3% typical |
| Interface | I²C up to 400 kHz |
| Boot | XSHUT pin for address change / reset |

---

## Protocol

**I²C** with 16-bit register index (big-endian address in API wrappers). Requires init sequence loading ST tuning settings — use proven register blob from reference driver.

See [17-i2c.md](../17-i2c.md).

---

## Register Map

| Addr | Name | Description |
|------|------|-------------|
| `0x0000` | IDENTIFICATION_MODEL_ID | `0xEE` |
| `0x0016` | FINAL_RANGE_CONFIG | Range setup |
| `0x0018` | SYSRANGE_START | Start ranging (`0x01`) |
| `0x0089` | RESULT_INTERRUPT_STATUS | New sample ready |
| `0x008E`–`0x0091` | RESULT_RANGE_STATUS | Distance mm |

Full init requires ~100 register writes from ST API sequence.

---

## ESP32-S3 Wiring

```
ESP32-S3          VL53L0X
────────          ───────
GPIO8  ─────────► SDA
GPIO9  ─────────► SCL
GPIO6  ─────────► XSHUT (optional reset)
GPIO7  ─────────► GPIO1 (interrupt, optional)
3V3    ─────────► VCC
GND    ─────────► GND
```

2.8 V sensor — most breakouts include regulator and level shifters.

---

## Swift Driver Sketch

```swift
struct VL53L0X<B: I2CBus> {
    var bus: B
    let address: UInt8 = 0x29

    mutating func initDevice() throws {
        // Load ST reference tuning sequence (omitted for brevity)
        try writeReg16(0x0016, 0x00)
    }

    mutating func startContinuous() throws {
        try writeReg8(0x0018, 0x02)
    }

    mutating func readRangeMm() throws -> UInt16? {
        let st = try readReg8(0x0089)
        guard st & 0x07 != 0 else { return nil }
        let d = try readBurst(from: 0x008E, count: 12)
        return UInt16(d[10]) << 8 | UInt16(d[11])
    }

    private mutating func writeReg8(_ reg: UInt16, _ val: UInt8) throws {
        try bus.write(to: address, register: UInt8(reg & 0xFF), data: [val])
    }
}
```

Use 16-bit indexed register helpers matching ST API.

---

## Bare-Metal Notes

- Cover glass in front of sensor affects calibration — use ST offset calibration for your housing.
- XSHUT low powers down device — useful for multi-sensor address assignment.
- I²C pull-ups 2.2 kΩ recommended at 400 kHz.

---

## HAL / Protocol-Oriented Driver Notes

Init blob in separate `VL53L0XConfig` resource. Implement `Rangefinder` protocol. Consider wrapping ST register sequence in generated Swift from reference C array.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always 8190 mm | Init failed | Re-run full tuning sequence |
| I²C NACK | Wrong address | Scan bus; check XSHUT |
| Jitter | Ambient IR | Enable longer timing budget |

---

## Example Project

**Liquid level:** VL53L0X downward-facing in tank. Map mm to fill percentage; alert when low.

---

## References

- [VL53L0X Datasheet (ST)](https://www.st.com/resource/en/datasheet/vl53l0x.pdf)
- [17-i2c.md](../17-i2c.md)
