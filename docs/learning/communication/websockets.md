# WebSockets — Full-Duplex over HTTP

**WebSockets** upgrade an HTTP connection to a **persistent, bidirectional** channel — ideal for live dashboards, remote control, and streaming sensor data without polling.

**Prerequisites:** [http.md](./http.md), [wifi.md](./wifi.md)

---

## Theory

Handshake: client sends HTTP **Upgrade** request; server responds **101 Switching Protocols**. Subsequent frames are **WebSocket framing**, not HTTP.

| Opcode | Meaning |
|--------|---------|
| 0x1 | Text frame |
| 0x2 | Binary frame |
| 0x8 | Close |
| 0x9 | Ping |
| 0xA | Pong |

**Masking:** client-to-server frames XOR payload with 4-byte key (RFC 6455).

---

## Timing Diagram (ASCII)

Upgrade handshake:

```
Client                                   Server
  │── GET /ws HTTP/1.1 ─────────────────►│
  │    Upgrade: websocket                 │
  │    Connection: Upgrade                │
  │    Sec-WebSocket-Key: dGhlIHNhbX...     │
  │    Sec-WebSocket-Version: 13          │
  │◄── HTTP/1.1 101 Switching Protocols ──│
  │    Upgrade: websocket                 │
  │    Sec-WebSocket-Accept: s3pPLMB...     │
  │══ WebSocket frames (bidirectional) ═══│
  │◄── TEXT {"temp":23.5} ────────────────│
  │─── TEXT {"cmd":"led_on"} ────────────►│
  │◄── PING ──────────────────────────────│
  │─── PONG ──────────────────────────────►│
```

---

## Packet Format

**WebSocket frame:**

| Field | Size |
|-------|------|
| FIN + opcode | 1 B |
| MASK + payload len | 1–9 B |
| Masking key | 4 B (client only) |
| Payload | N bytes |

**Sec-WebSocket-Accept** = Base64(SHA1(key + GUID)) — use `sha1` + `base64` crates.

---

## Electrical Characteristics

Same as [wifi.md](./wifi.md) / [ethernet.md](./ethernet.md). Long-lived connections increase average current vs sleep — design power budget accordingly.

---

## Rust HAL Sketch

```rust
use embedded_websocket::{WebSocket, WebSocketClient};

pub fn ws_handshake_request(key: &str, host: &str, path: &str) -> heapless::String<256> {
    let mut s = heapless::String::new();
    write!(s,
        "GET {} HTTP/1.1\r\n\
         Host: {}\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Key: {}\r\n\
         Sec-WebSocket-Version: 13\r\n\r\n",
        path, host, key
    ).ok();
    s
}

pub async fn ws_send_text(socket: &mut TcpSocket<'_>, msg: &str) {
    let mut frame_buf = [0u8; 128];
    let len = WebSocket::write_text_frame(&mut frame_buf, msg, true);
    socket.write_all(&frame_buf[..len]).await.ok();
}
```

**Ownership:** TCP socket exclusive during WebSocket session; ping/pong in background Embassy task.

---

## Bare-Metal Sketch (Concept)

After 101 response validated, parse incoming frames byte-by-byte — state machine for opcode, length, mask, payload. Respond to PING with PONG automatically.

---

## Example Projects

| Link | Use |
|------|-----|
| [projects/oled-dashboard.md](../projects/oled-dashboard.md) | Live web mirror |
| [projects/home-automation.md](../projects/home-automation.md) | Real-time control |
| [projects/digital-oscilloscope.md](../projects/digital-oscilloscope.md) | Stream samples |

---

## Common Mistakes

1. **Wrong Accept hash** — handshake fails (check GUID constant).
2. **Forgetting client masking** — server closes connection.
3. **Fragmentation not handled** — large JSON split across frames.
4. **No ping/pong** — NAT/proxy drops idle TCP.
5. **Parsing HTTP response as WebSocket** — wait for 101 before frame parser.

---

## Exercises

1. Connect to public echo WebSocket (`wss://echo.websocket.org` from PC first, then device).
2. Stream ADC values at 10 Hz as JSON text frames.
3. Handle server-initiated close (opcode 0x8).
4. Compare bandwidth WebSocket vs HTTP polling every second.

---

## References

- RFC 6455 (WebSocket Protocol)
- [embedded-websocket](https://crates.io/crates/embedded-websocket) crate
- Lesson: [http.md](./http.md)

---

*Prev: [http.md](./http.md) | Index: [README.md](./README.md)*
