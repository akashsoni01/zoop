# USB — Universal Serial Bus

**USB** connects embedded devices to hosts (PC, phone) with standardized **connectors**, **power**, and **protocol layers**. ESP32-S3 includes **USB OTG** via internal PHY — common classes: **CDC-ACM** (serial), **HID** (keyboard/mouse).

**Prerequisites:** [09-interrupts.md](../09-interrupts.md), [uart-usart.md](./uart-usart.md)

---

## Theory

USB is **host-centric**: the host initiates all transfers. Devices respond with **descriptors** defining identity and endpoints.

| Speed | Rate | ESP32-S3 |
|-------|------|----------|
| Full Speed | 12 Mbit/s | Supported |
| High Speed | 480 Mbit/s | Not on internal PHY |

**Endpoints:** Control (EP0, mandatory), Bulk, Interrupt, Isochronous. **CDC** uses bulk IN/OUT + interrupt status.

Device states: **Powered → Default → Address → Configured**.

---

## Timing Diagram (ASCII)

USB packet (simplified — SYNC, PID, DATA, CRC, handshake):

```
Host OUT transaction (token + data + ACK):

SYNC PID ADDR ENDP CRC5  │  DATA0 CRC16  │  ACK
├── token packet ────────┤  ├── data ────┤  ┤

Line (differential NRZI — conceptual):

D+/D-  ──|K|J|K|J|...|PID|...|DATA bytes|...|ACK|── idle
```

Frame (Full Speed): 1 ms; **microframes** in High Speed only.

---

## Packet Format

**Token packet (OUT):**

| Field | Size |
|-------|------|
| SYNC | 8 bits |
| PID | 8 bits (OUT = 0xE1) |
| ADDR | 7 bits |
| ENDP | 4 bits |
| CRC5 | 5 bits |

**Setup packet (control transfer):**

| Offset | Content |
|--------|---------|
| 0 | bmRequestType |
| 1 | bRequest |
| 2–3 | wValue |
| 4–5 | wIndex |
| 6–7 | wLength |

Descriptors are binary structs — see USB spec Chapter 9.

---

## Electrical Characteristics

| Parameter | Full Speed |
|-----------|------------|
| VBUS | 5 V (device may draw 100 mA default) |
| D+/D- | 3.3 V signaling, 90 Ω ±15% |
| Pull-up on D+ | Device indicates Full Speed |
| Cable | USB-C or Micro-B on DevKit |

DevKitC-1 routes USB through onboard bridge for JTAG + optional USB device stack.

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32USB

struct USBSerialDevice {
    var cdc: USBCDC

    mutating func poll() {
        cdc.processEvents()
    }

    mutating func write(_ text: String) {
        cdc.write(Array(text.utf8))
    }

    mutating func read(into buffer: inout [UInt8]) -> Int {
        cdc.read(into: &buffer)
    }
}

struct HIDKeyboard {
    mutating func keyDown(_ key: HIDKeyCode) {
        var report = HIDKeyboardReport(modifiers: 0, keys: [key.rawValue])
        USBHID.sendReport(report)
    }
}
```

**Ownership:** USB device state as a single struct in static/main scope — avoid multiple `class` owners of endpoint buffers.

**Compile:**

```bash
swift build -c release
espflash flash --monitor .build/release/UsbCdcDemo.bin
```

---

## Bare-Metal Sketch (Concept)

USB device stacks are complex — bare-metal without a stack is impractical. Minimum viable path:

1. Configure USB PHY clocks in TRM
2. Handle reset interrupt on EP0
3. Respond to GET_DESCRIPTOR with prebuilt byte array

Use vendor USB stack via Swift C interop for production.

---

## Example Projects

| Link | Description |
|------|-------------|
| [examples/usb-cdc.md](../examples/usb-cdc.md) | Virtual serial |
| [examples/usb-hid.md](../examples/usb-hid.md) | HID device |
| [projects/usb-keyboard.md](../projects/usb-keyboard.md) | Keyboard gadget |
| [projects/usb-serial-converter.md](../projects/usb-serial-converter.md) | USB ↔ UART |

---

## Common Mistakes

1. **Forgetting to poll USB stack** — device not enumerated.
2. **Wrong descriptor sizes** — host rejects device.
3. **Blocking in USB ISR** — keep ISR minimal.
4. **Drawing too much current** — negotiate with host / self-powered.
5. **Using UART pins instead of native USB** — check DevKit schematic.

---

## Exercises

1. Enumerate as CDC and echo serial input.
2. Send HID keyboard report on button press.
3. Build [projects/usb-serial-converter.md](../projects/usb-serial-converter.md).
4. Capture USB traffic with Wireshark / usbmon.

---

## References

- USB 2.0 Specification (Chapter 9)
- [examples/usb-cdc.md](../examples/usb-cdc.md)

---

*Prev: [can.md](./can.md) | Next: [ethernet.md](./ethernet.md)*

*Back to [Embedded Swift](../README.md)*
