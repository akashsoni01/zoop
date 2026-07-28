# LiDAR Sensors — TF-Luna, RPLidar

Guide to **UART/PWM LiDAR** modules for medium-range distance scanning and mapping.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

**Time-of-flight (ToF) laser** or **triangulation** measures distance to reflective surfaces. Scanning LiDAR (RPLidar) rotates a laser unit; solid-state modules (TF-Luna) return single-point range.

TF-Luna: 0.2–8 m, UART default 115200 baud, binary frame protocol.

---

## Datasheet Notes

| Parameter | TF-Luna |
|-----------|---------|
| Range | 0.2–8 m |
| Accuracy | ±6 cm |
| Interface | UART / I²C / PWM |
| Frame rate | 100 Hz |
| Baud | 115200 default |

RPLidar A1: 360° scan, 8000 samples/s, UART 115200 or 256000.

---

## Protocol

### TF-Luna UART Frame (9 bytes)

| Byte | Content |
|------|---------|
| 0 | Header `0x59` |
| 1 | Header `0x59` |
| 2–3 | Distance L/H (cm) |
| 4–5 | Strength L/H |
| 6–7 | Temperature raw |
| 8 | Checksum (sum bytes 0–7) & 0xFF |

Sync on dual `0x59` headers.

### RPLidar

Binary scan protocol — start scan command, receive variable-length scan nodes with angle + distance.

---

## Register Map

TF-Luna I²C mode uses register `0x01` for distance low/high. UART mode has no register access — frame-based only.

---

## ESP32-S3 Wiring

### TF-Luna (UART)

```
ESP32-S3          TF-Luna
────────          ───────
GPIO17 (TX) ────► RX (optional — config only)
GPIO18 (RX) ◄──── TX
3V3         ────► VCC
GND         ────► GND
```

### TF-Luna (PWM mode)

```
GPIO4 ◄──────── PWM output (pulse width ∝ distance)
```

---

## Swift Driver Sketch

```swift
struct TFLuna<U: UARTPort> {
    var uart: U
    private var buf: [UInt8] = []

    mutating func readDistanceCm() throws -> UInt16? {
        var tmp = [UInt8](repeating: 0, count: 32)
        let n = try uart.read(into: &tmp, maxBytes: 32)
        buf.append(contentsOf: tmp.prefix(n))
        while buf.count >= 9 {
            if buf[0] == 0x59 && buf[1] == 0x59 {
                let frame = Array(buf.prefix(9))
                buf.removeFirst(9)
                let sum = frame.prefix(8).reduce(0, +) & 0xFF
                guard sum == frame[8] else { continue }
                return UInt16(frame[2]) | (UInt16(frame[3]) << 8)
            }
            buf.removeFirst(1)
        }
        return nil
    }
}
```

---

## Bare-Metal Notes

- TF-Luna lens must stay clean — dust causes short readings.
- Glass and dark surfaces reduce range.
- RPLidar needs stable 5 V / adequate current for motor.

---

## HAL / Protocol-Oriented Driver Notes

Frame parser separate from UART. Expose `Rangefinder` protocol with `readDistanceM() -> Float?`. TF-Luna PWM mode uses `InputCapture` protocol.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No frames | Baud mismatch | Set 115200 8N1 |
| Checksum errors | EMI on UART | Short wires; ferrite |
| Short max range | Target absorption | Use reflective target |

---

## Example Project

**Obstacle avoider:** TF-Luna at 50 Hz; stop motor if distance < 30 cm. Display range on SSD1306.

---

## References

- [TF-Luna Datasheet (Benewake)](https://en.benewake.com/index.php/Support/info/68.html)
- [08-gpio.md](../08-gpio.md)
