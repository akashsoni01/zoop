# I²C EEPROM (AT24C256)

**Serial EEPROM** stores persistent configuration, calibration, and logs — **non-volatile** memory with byte/page write over **I²C**.

**Prerequisites:** [17-i2c.md](../17-i2c.md)

---

## Working Principle

**EEPROM (Electrically Erasable Programmable Read-Only Memory)** retains data without power. AT24C256 provides **256 Kbit (32 KB)** with 64-byte **page write** buffer and 400 kHz I²C.

Write cycle time: **~5 ms** per page — must wait or poll before next write.

---

## Datasheet Key Points (AT24C256)

| Parameter | Value |
|-----------|-------|
| Capacity | 32 KB (0x0000–0x7FFF) |
| I²C address | `0x50`–`0x57` (A0–A2 pins) |
| Page size | 64 bytes |
| Endurance | 1M write cycles per page |
| VCC | 1.7–5.5 V |

---

## Protocol

**Memory address:** 2-byte big-endian word sent after device address for read/write.

**Random read:** Write pointer (2 bytes addr) → repeated start → read.

**Page write:** Addr + up to 64 bytes within same page — do not cross page boundary.

---

## Register Map

Flat memory array — no device registers. Address space is direct byte index.

| Addr range | Content (your app) |
|------------|-------------------|
| 0x0000–0x00FF | Magic + schema version |
| 0x0100–... | Calibration blocks |
| ... | User data |

Design a **wear-leveled** layout for frequently updated values (circular log indices).

---

## ESP32-S3 Wiring

```
AT24C256         ESP32-S3
────────         ────────
VCC  ──────────► 3V3
GND  ──────────► GND
SDA  ──────────► GPIO8
SCL  ──────────► GPIO9
A0,A1,A2 ─ GND ► address 0x50
WP   ── GND ───► (write protect disabled)
```

Tie **WP** high to hardware-protect in production.

---

## Rust Driver Sketch

```rust
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x50;
const PAGE: usize = 64;

pub struct At24c256<I2C, DELAY> {
    i2c: I2C,
    delay: DELAY,
}

impl<I2C, DELAY, E> At24c256<I2C, DELAY>
where
    I2C: I2c<Error = E>,
    DELAY: DelayNs,
{
    pub fn write_byte(&mut self, addr: u16, byte: u8) -> Result<(), E> {
        let buf = [(addr >> 8) as u8, addr as u8, byte];
        self.i2c.write(ADDR, &buf)?;
        self.delay.delay_ms(5);
        Ok(())
    }

    pub fn write_page(&mut self, addr: u16, data: &[u8]) -> Result<(), E> {
        assert!(data.len() <= PAGE);
        let page_offset = (addr as usize) % PAGE;
        assert!(page_offset + data.len() <= PAGE);
        let mut buf = [0u8; 66];
        buf[0] = (addr >> 8) as u8;
        buf[1] = addr as u8;
        buf[2..2 + data.len()].copy_from_slice(data);
        self.i2c.write(ADDR, &buf[..2 + data.len()])?;
        self.delay.delay_ms(5);
        Ok(())
    }

    pub fn read(&mut self, addr: u16, out: &mut [u8]) -> Result<(), E> {
        let addr_bytes = [(addr >> 8) as u8, addr as u8];
        self.i2c.write_read(ADDR, &addr_bytes, out)?;
        Ok(())
    }
}
```

**Crates:** `eeprom24x`, `embedded-storage` traits.

---

## Bare-Metal Notes

- **Never write in ISR** — blocking 5 ms.
- **Page boundaries** — split multi-byte writes.
- **Wear:** rotate log sectors; EEPROM not for high-frequency streaming (use flash).
- Implement `embedded_storage::ReadStorage` / `WriteStorage` for bootloader compatibility.

---

## HAL / embedded-hal Notes

```rust
pub trait ByteStorage {
    type Error;
    fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), Self::Error>;
    fn write(&mut self, addr: u32, buf: &[u8]) -> Result<(), Self::Error>;
}
```

Higher layers (serde/postcard) serialize structs to EEPROM with CRC.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Write fails silently | WP pin high | Ground WP for development |
| Corrupt data | Page wrap | Respect 64-byte pages |
| NACK | Wrong address | Set A0–A2 straps |
| Slow throughput | 5 ms per write | Batch page writes |

---

## Example Project: Config Store

1. Store Wi-Fi credentials + BMP280 altitude offset in EEPROM.
2. Magic bytes `0xEEPROM` + version for migration.
3. Load at boot before sensor init.

---

## Exercises

1. Implement read-modify-write for single byte without corrupting page neighbors.
2. Benchmark sequential 1 KB write vs page-aligned write.
3. Add CRC16 footer to config block; reject invalid on boot.

---

## References

- Microchip AT24C256 datasheet
- [flash-memory.md](./flash-memory.md) — larger storage
- [17-i2c.md](../17-i2c.md)
