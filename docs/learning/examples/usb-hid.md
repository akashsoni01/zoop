# Example: USB HID Device

**Goal:** Emulate keyboard or consumer control device over USB.

**Prerequisites:** [19-usb.md](../19-usb.md), [communication/usb.md](../communication/usb.md)

---

## Rust Sketch

```rust
use usbd_hid::hid_class::HIDClass;
use usbd_hid::descriptor::{KeyboardReport, KeyboardUsage};

fn poll_usb(dev: &mut UsbDevice, hid: &mut HIDClass) {
    dev.poll(&mut [hid]);
    if hid.ready() {
        let report = KeyboardReport {
            keycodes: [KeyboardUsage::A as u8, 0, 0, 0, 0, 0],
            ..Default::default()
        };
        hid.write(&report).ok();
    }
}
```

See [projects/usb-keyboard.md](../projects/usb-keyboard.md).

*Next: [usb-cdc.md](./usb-cdc.md)*
