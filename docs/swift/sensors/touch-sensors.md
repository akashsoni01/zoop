# Touch Sensors — TTP223, MPR121

Guide to **capacitive touch** modules (TTP223) and **multi-channel touch ICs** (MPR121) for Embedded Swift.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [17-i2c.md](../17-i2c.md)

---

## Working Principle

**Capacitive sensing** detects body capacitance change near electrode. TTP223: single pad, digital output. MPR121: up to 12 electrodes over I²C with thresholds and filtering.

---

## Datasheet Notes

| Parameter | TTP223 |
|-----------|--------|
| Output | Digital HIGH when touched |
| Modes | Toggle / momentary (AHLB pin) |
| Response | ~60 ms (module dependent) |

MPR121: I²C `0x5A`/`0x5B`, 12 channels, interrupt pin.

---

## Protocol

**TTP223:** GPIO digital read per [08-gpio.md](../08-gpio.md).

**MPR121:** I²C register read for touch status bitmap.

See [17-i2c.md](../17-i2c.md).

---

## Register Map (MPR121)

| Addr | Name | Description |
|------|------|-------------|
| `0x00`–`0x01` | TOUCH_STATUS | 12-bit touch bitmap |
| `0x5E` | ELECTRODE_CONFIG | Run/stop |
| `0x41`–`0x5A` | Thresholds | Touch/release per channel |
| `0x5B` | AUTO_CONFIG | Electrode count |
| `0x80` | RESET | Soft reset `0x63` |

TTP223 has no registers.

---

## ESP32-S3 Wiring

### TTP223

```
ESP32-S3          TTP223
────────          ──────
GPIO4  ◄───────── OUT
3V3    ──────────► VCC
GND    ──────────► GND
```

### MPR121

```
GPIO8  ─────────► SDA
GPIO9  ─────────► SCL
GPIO7  ◄───────── IRQ
3V3    ─────────► VCC
```

---

## Swift Driver Sketch

```swift
struct TTP223<P: DigitalPin> {
    var pin: P
    mutating func isTouched() throws -> Bool { try pin.read() }
}

struct MPR121<B: I2CBus> {
    var bus: B
    let address: UInt8 = 0x5A

    mutating func initDevice() throws {
        try bus.write(to: address, register: 0x80, data: [0x63]) // reset
        try bus.write(to: address, register: 0x5E, data: [0x0C]) // start, 12 electrodes
    }

    mutating func touchedChannels() throws -> UInt16 {
        let lo = try bus.read(from: address, register: 0x00, count: 1)[0]
        let hi = try bus.read(from: address, register: 0x01, count: 1)[0]
        return UInt16(lo) | (UInt16(hi) << 8)
    }
}
```

---

## Bare-Metal Notes

- MPR121 electrodes connect via copper pads on PCB — tune thresholds per layout.
- TTP223 sensitive to moisture — not for outdoor unprotected use.
- Debounce touch in UI layer (50–100 ms).

---

## HAL / Protocol-Oriented Driver Notes

`TouchSensor` protocol with `isActive() -> Bool`. MPR121 implements `MultiTouch` returning channel bitmask.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Stuck on | Floating OUT | Check wiring |
| MPR121 NACK | Wrong address | Try `0x5B` |
| False touches | Noise | Raise touch threshold registers |

---

## Example Project

**Capacitive piano:** MPR121 12 keys → buzzer tones ([speakers.md](./speakers.md)). Map channel index to frequency.

---

## References

- [MPR121 Datasheet (NXP)](https://www.nxp.com/docs/en/data-sheet/MPR121.pdf)
- [08-gpio.md](../08-gpio.md), [17-i2c.md](../17-i2c.md)
