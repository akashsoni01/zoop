# MFRC522 — 13.56 MHz RFID Reader

The **NXP MFRC522** reads **ISO14443A** RFID tags (MIFARE Classic, NTAG) over **SPI**.

**Prerequisites:** [16-spi.md](../16-spi.md), [08-gpio.md](../08-gpio.md)

---

## Working Principle

RFID reader IC drives 13.56 MHz antenna. Tag harvests field energy and modulates backscatter. MFRC522 handles framing, anticollision, and crypto for MIFARE.

---

## Datasheet Notes

| Parameter | Value |
|-----------|-------|
| Interface | SPI (also UART/I²C variants) |
| Voltage | 2.5–3.3 V |
| Version reg (`0x37`) | `0x91` or `0x92` |
| Antenna | 13.56 MHz tuned LC |

---

## Protocol

**SPI mode 0** (CPOL=0, CPHA=0). Address byte: `(addr << 1) | read_bit`. FIFO for TX/RX.

See [16-spi.md](../16-spi.md) for ESP32-S3 SPI wiring.

---

## Register Map

| Addr | Name | Description |
|------|------|-------------|
| `0x01` | COMMAND | Idle, calc CRC, transceive |
| `0x04` | COMIRQ | Interrupt flags |
| `0x09` | FIFO_DATA | FIFO access |
| `0x0A` | FIFO_LEVEL | Bytes in FIFO |
| `0x0C` | CONTROL | Initiator settings |
| `0x14` | TX_CONTROL | Antenna drivers TX1/TX2 |
| `0x37` | VERSION | Chip version |

### Init essentials

1. Soft reset (`0x01 = 0x0F`)
2. Timer config
3. `TX_CONTROL = 0x03` — enable antenna
4. RFCfg, mode registers

---

## ESP32-S3 Wiring

```
ESP32-S3          MFRC522
────────          ───────
GPIO11 ─────────► MOSI
GPIO13 ◄───────── MISO
GPIO12 ─────────► SCK
GPIO10 ─────────► CS (SS)
GPIO14 ─────────► RST
3V3    ─────────► 3.3V
GND    ─────────► GND
```

IRQ optional on GPIO7.

---

## Swift Driver Sketch

```swift
protocol SPIBus {
    mutating func transfer(write: [UInt8], into read: inout [UInt8]) throws
}

struct MFRC522<S: SPIBus> {
    var spi: S
    var csPin: any DigitalPin
    var rstPin: any DigitalPin

    mutating func initDevice() throws {
        try hardReset()
        writeReg(0x01, 0x0F) // soft reset
        writeReg(0x2A, 0x8D) // timer
        writeReg(0x14, 0x03) // antenna on
    }

    mutating func requestTag() throws -> UInt16? {
        try writeReg(0x0D, 0x07) // bitrate
        var buf: [UInt8] = [0x26] // REQA
        guard try transceive(&buf, expectedBack: 2) else { return nil }
        return UInt16(buf[0]) << 8 | UInt16(buf[1])
    }

    private mutating func writeReg(_ addr: UInt8, _ val: UInt8) {
        var tx = [((addr << 1) & 0x7E), val]
        var rx = [UInt8](repeating: 0, count: 2)
        try? spi.transfer(write: tx, into: &rx)
    }
}
```

---

## Bare-Metal Notes

- Antenna tuning affects range — use module with presoldered antenna.
- Power supply must handle TX spikes (~100 mA).
- MIFARE auth requires key — default transport keys documented.

---

## HAL / Protocol-Oriented Driver Notes

Low-level `MFRC522` + high-level `ISO14443A` protocol for UID read. Keep SPI CS/RST as separate `DigitalPin` dependencies.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No tag detect | Antenna off | Set TX_CONTROL |
| SPI garbage | Wrong mode | SPI mode 0 |
| Short range | Detuned antenna | Check module caps |

---

## Example Project

**Door access:** Read UID; compare to allow-list in flash. Green NeoPixel on match ([displays/neopixel-ws2812.md](../displays/neopixel-ws2812.md)).

---

## References

- [MFRC522 Datasheet (NXP)](https://www.nxp.com/docs/en/data-sheet/MFRC522.pdf)
- [16-spi.md](../16-spi.md)
