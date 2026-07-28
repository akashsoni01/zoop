# Lesson 16 — SPI (Serial Peripheral Interface)

**Prerequisites:** [08-gpio.md](./08-gpio.md), [14-dma.md](./14-dma.md)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md). Examples also note STM32, RP2040.

---

## Theory

**SPI** (Serial Peripheral Interface) is a **synchronous** full-duplex bus: each clock edge shifts one bit on MOSI and one bit on MISO simultaneously. One **master** drives the clock (**SCLK**); one or more **slaves** share MOSI/MISO but have individual **CS** (Chip Select, active low).

### Signals

| Signal | Direction (master view) | Role |
|--------|-------------------------|------|
| SCLK | Output | Bit clock |
| MOSI | Output | Master Out, Slave In |
| MISO | Input | Master In, Slave Out |
| CS / SS | Output (per slave) | Select which device listens |

### SPI Modes (CPOL / CPHA)

**CPOL** (Clock Polarity): idle clock level — 0 = idle low, 1 = idle high.  
**CPHA** (Clock Phase): sample on leading (0) or trailing (1) edge.

| Mode | CPOL | CPHA | Idle clock |
|------|------|------|------------|
| 0 | 0 | 0 | Low — sample on rising |
| 1 | 0 | 1 | Low — sample on falling |
| 2 | 1 | 0 | High — sample on falling |
| 3 | 1 | 1 | High — sample on rising |

**Always check the slave datasheet** — displays (ST7789), flash (W25Q), and IMUs (BMI270) specify mode and max frequency.

### Transactions

A **transaction** = assert CS → exchange N bytes → deassert CS. Some devices require CS high between commands; others allow continuous streams.

---

## Hardware Overview

### ESP32-S3 SPI

ESP32-S3 provides **SPI0/1** (flash-bound) and **SPI2/SPI3** (general purpose, often called HSPI/VSPI in legacy docs):

- Master and slave modes (master is typical)
- Clock dividers up to ~80 MHz (board wiring often limits to 10–40 MHz)
- 8/16/32-bit transfers, configurable bit order
- GDMA support — [14-dma.md](./14-dma.md)

### Typical use cases

| Device | Speed | Mode | Notes |
|--------|-------|------|-------|
| ST7789 TFT | 20–40 MHz | 0 | DC pin for data vs command |
| W25Q flash | 40+ MHz | 0/3 | Quad-SPI for fast read |
| MPU6050 | — | — | **I²C**, not SPI — common mistake |
| BME280 | — | — | I²C or SPI variant |

---

## ASCII Wiring

### SPI master + ST7789 display (example pins)

```
ESP32-S3                ST7789 Display
┌──────────┐            ┌──────────┐
│ GPIO12   ├── SCLK ───►│ SCL      │
│ GPIO11   ├── MOSI ───►│ SDA/MOSI │
│ GPIO13   │◄─ MISO ────┤ (NC often)│
│ GPIO10   ├── CS ─────►│ CS       │
│ GPIO9    ├── DC ─────►│ DC/RS    │
│ GPIO14   ├── RST ────►│ RESET    │
│ 3V3      ├───────────►│ VCC      │
│ GND      ├───────────►│ GND      │
└──────────┘            └──────────┘
```

Keep wires short for high speeds; use series 33 Ω resistors on SCLK/MOSI if ringing occurs.

### SPI flash (on-module — do not wire externally)

The ESP32-S3 module's internal flash uses SPI0 — **do not use SPI0 pins for external devices**.

---

## Memory & Register Notes

### ESP32-S3 SPI registers (conceptual)

| Register | Purpose |
|----------|---------|
| `SPI_CTRL0` | Bit length, CPOL/CPHA |
| `SPI_CLOCK` | Divider, duty cycle |
| `SPI_USER` | Start transaction, CS setup |
| `SPI_W0..W15` | TX/RX FIFO via buffer registers |
| `SPI_DMA_CONF` | Link to GDMA |

### Buffer strategy

Small transfers: poll `SPI_USER` busy flag.  
Large framebuffer writes: **DMA from SRAM** — buffer must be contiguous and DMA-capable.

```rust
// Display line buffer — 240 pixels × 2 bytes RGB565
static LINE_BUF: StaticCell<[u8; 480]> = StaticCell::new([0u8; 480]);
```

---

## HAL Example — SPI write (esp-hal)

```rust
use esp_hal::{
    gpio::Io,
    peripherals::Peripherals,
    spi::master::{Spi, SpiBus},
};
use fugit::HertzU32;

fn main() -> ! {
    let peripherals = Peripherals::take();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    let sclk = io.pins.gpio12;
    let mosi = io.pins.gpio11;
    let mut cs = io.pins.gpio10.into_push_pull_output();

    // SPI mode 0, 20 MHz — verify against display datasheet
    let spi = Spi::new(
        peripherals.SPI2,
        sclk,
        mosi,
        None::<esp_hal::gpio::NoPin>, // MISO not used
        20u32.MHz(),
        esp_hal::spi::SpiMode::Mode0,
    )
    .unwrap();

    let mut bus = SpiBus::new(spi);

    // ST7789 init sequence (simplified — real driver crate preferred)
    cs.set_low().unwrap(); // Select display

    // Write command byte with DC low — omitted: use mipi-dbi or display driver
    let cmd = [0x01]; // SWRESET
    embedded_hal::spi::SpiBus::write(&mut bus, &cmd).unwrap();

    cs.set_high().unwrap(); // Deselect

    loop {
        // Frame updates...
    }
}
```

For production displays, use **`mipidsi`** + **`display-interface-spi`** crates with `embedded-graphics`.

---

## Sensor / Display Pattern (embedded-hal)

Portable driver code targets **`embedded-hal 1.0`** traits:

```rust
use embedded_hal::spi::SpiBus;
use embedded_hal::digital::OutputPin;

pub struct St7789<SPI, DC, RST, CS> {
    spi: SPI,
    dc: DC,
    rst: RST,
    cs: CS,
}

impl<SPI, DC, RST, CS, E> St7789<SPI, DC, RST, CS>
where
    SPI: SpiBus<Error = E>,
    DC: OutputPin,
    RST: OutputPin,
    CS: OutputPin,
{
    pub fn write_command(&mut self, cmd: u8) -> Result<(), E> {
        self.dc.set_low().map_err(|_| E::default())?; // command mode
        self.cs.set_low().map_err(|_| E::default())?;
        self.spi.write(&[cmd])?;
        self.cs.set_high().map_err(|_| E::default())?;
        Ok(())
    }

    pub fn write_data(&mut self, data: &[u8]) -> Result<(), E> {
        self.dc.set_high().map_err(|_| E::default())?; // data mode
        self.cs.set_low().map_err(|_| E::default())?;
        self.spi.write(data)?;
        self.cs.set_high().map_err(|_| E::default())?;
        Ok(())
    }
}
```

This pattern separates **protocol** from **HAL instance** — portable across ESP32, STM32, RP2040.

---

## Bare-Metal Sketch

```rust
unsafe fn spi_transfer_byte(byte: u8) -> u8 {
    // 1. Write byte to SPI_W0
    // 2. Set SPI_USR (start)
    // 3. Poll until SPI_USR done
    // 4. Read result from SPI_W0
    0
}
```

Always configure **IO_MUX** for SCLK/MOSI/MISO pins before enabling SPI clock.

---

## Step-by-Step

1. Read slave datasheet: **mode**, **max freq**, **CS timing**.
2. Wire SPI + power; leave MISO floating only if device never sends data.
3. Initialize SPI peripheral via HAL at low speed (1 MHz) for bring-up.
4. Send **WHO_AM_I** or **read ID** command — verify expected response.
5. Increase clock until failures, then back off 50%.
6. Add DMA for bulk transfers if needed.
7. Wrap in driver crate with `embedded-hal` traits.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Wrong SPI mode | `0xFF` reads, blank display | Match CPOL/CPHA |
| CS held low forever | Next device silent | Deassert CS between transactions |
| Using SPI0 pins | Boot failure | Use SPI2/SPI3 |
| Mode 3 vs Mode 0 typo | Intermittent garbage | Logic analyzer + datasheet |
| Full duplex when slave is TX-only | Random MISO bits | Use `write` not `transfer` if MISO NC |
| No level shifter (3.3 ↔ 5 V) | Damaged chip | Match voltages |

---

## Debugging Tips

- **Logic analyzer** on SCLK, MOSI, CS — compare to datasheet timing diagram.
- Scope **SCLK ringing** — add series resistor, shorten wires.
- Read ID register first — isolates wiring vs init sequence bugs.
- Use **half-speed** until basic exchange works.
- `defmt` log each command in init — compare against vendor C reference.

---

## Performance Tips

- **DMA** for display flush — CPU prepares next line while DMA sends current.
- Use **32-bit SPI transfers** when FIFO supports it (aligned buffers).
- **Quad SPI** for external flash reads (QIO mode) — bootloader/`esp-hal` config.
- Batch CS: some chips allow long data streams with CS low — reduces overhead.
- RP2040: consider **PIO** for custom SPI-like protocols — [boards/rp2040.md](./boards/rp2040.md).

---

## Exercises

1. **Read JEDEC ID** from external W25Q (if module exposes spare flash CS on dev board).
2. **ST7789 color fill** — red/green/blue screens using `embedded-graphics`.
3. **Portable driver:** Implement `SpiDevice` wrapper with CS management; test on STM32 Nucleo.
4. **Benchmark:** Compare polled vs DMA flush time for 240×240 RGB565 frame.
5. **Mode quiz:** Wire two SPI modes on analyzer; identify CPOL/CPHA from captures.

---

## References

- [ESP32-S3 TRM — SPI](https://www.espressif.com/en/products/socs/esp32-s3)
- [embedded-hal SpiBus](https://docs.rs/embedded-hal/latest/embedded_hal/spi/trait.SpiBus.html)
- [mipidsi crate](https://docs.rs/mipidsi/)
- [17-i2c.md](./17-i2c.md) — alternative bus for sensors
- [boards/esp32-s3.md](./boards/esp32-s3.md)

---

*Previous: [15-uart.md](./15-uart.md) · Next: [17-i2c.md](./17-i2c.md)*
