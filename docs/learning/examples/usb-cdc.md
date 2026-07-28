# Example: USB CDC (Virtual Serial)

**Goal:** USB serial port for logging without UART adapter.

**Prerequisites:** [19-usb.md](../19-usb.md), [communication/usb.md](../communication/usb.md)

---

## Rust Sketch

```rust
use usbd_serial::SerialPort;

fn main() -> ! {
    let usb = UsbDeviceBuilder::new(/* ... */).build();
    let mut serial = SerialPort::new(&usb_bus);

    loop {
        usb.poll(&mut [&mut serial]);
        let mut buf = [0u8; 64];
        if serial.read(&mut buf).ok() > 0 {
            serial.write(b"echo\r\n").ok();
        }
    }
}
```

DevKitC-1 may already expose JTAG CDC — avoid duplicate descriptors.

See [projects/usb-serial-converter.md](../projects/usb-serial-converter.md).

*Next: [file-system.md](./file-system.md)*
