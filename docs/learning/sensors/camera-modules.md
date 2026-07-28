# Camera Modules (OV2640, ESP32 Camera)

**Imaging** on ESP32-S3 uses the **ESP32 camera driver** pipeline with **OV2640** / **OV5640** sensors over **parallel DVP** interface and **I²C** configuration.

**Prerequisites:** [17-i2c.md](../17-i2c.md), [16-spi.md](../16-spi.md), PSRAM recommended

---

## Working Principle

**CMOS image sensor** exposes grid of photodiodes; on-chip ISP outputs **JPEG** or **RGB/YUV** frames. ESP32-S3 **LCD_CAM** peripheral captures parallel data with **XCLK** master clock from MCU.

OV2640: up to **UXGA (1600×1200)**, common **QVGA/VGA** for MCU RAM limits.

---

## Datasheet Key Points (OV2640)

| Parameter | Value |
|-----------|-------|
| Config interface | SCCB (I²C-like), addr `0x30` |
| Data interface | 8-bit parallel DVP |
| Formats | JPEG, RGB565, YUV422 |
| Clock | XCLK 10–24 MHz typical |
| Power | 1.3 V core + 2.5 V analog + 3.3 V I/O |

---

## Protocol

### SCCB (I²C) Sensor Configuration

Write **bank select** + register tables ( hundreds of entries ) for resolution, exposure, white balance.

### DVP Parallel Pins

| Signal | Description |
|--------|-------------|
| D0–D7 | Pixel data |
| VSYNC | Frame start |
| HREF | Line valid |
| PCLK | Pixel clock |
| XCLK | Input clock from ESP32 |

---

## Register Map (OV2640 Overview)

OV2640 uses **banked** registers via `0xFF`:

| Bank | Purpose |
|------|---------|
| 0x01 | DSP |
| 0x00 | Sensor |

Key registers (sensor bank):

| Reg | Description |
|-----|-------------|
| `0xFF` | Bank select |
| `0x12` | COM7 — reset, format select |
| `0x11` | CLKRC — clock prescale |
| `0x0C` | COM3 |
| PID `0x0A` | Product ID = 0x26 |

**Do not hand-init** — use Espressif sensor config tables or `esp32-camera` component port.

---

## ESP32-S3 Wiring (AI-Thinker Style Module)

Typical ESP32-CAM pin mapping (verify your module schematic):

```
Signal    ESP32-S3 (example)
──────    ──────────────────
SIOD      GPIO8  (I²C SDA)
SIOC      GPIO9  (I²C SCL)
Y2..Y9    GPIO10-17 (D0-D7)
VSYNC     GPIO6
HREF      GPIO7
PCLK      GPIO13
XCLK      GPIO15
PWDN      GPIO16
RESET     GPIO17
```

**PSRAM required** for VGA buffers — ESP32-S3-WROOM with OPI PSRAM recommended.

---

## Rust Driver Sketch

Rust ecosystem for ESP32 camera is evolving — typical approach uses `esp-hal` + community crates or FFI to sensor init tables:

```rust
// Conceptual — check esp-rs book for current camera support
pub struct CameraConfig {
    pub frame_size: FrameSize, // QVGA, VGA
    pub pixel_format: PixelFormat, // Jpeg, Rgb565
    pub fb_count: u8,
    pub grab_mode: GrabMode,
}

pub fn init_camera(cfg: CameraConfig) -> Result<Camera, CamError> {
    // 1. Enable XCLK via LEDC/MCPWM
    // 2. SCCB init register table for OV2640
    // 3. Configure LCD_CAM peripheral
    // 4. Allocate frame buffers in PSRAM
    todo!()
}

pub fn capture_frame(cam: &mut Camera) -> Result<&'static [u8], CamError> {
    // blocking or async DMA complete
    todo!()
}
```

**Resources:** `esp32-camera` (C), `esp-idf-hal` examples; search `esp-rs` for `esp32-camera-driver`.

---

## Bare-Metal Notes

- **PSRAM** — QVGA RGB565 ≈ 150 KB; double buffering needs external RAM.
- **JPEG** mode reduces size (10–50 KB) — preferred for streaming.
- **XCLK** must run before SCCB init.
- **Power sequencing** — PWDN/RESET timing per Omnivision guide.
- **Heat** — continuous streaming heats sensor; affects noise.

---

## HAL / embedded-hal Notes

Camera pipeline is **SOC-specific** — not portable via `embedded-hal`. Abstract at application:

```rust
pub trait FrameCapture {
    type Error;
    fn capture_jpeg(&mut self) -> Result<&[u8], Self::Error>;
}
```

Use for HTTP upload or SD card write on ESP32-S3.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Black image | Lens cap / PWDN | Deassert PWDN; remove cap |
| SCCB NACK | Wrong pins | Match module pinout |
| DMA fail | No PSRAM | Enable OPI PSRAM in sdkconfig |
| Green tint | WB not calibrated | Load AWB register set |
| Flicker | 50/60 Hz mains | Set banding filter regs |

---

## Example Project: Motion Snapshot

1. QVGA JPEG @ 1 fps normally.
2. VL53L0X distance drop triggers burst 5 fps ([tof-vl53l0x.md](./tof-vl53l0x.md)).
3. Store JPEG to flash ([flash-memory.md](./flash-memory.md)).

---

## Exercises

1. Capture QVGA JPEG; print size in bytes over `defmt`.
2. Stream MJPEG over HTTP (ESP32-S3 Wi-Fi).
3. Compare SCCB read of PID register vs expected 0x26.

---

## References

- Omnivision OV2640 datasheet
- ESP32-S3 Technical Reference — LCD_CAM chapter
- [Espressif Rust Book](https://esp-rs.github.io/book/) — camera sections
- [17-i2c.md](../17-i2c.md), [flash-memory.md](./flash-memory.md)
