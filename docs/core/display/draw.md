# display/draw.rs

- **Path:** `core/src/display/draw.rs`
- **Purpose:** 200×200 monochrome framebuffer primitives and 5×7 bitmap font (ports `draw.cpp`).
- **Key types / functions:**
  - Consts: `WIDTH`, `HEIGHT`, `BYTES`, `BLACK`, `WHITE`
  - Pixels: `set_pixel`, `get_pixel`, `fill_rect`, `hline`, `vline`, `line`
  - Shapes: `fill_circle`, `stroke_circle`, `draw_battery_ring`
  - Text: `text_width`, `draw_str`, `draw_str_centered`, `draw_header`, `draw_hints`
- **Dependencies:** None (stdlib `f32` for circles)
- **Tests:** `cargo test -p zoop-core display::draw::tests`
- **Status:** Host-verified
- **Related:** [ui.md](ui.md), [../../firmware/display/draw.md](../../firmware/display/draw.md)
