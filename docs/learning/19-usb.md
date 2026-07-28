# Lesson 19 — USB Device (HID, CDC)

**Prerequisites:** [08-gpio.md](./08-gpio.md), [15-uart.md](./15-uart.md)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md) — native USB OTG. Also: RP2040, nRF52840, STM32 with USB FS/HS.

---

## Theory

**USB** (Universal Serial Bus) is a host-driven serial bus. Embedded devices often act as **USB devices** (peripherals) talking to a **USB host** (PC, phone, Raspberry Pi).

### Layers

```
Application (your Rust firmware)
    ↓
Class driver (HID, CDC, MSC, ...)
    ↓
USB device stack (TinyUSB, esp-usb, embassy-usb, usb-device)
    ↓
Controller hardware (USB OTG, USB Device Controller)
    ↓
D+/D- differential pair
```

### Device classes (common in embedded)

| Class | Acronym expansion | Use case |
|-------|-------------------|----------|
| **HID** | Human Interface Device | Keyboard, mouse, custom gamepad, sensor reports |
| **CDC-ACM** | Communications Device Class — Abstract Control Model | Virtual serial port |
| **MSC** | Mass Storage Class | USB flash drive emulation |
| ** MIDI** | Musical Instrument Digital Interface | Controllers (via HID or MIDI class) |

### Endpoints

USB communication happens through **endpoints** — numbered pipes with direction:

| Type | Direction | Example |
|------|-----------|---------|
| **Control EP0** | Both | Enumeration, descriptors |
| **Interrupt IN** | Device → Host | HID reports (low latency) |
| **Bulk IN/OUT** | Both | CDC data (high throughput) |
| **Isochronous** | Both | Audio/video — timing-critical |

Each endpoint has a **max packet size** (e.g., 64 bytes full-speed bulk).

### Enumeration

On plug-in, the host:

1. Resets bus, assigns address on **EP0**.
2. Reads **device descriptor** (VID/PID — Vendor/Product ID).
3. Selects configuration, loads class drivers.
4. Application traffic begins on class endpoints.

---

## Hardware Overview

### ESP32-S3 USB

The ESP32-S3 integrates **USB OTG** with **full-speed** (12 Mbit/s) device (and host) capability:

- **GPIO19** = D−, **GPIO20** = D+ (default; check TRM for strap constraints).
- DevKitC-1 routes USB through a second USB-C port or shared bridge — read board schematic.
- **USB Serial/JTAG** is a separate built-in function on some pins — do not confuse with OTG.

### RP2040

Bootloader appears as **USB MSC** + **CDC** when BOOTSEL held — excellent reference implementation.

### nRF52840

Native USB device; popular with **embassy-usb** for HID/CDC composite devices.

---

## ASCII Wiring

DevKitC-1 typically needs **no external wiring** for USB device on the native port:

```
PC USB Host                    ESP32-S3 DevKitC-1
┌──────────┐                   ┌──────────────────┐
│ USB-C    │◄── cable ────────►│ USB OTG port     │
└──────────┘                   │  D+ / D- internal│
                               └──────────────────┘
```

Self-powered device must share **GND** with host or use proper VBUS sensing.

---

## Memory & Register Notes

### USB descriptors live in flash

```rust
// Device descriptor — read by host at enumeration
#[repr(C)]
struct DeviceDescriptor {
    b_length: u8,
    b_descriptor_type: u8,
    bcd_usb: u16,        // USB version e.g. 0x0200
    b_device_class: u8,
    // ...
    id_vendor: u16,      // VID — get unique for products
    id_product: u16,     // PID
}
```

**VID/PID:** Development often uses test IDs (`0x1209/0x0001` pid.codes); products need unique **USB-IF** vendor ID or licensee.

### ESP32-S3 USB registers

Managed by HAL/stack — direct register work is rare. Key concerns:

- **FIFO sizing** for endpoints
- **Pull-up** on D+ (device attach detection — often automatic)
- **Clock** must be accurate — 48 MHz USB clock domain

---

## HAL Example — CDC-ACM Virtual Serial (embassy-usb on ESP32-S3)

```rust
// embassy-usb + esp-hal pattern (illustrative)
// Cargo.toml: embassy-usb, embassy-sync, esp-hal, static_cell

use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::{Builder, Config, UsbDevice};
use static_cell::StaticCell;

static EP_OUT_BUFFER: StaticCell<[u8; 1024]> = StaticCell::new([0u8; 1024]);
static STATE: StaticCell<State<'static>> = StaticCell::new(State::new());

#[embassy_executor::task]
async fn usb_task(mut class: CdcAcmClass<'static, esp_hal::usb::Driver>) {
    loop {
        class.wait_connection().await;
        let mut buf = [0u8; 64];
        loop {
            let n = class.read_packet(&mut buf).await;
            if n > 0 {
                // Echo back to host terminal
                let _ = class.write_packet(&buf[..n]).await;
            }
        }
    }
}

fn build_usb<'d>(
    driver: esp_hal::usb::Driver<'d>,
) -> (UsbDevice<'d>, CdcAcmClass<'d>) {
    let mut config = Config::new(0x1209, 0x0001); // pid.codes test VID/PID
    config.manufacturer = Some("Learning");
    config.product = Some("ESP32-S3 CDC Echo");
    config.serial_number = Some("001");

    let mut builder = Builder::new(
        driver,
        config,
        EP_OUT_BUFFER.init([0u8; 1024]),
    );

    let class = CdcAcmClass::new(&mut builder, STATE.init(State::new()), 64);
    let usb = builder.build();
    (usb, class)
}
```

Open the serial port on the host (`/dev/ttyACM0`, `COM3`) — behaves like UART — [15-uart.md](./15-uart.md).

---

## HID Custom Device Example (conceptual)

```rust
use embassy_usb::class::hid::{HidReaderWriter, State as HidState, ReportId};
use embassy_usb::class::hid::{HidClass, Protocol, SubClass};

// 8-byte report: [buttons, x, y, wheel, ...]
#[repr(C)]
struct MouseReport {
    buttons: u8,
    x: i8,
    y: i8,
    wheel: i8,
}

// HidClass::new with report descriptor bytes from `rusb`/`hid-report-builder`
// On timer tick: send MouseReport when inputs change
```

HID uses **interrupt IN** endpoint — host polls at interval in descriptor (e.g., every 10 ms).

---

## Bare-Metal / Low-Level Notes

Without a stack, you must handle:

- Setup packets on EP0
- Standard requests (`GET_DESCRIPTOR`, `SET_ADDRESS`, `SET_CONFIGURATION`)
- Endpoint halts, stalls

**Never implement from scratch for production** — use `embassy-usb`, `esp-usb`, `tinyusb` bindings, or `usb-device`.

---

## Step-by-Step

1. Confirm board USB port wired to **OTG**, not only UART bridge.
2. Add `embassy-usb` (or platform stack) to `Cargo.toml`.
3. Build **CDC echo** example; flash and plug into PC.
4. Verify enumeration in `dmesg` / Device Manager — no "unknown device".
5. Open serial terminal on CDC interface; test echo.
6. Add **HID** interface for button reports — composite device.
7. Measure throughput and latency vs UART.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Wrong USB port on board | No enumeration | Use OTG-capable connector |
| Missing 48 MHz USB clock | Descriptor read fails | Fix clock init in HAL |
| EP0 buffer too small | Enumeration stall | Follow stack defaults |
| Duplicate VID/PID conflict | Host confusion | Unique PID per project |
| Blocking in USB IRQ | Deadlock | Defer to async task |
| Full-speed on HS hub only | Works but slow | Expected for FS device |
| D+/D- swapped | No attach | Swap or fix PCB |

---

## Debugging Tips

- **`lsusb -v`** (Linux) — descriptors, endpoints, configurations.
- **Wireshark + usbmon** — capture control transfers.
- Windows: **USB Device Tree Viewer** — configuration errors.
- Log **USB events** (reset, suspend, resume) via defmt.
- Compare descriptor hex against **USBlyzer** / **TinyUSB** reference.

---

## Performance Tips

- **Bulk CDC** for throughput (firmware updates, logs).
- **HID interrupt** for low-latency input (< 1 ms poll in descriptor).
- Double-buffer bulk endpoints on hardware that supports it.
- **Composite device:** CDC + HID in one firmware — one USB plug.
- Suspend handling — [24-low-power.md](./24-low-power.md) — detach/resume cleanly.

---

## Exercises

1. **CDC shell:** Parse `help` over USB serial; control LED.
2. **HID volume knob:** Rotary encoder sends consumer control reports.
3. **MSC readonly:** Expose small config file from flash (advanced).
4. **Descriptor dump:** Print EP0 setup packets in debug build.
5. **Port to RP2040:** Same CDC echo on Pico with `embassy-usb` — [boards/rp2040.md](./boards/rp2040.md).

---

## References

- [USB 2.0 specification (USB-IF)](https://www.usb.org/documents)
- [embassy-usb book](https://embassy.dev/book/)
- [ESP32-S3 USB OTG (ESP-IDF)](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/peripherals/usb_device.html)
- [pid.codes test VID](https://pid.codes/)
- [22-embassy.md](./22-embassy.md)

---

*Previous: [18-can.md](./18-can.md) · Next: [20-wifi.md](./20-wifi.md)*
