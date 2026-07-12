# display/epaper.rs

- **Path:** `firmware/src/display/epaper.rs`
- **Purpose:** `Display` trait stub for Waveshare 1.54″ e-Paper. Allocates a 5000-byte 1-bit framebuffer; `flush` logs until SPI partial/full refresh is implemented.

## Component in architecture

```mermaid
flowchart LR
  APP["App::redraw"]
  EPD["EpaperDisplay HIL stub"]
  SPI["EPD SPI pins"]
  APP --> EPD -.-> SPI
  style EPD fill:#f96,stroke:#333,stroke-width:3px
```

## Responsibilities

- **Does:** own framebuffer; implement `Display` (`flush`, `framebuffer_mut`, size getters)
- **Does not:** drive SPI BUSY/RST yet (HIL)

## Key types / functions

| Item | Role |
|------|------|
| `EpaperDisplay::new` / `init` | Allocate FB / log init |
| `Display` impl | `flush` (stub Ok), `framebuffer_mut`, `width`/`height` 200 |

## Constants / formats

200×200 1-bit; pins from `board::config`.

## Dependencies

- **Outbound:** `zoop_core::io::Display`, board config size constants
- **Inbound:** `main`, `FirmwareEngine`

## Tests

Build-only (`cd firmware && cargo build`).

## Status

HIL stub.

## Related

[../../core/display/ui.md](../../core/display/ui.md), [../../architecture.md](../../architecture.md)
