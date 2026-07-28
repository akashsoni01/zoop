# Zigbee — IEEE 802.15.4 Mesh Networking

**Zigbee** is a **mesh networking** protocol built on **IEEE 802.15.4** (2.4 GHz). Devices form self-healing networks: **coordinators**, **routers**, and **end devices** (battery-powered).

**Prerequisites:** [spi.md](./spi.md), [ble.md](./ble.md) (2.4 GHz coexistence)

---

## Theory

802.15.4 provides PHY/MAC; Zigbee adds **NWK** (routing) and **APS** (application profiles).

| Device type | Role |
|-------------|------|
| Coordinator | Forms network, assigns PAN ID |
| Router | Relays messages, mains powered |
| End Device | Sleepy, minimal routing |

Clusters (e.g., `0x0006` On/Off) live in **Home Automation** and **Light Link** profiles. ESP32-S3 typically uses coprocessors: **ESP32-H2**, **CC2652**, or **ZNP modules** over UART/SPI.

---

## Timing Diagram (ASCII)

802.15.4 data frame (O-QPSK 2.4 GHz):

```
|<─── 320 µs turn-around ───>|
Preamble │ SFD │ Len │ Seq │ PAN │ Dst │ Src │ Payload │ FCS
  4 B      1 B   1 B   1 B   2 B   2-8 B 2-8 B  0-102 B   2 B

Zigbee route (A → Router → B):

A ──[802.15.4 frame]──► Router ──[802.15.4 frame]──► B
     NWK dest = B              NWK dest = B
```

---

## Packet Format

**802.15.4 MAC header (simplified):**

| Field | Size |
|-------|------|
| Frame Control | 2 B |
| Sequence | 1 B |
| PAN ID | 2 B |
| Destination | 2 or 8 B |
| Source | 2 or 8 B |

**Zigbee APS payload:** `[Profile:2][Cluster:2][Cmd/Data...]`

---

## Electrical Characteristics

| Parameter | Typical |
|-----------|---------|
| Frequency | 2.405–2.480 GHz (16 channels) |
| TX power | 0–+20 dBm (module dependent) |
| Interface | UART to ZNP @ 115200, or SPI |
| Current | End device: µA sleep; router: tens of mA |

Keep 802.15.4 antenna away from Wi-Fi/BLE antenna on same PCB.

---

## Rust HAL Sketch

```rust
// UART-attached Zigbee coordinator (ZNP) — illustrative
use embedded_io::{Read, Write};

pub fn znp_send_af_data<W: Write, R: Read>(
    uart: &mut W,
    dst: u16,
    cluster: u16,
    payload: &[u8],
) -> Result<(), ()> {
    // ZNP frame: SOF | Len | Cmd0 | Cmd1 | Data... | FCS
    let mut frame = heapless::Vec::<u8, 128>::new();
    frame.push(0xFE).ok(); // SOF
    // build SYS_AF_DATA_REQUEST — consult TI ZNP spec
    let _ = uart.write_all(&frame);
    Ok(())
}
```

**Ownership:** UART owned by one task; frame buffer on stack (`heapless::Vec`).

---

## Bare-Metal Sketch (Concept)

802.15.4 MAC on dedicated radio chip — host sends high-level ZCL commands via AT-like ZNP protocol. Direct register SPI to CC2652 possible but use vendor stack for certification.

---

## Example Projects

| Link | Use |
|------|-----|
| [projects/smart-home-node.md](../projects/smart-home-node.md) | Zigbee end device |
| [projects/home-automation.md](../projects/home-automation.md) | Multi-protocol hub |
| [projects/environmental-monitor.md](../projects/environmental-monitor.md) | Sensor node |

---

## Common Mistakes

1. **PAN ID collision** with neighbor networks.
2. **Sleepy end device polling** not configured — missed commands.
3. **Insufficient router count** — range gaps in mesh.
4. **Mixing Zigbee 3.0 vs legacy** profiles without translation.
5. **2.4 GHz congestion** with Wi-Fi channel overlap — pick channel 15/20/25.

---

## Exercises

1. Pair a light bulb coordinator with a UART ZNP stick; toggle on/off cluster.
2. Map RSSI per hop in a three-node mesh.
3. Compare latency Zigbee vs [ble.md](./ble.md) for same toggle action.
4. Document channel plan vs home Wi-Fi AP channel.

---

## References

- Zigbee Specification (CSA)
- IEEE 802.15.4-2020
- TI Z-Stack ZNP UART interface
- Lesson: [21-bluetooth.md](../21-bluetooth.md) (wireless comparison)

---

*Prev: [lora.md](./lora.md) | Next: [rs232.md](./rs232.md)*
