# NeoPixel / WS2812 — Addressable RGB LEDs

**WS2812** (NeoPixel) RGB LEDs chain on a **single data line** — each pixel latches 24-bit color and forwards the rest.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [10-timers.md](../10-timers.md), [11-pwm.md](../11-pwm.md)

---

## Theory

Each LED contains **controller + RGB die**. Protocol is **self-clocked NRZ** — no separate clock wire.

Bit encoding (typical WS2812):

| Bit | High time | Low time |
|-----|-----------|----------|
| 0 | ~0.35 µs | ~0.80 µs |
| 1 | ~0.70 µs | ~0.60 µs |

Reset gap: **>50 µs** LOW ends latch frame.

Color order on wire is usually **GRB** (not RGB) — verify datasheet (SK6812 = RGBW).

---

## Wiring (ESP32-S3)

```
ESP32-S3              WS2812 Strip/Ring
────────              ────────────────
GPIO18 (DIN) ───────► DIN (first pixel)
5V (VIN/USB) ───────► VCC  (5V recommended for brightness)
GND          ───────► GND  (common ground with ESP32!)

Optional: 330 Ω resistor in series on DIN
Optional: 1000 µF cap across VCC/GND at strip start
```

```
     ESP32-S3                         NeoPixel Chain
   ┌──────────┐    330Ω    DIN   DOUT   DIN   DOUT
   │ GPIO18 ──┼────/\/\/───►[PX0]──────►[PX1]──────► ...
   │ GND    ──┼────────────────────────────────────── GND
   │ 5V     ──┼─── (external 5V supply recommended >8 LEDs)
   └──────────┘
```

**Power:** 60 mA per pixel full white — 30 pixels = 1.8 A. Do not power long strips from ESP32 3V3/5V pin alone.

Level shifting: ESP32-S3 GPIO is **3.3 V**; many WS2812 accept 3.3 V data at short distance; use 74HCT125 level shifter for reliability.

See [08-gpio.md](../08-gpio.md) for open-drain vs push-pull considerations.

---

## Protocol

Not I²C/SPI — **timing-critical one-wire**. Options on ESP32-S3:

| Method | Notes |
|--------|-------|
| RMT peripheral | Preferred — hardware timing |
| SPI abuse | Encode bits as SPI bytes @ ~2.4 MHz |
| PIO (RP2040) | Not on ESP32 |
| `smart-leds` + `esp-hal-rmt` | Rust ecosystem |

Frame = `[G,R,B]_0, [G,R,B]_1, ...` + reset quiet time.

---

## Framebuffer Concepts

Logical buffer: `[(u8,u8,u8); N]` or `[GRB; N]`.

**Gamma correction:** Human brightness perception is non-linear — apply:

```rust
fn gamma8(v: u8) -> u8 {
    GAMMA_TABLE[v as usize] // precomputed 256 entries
}
```

**HSL → RGB** for smooth fades without yellowing.

No display GRAM — latch entire chain on each update.

---

## Rust + embedded-graphics

NeoPixel isn't a raster display — but `smart-leds` + `smart-leds-trait` integrate with patterns:

```toml
[dependencies]
smart-leds = "0.4"
ws2812-spi = "0.5"  # SPI backend
# esp-hal RMT examples for ESP32-S3
```

```rust
use smart_leds::{SmartLedsWrite, RGB8};

struct StripWriter<Rmt> { rmt: Rmt }

impl<Rmt: SmartLedsWrite> StripWriter<Rmt> {
    fn set_pixel(&mut self, leds: &mut [RGB8]) -> Result<(), Rmt::Error> {
        self.rmt.write(leds.iter().cloned())
    }
}

// Map 1D strip to logical "display":
fn set_bar(leds: &mut [RGB8], level: u8) {
    let n = (level as usize * leds.len()) / 255;
    for (i, px) in leds.iter_mut().enumerate() {
        *px = if i < n { RGB8 { r: 0, g: 80, b: 0 } } else { RGB8::default() };
    }
}
```

For 2D matrices wired in serpentine order, map `(x,y)` → index:

```rust
fn xy_to_index(x: usize, y: usize, w: usize, h: usize) -> usize {
    if y % 2 == 0 { y * w + x } else { y * w + (w - 1 - x) }
}
```

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| No common GND | Random colors | Tie ESP32 GND to strip GND |
| Insufficient power | Brownout/reboot | External 5V supply |
| Wrong color order | Red/green swapped | Use GRB in driver |
| Interrupts during RMT | Glitch pixels | Use hardware RMT/DMA |
| First pixel wrong | Missing level shifter | 74HCT125 on DIN |
| Long strip + thin wire | Voltage drop | Inject power every 50 LEDs |

---

## Exercises

1. **Rainbow cycle** — HSV hue walk across 24 pixels @ 60 FPS.
2. **Audio VU** — Map microphone RMS ([microphones.md](../sensors/microphones.md)) to bar graph.
3. **Text scroll** — 8×8 font on 32×8 matrix; serpentine index math.
4. **Power measure** — INA219 on 5V rail during all-white ([current-ina219.md](../sensors/current-ina219.md)).
5. **Compare RMT vs SPI** — Timing jitter with logic analyzer.

---

## Example Project: Lux-Reactive Ambient Light

[BH1750](../sensors/light-bh1750.md) adjusts max NeoPixel brightness (0–128 cap) to avoid glare at night.

---

## References

- WorldSemi WS2812 datasheet / timing diagram
- Espressif RMT documentation
- [10-timers.md](../10-timers.md), [11-pwm.md](../11-pwm.md)
- `smart-leds`, `esp-hal` RMT examples
