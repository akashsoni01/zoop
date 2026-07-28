# External SPI Flash — W25Q128

Guide to **Winbond W25Q128** (16 MiB) **SPI NOR flash** for firmware storage, logs, and assets on ESP32-S3.

**Prerequisites:** [16-spi.md](../16-spi.md)

---

## Working Principle

**NOR flash** stores data in sectors (4 KB). Read via SPI command `0x03`; program page (256 B) with `0x02`; erase sector `0x20` or chip `0xC7`. ESP32-S3 can also boot from internal flash — external W25Q used for data partition.

---

## Datasheet Notes

| Parameter | W25Q128 |
|-----------|---------|
| Capacity | 128 Mbit (16 MB) |
| Interface | SPI up to 104 MHz |
| Page size | 256 bytes |
| Sector size | 4 KB |
| JEDEC ID | `EF 40 18` |

---

## Protocol

**SPI mode 0**. Command + address + dummy bytes for read. CS active low per transaction.

See [16-spi.md](../16-spi.md).

---

## Register Map (Commands)

| Cmd | Code | Description |
|-----|------|-------------|
| READ | `0x03` | Slow read |
| FAST_READ | `0x0B` | Fast read + dummy |
| PAGE_PROG | `0x02` | Program up to 256 B |
| SECTOR_ERASE | `0x20` | 4 KB erase |
| READ_ID | `0x9F` | JEDEC ID |
| WRITE_ENABLE | `0x06` | Enable program/erase |
| READ_STATUS | `0x05` | WIP bit |

Status register bit 0 (WIP): 1 = busy.

---

## ESP32-S3 Wiring

```
ESP32-S3          W25Q128
────────          ───────
GPIO11 ─────────► DI (MOSI)
GPIO13 ◄───────── DO (MISO)
GPIO12 ─────────► CLK
GPIO10 ─────────► CS
3V3    ─────────► VCC
GND    ─────────► GND
```

Some modules label pins IO0/IO1/CLK/CS.

---

## Swift Driver Sketch

```swift
struct W25Q128<S: SPIBus> {
    var spi: S
    var cs: any DigitalPin

    mutating func jedecID() throws -> (UInt8, UInt8, UInt8) {
        var tx: [UInt8] = [0x9F, 0, 0, 0]
        var rx = [UInt8](repeating: 0, count: 4)
        try csActive { try spi.transfer(write: tx, into: &rx) }
        return (rx[1], rx[2], rx[3])
    }

    mutating func read(offset: UInt32, count: Int) throws -> [UInt8] {
        var tx: [UInt8] = [0x03,
            UInt8((offset >> 16) & 0xFF), UInt8((offset >> 8) & 0xFF), UInt8(offset & 0xFF)]
        tx += [UInt8](repeating: 0, count: count)
        var rx = [UInt8](repeating: 0, count: tx.count)
        try csActive { try spi.transfer(write: tx, into: &rx) }
        return Array(rx.suffix(count))
    }

    mutating func waitReady() throws {
        repeat {
            var tx: [UInt8] = [0x05, 0]
            var rx = [UInt8](repeating: 0, count: 2)
            try csActive { try spi.transfer(write: tx, into: &rx) }
            if rx[1] & 0x01 == 0 { return }
        } while true
    }

    private mutating func csActive(_ body: () throws -> Void) rethrows {
        try cs.setLow(); defer { try? cs.setHigh() }
        try body()
    }
}
```

---

## Bare-Metal Notes

- Always erase before program — bits only go 1→0 via program, 0→1 via erase.
- Wear: ~100k erase cycles per sector — use log-structured layout.
- ESP32-S3 may share SPI bus with display — distinct CS per device.

---

## HAL / Protocol-Oriented Driver Notes

`SPIFlash` protocol: `read`, `program`, `eraseSector`. File-system layer (littlefs) optional on top.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Wrong JEDEC ID | Bad wiring / mode | Check SPI mode 0 |
| Program fail | Write disable | Send WRITE_ENABLE |
| Garbage read | Un erased sector | Erase sector first |

---

## Example Project

**Sensor black box:** Ring buffer of BME280 readings in flash sector; dump over UART on command.

---

## References

- [W25Q128JV Datasheet (Winbond)](https://www.winbond.com/hq/product/code-storage-flash-memory/serial-nor-flash/?__locale=en&partNo=W25Q128JV)
- [16-spi.md](../16-spi.md)
