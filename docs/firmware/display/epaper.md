# display/epaper.rs

- **Path:** `firmware/src/display/epaper.rs`
- **Purpose:** E-Paper display driver stub — in-memory framebuffer implementing `Display`.
- **Key types / functions:**
  - `EpaperDisplay::new`, `init`
  - `Display`: `flush` (logs), `framebuffer_mut`, `width`/`height`
- **Dependencies:** `zoop_core::{display::draw::BYTES, io::Display}`, `board::config`
- **Tests:** Framebuffer logic host-verified in `core/display`
- **Status:** HIL stub (SPI partial refresh pending)
- **Related:** [draw.md](draw.md), [../../core/display/ui.md](../../core/display/ui.md)
