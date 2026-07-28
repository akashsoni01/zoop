# CAN — Controller Area Network

**CAN** (Controller Area Network) is a **multi-master differential serial bus** designed for automotive and industrial environments. Messages are **broadcast by ID** — no explicit addressing.

**Prerequisites:** [09-interrupts.md](../09-interrupts.md), [uart-usart.md](./uart-usart.md)

---

## Theory

Nodes share **CAN_H** and **CAN_L** differential pair. Dominant (0) and recessive (1) bits use wired-AND arbitration — **lower ID wins** bus access.

Two common standards:

| Standard | ID width | Max data |
|----------|----------|----------|
| CAN 2.0A | 11-bit | 0–8 bytes |
| CAN 2.0B / CAN FD | 29-bit | 0–8 (classic) / 64 (FD) |

Each frame includes **CRC**, **ACK slot**, and **EOF**. Faulty nodes can enter **bus-off** state.

ESP32-S3 has **TWAI** (Two-Wire Automotive Interface) controller — compatible with ISO 11898-1.

---

## Timing Diagram (ASCII)

Bit stuffing omitted — CAN frame (data frame, 11-bit ID):

```
CAN_H/L (differential, conceptual):

Arbitration field     Control   Data field        CRC   ACK
|← ID (11) →|RTR|IDE|R0|DLC|← 0–8 bytes →|← CRC →|ACK|

Bus idle ────────────────────────────────────────────────────
         ┌─ SOF ─ arbitration ─ ... ─ EOF ─┐
         └─────────────────────────────────┘

SOF = Start of Frame (dominant)
RTR = Remote Transmission Request
DLC = Data Length Code (0–8)
```

Arbitration — two nodes transmit; ID 0x100 loses to 0x050:

```
Node A TX ID bits: 0 1 0 0 0 0 0 0 0 0 0  (0x100)
Node B TX ID bits: 0 0 1 0 1 0 0 0 0 0 0  (0x050)
                       ↑ B wins (dominant 0)
Node A backs off (reads mismatch)
```

---

## Packet Format

**CAN 2.0 data frame:**

| Field | Size | Description |
|-------|------|-------------|
| SOF | 1 bit | Dominant |
| Identifier | 11 or 29 bit | Message priority |
| RTR | 1 bit | Data vs remote request |
| IDE | 1 bit | Standard vs extended |
| r0 | 1 bit | Reserved |
| DLC | 4 bits | Data length 0–8 |
| Data | 0–64 bits | Payload |
| CRC | 15 bits + delimiter | Error detection |
| ACK | 2 bits | Receiver acknowledgment |
| EOF | 7 bits | End of frame |
| IFS | 3 bits | Inter-frame space |

Application layer often uses **DBC** files to decode signals (e.g., RPM in bytes 0–1).

---

## Electrical Characteristics

| Parameter | Typical |
|-----------|---------|
| Physical layer | ISO 11898-2 (high-speed) |
| Bit rates | 125 k, 250 k, 500 k, 1 Mbit/s |
| Termination | 120 Ω at **both** bus ends |
| Max nodes | ~30–110 (depends on transceiver) |
| Common transceivers | TJA1050, SN65HVD230, MCP2551 |

ESP32 TWAI outputs logic-level signals — **external transceiver required** for the bus.

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32Hardware

struct CANBus {
    var twai: TWAI0

    mutating func start(baud: TWAI.BaudRate = .kb500) throws {
        try twai.configure(baudRate: baud)
        try twai.start()
    }

    mutating func sendStandard(id: UInt16, data: [UInt8]) throws {
        let frame = TWAI.Frame.standard(id: id, payload: data)
        try twai.transmit(frame)
    }

    mutating func receive() throws -> TWAI.Frame? {
        twai.pollReceive()
    }
}
```

**Ownership:** One `CANBus` struct owns the TWAI controller. Receive callbacks should set atomic flags — defer parsing to main loop to avoid ARC in ISR.

---

## Bare-Metal Sketch (Concept)

```swift
// TWAI register access via MMIO — configure bit timing:
// BRP, TSEG1, TSEG2, SJW for target baud rate
// Sample point typically 75–87.5% of bit time
let btr = TWAI.calculateTiming(targetHz: 500_000, clockHz: 80_000_000)
twaiRegisters.bitTiming.write(btr.rawValue)
```

Use Espressif TRM TWAI chapter for BTR calculations.

---

## Example Projects

| Link | Use |
|------|-----|
| [projects/can-analyzer.md](../projects/can-analyzer.md) | Sniff & decode |
| [projects/motor-controller.md](../projects/motor-controller.md) | Drive commands over CAN |
| [projects/robot.md](../projects/robot.md) | Multi-node robot bus |

---

## Common Mistakes

1. **Missing 120 Ω termination** — reflections, errors.
2. **Single termination in middle** — worse than none on short bench setup.
3. **Ground loops** — isolate with proper transceiver GND reference.
4. **Wrong bit timing** — all nodes must match baud rate exactly.
5. **Ignoring bus-off recovery** — implement error handling.

---

## Exercises

1. Send a 500 kbit/s frame every 100 ms; verify with USB-CAN adapter.
2. Parse one signal from a DBC file in Swift.
3. Build [projects/can-analyzer.md](../projects/can-analyzer.md) filter by ID.
4. Measure differential voltage with oscilloscope.

---

## References

- ISO 11898-1 (CAN protocol)
- [ESP32-S3 TWAI documentation](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/peripherals/twai.html)
- [projects/can-analyzer.md](../projects/can-analyzer.md)

---

*Prev: [i2c.md](./i2c.md) | Next: [usb.md](./usb.md)*

*Back to [Embedded Swift](../README.md)*
