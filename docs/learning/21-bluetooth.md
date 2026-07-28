# Lesson 21 — Bluetooth Low Energy (BLE, GATT)

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [20-wifi.md](./20-wifi.md) (coexistence on ESP)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md), [nrf52.md](./boards/nrf52.md) (Nordic reference for BLE).

---

## Theory

**Bluetooth Low Energy (BLE)** is a short-range wireless protocol for low-power peripherals (sensors, fitness trackers, phone accessories). Unlike Classic Bluetooth, BLE emphasizes **low duty cycle** and **GATT** (Generic Attribute Profile) data exchange.

### Roles

| Role | Behavior |
|------|----------|
| **Peripheral** | Advertises, accepts connections — your sensor |
| **Central** | Scans, initiates connection — phone, USB dongle |
| **Broadcaster / Observer** | Connectionless advertising (beacons) |

Most embedded firmware implements **peripheral** mode.

### GATT hierarchy

```
GATT Server (your device)
  └── Service (e.g., Nordic UART Service UUID)
        └── Characteristic (e.g., RX — write from phone)
        └── Characteristic (e.g., TX — notify to phone)
              └── Descriptor (CCCD — Client Characteristic Configuration)
```

| Term | Meaning |
|------|---------|
| **UUID** | 128-bit identifier — 16-bit shortcuts for standard services |
| **Characteristic** | Value + properties (read, write, notify, indicate) |
| **Notify** | Server pushes updates without ACK (faster) |
| **Indicate** | Server pushes with ACK |
| **CCCD** | Client enables notify/indicate by writing `0x0001` / `0x0002` |

### Advertising

**Advertising packets** broadcast device name, UUIDs, and manufacturer data before connection:

```
ADV_IND: Connectable undirected advertising
  Flags | Complete Local Name | Service UUID list | ...
```

Interval trade-off: **20 ms–10.28 s** — faster = more discoverable, higher power.

### Nordic UART Service (NUS) style

De-facto pattern for serial-over-BLE:

| UUID (NUS) | Characteristic | Direction |
|------------|----------------|-----------|
| `6E400001-...` | Service | — |
| `6E400002-...` | RX | Central writes → device |
| `6E400003-...` | TX | Device notifies → central |

Works with nRF Connect mobile app and many USB BLE sniffer tools.

---

## Hardware Overview

### ESP32-S3 BLE

- **Bluetooth 5.0** LE — 1M PHY default; 2M/Coded on supported configs
- Shared radio with Wi-Fi — controller arbitrates time slices
- Rust paths: **esp-idf-svc** (`Ble`, `Bluedroid`/`NimBLE`), **esp-wifi** ecosystem expanding

### nRF52840 (Nordic)

- BLE is the native strength — **SoftDevice** (legacy) or **nRF Connect SDK / embassy-nrf**
- Excellent for learning GATT without Wi-Fi complexity — [boards/nrf52.md](./boards/nrf52.md)

---

## ASCII Wiring

No wiring — onboard antenna. Optional **LED** for connection status:

```
ESP32-S3 DevKitC-1
┌──────────────────┐
│ GPIO48 ──► LED ──► GND  (via resistor)
│ Built-in BLE antenna  │
└──────────────────┘
         │
    Phone (nRF Connect app) scans & connects
```

---

## Memory & Register Notes

### ATT MTU (Maximum Transmission Unit)

Default ATT MTU = **23 bytes** (20 payload). After **MTU exchange**, up to **517** — larger throughput for UART-style streaming.

### Connection parameters

| Parameter | Effect |
|-----------|--------|
| **Connection interval** | 7.5 ms–4 s between connection events |
| **Slave latency** | Skip N events to save power |
| **Supervision timeout** | Disconnect if packets missed |

Request reasonable defaults; phones may renegotiate.

### ESP BLE stack RAM

Similar to Wi-Fi — allocate **~40–80 KB** depending on features. Monitor heap after `ble_init`.

---

## HAL Example — NUS-style peripheral (esp-idf-svc outline)

```rust
use esp_idf_svc::ble::Ble;
use esp_idf_svc::ble::gap::{AdvConfiguration, BleGapEvent, RawAdvertisement};
use esp_idf_svc::ble::gatt::server::{GattServer, Service, Characteristic};

// Nordic UART Service UUIDs (128-bit)
const NUS_SERVICE: &str = "6E400001-B5A3-F393-E0A9-E50E24DCCA9E";
const NUS_RX: &str      = "6E400002-B5A3-F393-E0A9-E50E24DCCA9E"; // write
const NUS_TX: &str      = "6E400003-B5A3-F393-E0A9-E50E24DCCA9E"; // notify

fn main() -> anyhow::Result<()> {
    esp_idf_svc::sys::link_patches();

    let ble = Ble::new(/* event loop, nvs */)?;

    // Build GATT server
    let server = GattServer::new(&ble)?;
    let service = Service::new(NUS_SERVICE)?;
    let rx_char = Characteristic::new(
        NUS_RX,
        esp_idf_svc::ble::gatt::properties::WRITE | esp_idf_svc::ble::gatt::properties::WRITE_NR,
    )?;
    let tx_char = Characteristic::new(
        NUS_TX,
        esp_idf_svc::ble::gatt::properties::NOTIFY,
    )?;
    service.add_characteristic(rx_char)?;
    service.add_characteristic(tx_char)?;
    server.add_service(service)?;
    server.start()?;

    // Advertising
    let adv = AdvConfiguration {
        name: Some("ESP32-S3-NUS".into()),
        include_txpower: true,
        ..Default::default()
    };
    ble.advertise(adv)?;

    // Event loop: on write to RX char → echo notify on TX char
    loop {
        match ble.wait_event()? {
            BleGapEvent::Connected => log::info!("BLE connected"),
            BleGapEvent::Disconnected => {
                log::info!("BLE disconnected — restart advertising");
                ble.advertise(adv.clone())?;
            }
            _ => {}
        }
        // Handle GATT write events → ble.notify(NUS_TX, data)
    }
}
```

Exact API varies — consult [esp-idf-svc BLE module docs](https://docs.rs/esp-idf-svc/).

---

## embassy-nrf NUS (nRF52840 reference)

```rust
// embassy-nrf + trouble-host (BLE host stack) — illustrative
use trouble_host::prelude::*;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut resources = embassy_nrf::init(Default::default());
    let stack = /* build BLE stack with SoftDevice or s140 */;

    spawner.spawn(ble_task(stack)).unwrap();

    loop {
        embassy_futures::yield_now().await;
    }
}

#[embassy_executor::task]
async fn ble_task(mut stack: Stack<'_, '_, ...>) {
    let mut server = NusServer::new(/* gatt layers */);
    let adv = Advertisement {
        name: Some("nRF-NUS"),
        services: &[NUS_UUID],
        ..Default::default()
    };
    stack.set_advertisement(&adv).await;
    // Wait for writes, notify on TX
}
```

Nordic + **trouble-host** is the emerging pure-Rust BLE path — check [trouble repository](https://github.com/embassy-rs/trouble).

---

## Step-by-Step

1. Flash **BLE advertise** example; see device in **nRF Connect** scanner.
2. Add **device name** and **service UUID** in advertisement.
3. Implement **GATT server** with one read/write characteristic.
4. Enable **notifications** via CCCD from phone; send counter every second.
5. Port logic to **NUS UUIDs** for serial terminal apps.
6. If using Wi-Fi + BLE, test coexistence scenarios — [20-wifi.md](./20-wifi.md).

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Forgot CCCD enable | Phone sees no notifications | Client must write CCCD |
| Notify without connection | Error | Check connected state |
| MTU too small | Truncated messages | Negotiate MTU |
| Advertising stopped after disconnect | Invisible device | Re-start adv on disconnect |
| UUID endian typo | Service not discovered | Verify 128-bit string |
| Wi-Fi + BLE heavy traffic | Dropped packets | Throttle throughput; check coexistence config |
| 5 V on 3.3 V UART bridge to nRF | Hardware damage | Level shift |

---

## Debugging Tips

- **nRF Connect** (mobile) — scan, connect, explore GATT table, subscribe notify.
- **Wireshark + nRF Sniffer** — over-the-air packets.
- Log **connection interval**, **MTU**, **reason code** on disconnect.
- Compare against **Espressif AT firmware** on same board to isolate hardware.
- `defmt` trace GATT write lengths — [25-debugging.md](./25-debugging.md).

---

## Performance Tips

- **Notify** faster than **Indicate** for streaming sensor data.
- Batch small messages — overhead per notification is ~3 bytes ATT + radio schedule.
- Increase connection interval during idle; shorten during active transfer.
- Use **2M PHY** when both sides support it (lower air time).
- **Deep sleep** between advertisements for battery — [24-low-power.md](./24-low-power.md).

---

## Exercises

1. **BLE blink:** Write `0x01`/`0x00` to RX char → toggle LED.
2. **Sensor notify:** Stream fake temperature every 500 ms on TX char.
3. **Custom service:** Define your own 128-bit UUID service with three chars.
4. **Throughput test:** Measure bytes/sec NUS notify vs connection interval settings.
5. **nRF port:** Implement same NUS on nRF52840 DK — [boards/nrf52.md](./boards/nrf52.md).

---

## References

- [Bluetooth Core Specification (SIG)](https://www.bluetooth.com/specifications/specs/)
- [Nordic UART Service spec](https://developer.nordicsemi.com/nRF_Connect_SDK/doc/latest/nrf/libraries/bluetooth/services/nus.html)
- [esp-idf-svc BLE](https://docs.rs/esp-idf-svc/latest/esp_idf_svc/ble/index.html)
- [trouble BLE host](https://github.com/embassy-rs/trouble)
- [20-wifi.md](./20-wifi.md) — coexistence

---

*Previous: [20-wifi.md](./20-wifi.md) · Next: [22-embassy.md](./22-embassy.md)*
