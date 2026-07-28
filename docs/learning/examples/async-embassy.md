# Example: Async with Embassy

**Goal:** Run Wi-Fi, sensors, and UI concurrently without RTOS threads.

**Prerequisites:** [22-embassy.md](../22-embassy.md), [wifi-client.md](./wifi-client.md)

---

## Rust Sketch

```rust
use embassy_executor::Spawner;
use embassy_sync::channel::Channel;

static SENSOR_CH: Channel<CriticalSectionRawMutex, SensorReading, 4> = Channel::new();

#[embassy_executor::task]
async fn sensor_task() {
    loop {
        let reading = read_bme280().await;
        SENSOR_CH.send(reading).await;
        Timer::after_secs(5).await;
    }
}

#[embassy_executor::task]
async fn mqtt_task(stack: Stack<'static>) {
    loop {
        let r = SENSOR_CH.receive().await;
        publish_mqtt(&stack, &r).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(sensor_task()).unwrap();
    spawner.spawn(mqtt_task(stack)).unwrap();
}
```

### Ownership

Tasks `'static`; data via channels — no shared `RefCell` across await points without `Mutex`.

*Next: [rtic.md](./rtic.md)*
