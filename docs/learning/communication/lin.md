# LIN — Local Interconnect Network

**LIN** (Local Interconnect Network) is a **single-wire, master-slave** serial bus for **automotive sub-networks** — mirrors, seats, sensors — where CAN is overkill.

**Prerequisites:** [uart-usart.md](./uart-usart.md)

---

## Theory

One **master** (typically MCU with enhanced UART) schedules **frames** in a **LDF** (LIN Description File) schedule table. Slaves respond in predetermined slots.

Based on **UART-like** async serial:

| Parameter | Value |
|-----------|-------|
| Baud | 19200 typical (2400–19200) |
| Break + sync | 13+ bit break, sync 0x55 |
| PID | Protected identifier |
| Checksum | Classic or enhanced |

LIN transceiver (e.g., TJA1021) handles bus physical layer.

---

## Timing Diagram (ASCII)

LIN frame:

```
Break   Sync   PID   Data0..7   Checksum
│       │      │     │          │
▼       ▼      ▼     ▼          ▼

Break (dominant ≥13 bits):
─────┐                          
     └────────────────────────

Sync 0x55:
      ┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐
      └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘

Schedule table (master):

| Frame A | Frame B | Frame C | Frame A | ...
  10 ms     20 ms     10 ms
```

---

## Packet Format

| Field | Size | Notes |
|-------|------|-------|
| Break | ≥13 bits | Sync start |
| Sync | 1 B | 0x55 |
| PID | 1 B | 6-bit ID + parity |
| Data | 1–8 B | Payload |
| Checksum | 1 B | Classic: sum; Enhanced: includes PID |

---

## Electrical Characteristics

| Parameter | Typical |
|-----------|---------|
| Bus voltage | 12 V automotive |
| Transceiver | TJA1020/1021 |
| Topology | Bus, single wire + GND |
| Max nodes | 16 |
| ESD / EMC | Automotive requirements |

ESP32-S3 needs LIN transceiver — not 12 V tolerant on GPIO.

---

## Rust HAL Sketch

```rust
// LIN master send — requires UART that supports break generation
pub fn lin_send_frame(
    uart: &mut impl embedded_io::Write,
    pid: u8,
    data: &[u8],
) -> Result<(), ()> {
    // 1. Send break (hardware-specific API on esp-hal or bit-bang)
    send_break(uart)?;
    uart.write_all(&[0x55]).map_err(|_| ())?; // sync
    uart.write_all(&[pid]).map_err(|_| ())?;
    uart.write_all(data).map_err(|_| ())?;
    let cs = lin_checksum_classic(data);
    uart.write_all(&[cs]).map_err(|_| ())
}

fn lin_checksum_classic(data: &[u8]) -> u8 {
    let sum: u16 = data.iter().map(|b| *b as u16).sum();
    (!sum) as u8
}
```

---

## Bare-Metal Sketch (Concept)

Configure UART LIN mode if hardware supports break generation; else GPIO bit-bang dominant break on TX line via transceiver.

---

## Example Projects

| Link | Use |
|------|-----|
| [projects/can-analyzer.md](../projects/can-analyzer.md) | Automotive bench |
| [projects/robot.md](../projects/robot.md) | Sub-node actuator bus |

---

## Common Mistakes

1. **Break too short** — slaves don't sync.
2. **PID parity wrong** — frame ignored.
3. **Enhanced vs classic checksum** mismatch.
4. **Schedule drift** — master must hit LDF timing.
5. **12 V transceiver powered from 3.3 V logic** without level match.

---

## Exercises

1. Decode LIN trace with USB-LIN adapter; match PID to LDF.
2. Implement master schedule for two frames @ 10 ms.
3. Compare LIN bus load vs [can.md](./can.md) for same sensor set.
4. Calculate checksum by hand for sample data bytes.

---

## References

- LIN Specification Package (ISO 17987)
- TJA1021 datasheet
- Lesson: [15-uart.md](../15-uart.md)

---

*Prev: [modbus.md](./modbus.md) | Next: [mqtt.md](./mqtt.md)*
