# Example: DMA — Fast SPI Transfer

**Goal:** Stream framebuffer to display without CPU bit-banging every byte.

**Prerequisites:** [spi-display.md](./spi-display.md), [14-dma.md](../14-dma.md)

**Protocol:** [spi.md](../communication/spi.md)

---

## Rust Sketch

```rust
use esp_hal::dma::{Dma, DmaPriority};
use esp_hal::spi::master::Spi;

fn flush_frame(spi: &mut Spi<'_, esp_hal::Blocking>, dma: &Dma, buf: &[u8]) {
    // DMA TX: CPU sets up descriptor, hardware shifts bytes to MOSI
    spi.write_with_dma(buf, dma).unwrap();
    // Wait completion interrupt or poll busy
}

// Ownership: `buf` must stay valid until DMA completes — pin stack buffer
// or static frame buffer; do NOT use stack frame if function returns early
static FB: StaticCell<[u8; 11520]> = StaticCell::new(); // 240×240×2 bytes example
```

### Ownership (Critical)

DMA holds a **pointer** to your buffer. The buffer must live until transfer completes — `'static` or stack scope guaranteed. Rust cannot enforce this at compile time for all HALs; use `'static` for safety.

---

## Compile Notes

Enable `dma` feature in `esp-hal`. See [displays/](../displays/README.md) for panel sizes.

*Next: [uart-echo.md](./uart-echo.md)*
