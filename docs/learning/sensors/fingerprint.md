# Fingerprint Modules (R307, AS608)

Optical **fingerprint sensor modules** (R307, AS608, ZFM) provide enrollment, matching, and template storage over **UART**.

**Prerequisites:** [08-gpio.md](../08-gpio.md), UART lesson

---

## Working Principle

**Optical fingerprint:** LED illuminates finger; CMOS sensor captures ridge pattern; on-module DSP extracts **minutiae** and stores/compares **templates** (typically 256–512 bytes).

Host sends **command packets**; module responds with **acknowledge** + data.

---

## Datasheet Key Points (AS608 / R307 Family)

| Parameter | Value |
|-----------|-------|
| Interface | UART TTL 57600 default (38400–115200) |
| Template storage | ~300 on-module (module dependent) |
| Security | Template data on module; host gets match score |
| Supply | 3.3–6 V (use 3.3 V TTL with ESP32) |

---

## Protocol

### Packet Structure

```text
Header: 0xEF 0x01
Address: 4 bytes (0xFFFFFFFF default)
Type: 0x01 = command, 0x07 = ack, 0x02 = data
Length: 2 bytes big-endian (payload + checksum)
Payload: instruction + params
Checksum: sum of type+length+payload bytes
```

### Common Instructions

| Code | Command | Purpose |
|------|---------|---------|
| `0x01` | GenImg | Capture finger image |
| `0x02` | Img2Tz | Generate character file |
| `0x05` | RegModel | Combine chars → template |
| `0x06` | Store | Save template to flash ID |
| `0x03` | Search | 1:N match |
| `0x0D` | DeleteChar | Remove template |
| `0x13` | LED control | Sensor ring LED |

---

## Register Map

Not register-based — **command protocol** only. Template IDs are logical slots (0–299 typical).

---

## ESP32-S3 Wiring

```
Fingerprint       ESP32-S3
───────────       ────────
TX (module) ────► GPIO18 (ESP RX)
RX (module) ◄──── GPIO17 (ESP TX)
VCC  ───────────► 5V or 3V3 (per module spec)
GND  ───────────► GND
```

Use **3.3 V UART** modules or verify 5 V tolerant RX on ESP32 (often OK with divider on module TX).

---

## Rust Driver Sketch

```rust
use heapless::Vec;

const HEADER: [u8; 2] = [0xEF, 0x01];

pub struct Fingerprint<UART> {
    uart: UART,
}

impl<UART> Fingerprint<UART> {
    pub fn capture_image(&mut self) -> Result<(), FpError> {
        self.send_cmd(0x01, &[])?;
        match self.read_ack()? {
            0x00 => Ok(()),
            0x02 => Err(FpError::NoFinger),
            c => Err(FpError::Module(c)),
        }
    }

    pub fn search(&mut self, slot: u16) -> Result<(u16, u16), FpError> {
        // Img2Tz buffer 1, then search starting at slot
        todo!("command sequence")
    }

    fn send_cmd(&mut self, cmd: u8, params: &[u8]) -> Result<(), FpError> {
        let mut pkt: Vec<u8, 64> = Vec::new();
        // assemble header, address, type, length, cmd+params, checksum
        self.uart.write(&pkt)?;
        Ok(())
    }
}
```

**Crates:** `finger`, `as608` (community — verify maintenance).

---

## Bare-Metal Notes

- **57600 8N1** default — configure UART before commands.
- Commands can take **1–3 s** during enrollment — async state machine recommended.
- Store **template ID** on host, not full template, if module has flash.

---

## HAL / embedded-hal Notes

```rust
use embedded_io::{Read, Write};

pub trait BiometricSensor {
    type Error;
    fn identify(&mut self) -> Result<Option<u16>, Self::Error>;
}
```

Keep UX feedback (LED, display) in application layer.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No ack | Wrong baud | Try 57600, 38400 |
| Error 0x01 | Packet error | Fix checksum |
| Poor match | Dry finger | Moisturize; clean sensor glass |
| Timeout | Finger not placed | Call GenImg in loop |

---

## Example Project: Door Unlock

1. Enroll admin finger to slot 0 on module.
2. On boot, wait for finger; Search returns match ID.
3. Drive relay GPIO on match; log attempts to flash.

---

## Exercises

1. Implement packet parser with checksum validation.
2. Enroll two fingers; delete one via DeleteChar command.
3. Control module white LED during capture for user feedback.

---

## References

- AS608 / R307 user manual (HZ-09 protocol)
- [rfid-mfrc522.md](./rfid-mfrc522.md) — alternative auth
- [flash-memory.md](./flash-memory.md) — host-side template backup
