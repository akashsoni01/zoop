# Ethernet — IEEE 802.3 Wired LAN

**Ethernet** provides **wired** local area networking at **10/100/1000 Mbit/s** using twisted pair (RJ-45) or fiber. MCUs connect via an external **PHY** and **MAC** (or SPI Ethernet controller).

**Prerequisites:** [07-hal.md](../07-hal.md), [spi.md](./spi.md)

---

## Theory

**OSI mapping:**

| Layer | Ethernet component |
|-------|-------------------|
| Physical | PHY (signaling, auto-MDIX) |
| Data link | MAC (framing, CRC) |
| Network | IP (often lwIP / vendor stack) |

**Frame structure:** Preamble, dest MAC (6 B), src MAC (6 B), EtherType/Length (2 B), payload (46–1500 B), FCS (4 B).

ESP32-S3 has no built-in Ethernet MAC — use **SPI Ethernet** (W5500) or **RMII PHY** (LAN8720) with appropriate module.

---

## Timing Diagram (ASCII)

100BASE-TX symbol stream (conceptual — MLT-3 encoding on wire):

```
Manchester-like conceptual (10 Mbps half-duplex CSMA/CD):

Carrier sense ────────┐                    ┌── idle
                      └──── frame TX ──────┘
                            │
                     collision window
```

Full-duplex switched Ethernet avoids CSMA/CD collisions.

---

## Packet Format

**Ethernet II frame:**

| Field | Size |
|-------|------|
| Dest MAC | 6 B |
| Src MAC | 6 B |
| EtherType | 2 B (0x0800 = IPv4) |
| Payload | 46–1500 B |
| FCS | 4 B |

Above Ethernet: **IP → TCP/UDP → HTTP/MQTT**.

---

## Electrical Characteristics

| Parameter | 100BASE-TX |
|-----------|------------|
| Cable | Cat5e, max 100 m |
| Pairs | 2 (TX/RX) |
| Termination | Auto-MDIX on modern PHY |
| Magnetics | Required (integrated in RJ45 jack) |
| ESP32 interface | RMII or SPI |

Isolate chassis ground from Ethernet magnetics center tap per design guide.

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32Ethernet

struct EthernetInterface {
    var mac: EthernetMAC

    mutating func start(dhcp: Bool = true) async throws {
        try mac.initPHY(type: .lan8720)
        try mac.start()
        if dhcp {
            try await mac.dhcpWait(timeoutSeconds: 10)
        }
    }

    func ipAddress() -> IPv4Address? {
        mac.ipv4()
    }
}
```

**ARC note:** Network stack often uses singleton — wrap in struct facade for testability.

---

## Example Projects

| Link | Description |
|------|-------------|
| [projects/iot-gateway.md](../projects/iot-gateway.md) | Wired gateway |
| [examples/http-server.md](../examples/http-server.md) | Web UI over Ethernet |

---

## Common Mistakes

1. **Missing magnetics** — PHY won't link.
2. **Wrong REF_CLK frequency** — 50 MHz RMII clock required.
3. **MDIO address mismatch** — PHY not detected.
4. **Cable not connected** — link down, DHCP fails silently.
5. **Sharing SPI with display** — bus mutex required.

---

## Exercises

1. Print link speed and duplex after PHY init.
2. Ping device from PC after DHCP.
3. Serve HTTP page over Ethernet (no Wi-Fi).
4. Measure throughput vs Wi-Fi on same board.

---

## References

- IEEE 802.3
- W5500 / LAN8720 datasheets
- [projects/iot-gateway.md](../projects/iot-gateway.md)

---

*Prev: [usb.md](./usb.md) | Next: [wifi.md](./wifi.md)*

*Back to [Embedded Swift](../README.md)*
