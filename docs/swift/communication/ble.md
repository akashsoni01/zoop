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

ESP32-S3: use vendor BLE stack via C interop or Embedded Swift HAL wrapper.

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

## Swift HAL Sketch

```swift
import Embedded
import ESP32BLE

struct BatteryService {
    var level: UInt8 = 100

    func register(on server: inout GATTServer) {
        server.addService(uuid: 0x180F) { svc in
            svc.addCharacteristic(uuid: 0x2A19, properties: [.read, .notify]) {
                self.level
            }
        }
    }

    mutating func updateLevel(_ value: UInt8, server: inout GATTServer) {
        level = value
        server.notify(uuid: 0x2A19, payload: [value])
    }
}

struct BLEPeripheral {
    mutating func startAdvertising(name: String) throws {
        try BLE.setAdvertisingData([
            .flags(.generalDiscoverable),
            .completeLocalName(name)
        ])
        try BLE.startAdvertising()
    }
}
```

**ARC note:** GATT callbacks must not capture `self` strongly if `self` retains the server — use struct-based services or `[weak var]` delegate pattern.

---

## Bare-Metal Sketch (Concept)

```swift
// AD: flags + complete name "SwiftNode"
let advData: [UInt8] = [
    0x02, 0x01, 0x06,
    0x0A, 0x09, 0x53, 0x77, 0x69, 0x66, 0x74, 0x4E, 0x6F, 0x64, 0x65
]
bleController.setAdvData(advData)
```

---

## Example Projects

| Link | Description |
|------|-------------|
| [examples/ble-server.md](../examples/ble-server.md) | GATT peripheral |
| [projects/ble-beacon.md](../projects/ble-beacon.md) | iBeacon / Eddystone |
| [projects/battery-monitor.md](../projects/battery-monitor.md) | BLE telemetry |

---

## Common Mistakes

1. **Advertising without connectable flag** — central cannot connect.
2. **MTU too small for payload** — negotiate MTU exchange.
3. **Heavy work in GATT callback** — defer to main loop.
4. **Wi-Fi + BLE coexistence ignored** — throughput drops.
5. **Retain cycle in notify handler** — use struct services.

---

## Exercises

1. Advertise as "SwiftNode" and connect with nRF Connect app.
2. Implement Battery Service read + notify.
3. Build [projects/ble-beacon.md](../projects/ble-beacon.md) with fixed UUID.
4. Measure average current in connected vs advertising state.

---

## References

- Bluetooth Core Specification (GATT/ATT chapters)
- [examples/ble-server.md](../examples/ble-server.md)

---

*Prev: [wifi.md](./wifi.md) | Next: [lora.md](./lora.md)*

*Back to [Embedded Swift](../README.md)*
