# Example: MQTT Client

**Goal:** Publish sensor data to Mosquitto broker.

**Prerequisites:** [wifi-client.md](./wifi-client.md), [communication/mqtt.md](../communication/mqtt.md)

---

## Rust Sketch

```rust
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let stack = /* Wi-Fi up */;
    let mut mqtt = MqttClient::new(stack, "192.168.1.10", 1883);
    mqtt.connect("esp32s3-1", 60).await.unwrap();
    mqtt.subscribe("home/cmd").await.unwrap();

    loop {
        let temp = read_bme280(); // see sensors/bme280.md
        mqtt.publish("home/temp", temp.as_bytes(), QoS0).await.ok();
        embassy_time::Timer::after_secs(10).await;
    }
}
```

Implement reconnect with exponential backoff.

See [projects/wifi-sensor.md](../projects/wifi-sensor.md).

*Next: [ota-update.md](./ota-update.md)*
