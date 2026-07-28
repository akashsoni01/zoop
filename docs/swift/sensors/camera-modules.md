# Camera Modules — OV2640, ESP32-S3

Guide to **OV2640** and **ESP32-S3 camera** integration for Embedded Swift imaging projects.

**Prerequisites:** [16-spi.md](../16-spi.md), [08-gpio.md](../08-gpio.md)

---

## Working Principle

**OV2640** CMOS sensor outputs **parallel DVP** (8-bit data + pixel/line sync) or SCCB (I²C-like) for configuration. ESP32-S3 integrates **LCD_CAM** peripheral supporting DVP capture and JPEG encoding in hardware on some stacks.

Image path: lens → sensor → DVP → DMA → frame buffer → processing or Wi-Fi stream.

---

## Datasheet Notes

| Parameter | OV2640 |
|-----------|--------|
| Resolution | Up to UXGA 1600×1200 |
| Interface | DVP 8-bit, SCCB config |
| SCCB address | `0x30` |
| Formats | RGB565, YUV, JPEG |
| Clock | XCLK typically 20 MHz from MCU |

---

## Protocol

**SCCB** (I²C compatible) for hundreds of sensor registers — exposure, resolution, format.

**DVP parallel bus**: XCLK (MCU → sensor), PCLK, VSYNC, HREF, D0–D7.

See [17-i2c.md](../17-i2c.md) for SCCB; [08-gpio.md](../08-gpio.md) for parallel pins.

---

## Register Map (OV2640 essentials)

| Reg | Name | Description |
|-----|------|-------------|
| `0xFF` | BANK_SEL | Register bank 0/1 |
| `0x12` | COM7 | Reset, format select |
| `0x11` | CLKRC | Clock prescaler |
| `0x0C` | COM3 | |
| `0xDA`–`0xDD` | AVG window | AEC |

Full init tables are hundreds of registers — use vendor reference sequences.

---

## ESP32-S3 Wiring (Typical AI-Thinker style)

```
ESP32-S3          OV2640 Module
────────          ─────────────
GPIO15 ─────────► XCLK
GPIO4  ◄───────── D0
GPIO5  ◄───────── D1
...
GPIO11 ◄───────── D7
GPIO6  ◄───────── VSYNC
GPIO7  ◄───────── HREF
GPIO13 ◄───────── PCLK
GPIO8  ─────────► SDA (SCCB)
GPIO9  ─────────► SCL (SCCB)
3V3    ─────────► 3.3V
GND    ─────────► GND
```

Exact pin map varies by board — match your module schematic.

---

## Swift Driver Sketch

```swift
protocol SCCBBus {
    func writeReg(_ reg: UInt8, _ val: UInt8) throws
    func readReg(_ reg: UInt8) throws -> UInt8
}

protocol CameraCapture {
    mutating func initSensor() throws
    mutating func captureFrame(into buffer: inout [UInt8]) throws -> (width: Int, height: Int)
}

struct OV2640<C: SCCBBus> {
    var sccb: C

    mutating func reset() throws {
        try sccb.writeReg(0xFF, 0x01) // bank 1
        try sccb.writeReg(0x12, 0x80) // soft reset
        delayMs(100)
    }

    mutating func initQVGA_RGB565() throws {
        // Load reference register table for QVGA
        try loadRegisterTable(ov2640_qvga_rgb565)
    }

    private mutating func loadRegisterTable(_ table: [(UInt8, UInt8)]) throws {
        for (reg, val) in table {
            try sccb.writeReg(reg, val)
        }
    }
}
```

ESP32-S3 frame grab uses platform `LCD_CAM` + DMA — wrap in `ESPS3CameraCapture`.

---

## Bare-Metal Notes

- Camera draws 100–200 mA peak — robust 3.3 V supply required.
- PSRAM required for VGA+ buffers on ESP32-S3.
- Lens cap off during init — black image otherwise.

---

## HAL / Protocol-Oriented Driver Notes

Split SCCB config from DVP DMA capture. `Camera` protocol returns frame metadata + buffer view.

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Black image | Wrong init table | Use known-good sequence |
| SCCB NACK | Wrong pins | Verify SDA/SCL |
| Tear/corruption | DMA too slow | Reduce resolution; use PSRAM |

---

## Example Project

**Motion snapshot:** On PIR trigger, capture QVGA JPEG to flash; upload over Wi-Fi.

---

## References

- [OV2640 Datasheet (OmniVision)](https://www.uctronics.com/download/cam_module/OV2640DS.pdf)
- [ESP32-S3 Camera Driver Guide](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/peripherals/lcd_cam.html)
- [16-spi.md](../16-spi.md), [08-gpio.md](../08-gpio.md)
