# Modbus — Industrial Register Protocol

**Modbus** is a **master/slave** protocol for reading/writing **registers** and **coils** over serial (RTU/ASCII) or TCP. Ubiquitous in PLCs, VFDs, energy meters.

**Prerequisites:** [rs485.md](./rs485.md), [uart-usart.md](./uart-usart.md), [ethernet.md](./ethernet.md) (for Modbus TCP)

---

## Theory

Data model:

| Type | Access | Address range (typical) |
|------|--------|-------------------------|
| Coils | R/W bits | 00001–09999 |
| Discrete inputs | R bits | 10001–19999 |
| Holding registers | R/W 16-bit | 40001–49999 |
| Input registers | R 16-bit | 30001–39999 |

**Modbus RTU** on RS-485: binary, CRC16. **Modbus TCP** wraps same PDU in TCP port **502**.

Only **one master** per RTU bus (usually).

---

## Timing Diagram (ASCII)

Modbus RTU request/response @ 9600 baud:

```
Master TX:  [Addr:1][Func:1][StartHi:1][StartLo:1][CountHi:1][CountLo:1][CRC:2]
            |<──────────── ~8 ms ──────────────────────────────────────────>|

Bus quiet:  |── 3.5 char ──|

Slave TX:   [Addr:1][Func:1][ByteCount:1][Data...][CRC:2]

Func 0x03 Read Holding Registers example
Request:  01 03 00 00 00 0A C5 CD
Response: 01 03 14 [20 data bytes] [CRC]
```

---

## Packet Format

**Modbus RTU frame:**

| Field | Size | Description |
|-------|------|-------------|
| Unit ID | 1 B | Slave address 1–247 |
| Function | 1 B | 0x03 read, 0x06 write single, etc. |
| Data | N B | Start addr, count, values |
| CRC16 | 2 B | Lo byte first, poly 0xA001 |

**Modbus TCP MBAP header + PDU:**

| Field | Size |
|-------|------|
| Transaction ID | 2 B |
| Protocol ID | 2 B (0) |
| Length | 2 B |
| Unit ID | 1 B |
| PDU | same as RTU without CRC |

---

## Electrical Characteristics

Inherits [rs485.md](./rs485.md) or [ethernet.md](./ethernet.md) physical layer. Industrial installs: isolated transceivers (ADM2587), surge protection.

---

## Rust HAL Sketch

```rust
pub fn crc16_modbus(data: &[u8]) -> u16 {
    let mut crc = 0xFFFFu16;
    for b in data {
        crc ^= *b as u16;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xA001 } else { crc >> 1 };
        }
    }
    crc
}

pub fn build_read_holding(unit: u8, start: u16, count: u16) -> heapless::Vec<u8, 8> {
    let mut frame = heapless::Vec::new();
    frame.push(unit).ok();
    frame.push(0x03).ok();
    frame.extend_from_slice(&start.to_be_bytes()).ok();
    frame.extend_from_slice(&count.to_be_bytes()).ok();
    let crc = crc16_modbus(&frame);
    frame.push((crc & 0xFF) as u8).ok();
    frame.push((crc >> 8) as u8).ok();
    frame
}
```

Use with [rs485.md](./rs485.md) `Rs485Uart::transmit`.

---

## Bare-Metal Sketch (Concept)

State machine: IDLE → TX → WAIT 3.5 char → RX → validate CRC → parse registers.

---

## Example Projects

| Link | Use |
|------|-----|
| [projects/iot-gateway.md](../projects/iot-gateway.md) | Modbus to MQTT |
| [projects/environmental-monitor.md](../projects/environmental-monitor.md) | Read power meter |
| [projects/motor-controller.md](../projects/motor-controller.md) | VFD speed setpoint |

---

## Common Mistakes

1. **CRC byte order swapped** — common porting bug.
2. **Wrong register addressing** (0-based doc vs 40001 doc).
3. **Float registers** — byte/word order vendor-specific.
4. **Insufficient inter-frame delay** after TX before RX.
5. **Unit ID 0 on RTU** — broadcast only; no response.

---

## Exercises

1. Read 10 holding registers from Modbus slave simulator (`diagslave`).
2. Implement function 0x06 single register write.
3. Bridge one register to [mqtt.md](./mqtt.md) topic.
4. Compare RTU timing vs Modbus TCP on same register map.

---

## References

- Modbus Organization specification
- [modbus-core](https://crates.io/crates/modbus-core) crate
- Lesson: [18-can.md](../18-can.md) (field bus comparison)

---

*Prev: [rs485.md](./rs485.md) | Next: [lin.md](./lin.md)*
