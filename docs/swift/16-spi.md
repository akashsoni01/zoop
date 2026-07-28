# Lesson 16 — SPI (Serial Peripheral Interface)

**Prerequisites:** [08-gpio.md](./08-gpio.md), [14-dma.md](./14-dma.md)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md). Bare-metal SPI PoC exists on **ESP32-C6**; S3 via ESP-IDF is the practical Swift path today.

---

## Theory

**SPI** is a **synchronous** full-duplex bus: each clock edge shifts one bit on MOSI and MISO simultaneously. One **master** drives **SCLK**; slaves share MOSI/MISO with individual **CS** (active low).

### Signals

| Signal | Direction (master) | Role |
|--------|-------------------|------|
| SCLK | Output | Bit clock |
| MOSI | Output | Master Out, Slave In |
| MISO | Input | Master In, Slave Out |
| CS / SS | Output per slave | Chip select |

### SPI Modes (CPOL / CPHA)

| Mode | CPOL | CPHA | Idle clock |
|------|------|------|------------|
| 0 | 0 | 0 | Low — sample rising |
| 1 | 0 | 1 | Low — sample falling |
| 2 | 1 | 0 | High — sample falling |
| 3 | 1 | 1 | High — sample rising |

**Check the slave datasheet** — ST7789 displays, W25Q flash, and IMUs specify mode and max frequency.

### Transactions

Assert CS → exchange N bytes → deassert CS. Some devices require CS high between commands.

---

## Hardware Overview

### ESP32-S3 SPI

- **SPI0/1:** bound to internal flash — do not use for external devices.
- **SPI2/SPI3:** general purpose (legacy HSPI/VSPI naming).
- Clock up to ~80 MHz (wiring often limits to 10–40 MHz).
- GDMA support — [14-dma.md](./14-dma.md).

### Typical devices

| Device | Speed | Mode |
|--------|-------|------|
| ST7789 TFT | 20–40 MHz | 0 |
| W25Q flash | 40+ MHz | 0/3 |
| BME280 | — | I²C variant common — [17-i2c.md](./17-i2c.md) |

---

## ASCII Wiring

### SPI master + ST7789 display

```
ESP32-S3                ST7789 Display
┌──────────┐            ┌──────────┐
│ GPIO12   ├── SCLK ───►│ SCL      │
│ GPIO11   ├── MOSI ───►│ SDA/MOSI │
│ GPIO13   │◄─ MISO ────┤ (NC often)│
│ GPIO10   ├── CS ─────►│ CS       │
│ GPIO9    ├── DC ─────►│ DC/RS     │
│ GPIO14   ├── RST ────►│ RESET    │
│ 3V3      ├───────────►│ VCC      │
│ GND      ├───────────►│ GND      │
└──────────┘            └──────────┘
```

Keep wires short; 33 Ω series resistors on SCLK/MOSI if ringing occurs.

---

## Memory & Register Notes

### ESP32-S3 SPI registers (conceptual)

| Register | Purpose |
|----------|---------|
| `SPI_CTRL0` | Bit length, CPOL/CPHA |
| `SPI_CLOCK` | Divider, duty cycle |
| `SPI_USER` | Start transaction, CS setup |
| `SPI_W0..W15` | TX/RX FIFO buffers |
| `SPI_DMA_CONF` | GDMA link |

Display framebuffers often live in **PSRAM** on S3 — verify DMA can reach PSRAM for your HAL path.

---

## HAL Swift Example — SPI Transaction (ESP-IDF)

```swift
import ESPIDF

struct ST7789 {
    let spi: SPIBus
    let csPin: GPIO
    let dcPin: GPIO
    let rstPin: GPIO

    func writeCommand(_ cmd: UInt8) {
        dcPin.set(low: true)  // command mode
        csPin.set(low: true)
        spi.transfer([cmd])
        csPin.set(low: false)
    }

    func writeData(_ data: [UInt8]) {
        dcPin.set(low: false) // data mode
        csPin.set(low: true)
        spi.transfer(data)
        csPin.set(low: false)
    }

    func initDisplay() {
        rstPin.set(low: true)
        delay(ms: 10)
        rstPin.set(low: false)
        delay(ms: 120)
        writeCommand(0x11) // Sleep out
        delay(ms: 120)
        // ... mode-specific init sequence from datasheet
    }
}

@main
struct SpiDisplayApp {
    static func main() {
        let spi = SPIBus(
            host: 2,           // SPI2
            mosi: 11, sclk: 12, miso: 13,
            mode: 0,
            frequencyHz: 40_000_000
        )
        let display = ST7789(
            spi: spi,
            csPin: GPIO(10),
            dcPin: GPIO(9),
            rstPin: GPIO(14)
        )
        display.initDisplay()
    }
}
```

---

## Bare-Metal Swift Sketch

```swift
import MMIO

let spi2 = SPI2(baseAddress: 0x6002_4000)

func spiWriteByte(_ byte: UInt8) -> UInt8 {
    spi2.user.usr_mosi.set(true)
    spi2.user.usr_miso.set(true)
    spi2.w0.set(UInt32(byte))
    spi2.cmd.update.set(true)
    while spi2.cmd.update.get() { }
    return UInt8(truncatingIfNeeded: spi2.w0.get())
}

func spiTransfer(_ buffer: inout [UInt8]) {
    for i in buffer.indices {
        buffer[i] = spiWriteByte(buffer[i])
    }
}
```

---

## Host Swift — SPI via USB Bridge (development)

Many teams prototype display drivers on macOS with a USB-SPI adapter (e.g., FT232H + libmpsse), then port to MCU:

```swift
import Foundation

protocol SPIBusProtocol {
    func transfer(_ data: [UInt8]) -> [UInt8]
}

struct DisplayDriver<Bus: SPIBusProtocol> {
    let bus: Bus
    func fillScreen(color: UInt16) {
        // Send command + pixel data over SPI bridge
    }
}
```

Same driver logic compiles for host mock and embedded HAL with protocol abstraction — [27-design-patterns.md](./27-design-patterns.md).

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Wrong SPI mode | Garbage pixels / no response | Match CPOL/CPHA to datasheet |
| CS left low between commands | Device ignores commands | Deassert CS between transactions if required |
| Using SPI0 pins | Boot failure / flash corruption | Use SPI2/SPI3 only |
| MISO floating | Random RX bytes | Tie MISO or disable MISO in config |
| DC pin wrong | Solid color screen | DC low=cmd, high=data for ST7789 |
| Clock too fast on breadboard | Corrupted transfers | Reduce to 10 MHz for bring-up |

---

## Debugging Tips

- Logic analyzer on SCLK/MOSI/CS — verify mode and timing.
- Scope CS setup/hold vs clock edges.
- Read JEDEC ID from flash (`0x9F`) as first SPI test.
- Compare against Arduino/C reference on same wiring.

---

## Performance Tips

- Use **DMA** for display fills — [14-dma.md](./14-dma.md).
- Batch small writes into one CS assertion when protocol allows.
- Place framebuffer in PSRAM on S3 for large panels.
- Use **quad-SPI** for flash reads when supported (not general SPI lesson scope).

---

## Exercises

1. **JEDEC ID read:** Read manufacturer ID from W25Q on module flash pins (careful — not SPI0).
2. **ST7789 color bars:** Init display and draw vertical color bars.
3. **SPI loopback:** Connect MOSI→MISO; verify `transfer` returns sent bytes.
4. **DMA fill:** Fill 320×240 RGB565 buffer via GDMA — measure FPS.
5. **Protocol abstraction:** Write `SPIDevice` protocol with host mock — [26-testing.md](./26-testing.md).

---

## References

- [ESP32-S3 TRM — SPI](https://www.espressif.com/en/products/socs/esp32-s3)
- [swift-embedded-examples](https://github.com/swiftlang/swift-embedded-examples)
- [georgik/esp32-c6-swift-baremetal — SPI display PoC](https://github.com/georgik/esp32-c6-swift-baremetal)
- [14-dma.md](./14-dma.md)
- [17-i2c.md](./17-i2c.md)

---

*Previous: [15-uart.md](./15-uart.md) · Next: [17-i2c.md](./17-i2c.md)*
