# External SPI Flash (W25Q128)

**SPI NOR flash** (Winbond W25Q series) provides **megabytes** of storage for logs, assets, and OTA firmware slots.

**Prerequisites:** [16-spi.md](../16-spi.md)

---

## Working Principle

**NOR flash** stores bits in floating-gate cells — read like ROM, write requires **erase** (sector/block) then **program**. W25Q128: **128 Mbit (16 MB)**, SPI up to 80 MHz (board dependent).

---

## Datasheet Key Points (W25Q128JV)

| Parameter | Value |
|-----------|-------|
| Capacity | 16 MB |
| Sector size | 4 KB |
| Block size | 64 KB |
| Page program | 256 bytes max |
| SPI modes | 0 and 3 |
| ID | Manufacturer 0xEF, Device 0x4018 |

---

## Protocol

Standard SPI with **CS** active low. Command byte + address + data.

### Common Commands

| CMD | Code | Description |
|-----|------|-------------|
| Read JEDEC ID | `0x9F` | Manufacturer + device ID |
| Read Data | `0x03` | Slow read |
| Fast Read | `0x0B` | + dummy byte |
| Page Program | `0x02` | Up to 256 B |
| Sector Erase | `0x20` | 4 KB |
| Block Erase | `0xD8` | 64 KB |
| Chip Erase | `0xC7` | Entire chip |
| Read Status | `0x05` | BUSY bit |
| Write Enable | `0x06` | Before program/erase |

Status register bit 0 (**BUSY**) = 1 during internal erase/program.

---

## Register Map

Flash is **memory-mapped address space**, not registers. Optional **status/config** registers via commands above.

| Status bit | Meaning |
|------------|---------|
| 0 BUSY | Operation in progress |
| 1 WEL | Write enable latch |
| 2–4 BP | Block protect |

---

## ESP32-S3 Wiring

Many ESP32-S3 boards embed W25Q flash for firmware — **external** breakout for extra storage:

```
W25Q128          ESP32-S3
───────          ────────
VCC  ──────────► 3V3
GND  ──────────► GND
CS   ──────────► GPIO10
CLK  ──────────► GPIO12
DI   ◄────────── GPIO11 (MOSI)
DO   ──────────► GPIO13 (MISO)
```

Keep wires short at high SPI speeds. **Do not** conflict with internal flash CS on devkit.

---

## Rust Driver Sketch

```rust
use embedded_hal::delay::DelayNs;
use embedded_hal::spi::SpiDevice;
use embedded_hal::digital::OutputPin;

pub struct W25q128<SPI, CS, DELAY> {
    spi: SPI,
    cs: CS,
    delay: DELAY,
}

impl<SPI, CS, DELAY, E> W25q128<SPI, CS, DELAY>
where
    SPI: SpiDevice<Error = E>,
    CS: OutputPin<Error = E>,
    DELAY: DelayNs,
{
    pub fn read_jedec_id(&mut self) -> Result<u32, E> {
        let cmd = [0x9F, 0, 0, 0];
        let mut id = [0u8; 3];
        self.cs.set_low()?;
        self.spi.write(&cmd)?;
        self.spi.read(&mut id)?;
        self.cs.set_high()?;
        Ok(((id[0] as u32) << 16) | ((id[1] as u32) << 8) | id[2] as u32)
    }

    pub fn wait_busy(&mut self) -> Result<(), E> {
        loop {
            let st = self.read_status()?;
            if st & 1 == 0 { return Ok(()); }
            self.delay.delay_us(100);
        }
    }

    pub fn sector_erase(&mut self, addr: u32) -> Result<(), E> {
        self.write_enable()?;
        // cmd 0x20 + 24-bit address
        self.wait_busy()?;
        Ok(())
    }
}
```

**Crates:** `w25q`, `embedded-storage`, `littlefs2` (filesystem on flash).

---

## Bare-Metal Notes

- **Erase before write** — cannot flip 1→0 without erase (sector).
- **Wear** — 100k erase cycles per sector — use wear leveling (LittleFS).
- **Quad SPI (QPI)** — faster reads; requires 4 IO pins and mode switch command.
- ESP32-S3 can **memory-map** external flash for XIP (advanced).

---

## HAL / embedded-hal Notes

Implement `embedded_storage::NorFlash`:

```rust
// erase sector, write page aligned, read arbitrary
```

Filesystem layer (`littlefs2`) provides files for GPS logs ([gps.md](./gps.md)).

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| ID 0xFFFFFF | CS or wiring | Check CS polarity, SPI mode |
| Write fails | Write enable not sent | Command 0x06 first |
| Garbage read | Wrong address width | 24-bit addr for W25Q128 |
| Corruption | Erase not done | Erase sector before program |

---

## Example Project: GPS Track Logger

1. Append NMEA sentences to circular buffer in flash.
2. Index in last sector; wear-level index updates.
3. Dump over USB serial on command.

---

## Exercises

1. Read JEDEC ID; verify 0xEF4018.
2. Implement 4 KB sector erase + page program of test pattern.
3. Mount LittleFS; create `log.txt` and append lines.

---

## References

- Winbond W25Q128JV datasheet
- [16-spi.md](../16-spi.md)
- `littlefs2` crate documentation
- [eeprom.md](./eeprom.md) — small config storage
