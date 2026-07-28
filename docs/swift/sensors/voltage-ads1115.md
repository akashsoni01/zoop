# ADS1115 — 16-Bit External ADC

The **Texas Instruments ADS1115** provides **four multiplexed 16-bit ADC channels** over I²C — ideal for precision voltage and slow sensor sampling.

**Prerequisites:** [17-i2c.md](../17-i2c.md)

---

## Working Principle

Delta-sigma ADC with programmable gain amplifier (PGA). Single-ended or differential inputs. Programmable data rate 8–860 SPS. Built-in comparator optional.

---

## Datasheet Notes

| Parameter | Value |
|-----------|-------|
| I²C address | `0x48`–`0x4B` (ADDR pin) |
| Resolution | 16 bits |
| Channels | 4 single-ended or 2 differential |
| PGA | ±256 mV … ±6.144 V |
| Reference | Internal 2.048 V or VDD |

---

## Protocol

**I²C** pointer register + 16-bit config/result registers, big-endian.

See [17-i2c.md](../17-i2c.md).

---

## Register Map

| Addr | Name | Description |
|------|------|-------------|
| `0x00` | CONVERSION | Result (signed 16-bit) |
| `0x01` | CONFIG | MUX, PGA, mode, data rate |
| `0x02` | LO_THRESH | Comparator low |
| `0x03` | HI_THRESH | Comparator high |

### CONFIG bits

| Bits | Field |
|------|-------|
| 15 | OS — start single conversion |
| 14:12 | MUX channel select |
| 11:9 | PGA gain |
| 8 | Mode (single vs continuous) |
| 7:5 | Data rate |
| 4 | Comparator mode |

---

## ESP32-S3 Wiring

```
ESP32-S3          ADS1115
────────          ───────
GPIO8  ─────────► SDA
GPIO9  ─────────► SCL
3V3    ─────────► VDD
GND    ─────────► GND

A0   ◄────────── Sensor analog out
```

---

## Swift Driver Sketch

```swift
enum ADS1115Channel: UInt16 {
    case a0 = 0x4000  // MUX bits for AIN0 vs GND
    case a1 = 0x5000
    case a2 = 0x6000
    case a3 = 0x7000
}

enum ADS1115Gain: UInt16 {
    case v4_096 = 0x0200  // ±4.096 V
    case v2_048 = 0x0000  // ±2.048 V
}

struct ADS1115<B: I2CBus> {
    var bus: B
    let address: UInt8 = 0x48
    var gain: ADS1115Gain = .v4_096

    mutating func readSingle(channel: ADS1115Channel) throws -> Int16 {
        var cfg = channel.rawValue | gain.rawValue | 0x8000 | 0x0100 // OS + single-shot
        try writeReg(0x01, cfg)
        // Poll OS bit or wait 10 ms
        let raw = try readReg(0x00)
        return Int16(bitPattern: raw)
    }

    func toVoltage(_ raw: Int16) -> Float {
        let lsb: Float = 4.096 / 32768.0  // depends on gain
        return Float(raw) * lsb
    }
}
```

---

## Bare-Metal Notes

- For single-ended reads, connect unused inputs to GND.
- Add 0.1 µF on VDD near chip.
- Max input must not exceed VDD + 0.3 V.

---

## HAL / Protocol-Oriented Driver Notes

Implement `ADCChannel` protocol wrapping ADS1115 channel. Allows higher-level sensors to use external precision ADC transparently.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Saturated ±32768 | Input > PGA range | Lower gain or divider |
| Slow reads | Low data rate | Increase rate in CONFIG |
| Wrong channel | MUX bits | Verify channel enum |

---

## Example Project

**4-channel data logger:** ADS1115 A0–A3 for soil moisture sensors. Sample every 10 s; store to flash.

---

## References

- [ADS1115 Datasheet (TI)](https://www.ti.com/lit/ds/symlink/ads1115.pdf)
- [17-i2c.md](../17-i2c.md)
