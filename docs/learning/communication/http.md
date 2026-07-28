# HTTP — Hypertext Transfer Protocol

**HTTP** is the **request/response** application protocol of the web. Embedded devices act as **clients** (fetch API data) or **servers** (serve configuration UI, REST endpoints).

**Prerequisites:** [wifi.md](./wifi.md), [ethernet.md](./ethernet.md)

---

## Theory

HTTP/1.1 over TCP:

| Method | Use |
|--------|-----|
| GET | Read resource |
| POST | Submit data |
| PUT/PATCH | Update |
| DELETE | Remove |

Status codes: **200 OK**, **404 Not Found**, **500 Server Error**.

Embedded constraints: limited RAM for buffers, prefer **HTTP/1.0** or minimal **1.1** without chunked complexity. **HTTPS** (TLS) needs `esp-mbedtls` or Rustls — significant flash.

---

## Timing Diagram (ASCII)

HTTP GET request/response:

```
Client (ESP32-S3)                    Server
  │── TCP SYN ───────────────────────►│
  │◄── SYN-ACK ───────────────────────│
  │── ACK ────────────────────────────►│
  │── GET /api/status HTTP/1.1 ───────►│
  │    Host: device.local              │
  │    Connection: close               │
  │◄── HTTP/1.1 200 OK ───────────────│
  │    Content-Length: 42              │
  │    { "temp": 23.5 }                │
  │── FIN ────────────────────────────►│
```

---

## Packet Format

**Request line:**

```
GET /path HTTP/1.1\r\n
Host: example.com\r\n
\r\n
```

**Response:**

```
HTTP/1.1 200 OK\r\n
Content-Type: application/json\r\n
Content-Length: 15\r\n
\r\n
{"status":"ok"}
```

Below HTTP: **TCP segments** in **IP packets** on [wifi.md](./wifi.md) or [ethernet.md](./ethernet.md).

---

## Electrical Characteristics

Same as underlying network interface. HTTP itself has no electrical properties.

Design for **RAM**: fixed `[u8; 1024]` buffers, stream parsing, avoid loading full page into heap.

---

## Rust HAL Sketch

```rust
use embassy_net::tcp::TcpSocket;
use httparse::{Request, Headers};

pub async fn http_get(
    socket: &mut TcpSocket<'static>,
    host: &str,
    path: &str,
    buf: &mut [u8],
) -> usize {
    socket.connect((host, 80)).await.unwrap();
    let req = heapless::String::<128>::new();
    write!(req, "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n", path, host).ok();
    socket.write_all(req.as_bytes()).await.ok();
    socket.read(buf).await.unwrap_or(0)
}

// Server: esp-idf-svc HttpServer or picoserve (Embassy)
```

**Ownership:** Socket borrowed mutably for request lifetime; response buffer owned by caller stack array.

---

## Bare-Metal Sketch (Concept)

After TCP connect, manually format ASCII request bytes; parse response by finding `\r\n\r\n` header end, then read `Content-Length` body bytes.

---

## Example Projects

| Link | Use |
|------|-----|
| [examples/http-server.md](../examples/http-server.md) | On-device web UI |
| [examples/ota-update.md](../examples/ota-update.md) | Firmware download |
| [projects/oled-dashboard.md](../projects/oled-dashboard.md) | Status API |
| [projects/weather-station.md](../projects/weather-station.md) | Upload to cloud |

---

## Common Mistakes

1. **No timeout on read** — hang forever if server slow.
2. **Assuming single read returns full response** — loop until Content-Length satisfied.
3. **Missing Host header** — virtual hosts reject request.
4. **TLS without cert validation** — security risk (pin cert in prod).
5. **Blocking server in main loop** — use async or dedicated task.

---

## Exercises

1. GET weather API; parse JSON temperature with `serde-json-core`.
2. Serve `/` and `/api/sensors` from [examples/http-server.md](../examples/http-server.md).
3. Implement HEAD request for OTA size check.
4. Measure heap/flash with vs without TLS.

---

## References

- RFC 7230–7235 (HTTP/1.1)
- [picoserve](https://github.com/sammycorp/picoserve) — embedded HTTP server
- [esp-idf-svc HTTP client](https://docs.rs/esp-idf-svc/latest/esp_idf_svc/http/client/index.html)
- Lesson: [20-wifi.md](../20-wifi.md)

---

*Prev: [mqtt.md](./mqtt.md) | Next: [websockets.md](./websockets.md)*
