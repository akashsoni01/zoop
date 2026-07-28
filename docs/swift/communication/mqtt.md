# MQTT — Message Queuing Telemetry Transport

**MQTT** is a **lightweight pub/sub** protocol for IoT — clients publish to **topics**; brokers route messages to subscribers. Ideal for intermittent links and constrained devices.

**Prerequisites:** [wifi.md](./wifi.md), [ethernet.md](./ethernet.md)

---

## Theory

Roles:

| Component | Role |
|-----------|------|
| **Broker** | Central router (Mosquitto, HiveMQ, AWS IoT) |
| **Publisher** | Sends messages to topics |
| **Subscriber** | Receives messages on subscribed topics |

QoS levels:

| QoS | Guarantee |
|-----|-----------|
| 0 | At most once (fire and forget) |
| 1 | At least once (ACK) |
| 2 | Exactly once (four-way handshake) |

**Keep-alive** ping prevents broker disconnect. **Last Will** publishes on unexpected disconnect.

---

## Timing Diagram (ASCII)

MQTT CONNECT / CONNACK exchange:

```
Client                          Broker
  │── CONNECT (clientId, keepalive) ──►│
  │◄── CONNACK (session present, rc=0) ─│
  │── SUBSCRIBE [topic filter] ─────────►│
  │◄── SUBACK ────────────────────────────│
  │── PUBLISH topic=sensors/temp ───────►│
  │◄── PUBLISH topic=commands/led ────────│
  │── PINGREQ ───────────────────────────►│
  │◄── PINGRESP ──────────────────────────│
```

---

## Packet Format

**Fixed header:**

| Field | Size |
|-------|------|
| Type + flags | 1 B (upper nibble = type) |
| Remaining length | 1–4 B (variable) |

**PUBLISH variable header + payload:**

| Field | Content |
|-------|---------|
| Topic name | UTF-8 string |
| Packet ID | 2 B (QoS > 0) |
| Payload | binary / JSON |

Example topic: `home/room1/temperature` → payload `{"c":23.5}`.

---

## Electrical Characteristics

MQTT is application layer — physical layer is [wifi.md](./wifi.md) or [ethernet.md](./ethernet.md). TLS adds CPU/RAM cost; use **MQTTS** (port 8883) in production.

ESP32-S3: ensure stable power during Wi-Fi TX bursts.

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32MQTT

struct TelemetryPublisher {
    var client: MQTTClient

    mutating func connect(broker: String, clientId: String) async throws {
        try await client.connect(
            host: broker,
            port: 1883,
            clientId: clientId,
            keepAlive: 60
        )
    }

    mutating func publish(topic: String, json: [UInt8]) async throws {
        try await client.publish(
            topic: topic,
            payload: json,
            qos: .atMostOnce
        )
    }
}

func mqttTask() async {
    var pub = TelemetryPublisher(client: MQTTClient())
    try? await pub.connect(broker: "192.168.1.10", clientId: "esp32s3-node")
    try? await pub.client.subscribe("devices/esp32/cmd")
    while true {
        let payload: [UInt8] = Array("{\"t\":23.5}".utf8)
        try? await pub.publish(topic: "devices/esp32/temp", json: payload)
        try? await Task.sleep(nanoseconds: 30_000_000_000)
    }
}
```

**ARC note:** Use stack `[UInt8]` for payloads; avoid `String` allocation in publish loop. If `MQTTClient` is a class, hold one instance in a struct coordinator.

---

## Bare-Metal Sketch (Concept)

Minimal MQTT CONNECT over TCP socket:

```swift
// After TCP connect to broker:1883
let connectPacket: [UInt8] = [
    0x10, 0x0C,  // CONNECT, remaining length
    0x00, 0x04, 0x4D, 0x51, 0x54, 0x54,  // "MQTT"
    0x04, 0x02, 0x00, 0x3C  // v4, flags, keepalive 60
]
try tcpSocket.write(connectPacket)
```

Prefer a full client library for production.

---

## Example Projects

| Link | Description |
|------|-------------|
| [examples/mqtt-client.md](../examples/mqtt-client.md) | Pub/sub client |
| [projects/weather-station.md](../projects/weather-station.md) | Sensor MQTT |
| [projects/smart-home-node.md](../projects/smart-home-node.md) | Home automation |

---

## Common Mistakes

1. **Same client ID twice** — broker kicks previous session.
2. **No keep-alive / PING** — silent disconnect.
3. **Topic wildcards misunderstood** — `#` vs `+`.
4. **QoS 2 on MCU** — heavy; use QoS 0/1 for telemetry.
5. **Large JSON every 100 ms** — bandwidth and heap pressure.

---

## Exercises

1. Publish BME280 JSON every 30 s to `home/temp`.
2. Subscribe to `devices/esp32/led` and toggle GPIO.
3. Add Last Will message `offline` on unexpected disconnect.
4. Compare QoS 0 vs 1 latency with broker logs.

---

## References

- [MQTT 3.1.1 specification](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html)
- [examples/mqtt-client.md](../examples/mqtt-client.md)

---

*Prev: [wifi.md](./wifi.md) | Next: [http.md](./http.md)*

*Back to [Embedded Swift](../README.md)*
