# MQTT — Message Queuing Telemetry Transport

**MQTT** is a **lightweight pub/sub** protocol for IoT — clients publish to **topics**; brokers route messages to subscribers. Ideal for intermittent links and constrained devices.

**Prerequisites:** [wifi.md](./wifi.md), [tcp/ip via ethernet.md or wifi.md](./ethernet.md)

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

## Rust HAL Sketch

```rust
use embassy_executor::Spawner;
// smoltcp + mqtt or esp-idf-svc — illustrative with embedded-friendly client

pub async fn mqtt_task(
    stack: embassy_net::Stack<'static>,
    broker: &'static str,
) {
    let mut client = MqttClient::new(stack, broker, 1883);
    client.connect("esp32s3-node", 60).await.unwrap();
    client.subscribe("devices/esp32/cmd").await.unwrap();

    loop {
        client.publish("devices/esp32/temp", b"23.5", QoS::AtMostOnce).await.ok();
        if let Some(msg) = client.poll_message().await {
            defmt::info!("Got: {:?}", msg.topic);
        }
        embassy_time::Timer::after_secs(30).await;
    }
}
```

**Ownership:** `Stack` and client in `'static` Embassy task; no `String` — use `heapless::String` for topics.

**Compile:**

```toml
# picMQTT, rumqttc (std), or embassy-compatible fork
heapless = "0.8"
serde-json-core = "0.5"  # optional JSON payloads
```

---

## Bare-Metal Sketch (Concept)

Minimal MQTT CONNECT over TCP socket:

```rust
// After TCP connect to broker:1883
const CONNECT: &[u8] = &[
    0x10, 0x0C,             // CONNECT, remaining length
    0x00, 0x04, b'M', b'Q', b'T', b'T', // protocol name
    0x04, 0x02, 0x00, 0x3C, // level 4, clean session, keepalive 60
    // + client ID length + bytes
];
```

Use a crate for production — hand-rolling is error-prone.

---

## Example Projects

| Link | Use |
|------|-----|
| [examples/mqtt-client.md](../examples/mqtt-client.md) | Basic pub/sub |
| [projects/wifi-sensor.md](../projects/wifi-sensor.md) | Telemetry |
| [projects/iot-gateway.md](../projects/iot-gateway.md) | Protocol bridge |
| [projects/smart-home-node.md](../projects/smart-home-node.md) | Home Assistant |

---

## Common Mistakes

1. **Client ID collision** — broker kicks previous session.
2. **Keep-alive too long** — NAT timeout drops TCP silently.
3. **Huge payloads** — split or use binary formats.
4. **No reconnect/backoff** — device stays offline after AP glitch.
5. **Credentials in source** — use NVS / provisioning.

---

## Exercises

1. Publish BME280 readings every 10 s — see [sensors/bme280.md](../sensors/bme280.md).
2. Subscribe to `cmd/led` and toggle GPIO from payload.
3. Implement exponential backoff reconnect.
4. Compare QoS 0 vs 1 latency under packet loss.

---

## References

- [MQTT 3.1.1 Specification](https://docs.oasis-open.org/mqtt/mqtt/v3.1.1/mqtt-v3.1.1.html)
- [esp-idf-svc MQTT](https://docs.rs/esp-idf-svc/latest/esp_idf_svc/mqtt/index.html)
- Lesson: [20-wifi.md](../20-wifi.md)

---

*Prev: [lin.md](./lin.md) | Next: [http.md](./http.md)*
