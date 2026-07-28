# WebSockets — Full-Duplex over HTTP

**WebSockets** provide **bidirectional**, **full-duplex** communication over a single TCP connection, initiated by an HTTP **Upgrade** handshake.

**Prerequisites:** [http.md](./http.md), [wifi.md](./wifi.md)

---

## Theory

1. Client sends HTTP GET with `Upgrade: websocket` and `Sec-WebSocket-Key`.
2. Server responds `101 Switching Protocols` with `Sec-WebSocket-Accept`.
3. Binary **framed** messages follow — no longer HTTP.

Use cases: live dashboards, remote scope UIs, gaming controllers, chat.

Embedded: prefer small JSON messages; avoid large binary streams without backpressure.

---

## Timing Diagram (ASCII)

Handshake then data frames:

```
Client                              Server
  │── GET /ws HTTP/1.1 ───────────────►│
  │    Upgrade: websocket              │
  │    Sec-WebSocket-Key: dGhlIHNhbX...  │
  │◄── 101 Switching Protocols ────────│
  │    Sec-WebSocket-Accept: ...       │
  │── WS frame (text FIN=1) ──────────►│
  │◄── WS frame (text pong) ───────────│
  │── WS ping ────────────────────────►│
  │◄── WS pong ────────────────────────│
```

---

## Packet Format

**WebSocket frame (simplified):**

| Field | Size | Notes |
|-------|------|-------|
| FIN + opcode | 1 B | 0x1 = text, 0x2 = binary, 0x9 = ping |
| Mask + len | 1–9 B | Client frames masked |
| Masking key | 4 B | Client only |
| Payload | N bytes | UTF-8 text or binary |

**HTTP Upgrade request (excerpt):**

```
GET /ws HTTP/1.1\r\n
Host: device.local\r\n
Upgrade: websocket\r\n
Connection: Upgrade\r\n
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n
Sec-WebSocket-Version: 13\r\n
\r\n
```

---

## Electrical Characteristics

Same as [wifi.md](./wifi.md) / [ethernet.md](./ethernet.md). WebSockets add minimal overhead vs raw TCP for persistent sessions.

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32Net

struct WebSocketClient {
    var tcp: TcpSocket
    var masked: Bool = true

    mutating func connect(url: String) async throws {
        // Parse ws://host/path, perform HTTP upgrade
        try await performUpgrade(host: url)
    }

    mutating func sendText(_ message: String) async throws {
        let payload = Array(message.utf8)
        try await sendFrame(opcode: .text, payload: payload)
    }

    mutating func receive(into buffer: inout [UInt8]) async throws -> WebSocketOpcode {
        try await readFrame(into: &buffer)
    }
}

struct LiveDashboard {
    var ws: WebSocketClient

    mutating func streamSamples(_ sample: SensorSample) async throws {
        let json = sample.toJSONBytes()
        try await ws.sendText(String(decoding: json, as: UTF8.self))
    }
}
```

**ARC note:** Long-lived WebSocket `class` clients should use weak delegates for UI/update callbacks.

---

## Example Projects

| Link | Description |
|------|-------------|
| [projects/oled-dashboard.md](../projects/oled-dashboard.md) | Live browser UI |
| [projects/digital-oscilloscope.md](../projects/digital-oscilloscope.md) | ADC streaming |

---

## Common Mistakes

1. **Wrong Accept hash** — handshake fails (SHA-1 + Base64).
2. **Forgetting client mask bit** — server rejects frames.
3. **No ping/pong** — NAT timeouts kill connection.
4. **Sending huge JSON blobs** — fragment or throttle.
5. **Parsing HTTP and WS on same buffer** — separate state machines.

---

## Exercises

1. Complete WebSocket upgrade manually over TCP.
2. Echo server: receive text, send back uppercase.
3. Stream ADC samples at 10 Hz to browser chart.
4. Implement ping every 30 s keepalive.

---

## References

- RFC 6455 (WebSocket Protocol)
- [projects/oled-dashboard.md](../projects/oled-dashboard.md)

---

*Prev: [http.md](./http.md)*

*Back to [Embedded Swift](../README.md)*
