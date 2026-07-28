# LIN — Local Interconnect Network

**LIN** is a **single-wire**, **master-scheduled** serial bus for automotive subsystems (sensors, actuators). Lower cost than CAN for non-critical nodes.

**Prerequisites:** [uart-usart.md](./uart-usart.md)

---

## Theory

**Topology:** one master (typically MCU), up to **15 slaves** on one bus.

**Physical layer:** UART-like **NRZ** at 9600–19200 baud with **break/sync** header for synchronization.

| Field | Description |
|-------|-------------|
| Break | Dominant ≥ 13 bit times |
| Sync | 0x55 |
| PID | Protected identifier (6 ID + 2 parity) |
| Data | 1–8 bytes |
| Checksum | Classic or enhanced |

Master publishes **schedule table** — each frame slot periodic.

---

## Timing Diagram (ASCII)

LIN frame:

```
Break   Sync  PID   Data0  Data1  ...  Checksum
│││││   0x55  0x3C  ─── payload ───  CS

Break field (13+ bits low):

TX  ────┐                              ┌──
        └──────────────────────────────┘
        |← ≥13 bit times dominant ─→|
```

---

## Packet Format

| Field | Size | Notes |
|-------|------|-------|
| Break | ≥13 bits | Sync start |
| Sync byte | 1 B | Always 0x55 |
| PID | 1 B | Frame ID + parity |
| Data | 1–8 B | Payload |
| Checksum | 1 B | Classic: sum only; Enhanced: includes PID |

**PID example:** ID 0x0C → PID byte 0xCC (with parity bits).

---

## Electrical Characteristics

| Parameter | Typical |
|-----------|---------|
| Bus voltage | 12 V automotive (via transceiver) |
| MCU interface | LIN transceiver (TJA1021, MCP2003) |
| Max nodes | 16 |
| Cable length | Up to 40 m @ 19.2 kbaud |
| Baud | 9600, 19200 common |

ESP32 connects via **LIN transceiver** to UART — not direct to 12 V bus.

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32Hardware

struct LINMaster {
    var uart: UART1

    mutating func sendFrame(pid: UInt8, data: [UInt8]) {
        uart.sendBreak(durationBits: 13)
        uart.write(0x55)  // sync
        uart.write(protectedID(pid))
        uart.write(data)
        uart.write(classicChecksum(data))
    }

    mutating func readSlaveResponse(into buffer: inout [UInt8], count: Int) -> Int {
        uart.read(into: &buffer, count: count, timeoutMs: 100)
    }
}

func protectedID(_ id: UInt8) -> UInt8 {
    // Compute parity bits P0, P1 per LIN spec
    id & 0x3F | parityBits(for: id)
}
```

---

## Example Projects

Automotive sensor nodes, door modules — often combined with [can.md](./can.md) gateways.

---

## Common Mistakes

1. **Break too short** — slaves don't sync.
2. **Wrong checksum type** — classic vs enhanced mismatch.
3. **Direct 12 V on UART** — use LIN transceiver.
4. **Schedule drift** — master must hit frame deadlines.
5. **PID parity error** — frame ignored by slaves.

---

## Exercises

1. Send master request frame PID 0x3C; read 2-byte response.
2. Implement enhanced checksum for diagnostic frames.
3. Build schedule table for 5 frames at 10 ms slots.
4. Sniff LIN bus with UART + break detection.

---

## References

- LIN Specification (ISO 17987)
- TJA1021 LIN transceiver datasheet
- [uart-usart.md](./uart-usart.md)

---

*Prev: [modbus.md](./modbus.md)*

*Back to [Embedded Swift](../README.md)*
