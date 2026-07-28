# Wi-Fi — IEEE 802.11 Wireless LAN

**Wi-Fi** connects embedded devices to IP networks via **2.4 GHz** radio. ESP32-S3 integrates **802.11 b/g/n** with onboard antenna or U.FL connector.

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

Swift paths on ESP32:

- **Embedded Swift + vendor Wi-Fi stack** (C interop via clang module)
- **ESP-IDF** bridge for production stacks

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

## Swift HAL Sketch

```swift
import Embedded
import ESP32WiFi

struct WiFiStation {
    mutating func connect(ssid: String, password: String) async throws {
        try await ESPWiFi.setMode(.station)
        try await ESPWiFi.connect(ssid: ssid, password: password)
        try await ESPWiFi.waitForIP(timeoutSeconds: 30)
    }

    func ipAddress() -> IPv4Address? {
        ESPWiFi.stationIP()
    }
}

// Network stack task — prefer struct coordinator over singleton class
func runNetworkStack() async {
    await ESPNetStack.run()
}
```

**ARC note:** Wi-Fi manager as a `class` is common in SDK wrappers — use `[weak self]` in connection callbacks or wrap in a single `struct AppCoordinator` that owns lifecycle.

**Compile (ESP32-S3):**

```bash
swift build -c release
espflash flash --monitor .build/release/WiFiClient.bin
```

---

## Bare-Metal Sketch (Concept)

Wi-Fi MAC is closed-source binary (Wi-Fi blob) on Espressif — true bare-metal without vendor library is not practical. Low-level interaction goes through **Wi-Fi driver API**.

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
4. **Hard-coded credentials in flash** — use NVS / provisioning.
5. **Ignoring reconnect** — handle AP roam and drop.

---

## Exercises

1. Connect to home AP; print DHCP-assigned IP over USB-CDC.
2. Implement Wi-Fi scan and print SSID list sorted by RSSI.
3. Build [examples/wifi-client.md](../examples/wifi-client.md) with exponential backoff reconnect.
4. Measure current with multimeter during idle vs download.

---

## References

- [Espressif ESP-IDF Wi-Fi guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-guides/wifi.html)
- [examples/wifi-client.md](../examples/wifi-client.md)

---

*Prev: [ethernet.md](./ethernet.md) | Next: [ble.md](./ble.md)*

*Back to [Embedded Swift](../README.md)*
