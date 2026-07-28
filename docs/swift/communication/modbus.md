# Modbus — Industrial Register Protocol

**Modbus** is a **master/slave** protocol for reading/writing **registers** and **coils** over serial (**RTU**) or TCP. Dominant in PLCs, VFDs, energy meters.

**Prerequisites:** [rs485.md](./rs485.md), [uart-usart.md](./uart-usart.md)

---

## Theory

**Modbus RTU** (binary, CRC):

| Function | Code | Action |
|----------|------|--------|
| Read coils | 0x01 | Read discrete outputs |
| Read discrete inputs | 0x02 | Read discrete inputs |
| Read holding registers | 0x03 | Read 16-bit registers |
| Read input registers | 0x04 | Read read-only registers |
| Write single register | 0x06 | Write one register |
| Write multiple registers | 0x10 | Write block |

**Modbus TCP:** same PDU wrapped in MBAP header over TCP port **502**.

Master sends request; slave responds or exception (function + 0x80).

---

## Timing Diagram (ASCII)

Modbus RTU frame gap (3.5 character times silence between frames):

```
Bus idle ────┐                              ┌────
             └──── [Addr|Func|Data|CRC] ────┘
                  |←── frame ──→|
             ↑                  ↑
        3.5 char silence    3.5 char silence

@ 9600 baud, char ~1 ms → ~3.5 ms inter-frame delay
```

---

## Packet Format

**Read Holding Registers (0x03) request:**

| Byte | Content |
|------|---------|
| 0 | Slave address |
| 1 | Function 0x03 |
| 2–3 | Start address (big-endian) |
| 4–5 | Quantity of registers |
| 6–7 | CRC-16 (LSB first) |

**Response:**

| Byte | Content |
|------|---------|
| 0 | Address |
| 1 | 0x03 |
| 2 | Byte count |
| 3.. | Register data (big-endian per register) |
| n–1,n | CRC |

---

## Electrical Characteristics

Modbus RTU physical layer is [rs485.md](./rs485.md) or [rs232.md](./rs232.md). Modbus TCP uses [ethernet.md](./ethernet.md) / [wifi.md](./wifi.md).

Timing-critical: inter-frame delay on RTU — implement in software after UART flush.

---

## Swift HAL Sketch

```swift
import Embedded

struct ModbusRTU {
    var port: RS485Port

    mutating func readHoldingRegisters(
        slave: UInt8,
        start: UInt16,
        count: UInt16
    ) throws -> [UInt16] {
        var req: [UInt8] = [
            slave, 0x03,
            UInt8(start >> 8), UInt8(start & 0xFF),
            UInt8(count >> 8), UInt8(count & 0xFF)
        ]
        appendCRC16(&req)
        port.transmit(req)
        var resp = [UInt8](repeating: 0, count: 5 + Int(count) * 2)
        let n = port.receive(into: &resp, timeoutMs: 500)
        try validateCRC(resp, length: n)
        return parseRegisters(resp, count: count)
    }
}

func appendCRC16(_ data: inout [UInt8]) {
    let crc = modbusCRC(data)
    data.append(UInt8(crc & 0xFF))
    data.append(UInt8(crc >> 8))
}
```

**ARC note:** Pure struct protocol layer — no heap in request/response path.

---

## Example Projects

| Link | Description |
|------|-------------|
| [projects/iot-gateway.md](../projects/iot-gateway.md) | Modbus → MQTT |
| [projects/home-automation.md](../projects/home-automation.md) | PLC integration |

---

## Common Mistakes

1. **Wrong CRC byte order** — Modbus CRC LSB first.
2. **Big-endian register confusion** — addresses vs values.
3. **Insufficient inter-frame delay** — merged frames.
4. **Address 0 broadcast** — no response expected.
5. **Reading float as single register** — need 2 registers + IEEE754 decode.

---

## Exercises

1. Read 2 registers from energy meter simulator.
2. Write setpoint register 0x0064 = 1000.
3. Bridge register map to MQTT JSON in gateway project.
4. Implement Modbus TCP read over Wi-Fi.

---

## References

- Modbus Organization specification
- [rs485.md](./rs485.md)
- [projects/iot-gateway.md](../projects/iot-gateway.md)

---

*Prev: [rs485.md](./rs485.md) | Next: [lin.md](./lin.md)*

*Back to [Embedded Swift](../README.md)*
