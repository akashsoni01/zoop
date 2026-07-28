# Example: Wi-Fi Client

**Goal:** Connect ESP32-S3 to access point and print IP address.

**Prerequisites:** [20-wifi.md](../20-wifi.md), [communication/wifi.md](../communication/wifi.md)

---

## Rust Sketch

```rust
use esp_wifi::wifi::{ClientConfiguration, Configuration, WifiController};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let (wifi, net) = /* init esp-wifi + embassy-net stack */;
    spawner.spawn(net_task(net)).unwrap();

    let mut wifi = wifi;
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: "YOUR_SSID".try_into().unwrap(),
        password: "YOUR_PASS".try_into().unwrap(),
        ..Default::default()
    })).unwrap();
    wifi.start().unwrap();
    wifi.connect().unwrap();

    net.wait_config_up().await;
    defmt::info!("IP acquired!");
    loop { embassy_time::Timer::after_secs(60).await; }
}
```

Store credentials in NVS — not source code.

*Next: [ble-server.md](./ble-server.md)*
