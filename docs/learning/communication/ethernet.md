# Ethernet — IEEE 802.3 Wired Networking

**Ethernet** provides **layer-2** frame delivery over twisted pair (10/100/1000BASE-T) or fiber. MCUs with **MAC + external PHY** (e.g., W5500, LAN8720) gain TCP/IP capability.

**Prerequisites:** [07-hal.md](../07-hal.md), [spi.md](./spi.md) (for SPI Ethernet modules)

---

## Theory

Stack layers (simplified):

```
Application  │ HTTP, MQTT
Transport    │ TCP, UDP
Network      │ IP, ICMP
Link         │ Ethernet (MAC addresses)
Physical     │ PHY (magnetics, RJ45)
```

**MAC address:** 48-bit unique ID (OUI + device). **Frames** carry ethertype (0x0800 = IPv4).

ESP32-S3 has **no built-in Ethernet MAC** — use SPI/W5500 or RMII PHY on custom boards. ESP32 (original) had EMAC — note difference when reading docs.

---

## Timing Diagram (ASCII)

Manchester-ish concept — 100BASE-TX on wire pair (simplified):

```
100BASE-TX symbol stream (conceptual):

Data bits ── 1 0 1 1 0 0 1 0 ...
Line pair ── transitions encode clock + data (MLT-3 encoding on TX pair)

Inter-frame gap (IFG): 96 bit times minimum between frames
```

SPI read of W5500 RX buffer (separate from line coding):

```
CS  ──┐                                    ┌──
      └────────────────────────────────────┘
SCK ──┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐
      └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘
MOSI──[Addr][Control][Data...]
MISO──[...............][Data...]
```

---

## Packet Format

**Ethernet II frame:**

| Field | Size | Example |
|-------|------|---------|
| Preamble + SFD | 8 bytes | (hardware) |
| Dest MAC | 6 bytes | FF:FF:FF:FF:FF:FF (broadcast) |
| Src MAC | 6 bytes | Device MAC |
| EtherType | 2 bytes | 0x0800 IPv4 |
| Payload | 46–1500 bytes | IP packet |
| FCS | 4 bytes | CRC32 |

**IPv4 header** (inside payload) adds src/dst IP, TTL, protocol (6=TCP, 17=UDP).

---

## Electrical Characteristics

| Parameter | 100BASE-TX |
|-----------|------------|
| Cable | Cat5e, max 100 m |
| Magnetics | Required (integrated in RJ45 jack) |
| PHY interface | RMII / MII / SPI (W5500) |
| Power | 3.3 V I/O, PHY may need 1.2 V core |

W5500 SPI: 3.3 V, ~30 MHz SPI typical, separate socket buffers (8 KB each direction).

---

## Rust HAL Sketch (W5500 + embedded-nal)

```rust
use embedded_nal::{TcpStack, ConnectedTcpSocket};
// w5500-hl or embassy-net integration — illustrative

pub async fn http_get_ip(stack: &mut impl TcpStack, host: &str) {
    let mut socket = stack.connect(host, 80).await.unwrap();
    let req = b"GET / HTTP/1.0\r\nHost: example.com\r\n\r\n";
    socket.write(req).await.ok();
    let mut buf = [0u8; 512];
    let n = socket.read(&mut buf).await.unwrap_or(0);
    defmt::info!("Received {} bytes", n);
}
```

**Ownership:** Socket structs own buffer slots in W5500 hardware; drop closes connection.

**Compile:** Use `embassy-net` + `w5500-net` or ESP-IDF `esp-netif` for integrated stacks.

---

## Bare-Metal Sketch (Concept)

W5500 register access via SPI:

```rust
fn w5500_write_reg(cs: &mut OutputPin, spi: &mut impl SpiDevice, addr: u16, val: u8) {
    let cmd = [(addr >> 8) as u8, (addr & 0xFF) as u8 | 0x04, val];
    let _ = cs.set_low();
    let _ = spi.write(&cmd);
    let _ = cs.set_high();
}
```

Initialize PHY mode, MAC, IP, gateway via common register block (0x0000–0x002F).

---

## Example Projects

| Link | Role |
|------|------|
| [projects/iot-gateway.md](../projects/iot-gateway.md) | Ethernet uplink |
| [examples/http-server.md](../examples/http-server.md) | TCP HTTP |
| [mqtt.md](./mqtt.md) | MQTT over TCP |

---

## Common Mistakes

1. **Missing magnetics** — link never comes up.
2. **Wrong RMII ref clock** — 50 MHz timing critical.
3. **Hard-coded IP on DHCP network** — use DHCP client.
4. **Single socket exhaustion on W5500** — manage 8 hardware sockets.
5. **No ARP** — cannot reach local gateway.

---

## Exercises

1. Ping device from PC after static IP config.
2. Implement TCP echo server on port 7777.
3. Measure throughput SPI Ethernet vs Wi-Fi on same board class.
4. Capture Wireshark trace of DHCP exchange.

---

## References

- IEEE 802.3
- W5500 datasheet (WIZnet)
- [embedded-nal](https://docs.rs/embedded-nal/latest/embedded_nal/)
- [embassy-net](https://embassy.dev/book/dev/net.html)
- Lesson: [05-embedded-architecture.md](../05-embedded-architecture.md)

---

*Prev: [usb.md](./usb.md) | Next: [wifi.md](./wifi.md)*
