# Embedded Swift Display Guides

Guides for driving OLED, TFT, character LCD, and addressable LED displays from **Embedded Swift** using protocol-oriented drawing APIs and board HALs.

**Primary target board:** ESP32-S3 DevKitC-1.

---

## Prerequisites

| Lesson | Why You Need It |
|--------|-----------------|
| [08-gpio.md](../08-gpio.md) | Reset, DC/RS, backlight pins |
| [16-spi.md](../16-spi.md) | SPI displays (ILI9341, ST7789, GC9A01) |
| [17-i2c.md](../17-i2c.md) | I²C OLED (SSD1306, SH1106) |

---

## Learning Objectives

1. **Understand display controller architecture** — GRAM, scan direction, color depth, partial updates.
2. **Wire displays correctly** — SPI vs I²C, CS/DC/RST, level shifters, backlight current.
3. **Use Swift drawing protocols** — `Drawable`, fixed-size buffers, and UI layout without heap allocation.
4. **Manage framebuffers** — full buffer vs line buffer vs direct draw trade-offs on RAM-limited MCUs.
5. **Avoid common pitfalls** — wrong init sequence, endianness, ghosting, SPI mode mismatch.

---

## Folder Structure

```
displays/
├── README.md
├── ssd1306.md          ← 128×64 OLED (SSD1306, I²C/SPI)
├── sh1106.md           ← 128×64 OLED (SH1106, offset quirk)
├── ili9341.md          ← 240×320 color TFT
├── st7789.md           ← 240×240 / 135×240 IPS
├── gc9a01.md           ← 240×240 round TFT
├── neopixel-ws2812.md  ← Addressable RGB LEDs
└── lcd-character.md    ← HD44780 16×2 / 20×4
```

---

## Display Index

| Guide | Resolution | Interface | Color | RAM Needed* | Difficulty |
|-------|------------|-----------|-------|-------------|------------|
| [ssd1306.md](./ssd1306.md) | 128×64 | I²C / SPI | Mono | 1 KB | ★★☆ |
| [sh1106.md](./sh1106.md) | 128×64 | I²C / SPI | Mono | 1 KB | ★★☆ |
| [ili9341.md](./ili9341.md) | 240×320 | SPI | 16-bit RGB565 | 150 KB full / 4.8 KB line | ★★★ |
| [st7789.md](./st7789.md) | 240×240 | SPI | 16-bit RGB565 | 115 KB full / 3.6 KB line | ★★★ |
| [gc9a01.md](./gc9a01.md) | 240×240 round | SPI | 16-bit RGB565 | 115 KB | ★★★ |
| [neopixel-ws2812.md](./neopixel-ws2812.md) | 1×N chain | One-wire | RGB | Minimal | ★★☆ |
| [lcd-character.md](./lcd-character.md) | 16×2 chars | Parallel 4-bit / I²C backpack | Mono | Minimal | ★★☆ |

\*Full framebuffer at RGB565 (2 bytes/pixel). Line-buffer rendering uses ~1–2 scan lines instead.

---

## Recommended Study Order

```
1. ssd1306.md       ← Easiest: small, I²C, monochrome
2. sh1106.md        ← Compare offset/column differences
3. lcd-character.md ← Text-only, good for debug output
4. st7789.md        ← Color TFT with line-buffer pattern
5. ili9341.md       ← Larger panel, same SPI patterns
6. gc9a01.md        ← Round variant, init sequence differences
7. neopixel-ws2812  ← Different paradigm (timing, not GRAM)
```

Pair any display guide with a sensor from [sensors/](../sensors/README.md) for live data visualization.

---

## ESP32-S3 Default Wiring

| Signal | GPIO | Used By |
|--------|------|---------|
| I²C SDA | 8 | SSD1306, SH1106 (I²C) |
| I²C SCL | 9 | SSD1306, SH1106 (I²C) |
| SPI MOSI | 11 | TFT panels |
| SPI SCK | 12 | TFT panels |
| SPI MISO | 13 | (often NC on displays) |
| SPI CS | 10 | TFT chip select |
| DC / RS | 14 | Data/command select (TFT) |
| RST | 15 | Hardware reset (TFT/OLED) |
| Backlight | 16 | PWM optional |
| WS2812 DIN | 18 | NeoPixel data in |

See individual guides for module-specific pinouts.

---

## Related Sections

- [sensors/README.md](../sensors/README.md) — data sources for UI
- [16-spi.md](../16-spi.md) — SPI clock, mode, DMA
- [17-i2c.md](../17-i2c.md) — I²C pull-ups and scanning

---

*Start with [ssd1306.md](./ssd1306.md) for your first display.*
