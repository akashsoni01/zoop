# Core network

Host-testable WiFi connect policy, Whisper transcription orchestration, and HTTP transfer portal handlers. Firmware supplies TLS/HTTP/WiFi stacks as HIL stubs wrapping these APIs.

## Component in architecture

```mermaid
flowchart TB
  APP["app / sync UX"]
  WIFI["network/wifi"]
  WH["network/whisper"]
  PORTAL["network/portal"]
  API["STT API"]
  BROWSER["LAN browser"]
  APP --> WIFI --> WH --> API
  APP --> PORTAL --> BROWSER
  style WIFI fill:#f96,stroke:#333,stroke-width:2px
  style WH fill:#f96,stroke:#333,stroke-width:2px
  style PORTAL fill:#f96,stroke:#333,stroke-width:2px
```

[architecture.md](../../architecture.md).

## Child docs

| Doc | Source |
|-----|--------|
| [mod.md](mod.md) | `mod.rs` |
| [wifi.md](wifi.md) | `wifi.rs` |
| [portal.md](portal.md) | `portal.rs` |
| [whisper.md](whisper.md) | `whisper.rs` |

## How components interact

Sync: `advance_wifi_connect` → Connected → `transcribe_all` → optional Transfer mode with `handle_portal_request` / `serve_portal`.

## Status

Host-verified; device networking is HIL stub.
