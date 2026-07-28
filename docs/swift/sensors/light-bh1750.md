# BH1750 — Ambient Light Sensor

The **ROHM BH1750** measures **ambient illuminance in lux** over I²C.

**Prerequisites:** [17-i2c.md](../17-i2c.md)

---

## Working Principle

Digital ambient light sensor using photodiode array. Internal ADC converts to lux value — no external calculation required beyond mode scaling.

---

## Datasheet Notes

| Parameter | Value |
|-----------|-------|
| I²C address | `0x23` (ADDR low) / `0x5C` (ADDR high) |
| Range | 1–65535 lx (mode dependent) |
| Resolution | 0.5–1 lx depending on mode |
| VDD | 2.4–3.6 V |

---

## Protocol

**I²C command protocol** — no register address byte. Send one-byte **command**, then read 2-byte result.

See [17-i2c.md](../17-i2c.md).

---

## Register Map (Commands)

| Cmd | Name | Description |
|-----|------|-------------|
| `0x00` | POWER_DOWN | Low power |
| `0x01` | POWER_ON | Wake |
| `0x07` | RESET | Reset data register |
| `0x10` | CONT_H_RES | Continuous high-res (1 lx) |
| `0x13` | CONT_L_RES | Continuous low-res (4 lx) |
| `0x20` | ONE_HIRES | One-shot high-res |

Result: 16-bit big-endian, divide by 1.2 for lux in high-res mode.

---

## ESP32-S3 Wiring

```
ESP32-S3          BH1750
────────          ───────
GPIO8  ─────────► SDA
GPIO9  ─────────► SCL
3V3    ─────────► VCC
GND    ─────────► GND
```

ADDR pin: GND → `0x23`, VCC → `0x5C`.

---

## Swift Driver Sketch

```swift
struct BH1750<B: I2CBus> {
    var bus: B
    let address: UInt8 = 0x23

    mutating func initDevice() throws {
        try bus.writeCommand(to: address, command: 0x01) // power on
        try bus.writeCommand(to: address, command: 0x10) // continuous H-res
    }

    mutating func readLux() throws -> Float {
        let d = try bus.read(from: address, count: 2)
        let raw = UInt16(d[0]) << 8 | UInt16(d[1])
        return Float(raw) / 1.2
    }
}

extension I2CBus {
    func writeCommand(to address: UInt8, command: UInt8) throws {
        try write(to: address, register: command, data: [])
    }
}
```

---

## Bare-Metal Notes

- First measurement after power-on takes ~120 ms in high-res mode.
- Window in front of sensor affects reading — keep clear.
- I²C clock stretch supported but rare on ESP32.

---

## HAL / Protocol-Oriented Driver Notes

Simple command/response — wrap in `LightSensor` protocol returning lux `Float`. Power-down between readings for battery projects.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Always 0 | Power down | Send POWER_ON |
| I²C NACK | ADDR pin | Try `0x5C` |
| Slow updates | One-shot mode | Use continuous mode |

---

## Example Project

**Auto-brightness:** BH1750 + PWM backlight on ST7789 ([displays/st7789.md](../displays/st7789.md)). Map lux to backlight duty cycle.

---

## References

- [BH1750 Datasheet (ROHM)](https://www.mouser.com/datasheet/2/588/bh1750fvi-e-186247.pdf)
- [17-i2c.md](../17-i2c.md)
