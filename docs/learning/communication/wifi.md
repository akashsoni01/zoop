# Wi-Fi — IEEE 802.11 Wireless LAN

**Wi-Fi** connects embedded devices to IP networks via **2.4 GHz** and/or **5 GHz** radio. ESP32-S3 integrates **802.11 b/g/n** with onboard antenna or U.FL connector.

**Prerequisites:** [04-cargo.md](../04-cargo.md), [http.md](./http.md) / [mqtt.md](./mqtt.md) for application layer

---

## Theory

Operating modes:

| Mode | Description |
|------|-------------|
| **STA** (Station) | Client — joins existing AP |
| **AP** (Access Point) | Creates network for others |
| **STA+AP** | Concurrent (limited throughput) |

Connection flow: **Scan → Associate → 4-way handshake (WPA2) → DHCP → IP**.

Rust paths on ESP32:

- **esp-wifi** (bare-metal / esp-hal ecosystem)
- **ESP-IDF** via `std` (`esp-idf-svc`)

Coexistence: Wi-Fi + BLE share radio — schedule carefully.

---

## Timing Diagram (ASCII)

802.11 frame (simplified — data frame):

```
┌──────────┬─────────┬──────────┬─────────┬──────┬─────────┐
│ Preamble │ Header  │ MAC hdr  │ Payload │ MIC  │ FCS     │
└──────────┴─────────┴──────────┴─────────┴──────┴─────────┘
  (PHY)      (PHY)     Addr1/2/3   LLC/IP     (WPA)  CRC

Channel access (DCF — backoff before transmit):

Busy ━━━━━━━━━━━━━━━━━━━┓
                        ┗━━ idle ━━ [random backoff slots] ━━ TX
```

Power save (DTIM): STA wakes periodically to receive buffered frames.

---

## Packet Format

**802.11 MAC header (key fields):**

| Field | Size | Notes |
|-------|------|-------|
| Frame Control | 2 B | Type/subtype |
| Duration | 2 B | NAV |
| Addr1 | 6 B | Destination |
| Addr2 | 6 B | Source |
| Addr3 | 6 B | BSSID |
| Sequence | 2 B | Fragment/retry |
| Body | 0–2312 B | LLC/SNAP + IP |

Above Wi-Fi: **IP → TCP/UDP → application** (HTTP, MQTT).

---

## Electrical Characteristics

| Parameter | ESP32-S3 |
|-----------|----------|
| TX power | up to +20 dBm (region limited) |
| Antenna | PCB trace or external via U.FL |
| Current | 150–300 mA peak TX — size power supply |
| 5 GHz | **Not supported** on ESP32-S3 (2.4 GHz only) |

Keep ground plane solid under antenna area per hardware design guide.

---

## Rust HAL Sketch (esp-wifi + embassy)

```rust
use embassy_net::{Stack, Config as NetConfig, Ipv4Cidr};
use esp_wifi::wifi::{ClientConfiguration, Configuration, WifiController};

// Ownership: `Stack` owns network interface; `'static` spawn for background task

#[embassy_executor::task]
async fn net_task(stack: Stack<'static>) {
    stack.run().await;
}

pub async fn connect_wifi(
    wifi: &mut WifiController<'static>,
    ssid: &str,
    password: &str,
) {
    let config = Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().unwrap(),
        password: password.try_into().unwrap(),
        ..Default::default()
    });
    wifi.set_configuration(&config).unwrap();
    wifi.start().unwrap();
    wifi.connect().unwrap();
    // wait for DHCP via stack.wait_config_up()
}
```

**Compile (ESP32-S3):**

```toml
esp-wifi = { version = "0.9", features = ["esp32s3", "wifi"] }
embassy-net = { version = "0.4", features = ["dhcpv4", "tcp", "udp"] }
```

Requires `xtensa-esp32s3-none-elf` + `espup` toolchain.

---

## Bare-Metal Sketch (Concept)

Wi-Fi MAC is closed-source binary (Wi-Fi blob) on Espressif — true bare-metal without vendor library is not practical. Low-level interaction goes through **Wi-Fi driver API** (IDF or esp-wifi).

---

## Example Projects

| Link | Description |
|------|-------------|
| [examples/wifi-client.md](../examples/wifi-client.md) | STA connect |
| [examples/http-server.md](../examples/http-server.md) | Web UI |
| [examples/mqtt-client.md](../examples/mqtt-client.md) | Cloud publish |
| [projects/wifi-sensor.md](../projects/wifi-sensor.md) | Telemetry node |
| [projects/iot-gateway.md](../projects/iot-gateway.md) | Protocol bridge |

---

## Common Mistakes

1. **Insufficient power supply** — brownout during TX spikes.
2. **Wrong country code / TX power** — regulatory violations.
3. **Blocking connect in ISR** — use async task.
4. **Hard-coded credentials in flash** — use NVS / provisioning ([projects/smart-home-node.md](../projects/smart-home-node.md)).
5. **Ignoring reconnect** — handle AP roam and drop.

---

## Exercises

1. Connect to home AP; print DHCP-assigned IP via `defmt`.
2. Implement Wi-Fi scan and print SSID list sorted by RSSI.
3. Build [examples/wifi-client.md](../examples/wifi-client.md) with exponential backoff reconnect.
4. Measure current with multimeter during idle vs download.

---

## References

- [Espressif Rust Book — Wi-Fi](https://esp-rs.github.io/book/)
- [esp-wifi docs](https://docs.rs/esp-wifi/latest/esp_wifi/)
- IEEE 802.11-2020 (reference)
- Lesson: [04-cargo.md](../04-cargo.md)

---

*Prev: [ethernet.md](./ethernet.md) | Next: [ble.md](./ble.md)*
