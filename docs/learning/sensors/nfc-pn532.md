# PN532 — NFC Reader/Writer

The **NXP PN532** is a versatile **NFC** controller supporting **ISO14443A/B**, **MIFARE**, **FeliCa**, and **NFC Forum** tag formats over **I²C**, **SPI**, or **UART**.

**Prerequisites:** [17-i2c.md](../17-i2c.md), [16-spi.md](../16-spi.md)

---

## Working Principle

**NFC (Near Field Communication)** extends RFID to two-way peer communication at 13.56 MHz. PN532 integrates RF analog front-end, protocol stack, and host interface.

Use cases: read NTAG stickers, emulate cards (limited), P2P (deprecated in many apps).

---

## Datasheet Key Points

| Parameter | Value |
|-----------|-------|
| Host interfaces | I²C, SPI, HSU (UART) |
| I²C address | `0x24` (common) |
| Max RF field | ~5 cm practical |
| Firmware | Internal; host sends command frames |

---

## Protocol

### Host Frame Format (I²C)

```text
[Preamble 0x00] [Start 0x00] [0xFF] [LEN] [LCS] [TFI 0xD4] [CMD] [DATA...] [DCS] [Postamble 0x00]
```

Response TFI: `0xD5`. LCS = `~LEN + 1`; DCS = checksum over data.

### Common Commands

| CMD | Code | Purpose |
|-----|------|---------|
| GetFirmwareVersion | `0x02` | Returns IC version |
| SAMConfiguration | `0x14` | Setup security |
| InListPassiveTarget | `0x4A` | Detect tag, get UID |
| InDataExchange | `0x40` | Read/write tag |
| TgInitAsTarget | `0x8C` | Card emulation |

---

## Register Map

PN532 is **command-driven** — no simple register file like MFRC522. Host communicates via packet interface above.

---

## ESP32-S3 Wiring (I²C Mode)

Set module jumpers to **I²C** (board silkscreen).

```
PN532            ESP32-S3
─────            ────────
SDA  ──────────► GPIO8
SCL  ──────────► GPIO9
IRQ  ──────────► GPIO7 (optional)
RST  ──────────► GPIO15
VCC  ──────────► 3V3
GND  ──────────► GND
```

Some modules need **1.8 V–3.6 V** — avoid 5 V unless regulator onboard.

---

## Rust Driver Sketch

```rust
use embedded_hal::i2c::I2c;

const ADDR: u8 = 0x24;

pub struct Pn532<I2C> {
    i2c: I2C,
}

impl<I2C, E> Pn532<I2C>
where
    I2C: I2c<Error = E>,
{
    pub fn get_firmware_version(&mut self) -> Result<u32, E> {
        let frame = build_frame(0x02, &[]);
        self.i2c.write(ADDR, &frame)?;
        self.wait_ready()?;
        let resp = self.read_response()?;
        Ok(parse_version(&resp))
    }

    pub fn read_passive_target_uid(&mut self) -> Result<[u8; 7], E> {
        let frame = build_frame(0x4A, &[0x01, 0x00]); // max 1 tag, 106 kbps
        self.i2c.write(ADDR, &frame)?;
        self.wait_ready()?;
        let resp = self.read_response()?;
        parse_uid(&resp)
    }
}
```

**Crates:** `pn532`, `pn532-i2c`.

---

## Bare-Metal Notes

- **SAMConfiguration** required after power-up (`0x14`, normal mode).
- Long frames need **timeout** — tag removed mid-command.
- **Wake up** from power-down via GPIO reset or specific wake frame.

---

## HAL / embedded-hal Notes

Abstract NFC operations:

```rust
pub trait NfcReader {
    type Error;
    fn poll_tag(&mut self) -> Result<Option<Uid>, Self::Error>;
}
```

Works over `I2c`, `SpiDevice`, or UART `Read`/`Write` with feature flags.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No response | Wrong interface jumper | Set I²C vs SPI vs UART |
| Checksum error | Frame format bug | Verify LCS/DCS |
| Detects nothing | Tag type unsupported | Use ISO14443A NTAG |
| Intermittent | Poor antenna coupling | Hold tag steady over coil |

---

## Example Project: NFC URL Launcher

1. Read NDEF URI from NTAG213.
2. Display URL on SSD1306.
3. Optional: ESP32-S3 sends URL over Wi-Fi (if connected).

---

## Exercises

1. Implement frame builder/parser with checksum validation.
2. Write text NDEF record to blank NTAG (with appropriate app).
3. Compare detection speed PN532 vs MFRC522 ([rfid-mfrc522.md](./rfid-mfrc522.md)).

---

## References

- NXP PN532 User Manual (UM0701-02)
- NFC Forum Type 2 Tag specification
- [rfid-mfrc522.md](./rfid-mfrc522.md)
- [17-i2c.md](../17-i2c.md)
