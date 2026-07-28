# BLE — Bluetooth Low Energy

**BLE** (Bluetooth Low Energy) is a **low-power** 2.4 GHz protocol for short-range links. Devices expose **GATT services** (collections of **characteristics**) read/written by a central (phone, PC).

**Prerequisites:** [09-interrupts.md](../09-interrupts.md), [wifi.md](./wifi.md) (coexistence on ESP32-S3)

---

## Theory

Roles:

| Role | Behavior |
|------|----------|
| **Peripheral** (server) | Advertises, accepts connections |
| **Central** (client) | Scans, initiates connection |

**GATT hierarchy:** Service → Characteristic → Descriptor.

Example: Heart Rate Service `0x180D` → Measurement `0x2A37`.

Connection interval: 7.5 ms–4 s (negotiated). **MTU** up to 517 bytes (negotiated).

ESP32-S3: use `esp-wifi` BLE stack or ESP-IDF `BleServer`.

---

## Timing Diagram (ASCII)

Advertising (connectable undirected):

```
RF channel hop ── 37 ── 38 ── 39 ── 37 ── ...
                 │ adv PDU │    │ adv PDU │

Adv packet on one channel:

Preamble │ Access Addr │ PDU (AdvA + AdvData) │ CRC
         │  0x8E89BED6 │  6 B MAC + 0–31 B    │

Connection event (after connect):

|<── connInterval ──>|<── connInterval ──>|
   Central TX/RX          Peripheral TX/RX
```

---

## Packet Format

**Advertising PDU (legacy):**

| Field | Size |
|-------|------|
| PDU Type | 4 bits |
| Length | 6 bits |
| AdvA | 6 bytes (MAC) |
| AdvData | 0–31 bytes (AD structures) |

**AD structure:** `[len][type][data...]` — e.g., type 0x09 = Complete Local Name.

**ATT Write Request:** opcode (0x12) + handle (2 B) + value (0–MTU-3).

---

## Electrical Characteristics

| Parameter | Typical |
|-----------|---------|
| Frequency | 2.402–2.480 GHz, 40 channels |
| TX power | −12 to +9 dBm (ESP32-S3 configurable) |
| Current | ~15 mA average (connection), µA in sleep |
| Antenna | Shared with Wi-Fi — layout critical |

No external PHY required — RF integrated in ESP32-S3.

---

## Rust HAL Sketch (esp-wifi BLE peripheral)

```rust
use trouble_host::prelude::*; // or esp-idf BLE — API illustrative

// GATT server: battery service example
pub struct BatteryService {
    level: u8,
}

impl BatteryService {
    pub fn read_level(&self) -> u8 {
        self.level
    }

    pub fn notify_level(&mut self, level: u8) {
        self.level = level;
        // notify subscribed centrals via stack
    }
}

// Ownership: Service lives in static or stack-allocated server struct
// `'static` for spawn; no heap needed with fixed attribute table
```

**Compile:**

```toml
esp-wifi = { features = ["esp32s3", "ble"] }
# or trouble-host / bt-hci crates for portable BLE host
```

---

## Bare-Metal Sketch (Concept)

BLE link layer is implemented in controller firmware (Espressif binary). Application sets advertising data:

```rust
// AD: flags + complete name "ZoopNode"
const ADV_DATA: &[u8] = &[
    0x02, 0x01, 0x06,       // LE General Discoverable
    0x09, 0x09, b'Z', b'o', b'o', b'p', b'N', b'o', b'd', b'e',
];
```

Controller handles hopping and timing.

---

## Example Projects

| Link | Role |
|------|------|
| [examples/ble-server.md](../examples/ble-server.md) | GATT peripheral |
| [projects/ble-beacon.md](../projects/ble-beacon.md) | iBeacon / Eddystone |
| [projects/battery-monitor.md](../projects/battery-monitor.md) | BLE telemetry |
| [projects/smart-home-node.md](../projects/smart-home-node.md) | Provisioning |

---

## Common Mistakes

1. **Advertising without connectable flag** when connection expected.
2. **MTU default 23 bytes** — negotiate before long writes.
3. **Blocking in GATT callback** — defer heavy work to task.
4. **Wi-Fi + BLE high duty cycle** — coexistence crashes or disconnects.
5. **Missing bonding/security** for sensitive data.

---

## Exercises

1. Advertise device name; discover with nRF Connect app.
2. Add custom UUID characteristic; read/write from phone.
3. Implement [projects/ble-beacon.md](../projects/ble-beacon.md) with battery in ADV.
4. Measure sleep current with advertising interval 1 s vs 100 ms.

---

## References

- [Bluetooth Core Specification](https://www.bluetooth.com/specifications/specs/)
- [Espressif BLE API](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/bluetooth/index.html)
- [trouble-host](https://github.com/embassy-rs/trouble) (Embassy BLE)
- Lesson: [09-interrupts.md](../09-interrupts.md)

---

*Prev: [wifi.md](./wifi.md) | Next: [lora.md](./lora.md)*
