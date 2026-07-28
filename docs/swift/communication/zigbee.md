# Zigbee — IEEE 802.15.4 Mesh

**Zigbee** is a **low-power mesh** protocol built on **IEEE 802.15.4** (2.4 GHz). Devices join a network coordinated by a **coordinator**; routers extend range; end devices sleep.

**Prerequisites:** [spi.md](./spi.md), [ble.md](./ble.md)

---

## Theory

**802.15.4 MAC:** CSMA-CA channel access, 16 channels (2.405–2.480 GHz), 250 kbit/s.

**Zigbee NWK layer:** star, tree, or mesh routing. **APS** application profiles (Home Automation, Light Link).

| Device type | Role |
|-------------|------|
| Coordinator | Forms network, PAN ID |
| Router | Relays, mains powered |
| End Device | Battery, polls parent |

ESP32-H2/C6 integrate 802.15.4; ESP32-S3 needs external radio (e.g., CC2652 via UART/SPI).

---

## Timing Diagram (ASCII)

802.15.4 data frame (simplified):

```
Preamble │ SFD │ Len │ Flags │ Seq │ PAN │ Dest │ Src │ Payload │ FCS
  4 B    │ 1 B │ 1 B │  2 B  │ 1 B │ 2 B │  2 B  │ 2 B │  0–102 B │ 2 B
```

Zigbee routing (mesh):

```
ED ──► Router ──► Router ──► Coordinator ──► IP gateway
```

---

## Packet Format

**802.15.4 MAC header (key fields):**

| Field | Size |
|-------|------|
| Frame control | 2 B |
| Sequence | 1 B |
| PAN ID | 2 B |
| Destination | 2 B (short) or 8 B (extended) |
| Source | 2 B or 8 B |
| Payload | MAC payload + Zigbee headers |

**Zigbee cluster:** endpoint + cluster ID (e.g., OnOff 0x0006).

---

## Electrical Characteristics

| Parameter | Typical |
|-----------|---------|
| Frequency | 2.4 GHz ISM |
| TX power | 0 to +20 dBm (chip dependent) |
| Current | µA sleep, mA TX |
| Antenna | PCB or ceramic 2.4 GHz |

Same RF cautions as [ble.md](./ble.md) — ground plane and keep-out.

---

## Swift HAL Sketch

```swift
import Embedded
import ZigbeeStack

struct ZigbeeLight {
    var endpoint: Endpoint
    var on: Bool = false

    mutating func register(onOffCluster() {
        endpoint.addCluster(id: 0x0006) { cluster in
            cluster.onWrite { value in
                self.on = value
                GPIO.relay.set(value)
            }
            cluster.onRead { self.on }
        }
    }
}

struct ZigbeeCoordinator {
    mutating func formNetwork(panId: UInt16, channel: UInt8) throws {
        try Zigbee.nwkForm(panId: panId, channelMask: 1 << channel)
    }
}
```

**ARC note:** Zigbee stack often C-based — thin Swift struct wrappers avoid Swift `class` in callbacks.

---

## Example Projects

| Link | Description |
|------|-------------|
| [projects/smart-home-node.md](../projects/smart-home-node.md) | Zigbee + MQTT |
| [projects/home-automation.md](../projects/home-automation.md) | Multi-protocol hub |

---

## Common Mistakes

1. **PAN ID collision** — won't join intended network.
2. **End device not polling parent** — messages dropped.
3. **Mixing Zigbee 3.0 security without install codes** — join fails.
4. **802.15.4 channel overlap with Wi-Fi** — choose clear channel (11, 15, 20, 25).
5. **Insufficient router nodes** — poor mesh coverage.

---

## Exercises

1. Form network on channel 15; join light bulb module.
2. Toggle OnOff cluster from coordinator.
3. Bridge OnOff events to MQTT topic.
4. Map RSSI/LQI per hop in mesh.

---

## References

- Zigbee Specification (Zigbee Alliance)
- IEEE 802.15.4
- [projects/smart-home-node.md](../projects/smart-home-node.md)

---

*Prev: [lora.md](./lora.md) | Next: [rs232.md](./rs232.md)*

*Back to [Embedded Swift](../README.md)*
