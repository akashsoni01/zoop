# Example: OTA Firmware Update

**Goal:** Download new firmware over HTTP/HTTPS and swap partitions.

**Prerequisites:** [http-server.md](./http-server.md), [wifi-client.md](./wifi-client.md)

---

## Rust Sketch

```rust
// ESP-IDF style OTA — esp-idf-svc or esp-hal ota partition API

async fn ota_update(url: &str) -> Result<(), OtaError> {
    let mut client = HttpClient::new();
    let mut response = client.get(url).await?;
    let mut ota = OtaUpdate::begin()?;
    let mut buf = [0u8; 4096];
    while let Ok(n) = response.read(&mut buf).await {
        if n == 0 { break; }
        ota.write(&buf[..n])?;
    }
    ota.finish()?;
    esp_hal::system::restart();
}

// Ownership: Ota partition handle exclusive during write — do not run from two tasks
```

Validate image signature in production. See Espressif secure boot docs.

*Next: [usb-hid.md](./usb-hid.md)*
