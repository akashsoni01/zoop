# I²C EEPROM — AT24C256

Guide to **AT24C256** (32 KiB) and similar **I²C serial EEPROM** for non-volatile configuration and logging.

**Prerequisites:** [17-i2c.md](../17-i2c.md)

---

## Working Principle

**Floating-gate memory** retains bytes without power. I²C address selects chip; internal address pointer selects location. Page writes (64 bytes on AT24C256) buffer before program cycle (~5 ms).

---

## Datasheet Notes

| Parameter | AT24C256 |
|-----------|----------|
| Capacity | 256 Kbit (32 KB) |
| I²C address | `0x50`–`0x57` (A0–A2 pins) |
| Page size | 64 bytes |
| Endurance | 1M write cycles per page |
| Write time | ~5 ms max |

---

## Protocol

**I²C** memory protocol: write `[addr_hi, addr_lo, data...]` for write; write address pointer then read for read.

See [17-i2c.md](../17-i2c.md).

---

## Register Map

Flat memory array — no device registers. **16-bit word address** (two bytes) precedes data.

| Address range | Size |
|---------------|------|
| `0x0000`–`0x7FFF` | 32 KB |

---

## ESP32-S3 Wiring

```
ESP32-S3          AT24C256
────────          ────────
GPIO8  ─────────► SDA
GPIO9  ─────────► SCL
3V3    ─────────► VCC
GND    ─────────► GND
```

A0–A2 pins set I²C address LSBs.

---

## Swift Driver Sketch

```swift
struct AT24C256<B: I2CBus> {
    var bus: B
    let address: UInt8 = 0x50
    let pageSize: Int = 64

    mutating func read(offset: UInt16, count: Int) throws -> [UInt8] {
        try bus.writeAddressRead(from: address, memAddr: offset, count: count)
    }

    mutating func writePage(offset: UInt16, data: [UInt8]) throws {
        guard data.count <= pageSize else { throw StorageError.pageOverflow }
        guard Int(offset) % pageSize + data.count <= pageSize else {
            throw StorageError.pageCross
        }
        try bus.writeMemory(to: address, memAddr: offset, data: data)
        delayMs(6) // write cycle time
    }

    mutating func write(offset: UInt16, data: [UInt8]) throws {
        var off = offset
        var remaining = data
        while !remaining.isEmpty {
            let pageOffset = Int(off) % pageSize
            let chunk = min(remaining.count, pageSize - pageOffset)
            try writePage(offset: off, data: Array(remaining.prefix(chunk)))
            off += UInt16(chunk)
            remaining.removeFirst(chunk)
        }
    }
}
```

---

## Bare-Metal Notes

- Wear leveling for frequently updated counters — rotate sectors.
- WP pin (if present): tie high to allow writes.
- Do not power off during write cycle.

---

## HAL / Protocol-Oriented Driver Notes

Implement `NonVolatileStorage` protocol over byte offset API. Higher layers use typed Codable-like serialization with CRC.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Write fails | WP enabled | Check WP pin |
| Corrupt data | Page cross write | Split at page boundaries |
| NACK | Wrong address | Check A0–A2 straps |

---

## Example Project

**Config store:** Save Wi-Fi credentials and calibration constants in EEPROM; load at boot before sensor init.

---

## References

- [AT24C256 Datasheet (Microchip)](https://ww1.microchip.com/downloads/en/devicedoc/atmel-8560-seeprom-at24c256c-datasheet.pdf)
- [17-i2c.md](../17-i2c.md)
