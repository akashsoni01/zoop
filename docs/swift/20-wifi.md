# Lesson 20 — Wi-Fi on ESP (Station Mode, TLS)

**Prerequisites:** [02-no-stdlib.md](./02-no-stdlib.md), [09-interrupts.md](./09-interrupts.md), [22-swift-concurrency.md](./22-swift-concurrency.md) (recommended)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md), [esp32.md](./boards/esp32.md). Notes on ESP32-C6 (Wi-Fi 6).

**Maturity note:** **Wi-Fi requires the Espressif binary MAC/baseband.** Embedded Swift Wi-Fi today means **Swift application code + ESP-IDF Wi-Fi C stack** (FreeRTOS underneath). Pure bare-metal Swift Wi-Fi is **not feasible** without closed blobs. **Host Swift** (`Network` framework) complements MCU firmware for gateway apps and provisioning UIs.

---

## Theory

**Wi-Fi** (IEEE 802.11) lets MCUs join access points or create hotspots. On Espressif chips, Wi-Fi runs on a **dedicated processor** alongside your app — not bit-banged from GPIO.

### Modes

| Mode | Role |
|------|------|
| **Station (STA)** | Client to home router — IoT sensor → cloud |
| **SoftAP** | Access point — captive portal provisioning |
| **STA+AP** | Both — configure while connected |

This lesson focuses on **station mode**: join network, obtain IP, speak TCP/TLS.

### Stack architecture (ESP)

```
Embedded Swift app (ESP-IDF bindings)
        ↓
ESP-IDF Wi-Fi driver API (C)
        ↓
Wi-Fi binary (closed-source MAC/baseband)
        ↓
Radio front-end (2.4 GHz)
```

| Approach | Runtime | Notes |
|----------|---------|-------|
| **Embedded Swift + ESP-IDF** | FreeRTOS + ESP-IDF | **Recommended today** |
| **Bare-metal Swift** | No OS | **No Wi-Fi** — radio blobs require RTOS |
| **Host Swift (iOS/macOS)** | Network.framework | Gateway, provisioning, testing |

### TLS

Encrypt TCP (HTTPS, MQTTS). On ESP-IDF:

- **mbedTLS** via ESP-IDF — mature on device.
- Store **CA certificates** in flash as `[UInt8]` constants.
- **Certificate validation** needs correct time — sync **NTP** first.

---

## Hardware Overview

### ESP32-S3 Wi-Fi

- 802.11 b/g/n 2.4 GHz
- Integrated or U.FL antenna
- **Coexistence** with BLE — time-sliced by controller

### ESP32-C6

- Adds **Wi-Fi 6 (802.11ax)** on 2.4 GHz — [boards/esp32-c6.md](./boards/esp32-c6.md).

---

## ASCII Wiring

No external wiring — antenna on-module:

```
┌─────────────────────┐
│  ESP32-S3 DevKitC-1 │
│  ┌───────────────┐  │
│  │ PCB Antenna   │  │  ← keep clear of metal enclosure
│  └───────────────┘  │
│  USB-C (power/flash)│
└─────────────────────┘
```

For metal enclosures, use module with **U.FL** connector and external antenna.

---

## Memory & Register Notes

Wi-Fi buffers live in **internal RAM** — Wi-Fi stack consumes a large fraction of ESP32-S3's 512 KB SRAM. Monitor with ESP-IDF heap APIs.

```swift
// Embed CA cert in flash
let caCert: [UInt8] = [
    // PEM or DER bytes from include_file at build time
]
```

NVS (Non-Volatile Storage) holds Wi-Fi credentials — partition in flash.

---

## HAL Swift Example — Station + HTTPS (ESP-IDF)

```swift
import ESPIDF

@main
struct WiFiClientApp {
    static func main() async {
        // Initialize TCP/IP stack
        ESPNetif.init()
        ESPWiFi.init()

        // Connect to AP
        try await ESPWiFi.connect(
            ssid: "YourNetwork",
            password: "YourPassword"
        )

        print("Connected, IP: \(ESPNetif.staIP())")

        // NTP for TLS time validation
        try await ESPNTP.sync()

        // HTTPS GET
        let client = HTTPSClient(caCert: caCert)
        let response = try await client.get(
            url: "https://api.example.com/sensor"
        )
        print("Response: \(response.statusCode)")
    }
}
```

Use Swift **async/await** with ESP-IDF event loop integration — [22-swift-concurrency.md](./22-swift-concurrency.md).

---

## Bare-Metal Swift Sketch

**Not applicable for Wi-Fi.** Document this limitation explicitly in project README. Alternatives:

1. **ESP-IDF + Embedded Swift** (this lesson).
2. **External Wi-Fi module** (AT commands over UART) — Swift parses UART, module handles Wi-Fi.
3. **Host gateway** — MCU on SPI/UART; iOS/macOS app uses `Network` framework.

---

## Host Swift — Network Framework (gateway / companion)

```swift
import Foundation
import Network

func fetchSensorData() async throws -> Data {
    let connection = NWConnection(
        host: "192.168.1.100",
        port: 8080,
        using: .tcp
    )

    return try await withCheckedThrowingContinuation { continuation in
        connection.stateUpdateHandler = { state in
            if case .ready = state {
                connection.receive(minimumIncompleteLength: 1,
                                   maximumLength: 4096) { data, _, _, error in
                    if let error { continuation.resume(throwing: error) }
                    else if let data { continuation.resume(returning: data) }
                }
            }
        }
        connection.start(queue: .global())
    }
}
```

Build provisioning apps on iOS that send credentials to MCU over BLE — [21-bluetooth.md](./21-bluetooth.md).

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Wrong Wi-Fi credentials | Endless reconnect | Log disconnect reason codes |
| No NTP before TLS | Cert validation fails | Sync time first |
| ADC2 + Wi-Fi on ESP32 | ADC reads fail | Use ADC1 only when radio active |
| Insufficient heap | `ESP_ERR_NO_MEM` | Reduce buffers; enable PSRAM carefully |
| 5 GHz-only network | Won't connect | ESP32 is 2.4 GHz only |
| Blocking Wi-Fi in ISR | Watchdog reset | Wi-Fi only from task/main context |

---

## Debugging Tips

- `ESP_LOGI` / UART log Wi-Fi events: `WIFI_EVENT_STA_DISCONNECTED`, reason code.
- `esp_wifi_scan_get_ap_records` — verify SSID visible.
- Wireshark on AP side — see if device associates.
- Check **coexistence** if BLE + Wi-Fi together — [21-bluetooth.md](./21-bluetooth.md).

---

## Performance Tips

- **Modem sleep** when throughput low — [24-low-power.md](./24-low-power.md).
- Reuse TCP connections; TLS handshakes are expensive.
- Batch sensor uploads; avoid waking radio every second if battery-powered.
- Use **MQTT** with QoS 0 for telemetry; reserve TLS for provisioning.

---

## Exercises

1. **Scan:** Print nearby SSIDs and RSSI.
2. **Connect:** Join home AP; print IP on UART.
3. **HTTPS GET:** Fetch JSON from a public API with mbedTLS.
4. **Captive portal:** SoftAP + HTTP server for credential entry (ESP-IDF C HTTP + Swift UI logic).
5. **Host companion:** macOS Swift app displays data fetched from MCU HTTP server on LAN.

---

## References

- [ESP-IDF Wi-Fi API](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/network/esp_wifi.html)
- [swift-matter-examples](https://github.com/swiftlang/swift-matter-examples) — ESP32-C6 Wi-Fi + Matter
- [Espressif Embedded Swift blog](https://developer.espressif.com/blog/build-embedded-swift-application-for-esp32c6/)
- [boards/esp32-s3.md](./boards/esp32-s3.md)
- [22-swift-concurrency.md](./22-swift-concurrency.md)

---

*Previous: [19-usb.md](./19-usb.md) · Next: [21-bluetooth.md](./21-bluetooth.md)*
