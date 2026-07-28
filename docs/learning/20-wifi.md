# Lesson 20 — Wi-Fi on ESP (Station Mode, TLS)

**Prerequisites:** [02-no-std.md](./02-no-std.md), [09-interrupts.md](./09-interrupts.md), [22-embassy.md](./22-embassy.md) (recommended)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md), [esp32.md](./boards/esp32.md). Brief notes on ESP32-C6 (Wi-Fi 6).

---

## Theory

**Wi-Fi** (IEEE 802.11 wireless LAN) lets MCUs connect to access points (APs) or create their own hotspot. On Espressif chips, Wi-Fi is handled by a **binary blob** running on a dedicated processor alongside your Rust app — not bit-banged from GPIO.

### Modes

| Mode | Role | Typical use |
|------|------|-------------|
| **Station (STA)** | Client to home router | IoT sensor → cloud |
| **SoftAP** | Access point | Captive portal provisioning |
| **STA+AP** | Both simultaneously | Configure while connected |

This lesson focuses on **station mode**: join an existing network, obtain IP, speak TCP/TLS.

### Stack architecture (ESP)

```
Your Rust app (esp-std / esp-idf-svc / esp-wifi)
        ↓
ESP-IDF Wi-Fi driver API (C)
        ↓
Wi-Fi binary (closed-source MAC/baseband)
        ↓
Radio front-end (2.4 GHz; 5 GHz on some chips)
```

Rust options:

| Crate / approach | Runtime | Notes |
|------------------|---------|-------|
| **esp-idf-svc** | `std` on ESP-IDF | Mature Wi-Fi + MQTT + HTTP |
| **esp-wifi** | `no_std` + alloc | Community Wi-Fi stack for esp-hal |
| **embassy-net** + esp-wifi | async | Growing ecosystem |

See [boards/esp32-s3.md](./boards/esp32-s3.md) for radio coexistence with Bluetooth.

### TLS (Transport Layer Security)

**TLS** encrypts TCP connections (HTTPS, MQTTS). On MCUs:

- Use **mbedTLS** (via ESP-IDF) or **`rustls`** (pure Rust, needs entropy + time).
- Store **CA certificates** in flash — embed with `include_bytes!`.
- **Certificate validation** requires correct system time — sync via **NTP** (Network Time Protocol) first.

---

## Hardware Overview

### ESP32-S3 Wi-Fi

- **2.4 GHz** 802.11 b/g/n
- Integrated antenna or U.FL on some modules
- **Coexistence** with BLE — time-sliced by controller

### ESP32-C6

- Adds **Wi-Fi 6 (802.11ax)** on 2.4 GHz — see [boards/esp32-c6.md](./boards/esp32-c6.md).

### Non-Espressif

Wi-Fi on STM32/nRF usually means **external module** (ESP8266 AT commands, WIZnet, RW612) — different lesson path.

---

## ASCII Wiring

No external wiring for dev — antenna is on-module:

```
┌─────────────────────┐
│  ESP32-S3 DevKitC-1 │
│  ┌───────────────┐  │
│  │ PCB antenna   │  │  ← keep clear of metal enclosure
│  └───────────────┘  │
│  USB-C (power/flash)│
└─────────────────────┘
        │
        ▼
   Home Wi-Fi Router (2.4 GHz SSID)
```

For metal enclosures, use **IPEX/U.FL** external antenna variant and RF keep-out rules from Espressif layout guides.

---

## Memory & Register Notes

### RAM budget

Wi-Fi stacks are **RAM-heavy**:

| Component | Approx RAM |
|-----------|------------|
| Wi-Fi driver | 50–100+ KB |
| LWIP TCP/IP | 20–40 KB |
| TLS session | 20–60 KB |
| App heap | remainder |

ESP32-S3 with **8 MB flash / 512 KB SRAM** is comfortable with `std`; PSRAM helps large buffers.

### Credentials storage

**Never hardcode** production Wi-Fi passwords in git:

```rust
// Development only — use NVS / provisioning in production
const WIFI_SSID: &str = env!("WIFI_SSID");
const WIFI_PASS: &str = env!("WIFI_PASS");
```

Use **`secrets.example.md`** patterns from your project docs or ESP **NVS** (Non-Volatile Storage).

### Wi-Fi config (conceptual)

Managed by ESP-IDF — not direct MMIO registers in application code. Key structs: `wifi_config_t`, `wifi_sta_config_t`.

---

## Example — Station connect (esp-idf-svc)

```rust
use esp_idf_svc::wifi::{ClientConfiguration, Configuration, WifiDriver};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::hal::peripherals::Peripherals;
use esp_idf_svc::wifi::AuthMethod;

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take()?;
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    let mut wifi = WifiDriver::new(
        peripherals.modem,
        sys_loop.clone(),
        Some(nvs),
    )?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: "YourSSID".into(),
        password: "YourPassword".into(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))?;

    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?; // Blocks until DHCP IP assigned

    log::info!("Wi-Fi connected!");
    log::info!("IP: {:?}", esp_idf_svc::ifconfig::get_ip_info("sta")?);

    // Now TCP/HTTP/MQTT...
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
```

Build with `esp-idf` toolchain; set SSID/password via sdkconfig or runtime provisioning.

---

## Example — HTTPS fetch with TLS notes (esp-idf-svc)

```rust
use esp_idf_svc::http::client::{Configuration, EspHttpConnection};
use embedded_svc::http::client::Client;

fn fetch_https() -> anyhow::Result<()> {
    // 1. Sync time via SNTP before cert validation
    // esp_idf_svc::sntp::EspSntp::new_default()?.wait_for_sync();

    let config = Configuration {
        use_global_ca_store: true, // or embed CA with crt_bundle_attach
        ..Default::default()
    };

    let connection = EspHttpConnection::new(&config)?;
    let mut client = Client::wrap(connection);

    let request = client.request(
        embedded_svc::http::Method::Get,
        "https://example.com/",
        &[],
    )?;
    let mut response = request.submit()?;

    let mut buf = [0u8; 256];
    while let n = response.read(&mut buf)? {
        // process chunk
        let _ = n;
    }
    Ok(())
}
```

**TLS pitfalls:** expired cert without valid time; missing CA; SNI (Server Name Indication) required by many hosts.

---

## esp-wifi + embassy (no_std direction)

```rust
// Illustrative — check esp-wifi book for current API
// esp-wifi + embassy-executor on ESP32-S3

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(...);
    let wifi = esp_wifi::init(...).await;
    let config = esp_wifi::wifi::Configuration::Client(
        esp_wifi::wifi::ClientConfiguration {
            ssid: "YourSSID".try_into().unwrap(),
            password: "YourPassword".try_into().unwrap(),
            ..Default::default()
        },
    );
    wifi.set_configuration(&config).await;
    wifi.connect().await.expect("connect failed");
    // embassy-net stack for TCP
}
```

---

## Step-by-Step

1. Flash **Wi-Fi scan example** — confirm SSIDs visible (RF working).
2. Connect to home AP with WPA2 — log IP address.
3. **Ping** gateway from example or TCP connect to `example.com:80`.
4. Enable **SNTP**; verify UTC time in logs.
5. HTTPS GET with bundled CA store.
6. Move credentials to **NVS** or BLE provisioning — [21-bluetooth.md](./21-bluetooth.md).

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| 5 GHz-only SSID | Scan misses network | Use 2.4 GHz SSID |
| Wrong auth mode | `Auth fail` | Match WPA2/WPA3/Open |
| No SNTP before TLS | Cert verify fail | Sync time first |
| Heap too small | Connect then panic | Increase heap in sdkconfig |
| Antenna blocked | Weak RSSI | Layout / external antenna |
| Power supply weak | Brownout on TX | ≥500 mA USB supply |
| Country code unset | Limited channels | Set `wifi_country_t` |

---

## Debugging Tips

- Log **RSSI** (Received Signal Strength Indicator) and disconnect reason codes.
- **`esp_wifi_scan_get_ap_records`** — channel, auth, RSSI.
- Wireshark on AP mirror port for over-the-air (advanced).
- Enable **Wi-Fi debug logs** temporarily — verbose but diagnostic.
- Test with phone hotspot to isolate router issues.

---

## Performance Tips

- **Power save:** modem sleep between bursts — [24-low-power.md](./24-low-power.md).
- Reuse **TLS session** / single long-lived MQTT connection vs repeated handshakes.
- Prefer **MQTT** over polling HTTPS for IoT telemetry.
- Fix Wi-Fi channel congestion — scan and pick quiet channel for SoftAP.
- Use **async** (`embassy-net`) to overlap I/O and sensor reads.

---

## Exercises

1. **RSSI logger:** Connect STA; print signal strength every 5 s.
2. **Reconnect policy:** Exponential backoff on disconnect.
3. **Captive portal:** SoftAP + HTTP server to receive SSID/password form.
4. **HTTPS weather:** Parse JSON from a public API after SNTP sync.
5. **Compare stacks:** Note RAM/flash diff between esp-idf-svc and esp-wifi hello.

---

## References

- [Espressif Rust Book — Wi-Fi](https://esp-rs.github.io/book/)
- [esp-wifi repository](https://github.com/esp-rs/esp-wifi)
- [ESP-IDF Wi-Fi API](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/network/esp_wifi.html)
- [embassy-net](https://embassy.dev/book/#_embassy_net)
- [21-bluetooth.md](./21-bluetooth.md) — coexistence
- [24-low-power.md](./24-low-power.md)

---

*Previous: [19-usb.md](./19-usb.md) · Next: [21-bluetooth.md](./21-bluetooth.md)*
