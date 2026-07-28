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

Embedded constraints: limited RAM for buffers, prefer **HTTP/1.0** or minimal **1.1** without chunked complexity. **HTTPS** (TLS) needs mbedtls — significant flash.

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

Design for **RAM**: fixed `[UInt8; 1024]` buffers, stream parsing, avoid loading full page into heap.

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32Net

func httpGet(host: String, path: String, buffer: inout [UInt8]) async throws -> Int {
    var socket = TcpSocket()
    try await socket.connect(host: host, port: 80)
    let request = "GET \(path) HTTP/1.1\r\nHost: \(host)\r\nConnection: close\r\n\r\n"
    try await socket.write(Array(request.utf8))
    return try await socket.read(into: &buffer)
}

struct HTTPServer {
    var routes: [String: () -> HTTPResponse] = [:]

    mutating func get(_ path: String, handler: @escaping () -> HTTPResponse) {
        routes[path] = handler
    }

    func run(port: UInt16) async throws {
        let listener = try TcpListener(port: port)
        while true {
            var conn = try await listener.accept()
            if let req = try await conn.readRequest() {
                let resp = routes[req.path]?() ?? HTTPResponse.notFound
                try await conn.write(resp)
            }
        }
    }
}
```

**ARC note:** Prefer struct-based routing table over closure-heavy class routers to avoid retain cycles with connection objects.

---

## Example Projects

| Link | Description |
|------|-------------|
| [examples/http-server.md](../examples/http-server.md) | Embedded web UI |
| [projects/oled-dashboard.md](../projects/oled-dashboard.md) | Local + HTTP API |

---

## Common Mistakes

1. **No Content-Length** — client hangs waiting for body end.
2. **Buffer too small** — parse headers incrementally.
3. **Blocking server loop** — use async accept per connection.
4. **TLS without enough heap** — measure before enabling HTTPS.
5. **String concatenation in hot path** — preformat responses.

---

## Exercises

1. GET weather API and parse JSON temperature field.
2. Serve `/api/status` JSON from [examples/http-server.md](../examples/http-server.md).
3. Add POST `/config` to store settings in NVS.
4. Measure RAM usage with 3 concurrent connections.

---

## References

- RFC 9110 (HTTP Semantics)
- [examples/http-server.md](../examples/http-server.md)

---

*Prev: [mqtt.md](./mqtt.md) | Next: [websockets.md](./websockets.md)*

*Back to [Embedded Swift](../README.md)*
