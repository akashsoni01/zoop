# GPS Modules — NMEA over UART

Guide to **NEO-6M**, **u-blox NEO-M8N**, and similar UART GPS modules for Embedded Swift.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

GPS receivers listen to **satellite navigation signals** (GPS/GLONASS/Galileo). A onboard RF front-end and correlator compute position, velocity, and time. The module outputs human-readable **NMEA 0183** sentences over UART (typically 9600 baud).

Key sentence: **`$GPRMC`** or **`$GNRMC`** — time, lat, lon, speed, fix validity.

---

## Datasheet Notes

| Parameter | NEO-6M typical |
|-----------|----------------|
| Supply | 3.3 V (some boards include LDO for 5 V in) |
| UART baud | 9600 default (configurable) |
| Fix time (cold) | 27–30 s |
| Accuracy | 2.5 m CEP |
| Update rate | 1 Hz default (up to 10 Hz on M8N) |

---

## Protocol

**UART** 8N1. NMEA sentences start with `$` and end with `*` + 2-char hex checksum.

Example:
```
$GNRMC,123519,A,4807.038,N,01131.000,E,022.4,084.4,230394,003.1,W*6A
```

Fields: time, status (A=valid), lat, N/S, lon, E/W, speed, course, date.

No register map — configuration via **UBX binary protocol** (u-blox) or **PUBX** text commands.

---

## Register Map

GPS modules do not expose a register map over I²C in standard hobby boards. u-blox **UBX protocol** uses class/id/message structure:

| Class | ID | Purpose |
|-------|-----|---------|
| `0x06` | `0x00` | CFG-PRT (port config) |
| `0x06` | `0x08` | CFG-RATE (measurement rate) |
| `0x01` | `0x07` | NAV-PVT (position fix) |

For NMEA-only projects, parse text sentences — no UBX required.

---

## ESP32-S3 Wiring

```
ESP32-S3          GPS Module
────────          ──────────
GPIO17 (TX) ────► RX  (module RX ← ESP TX)
GPIO18 (RX) ◄──── TX  (module TX → ESP RX)
3V3         ────► VCC
GND         ────► GND
```

Optional: GPIO for PPS (pulse-per-second) for precise timing.

---

## Swift Driver Sketch

```swift
protocol UARTPort {
    mutating func write(_ data: [UInt8]) throws
    mutating func read(into buffer: inout [UInt8], maxBytes: Int) throws -> Int
    var bytesAvailable: Int { get }
}

struct NMEAFix: Sendable {
    var valid: Bool
    var latitude: Double
    var longitude: Double
    var speedKnots: Float
    var course: Float
}

struct NMEAParser {
    private var lineBuffer: [UInt8] = []

    mutating func feed(_ byte: UInt8) -> NMEAFix? {
        if byte == 0x0A {
            defer { lineBuffer.removeAll(keepingCapacity: true) }
            guard let line = String(bytes: lineBuffer, encoding: .ascii),
                  line.hasPrefix("$GNRMC") || line.hasPrefix("$GPRMC") else { return nil }
            return parseRMC(line)
        }
        if byte != 0x0D { lineBuffer.append(byte) }
        return nil
    }

    func parseRMC(_ line: String) -> NMEAFix? {
        let parts = line.split(separator: ",")
        guard parts.count >= 10, parts[2] == "A" else { return NMEAFix(valid: false, latitude: 0, longitude: 0, speedKnots: 0, course: 0) }
        let lat = nmeaToDecimal(String(parts[3]), hemisphere: Character(parts[4]))
        let lon = nmeaToDecimal(String(parts[5]), hemisphere: Character(parts[6]))
        let speed = Float(parts[7]) ?? 0
        let course = Float(parts[8]) ?? 0
        return NMEAFix(valid: true, latitude: lat, longitude: lon, speedKnots: speed, course: course)
    }

    func nmeaToDecimal(_ coord: String, hemisphere: Character) -> Double { 0 }
}

struct GPS<U: UARTPort> {
    var uart: U
    var parser = NMEAParser()

    mutating func poll() throws -> NMEAFix? {
        var buf = [UInt8](repeating: 0, count: 64)
        let n = try uart.read(into: &buf, maxBytes: 64)
        for i in 0..<n {
            if let fix = parser.feed(buf[i]) { return fix }
        }
        return nil
    }
}
```

---

## Bare-Metal Notes

- GPS modules need **clear sky view** — indoor fixes fail or drift.
- First fix after power-on takes 30+ s (cold start). Backup battery on module preserves almanac.
- UART RX buffer must drain continuously — NMEA at 1 Hz is ~80 bytes/s but bursts occur.

---

## HAL / Protocol-Oriented Driver Notes

Separate `NMEAParser` (pure Swift, host-testable) from `GPS` UART wrapper. For u-blox binary, add `UbloxUBX` layer with checksum and struct decoding.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No output | TX/RX swapped | Cross-connect ESP TX→GPS RX |
| `$` garbage | Baud mismatch | Match 9600 8N1 |
| No fix (V=invalid) | Indoor / antenna | Move outdoors; check antenna connection |
| 5 V module on 3.3 V UART | Level mismatch | Use 3.3 V module or level shifter |

---

## Example Project

**GPS tracker:** Log RMC sentences to flash ([flash-memory.md](./flash-memory.md)). Display lat/lon on SSD1306. Blink LED when fix valid.

---

## References

- [NMEA 0183 specification](https://www.nmea.org/)
- [u-blox NEO-6M datasheet](https://content.u-blox.com/sites/default/files/products/documents/NEO-6_DataSheet_%28GPS.G6-HW-09005%29.pdf)
- [08-gpio.md](../08-gpio.md) — UART pins
