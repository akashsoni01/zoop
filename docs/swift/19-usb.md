# Lesson 19 — USB Device (HID, CDC)

**Prerequisites:** [08-gpio.md](./08-gpio.md), [15-uart.md](./15-uart.md)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md) — native USB OTG. Also: RP2040, nRF52840, STM32.

**Maturity note:** **USB device stacks in pure Embedded Swift are immature.** ESP32-S3 USB via **ESP-IDF TinyUSB** (C) with Swift application logic is the practical path. **Host-side Swift** (macOS/iOS) for USB device testing uses **IOKit** / **USBSerial**. RP2040 and nRF52840 have community Embedded Swift blink examples but not full USB class drivers yet.

---

## Theory

**USB** is a host-driven serial bus. Embedded devices act as **USB devices** (peripherals) to a **USB host** (PC, phone).

### Layers

```
Application (Embedded Swift firmware)
    ↓
Class driver (HID, CDC, MSC)
    ↓
USB device stack (TinyUSB, esp-usb)
    ↓
Controller hardware (USB OTG)
    ↓
D+/D- differential pair
```

### Device classes

| Class | Use case |
|-------|----------|
| **HID** | Keyboard, mouse, gamepad, custom sensors |
| **CDC-ACM** | Virtual serial port |
| **MSC** | USB flash drive emulation |

### Endpoints

| Type | Direction | Example |
|------|-----------|---------|
| **Control EP0** | Both | Enumeration |
| **Interrupt IN** | Device → Host | HID reports |
| **Bulk IN/OUT** | Both | CDC data |

### Enumeration

1. Host resets bus, assigns address on EP0.
2. Reads device descriptor (VID/PID).
3. Selects configuration, loads class drivers.
4. Application traffic on class endpoints.

---

## Hardware Overview

### ESP32-S3 USB

- **USB OTG** full-speed (12 Mbit/s).
- **GPIO19** = D−, **GPIO20** = D+ (default; strap-sensitive).
- **USB Serial/JTAG** is separate from OTG — don't confuse them.

### RP2040 / nRF52840

Native USB device; RP2040 BOOTSEL appears as MSC + CDC — excellent reference.

---

## ASCII Wiring

### ESP32-S3 USB OTG to PC

```
PC USB-C port          ESP32-S3 DevKitC-1
┌──────────┐           ┌──────────────┐
│ D+ ◄─────┼───────────┤ GPIO20 (D+)  │
│ D- ◄─────┼───────────┤ GPIO19 (D-)  │
│ GND ─────┼───────────┤ GND          │
│ 5V  ─────┼───────────┤ 5V (optional)│
└──────────┘           └──────────────┘
```

Use a **data-capable** USB-C cable. Some DevKit boards route OTG to a dedicated port — check schematic.

---

## Memory & Register Notes

### USB descriptors (conceptual)

| Descriptor | Contents |
|------------|----------|
| Device | VID, PID, USB version, class |
| Configuration | Power, interface count |
| Interface | Class, subclass, protocol |
| Endpoint | Address, max packet size, type |
| HID report | Input/output report sizes |

Descriptor bytes typically live in **flash** as `const` arrays.

```swift
let deviceDescriptor: [UInt8] = [
    0x12, 0x01, // bLength, bDescriptorType (Device)
    0x00, 0x02, // bcdUSB 2.0
    // ... VID, PID, endpoints
]
```

---

## HAL Swift Example — CDC Serial (ESP-IDF + TinyUSB)

```swift
import ESPIDF

@main
struct UsbCdcApp {
    static func main() {
        let usb = USBDevice(
            vid: 0x303A,  // Espressif example VID — use your own for product
            pid: 0x4001,
            classes: [.cdcACM]
        )

        usb.start()

        print("USB CDC ready — open /dev/ttyACM* on host\r\n")

        while true {
            if let line = usb.cdc.readLine() {
                usb.cdc.write("Echo: \(line)\r\n")
            }
        }
    }
}
```

Configure ESP-IDF `CONFIG_TINYUSB_CDC_ENABLED=y` in `sdkconfig`.

---

## Bare-Metal Swift Sketch (illustrative)

Full USB stack in bare-metal Swift is **not recommended yet** — enumeration alone is thousands of lines. If exploring:

```swift
// Illustrative — EP0 setup packet handler outline
struct SetupPacket {
    var bmRequestType: UInt8
    var bRequest: UInt8
    var wValue: UInt16
    var wIndex: UInt16
    var wLength: UInt16
}

func handleSetup(_ setup: SetupPacket) {
    switch (setup.bmRequestType, setup.bRequest) {
    case (0x00, 0x05): // SET_ADDRESS
        break
    case (0x80, 0x06): // GET_DESCRIPTOR
        break
    default:
        break
    }
}
```

Use **TinyUSB** via ESP-IDF until a Swift-native stack exists.

---

## Host Swift — USB Serial on macOS

Test your CDC firmware from a macOS companion app:

```swift
import Foundation
import ORSSerial

class USBSerialMonitor: ORSSerialPortDelegate {
    let port: ORSSerialPort

    init(path: String) {
        port = ORSSerialPort(path: path)!
        port.baudRate = 115_200 // ignored for USB CDC but required by API
        port.delegate = self
    }

    func start() {
        port.open()
        port.sendData("hello\n".data(using: .utf8)!)
    }

    func serialPort(_ serialPort: ORSSerialPort, didReceive data: Data) {
        print("Device:", String(data: data, encoding: .utf8) ?? "")
    }
}
```

For HID devices, use **IOHIDManager** on macOS — see [21-bluetooth.md](./21-bluetooth.md) for BLE HID parallels.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| D+/D- swapped | Not enumerated | Swap data lines |
| Missing pull-up on D+ (FS device) | Host ignores device | 1.5 kΩ to 3.3 V (often internal) |
| Wrong endpoint max packet size | STALL on transfer | 64 bytes for FS bulk |
| Confusing JTAG port with OTG | Wrong driver | Use correct USB port on DevKit |
| Duplicate VID/PID | Driver conflict | Unique PID per product |
| HID report descriptor errors | Device seen but no input | Validate with `hid-report-parser` tools |

---

## Debugging Tips

- **macOS System Information → USB** — verify enumeration and descriptors.
- Wireshark **USB capture** (with appropriate hardware) for packet-level debug.
- Compare descriptor hex against working TinyUSB example.
- `ls /dev/tty.usb*` (macOS) or `ls /dev/ttyACM*` (Linux) after plug-in.

---

## Performance Tips

- **Interrupt IN** for low-latency HID; **bulk** for high-throughput CDC.
- Double-buffer OUT endpoints for CDC to avoid NAK storms.
- Composite device (CDC + HID) saves one USB port — plan interface layout early.
- Keep EP0 handler fast — defer work to main loop.

---

## Exercises

1. **CDC echo:** Virtual serial echo on ESP32-S3 OTG.
2. **macOS terminal:** Open CDC port; send commands from Swift host app.
3. **HID button:** Send one-byte report on GPIO press (via TinyUSB C + Swift handler).
4. **Descriptor dump:** Print parsed descriptors from macOS IOKit.
5. **Composite:** CDC + HID on one device — verify both interfaces on host.

---

## References

- [ESP32-S3 TRM — USB OTG](https://www.espressif.com/en/products/socs/esp32-s3)
- [TinyUSB](https://docs.tinyusb.org/)
- [ESP-IDF USB Device Stack](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/peripherals/usb_device.html)
- [25-debugging.md](./25-debugging.md)

---

*Previous: [18-can.md](./18-can.md) · Next: [20-wifi.md](./20-wifi.md)*
