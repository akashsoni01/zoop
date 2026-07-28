# SPI — Serial Peripheral Interface

**SPI** (Serial Peripheral Interface) is a **synchronous, full-duplex** bus: one **clock (SCK)**, **Master Out Slave In (MOSI)**, **Master In Slave Out (MISO)**, plus per-device **Chip Select (CS)** lines.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [gpio.md](./gpio.md)

---

## Theory

The **master** generates SCK and selects a slave by driving CS **LOW**. On each clock edge (mode-dependent), one bit shifts out on MOSI and one bit shifts in on MISO — **simultaneous** transfer.

**SPI modes** (CPOL, CPHA):

| Mode | CPOL | CPHA | Sample edge |
|------|------|------|-------------|
| 0 | 0 | 0 | Rising |
| 1 | 0 | 1 | Falling |
| 2 | 1 | 0 | Falling |
| 3 | 1 | 1 | Rising |

ESP32-S3 SPI hosts support high clock rates (tens of MHz) for displays and flash. Each slave needs a dedicated **CS**; sharing MOSI/MISO/SCK is standard.

---

## Timing Diagram (ASCII)

Mode 0 — CS active LOW, data sampled on rising edge:

```
CS   ────┐                              ┌────
         └──────────────────────────────┘

SCK  ────┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌────
         └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘

MOSI ────X─D7─X─D6─X─D5─X─D4─X─D3─X─D2─X─D1─X─D0─X───
         (MSB or LSB first — check datasheet!)

MISO ────X─Q7─X─Q6─X─Q5─X─Q4─X─Q3─X─Q2─X─Q1─X─Q0─X───
```

CS must stay LOW for the entire transaction (command + data).

---

## Packet Format

SPI has no universal packet — device datasheets define **command bytes** and **payloads**:

| Device | Typical transaction |
|--------|---------------------|
| SPI Flash | `[CMD:8][ADDR:24][DUMMY:n][DATA...]` |
| ST7789 display | `[CMD/DATA bit via D/C pin][byte...]` |
| LoRa SX1262 | `[opcode:8][params...]` |

Generic transfer pattern:

```swift
// Write: MOSI only (MISO ignored or 0xFF dummy read)
// Transfer: simultaneous write/read buffers of equal length
```

---

## Electrical Characteristics

| Parameter | Guideline |
|-----------|-----------|
| Logic level | 3.3 V on ESP32-S3 |
| Max SCK (short wire) | 10–80 MHz (device dependent) |
| CS idle | HIGH (inactive) |
| MISO when CS high | High-Z (tri-state) |
| Trace length | Keep < 20 cm for high speed |

Use **series 33–100 Ω** resistors on SCK/MOSI to reduce ringing. For multiple slaves, **one CS per slave** — never tie CS lines together.

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32Hardware

struct SPIDisplay {
    var bus: SPI2
    var dc: GPIO.Output<GPIO4>
    var cs: GPIO.Output<GPIO5>

    mutating func writeCommand(_ cmd: UInt8) throws {
        cs.setLow()
        dc.setLow()
        try bus.write([cmd])
        cs.setHigh()
    }

    mutating func writeData(_ bytes: [UInt8]) throws {
        cs.setLow()
        dc.setHigh()
        try bus.write(bytes)
        cs.setHigh()
    }
}
```

**Ownership:** `SPIDisplay` owns CS and D/C pins exclusively — compile-time single owner prevents bus conflicts when modeled as separate structs sharing one `SPI2` via a mutex wrapper.

**Compile (ESP32-S3):**

```bash
swift build -c release --product SpiDisplayDemo
espflash flash --monitor .build/release/SpiDisplayDemo.bin
```

---

## Bare-Metal Sketch (Concept)

```swift
// Bit-bang SPI Mode 0 — slow but educational
func spiWriteByte(
    sck: inout GPIO.Output<GPIO6>,
    mosi: inout GPIO.Output<GPIO7>,
    byte: UInt8
) {
    for i in stride(from: 7, through: 0, by: -1) {
        let bit = (byte >> i) & 1
        mosi.set(bit != 0)
        sck.setHigh()
        sck.setLow()
    }
}
```

Prefer hardware SPI + DMA for displays — see [examples/dma.md](../examples/dma.md).

---

## Example Projects

| Link | Device |
|------|--------|
| [examples/spi-display.md](../examples/spi-display.md) | ST7789 TFT |
| [examples/oled.md](../examples/oled.md) | SPI OLED variant |
| [lora.md](./lora.md) | SX1262 radio |
| [projects/oled-dashboard.md](../projects/oled-dashboard.md) | SPI dashboard |

---

## Common Mistakes

1. **Wrong SPI mode** — read datasheet CPOL/CPHA.
2. **CS toggled mid-transaction** — corrupt command sequences.
3. **MSB vs LSB first** — mismatched bit order.
4. **Sharing bus without mutex** — two drivers fight for CS/SCK.
5. **No D/C pin on displays** — command/data confused.

---

## Exercises

1. Bit-bang read JEDEC ID from SPI flash (0x9F command).
2. Drive [examples/spi-display.md](../examples/spi-display.md) at 40 MHz; measure with logic analyzer.
3. Share one SPI bus between display and LoRa using a `BusMutex` struct.
4. Compare blocking SPI vs DMA throughput.

---

## References

- ST7789 datasheet
- [examples/spi-display.md](../examples/spi-display.md)
- Lesson: [07-hal.md](../07-hal.md)

---

*Prev: [uart-usart.md](./uart-usart.md) | Next: [i2c.md](./i2c.md)*

*Back to [Embedded Swift](../README.md)*
