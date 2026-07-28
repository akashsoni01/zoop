# Fingerprint Modules — R307, AS608

Guide to **optical UART fingerprint modules** (R307, AS608) for Embedded Swift.

**Prerequisites:** [08-gpio.md](../08-gpio.md)

---

## Working Principle

Module captures fingerprint image internally, extracts minutiae template, stores in onboard flash (512+ templates). Host sends **packet commands** over UART; module responds with ACK + data.

---

## Datasheet Notes

| Parameter | R307 typical |
|-----------|--------------|
| Interface | UART TTL 57600 default |
| Template storage | 162–1000 (model dependent) |
| False accept rate | 0.001% |
| Supply | 3.3–6 V (use 3.3 V TTL with ESP32) |

---

## Protocol

**UART** 57600 8N1 (configurable). Packet structure:

```
[0xEF01][ADDR 4B][PKT_TYPE][LEN 2B][DATA...][CHECKSUM 2B]
```

Commands: `0x01` Verify password, `0x01` GenImg, `0x02` Img2Tz, `0x05` Search, `0x08` Store.

See [08-gpio.md](../08-gpio.md) for UART pins.

---

## Register Map

Not register-based — command/response packet protocol. System parameters readable via `ReadSysPara` command.

---

## ESP32-S3 Wiring

```
ESP32-S3          R307 Module
────────          ───────────
GPIO17 (TX) ────► RX (white)
GPIO18 (RX) ◄──── TX (green)
3V3         ────► VCC (red)
GND         ────► GND (black)
GPIO14      ────► WAKEUP (optional)
```

Cross-connect TX/RX.

---

## Swift Driver Sketch

```swift
protocol UARTPort {
    mutating func write(_ data: [UInt8]) throws
    mutating func read(into buffer: inout [UInt8], maxBytes: Int) throws -> Int
}

enum FingerprintError: Error { case noFinger, noMatch, packetError }

struct R307<U: UARTPort> {
    var uart: U
    let address: UInt32 = 0xFFFFFFFF

    mutating func verifyPassword(_ pwd: UInt32 = 0) throws {
        var data = [UInt8((pwd >> 24) & 0xFF), UInt8((pwd >> 16) & 0xFF),
                    UInt8((pwd >> 8) & 0xFF), UInt8(pwd & 0xFF)]
        try sendPacket(type: 0x01, command: 0x13, data: &data)
        try waitAck()
    }

    mutating func captureImage() throws {
        try sendPacket(type: 0x01, command: 0x01, data: &[])
        let ack = try readResponse()
        guard ack.confirmCode == 0x00 else { throw FingerprintError.noFinger }
    }

    mutating func searchTemplate(inCount: UInt16) throws -> (id: UInt16, score: UInt16)? {
        var data: [UInt8] = [0x01, 0x00, 0x00, UInt8(inCount >> 8), UInt8(inCount & 0xFF)]
        try sendPacket(type: 0x01, command: 0x04, data: &data)
        let resp = try readResponse()
        // Parse page ID and match score from resp.data
        return nil
    }
}
```

---

## Bare-Metal Notes

- Enrollment requires multiple captures (usually 2) merged to template.
- Module LED indicates states — use for user feedback.
- Buffer UART RX — packets up to 256 bytes.

---

## HAL / Protocol-Oriented Driver Notes

Packet codec separate from `R307` command API. `BiometricStore` protocol for enroll/verify/search.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No ACK | Baud mismatch | Default 57600 |
| Error 0x02 | No finger | Press firmly |
| Error 0x09 | No match | Re-enroll template |

---

## Example Project

**Attendance system:** Enroll up to 50 users. On match, log ID + timestamp to flash and show name on LCD.

---

## References

- [R307 Technical Manual](https://cdn-shop.adafruit.com/datasheets/379fingerprint.pdf)
- [08-gpio.md](../08-gpio.md)
