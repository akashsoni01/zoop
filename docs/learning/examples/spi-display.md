# Example: SPI Display (ST7789)

**Goal:** Draw pixels on 240×240 TFT over SPI.

**Prerequisites:** [16-spi.md](../16-spi.md), [communication/spi.md](../communication/spi.md)

**Display docs:** [displays/README.md](../displays/README.md)

---

## Wiring

```
GPIO11 MOSI ── DIN
GPIO12 SCK  ── CLK
GPIO10 CS   ── CS
GPIO9  DC   ── D/C
GPIO46 RST  ── RST
3V3, GND
BL ── 3V3 (backlight)
```

---

## Rust Sketch

```rust
use mipidsi::{Builder, Orientation, models::ST7789};
use display_interface_spi::SPIInterface;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};

fn main() -> ! {
    // spi_bus + cs + dc from esp-hal
    let di = SPIInterface::new(spi, dc, cs);
    let mut display = Builder::new(ST7789, di)
        .display_size(240, 240)
        .init(&mut delay).unwrap();

    display.clear(Rgb565::BLACK).unwrap();
    Circle::new(Point::new(120, 120), 50)
        .into_styled(embedded_graphics::primitives::PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(&mut display).unwrap();
    loop {}
}
```

### Ownership

`display` owns SPI interface; exclusive access for draw calls.

---

## Compile Notes

```toml
mipidsi = "0.8"
embedded-graphics = "0.8"
embedded-hal-bus = "0.1"
```

Use [dma.md](./dma.md) for full-frame refresh.

*Next: [i2c-scanner.md](./i2c-scanner.md)*
