# Lesson 18 — CAN Bus (Controller Area Network)

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [10-timers.md](./10-timers.md)

**Board focus:** ESP32 TWAI (Two-Wire Automotive Interface). STM32 is the industry reference.

**Maturity note:** **No production Embedded Swift CAN stack** exists today. Use **ESP-IDF TWAI C API** from Swift via ESP-IDF bindings, or implement register access with **swift-mmio**. Host Swift (macOS) CAN via USB adapters is useful for protocol development and HIL.

---

## Theory

**CAN** is a **multi-master serial bus** for noisy automotive/industrial environments. **Differential signaling** on **CAN_H** and **CAN_L**.

### Key properties

- **Broadcast:** All nodes see all frames; local acceptance filtering.
- **Arbitration:** Non-destructive priority by **ID** — lower ID wins.
- **No host required:** Any node transmits when bus is idle.
- **Robust:** CRC, ACK slot, error confinement.

### Frame format (Classical CAN 2.0A)

```
 SOF | ID (11b) | RTR | IDE | r0 | DLC | DATA (0–8) | CRC | ACK | EOF
```

| Field | Meaning |
|-------|---------|
| **ID** | Message identifier — priority |
| **DLC** | Data length 0–8 bytes |
| **RTR** | Remote transmission request |

**CAN FD:** 64-byte payload, dual bit rates — check transceiver support.

### Bit timing

```
Bit time = Sync + Prop + Phase1 + Phase2
Sample point ~75–87.5%
```

Common bitrates: **125**, **250**, **500**, **1000 kbit/s**.

---

## Hardware Overview

### ESP32 / ESP32-S3 — TWAI

- Integrated controller — **requires external transceiver** (SN65HVD230, TJA1050).
- Hardware ID/mask filters, bus-off recovery, listen-only mode.
- DevKitC-1 has **no onboard transceiver** — add CAN module/HAT.

### Physical layer

```
        CAN_H ───┬───────────────┬───
                 │   120 Ω       │
Node A ══════════╪═══════════════╪══════════ Node B
                 │   (bus term)  │
        CAN_L ───┴───────────────┴───
```

**120 Ω termination** at **both bus ends**.

---

## ASCII Wiring

```
ESP32-S3                SN65HVD230 Module
┌──────────┐            ┌──────────┐
│ GPIO4    ├── TX ─────►│ TXD      │
│ GPIO5    │◄─ RX ──────┤ RXD      │
│ 3V3      ├───────────►│ VCC      │
│ GND      ├───────────►│ GND      │
└──────────┘            │ CANH ────┼──► to bus CAN_H
                        │ CANL ────┼──► to bus CAN_L
                        └──────────┘
```

Verify TWAI pin mapping in TRM — GPIO4/5 are common examples; adjust for your board.

---

## Memory & Register Notes

### ESP32 TWAI registers (conceptual)

| Register | Purpose |
|----------|---------|
| `TWAI_MODE` | Normal / listen-only / loopback |
| `TWAI_BTR` | Bit timing (prescaler, segments) |
| `TWAI_STATUS` | TX/RX buffers, error state |
| `TWAI_INT_ENA` | RX, TX, error, bus-off interrupts |
| `TWAI_ARB_LOST` | Arbitration lost counter |

### Message struct

```swift
struct CANFrame {
    var id: UInt32        // 11-bit standard or 29-bit extended
    var extended: Bool
    var data: [UInt8]     // 0..8 bytes
    var dlc: UInt8
}
```

---

## HAL Swift Example — TWAI via ESP-IDF

```swift
import ESPIDF

@main
struct CanNodeApp {
    static func main() {
        let twai = TWAI(
            txPin: 4,
            rxPin: 5,
            bitrate: .kbps500,
            mode: .normal
        )

        twai.installFilter(
            acceptance: .single(id: 0x100, mask: 0x7FF)
        )

        twai.start()

        // Transmit engine RPM frame every 100 ms
        var rpm: UInt16 = 800
        while true {
            let payload: [UInt8] = [
                UInt8(rpm >> 8), UInt8(rpm & 0xFF), 0, 0, 0, 0, 0, 0
            ]
            twai.transmit(id: 0x100, data: payload)
            rpm &+= 10
            delay(ms: 100)
        }
    }
}
```

> Wire `TWAI` to ESP-IDF `twai_driver_install`, `twai_transmit`, `twai_receive` C APIs.

---

## Bare-Metal Swift Sketch

```swift
import MMIO

let twai = TWAI(baseAddress: 0x6002_5000)

func twaiInit500K() {
    // Configure bit timing for 500 kbit/s at APB clock
    twai.btr0.set(0x00) // prescaler — compute from TRM
    twai.btr1.set(0x1C) // sample point, SJW
    twai.mode.normal.set(true)
    twai.cmd.tx_request.set(true)
}

func twaiReceive() -> CANFrame? {
    guard twai.status.rx_ready.get() else { return nil }
    let id = twai.id.get()
    var data: [UInt8] = []
    for _ in 0..<8 { data.append(UInt8(twai.data_fifo.get())) }
    return CANFrame(id: id, extended: false, data: data, dlc: 8)
}
```

---

## Host Swift — CAN via USB (macOS development)

Use **PCAN**, **CANable**, or similar USB-CAN adapters for protocol testing:

```swift
import Foundation

protocol CANBusProtocol {
    func send(frame: CANFrame) throws
    func receive(timeout: TimeInterval) throws -> CANFrame?
}

struct EngineSimulator {
    let bus: CANBusProtocol

    func run() throws {
        var rpm: UInt16 = 800
        while true {
            let frame = CANFrame(
                id: 0x100, extended: false,
                data: [UInt8(rpm >> 8), UInt8(rpm & 0xFF)],
                dlc: 2
            )
            try bus.send(frame: frame)
            rpm &+= 10
            Thread.sleep(forTimeInterval: 0.1)
        }
    }
}
```

Develop parsers on macOS; port frame structs to Embedded Swift unchanged — [26-testing.md](./26-testing.md).

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| No transceiver | TWAI works, bus silent | Add SN65HVD230 or equivalent |
| Missing termination | Reflections, errors | 120 Ω at both ends |
| CANH/CANL swapped | No communication | Swap differential pair |
| Bitrate mismatch | Error passive / bus-off | Same bitrate on all nodes |
| Wrong sample point | Intermittent errors on long bus | Retune BTR segments |
| TX without bus partner | Error frames | Use loopback mode for solo test |

---

## Debugging Tips

- Start in **loopback mode** — verify controller without transceiver.
- Monitor **TEC/REC** error counters — rising = wiring/timing issue.
- CAN analyzer (PCAN-View, SavvyCAN) on bus alongside your node.
- Listen-only mode to snoop without affecting arbitration.

---

## Performance Tips

- Use **hardware filters** to drop irrelevant IDs in ISR.
- Batch application logic — don't parse in ISR; queue frames.
- CAN FD only if all nodes and transceiver support it.
- For OBD-II, standard IDs and 500 kbit/s dominate — profile accordingly.

---

## Exercises

1. **Loopback:** Send and receive same ID in TWAI loopback mode.
2. **Bus sniffer:** Listen-only; print all IDs to UART.
3. **OBD-II PID:** Request `0x7DF` mode 01 PID 0x0C (RPM) — parse response.
4. **Host parser:** macOS Swift decodes logged candump CSV.
5. **Error recovery:** Force bus-off; verify auto-recovery after 128 occurrences.

---

## References

- [ESP32-S3 TRM — TWAI](https://www.espressif.com/en/products/socs/esp32-s3)
- [ISO 11898-1 Classical CAN](https://www.iso.org/standard/63648.html)
- [ESP-IDF TWAI driver](https://docs.espressif.com/projects/esp-idf/en/latest/esp32/api-reference/peripherals/twai.html)
- [boards/esp32-s3.md](./boards/esp32-s3.md)

---

*Previous: [17-i2c.md](./17-i2c.md) · Next: [19-usb.md](./19-usb.md)*
