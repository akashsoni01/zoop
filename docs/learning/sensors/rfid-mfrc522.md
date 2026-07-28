# MFRC522 — RFID Reader (13.56 MHz)

The **NXP MFRC522** reads **ISO14443A** RFID tags (MIFARE Classic, NTAG) over **SPI** or **I²C**.

**Prerequisites:** [16-spi.md](../16-spi.md), [17-i2c.md](../17-i2c.md)

---

## Working Principle

**RFID (Radio-Frequency Identification):** reader antenna emits 13.56 MHz field; passive tag harvests power and modulates backscatter to send UID and data.

MFRC522 handles modulation/demodulation; host reads/writes **FIFO** and **command registers**.

---

## Datasheet Key Points

| Parameter | Value |
|-----------|-------|
| Interface | SPI (10 MHz max), I²C, UART |
| Supply | 2.5–3.3 V |
| Antenna | 13.56 MHz tuned circuit on module |
| Card types | MIFARE Classic 1K/4K, Ultralight |

---

## Protocol

### SPI (Most Common Breakouts)

| Pin | Function |
|-----|----------|
| CS | Active low chip select |
| SCK, MOSI, MISO | SPI |
| IRQ | Optional interrupt |

**SPI Mode 0**, MSB first. Address byte: `(reg << 1) | rw` (rw: 0=write, 1=read).

### ISO14443A Sequence (High Level)

1. `REQA` / `WUPA` — wake tag
2. Anticollision → **UID**
3. Select tag
4. Authenticate (MIFARE) / read blocks

---

## Register Map (Essential)

| Addr | Name | Description |
|------|------|-------------|
| `0x01` | CommandReg | Start commands |
| `0x04` | ComIrqReg | Interrupt flags |
| `0x06` | ErrorReg | Protocol errors |
| `0x07` | Status1Reg | Crypto, CRC status |
| `0x09` | FIFODataReg | FIFO access |
| `0x0A` | FIFOLevelReg | Bytes in FIFO |
| `0x0C` | ControlReg | Initiator control |
| `0x0D` | BitFramingReg | |
| `0x14` | TxControlReg | Antenna drivers TX1/TX2 |
| `0x15` | TxAutoReg | |
| `0x37` | VersionReg | `0x91` or `0x92` |

### Commands (CommandReg)

| Cmd | Value | Action |
|-----|-------|--------|
| Idle | `0x00` | Cancel |
| MFAuthent | `0x0E` | MIFARE auth |
| Transceive | `0x0C` | Send/receive FIFO |

---

## ESP32-S3 Wiring (SPI)

```
MFRC522          ESP32-S3
───────          ────────
SDA (CS) ──────► GPIO10
SCK  ──────────► GPIO12
MOSI ──────────► GPIO11
MISO ──────────► GPIO13
IRQ  ──────────► GPIO7 (optional)
RST  ──────────► GPIO15
3.3V / GND       3V3 / GND
```

**Antenna must face tag** within ~3 cm for typical modules.

---

## Rust Driver Sketch

```rust
use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiDevice;

pub struct Mfrc522<SPI, CS, RST> {
    spi: SPI,
    cs: CS,
    rst: RST,
}

impl<SPI, CS, RST, E> Mfrc522<SPI, CS, RST>
where
    SPI: SpiDevice<Error = E>,
    CS: OutputPin<Error = E>,
    RST: OutputPin<Error = E>,
{
    pub fn init(&mut self) -> Result<(), E> {
        self.rst.set_low()?;
        self.delay_ms(10);
        self.rst.set_high()?;
        self.delay_ms(50);
        self.write_reg(0x14, 0x03)?; // TxControlReg: antenna on
        Ok(())
    }

    pub fn transceive(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<usize, E> {
        self.write_reg(0x0A, 0x80)?; // flush FIFO
        for &b in tx {
            self.write_reg(0x09, b)?;
        }
        self.write_reg(0x01, 0x0C)?; // Transceive
        self.wait_irq()?;
        let level = self.read_reg(0x0A)? & 0x7F;
        for i in 0..level as usize {
            rx[i] = self.read_reg(0x09)?;
        }
        Ok(level as usize)
    }
}
```

**Crates:** `mfrc522`, `mfrc522-embedded`.

---

## Bare-Metal Notes

- Enable antenna (`TxControlReg`) or **no tag detection**.
- Set timer and CRC presets per card type (MIFARE uses Crypto1 — legal use only on your tags).
- **Power spikes** when field active — bulk cap on 3.3 V.

---

## HAL / embedded-hal Notes

Use `SpiDevice` (embedded-hal 1.0) for CS-aware transactions. Separate **card detection** task from **UI** task in async firmware.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No UID | Antenna off | Set TxControlReg |
| Version 0x00 | SPI wiring | Check MISO, mode 0 |
| Auth fail | Wrong key | Default MIFARE key `FF FF FF FF FF FF` |
| Short range | Detuned antenna | Use module as designed; no metal under coil |

---

## Example Project: Access Badge Reader

1. Read UID on tag present.
2. Compare to allow-list in flash ([flash-memory.md](./flash-memory.md)).
3. Show name on character LCD ([lcd-character.md](../displays/lcd-character.md)).

---

## Exercises

1. Implement REQA/WUPA and print 4-byte UID.
2. Read NTAG215 NDEF URL block (if tag available).
3. Measure current draw antenna on vs off.

---

## References

- NXP MFRC522 datasheet (MFRC522.pdf)
- ISO/IEC 14443-3 specification overview
- [nfc-pn532.md](./nfc-pn532.md) — alternative reader
- [16-spi.md](../16-spi.md)
