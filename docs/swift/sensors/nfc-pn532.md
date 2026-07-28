# PN532 — NFC Reader/Writer

The **NXP PN532** supports **NFC** (ISO14443A/B, FeliCa) over **I²C**, UART, or SPI.

**Prerequisites:** [17-i2c.md](../17-i2c.md), [16-spi.md](../16-spi.md)

---

## Working Principle

NFC controller manages RF field and high-level commands (InListPassiveTarget, ReadBlock). Host sends **command frames** with preamble, length, checksum; PN532 responds with status + data.

---

## Datasheet Notes

| Parameter | Value |
|-----------|-------|
| I²C address | `0x24` (7-bit) |
| Interfaces | I²C / UART / SPI (mode select pins) |
| RF standards | ISO14443A/B, Mifare, FeliCa |
| Firmware | Internal; host uses command set |

---

## Protocol

**I²C:** Write frame to PN532; read status byte `0x01` = ready; read response frame.

Frame: `00 00 FF LEN LCS [DATA...] DCS 00`

See [17-i2c.md](../17-i2c.md).

---

## Register Map

PN532 uses **command frames**, not flat register map:

| Cmd | Code | Description |
|-----|------|-------------|
| SAMConfiguration | `0x14` | Security adaptation |
| InListPassiveTarget | `0x4A` | Detect tag |
| InDataExchange | `0x40` | Mifare read/write |
| GetFirmwareVersion | `0x02` | Returns IC version |

---

## ESP32-S3 Wiring (I²C)

```
ESP32-S3          PN532
────────          ─────
GPIO8  ─────────► SDA
GPIO9  ─────────► SCL
GPIO14 ─────────► RST
GPIO7  ◄───────── IRQ
3V3    ─────────► VCC
GND    ─────────► GND
```

Set mode switches on module to I²C.

---

## Swift Driver Sketch

```swift
struct PN532<B: I2CBus> {
    var bus: B
    let address: UInt8 = 0x24

    mutating func getFirmwareVersion() throws -> (ver: UInt8, rev: UInt8, support: UInt8) {
        let frame: [UInt8] = [0x00, 0x00, 0xFF, 0x02, 0xFE, 0xD4, 0x02, 0x2A, 0x00]
        try writeFrame(frame)
        let resp = try readFrame()
        return (resp[7], resp[8], resp[9])
    }

    mutating func readPassiveTargetUID() throws -> [UInt8]? {
        let cmd: [UInt8] = [0xD4, 0x4A, 0x01, 0x00] // InListPassiveTarget
        try sendCommand(cmd)
        let resp = try readFrame()
        guard resp.count > 10 else { return nil }
        let uidLen = resp[7]
        return Array(resp[8..<(8 + Int(uidLen))])
    }

    private mutating func writeFrame(_ data: [UInt8]) throws { /* I2C write */ }
    private mutating func readFrame() throws -> [UInt8] { [] }
}
```

---

## Bare-Metal Notes

- SAM config (`0x14`) required after reset — normal mode timeout settings.
- IRQ pin indicates data ready — poll or interrupt.
- 3.3 V only; some modules include level shifters.

---

## HAL / Protocol-Oriented Driver Notes

Frame builder/parser as pure Swift types. `NFCReader` protocol abstracts UID read and NDEF parse layer.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No response | Wrong interface mode | Set DIP to I²C |
| Checksum fail | Frame format | Verify LCS/DCS bytes |
| Short range | Low RF | Check antenna connection |

---

## Example Project

**NDEF URL writer:** Detect NTAG213; write URI record. Show UID on SSD1306.

---

## References

- [PN532 User Manual (NXP)](https://www.nxp.com/docs/en/user-guide/141540.pdf)
- [17-i2c.md](../17-i2c.md)
