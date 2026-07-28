# Example: BLE GATT Server

**Goal:** Advertise custom service readable from nRF Connect app.

**Prerequisites:** [21-bluetooth.md](../21-bluetooth.md), [communication/ble.md](../communication/ble.md)

---

## Rust Sketch

```rust
// Illustrative — use esp-wifi BLE or trouble-host per template

const SERVICE_UUID: &str = "0000180f-0000-1000-8000-00805f9b34fb"; // Battery

fn main() -> ! {
    // init BLE stack
    // add characteristic `level: u8`
    // advertise name "ESP32-S3 Node"
    loop {
        stack.poll();
        // update characteristic on sensor read
    }
}
```

### Ownership

Attribute table typically `'static`; connection handles in stack.

See [projects/ble-beacon.md](../projects/ble-beacon.md).

*Next: [http-server.md](./http-server.md)*
