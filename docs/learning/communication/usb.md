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

## Rust HAL Sketch (esp-hal / esp-usb)

```rust
// Illustrative — API evolves; check esp-rs book for current crate
use usb_device::prelude::*;
use usbd_serial::SerialPort;

pub fn usb_cdc_device<'d, B: UsbBus>(
    usb_bus: &'d UsbBusAllocator<B>,
) -> UsbDevice<'d, B> {
    let serial = SerialPort::new(usb_bus);
    let mut dev = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x303a, 0x1001))
        .product("ESP32-S3 Serial")
        .device_class(0x02)
        .build();
    // poll in main loop: dev.poll(&mut [&mut serial])
    dev
}
```

**Ownership:** `UsbDevice` owns endpoint allocators; `SerialPort` holds bulk endpoints — static lifetime typical in `no_std`.

**Compile:**

```toml
usb-device = "0.3"
usbd-serial = "0.2"
# esp-idf or esp-hal USB feature flags per template
```

---

## Bare-Metal Sketch (Concept)

USB device stacks are complex — bare-metal without a stack is impractical. Minimum viable path:

1. Configure USB PHY clocks in TRM
2. Handle reset interrupt on EP0
3. Respond to GET_DESCRIPTOR with prebuilt byte array

Use `esp-usb` or ESP-IDF VFS for production.

---

## Example Projects

| Link | Class |
|------|-------|
| [examples/usb-cdc.md](../examples/usb-cdc.md) | Virtual serial |
| [examples/usb-hid.md](../examples/usb-hid.md) | HID device |
| [projects/usb-keyboard.md](../projects/usb-keyboard.md) | Keyboard |
| [projects/usb-mouse.md](../projects/usb-mouse.md) | Mouse |
| [projects/usb-serial-converter.md](../projects/usb-serial-converter.md) | UART bridge |

---

## Common Mistakes

1. **Missing D+ pull-up** — host never enumerates device.
2. **Not polling `UsbDevice::poll`** — transfers stall.
3. **Wrong descriptor lengths** — enumeration fails silently.
4. **Drawing > 500 mA** without negotiation — host resets port.
5. **Blocking in interrupt context** during USB ISR.

---

## Exercises

1. Enumerate CDC device on Linux (`dmesg`, `/dev/ttyACM0`).
2. Implement [examples/usb-hid.md](../examples/usb-hid.md) consumer control (volume).
3. Measure current draw during active bulk transfer.
4. Compare USB-CDC vs UART debug latency.

---

## References

- [USB 2.0 Specification](https://www.usb.org/documents)
- [Espressif Rust Book — USB](https://esp-rs.github.io/book/)
- [usb-device crate](https://docs.rs/usb-device/latest/usb_device/)
- Lesson: [09-interrupts.md](../09-interrupts.md)

---

*Prev: [can.md](./can.md) | Next: [ethernet.md](./ethernet.md)*
