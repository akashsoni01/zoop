# Example: EEPROM / Persistent Config

**Goal:** Store Wi-Fi credentials and settings across power cycles.

**Prerequisites:** [flash-storage.md](./flash-storage.md)

---

## Rust Sketch

```rust
use esp_bootloader_esp_idf::NvsDefault;

#[derive(Serialize, Deserialize)]
struct Config {
    ssid: heapless::String<32>,
    mqtt_broker: heapless::String<64>,
}

fn load_config(nvs: &mut NvsDefault) -> Config {
    nvs.get("config").unwrap_or_default()
}

fn save_config(nvs: &mut NvsDefault, cfg: &Config) {
    nvs.set("config", cfg).unwrap();
}
```

Prefer **NVS** over emulating EEPROM — handles wear leveling.

See [projects/smart-home-node.md](../projects/smart-home-node.md).

*Next: [async-embassy.md](./async-embassy.md)*
