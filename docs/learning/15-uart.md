# Lesson 15 — UART / USART Serial Communication

**Prerequisites:** [08-gpio.md](./08-gpio.md), [09-interrupts.md](./09-interrupts.md), [14-dma.md](./14-dma.md) (optional)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md) — UART0 via USB. Also: STM32, RP2040, nRF52.

---

## Theory

**UART** (Universal Asynchronous Receiver-Transmitter) is a **serial** protocol for byte-oriented communication between two devices over two wires (plus ground). It is **asynchronous** — there is no shared clock wire; both sides agree on a **baud rate** (bits per second) in advance.

**USART** (Universal Synchronous/Asynchronous Receiver-Transmitter) adds optional **synchronous** clock mode; in practice embedded developers say "UART" for the common async case.

### Framing

Each byte is sent as a frame:

```
Idle (high) ──┐                              ┌── Idle
              │ START │ D0 │ D1 │ ... │ D7 │ STOP │
              └── 0 ──┴──── 8 data bits ────┴── 1 ──┘
              (LSB first)                  (usually 1 stop bit)
```

| Parameter | Typical value | Notes |
|-----------|---------------|-------|
| Baud rate | 115200 | Common for dev consoles; 9600 for legacy sensors |
| Data bits | 8 | `8N1` = 8 data, no parity, 1 stop |
| Parity | None / Even / Odd | Detect single-bit errors |
| Stop bits | 1 or 2 | Receiver recovery time |

**Baud error:** Both sides must be within ~2–3% of the nominal rate. Derive baud from your **APB/AHB clock** — see clock tree in [boards/esp32-s3.md](./boards/esp32-s3.md).

### Flow control (optional)

- **RTS/CTS** (Request To Send / Clear To Send): hardware handshaking to prevent FIFO overflow.
- **XON/XOFF:** Software flow control bytes — rare in embedded.

---

## Hardware Overview

### ESP32-S3 UART

The ESP32-S3 has **three UART controllers** (UART0–UART2):

| UART | Typical use on DevKitC-1 |
|------|--------------------------|
| UART0 | Default console — often wired to USB-serial/JTAG bridge |
| UART1 | Free for peripherals (GPS, LoRa module) |
| UART2 | Free |

Each UART has:

- TX/RX FIFOs (128 bytes on many Espressif chips)
- Programmable baud generator
- Optional hardware flow control pins
- Interrupts: RX threshold, TX done, parity error, frame error
- **GDMA** support — see [14-dma.md](./14-dma.md)

### Other MCUs

| MCU | Notes |
|-----|-------|
| STM32 | Multiple USARTs; remappable pins via AF (Alternate Function) |
| RP2040 | Two UARTs; easy with `rp2040-hal` |
| nRF52840 | UARTE with EasyDMA — Nordic's DMA-backed UART |

---

## ASCII Wiring

### USB console (DevKitC-1 — no extra wiring)

```
PC ◄──── USB-C ────► ESP32-S3 DevKitC-1
                      (internal USB-JTAG/UART bridge)
```

### External USB-UART adapter (3.3 V logic!)

```
USB-UART          ESP32-S3
┌──────────┐      ┌──────────┐
│ TX ──────┼──────►│ GPIO44   │  RX (crossover)
│ RX ◄─────┼──────┤ GPIO43   │  TX
│ GND ─────┼──────┤ GND      │
└──────────┘      └──────────┘
     3V3 logic — NOT 5 V TTL unless board is 5 V tolerant
```

### Loopback test (no adapter)

```
GPIO43 (TX) ──── jumper wire ────► GPIO44 (RX)
```

---

## Memory & Register Notes

### ESP32-S3 UART registers (selected)

| Register | Offset | Purpose |
|----------|--------|---------|
| `UART_CONF0` | — | Parity, stop bits, bit length |
| `UART_CLKDIV` | — | Baud divisor = f_uart / baud |
| `UART_STATUS` | — | TX idle, RX FIFO count |
| `UART_FIFO` | — | Read/write data byte |
| `UART_INT_RAW` | — | Interrupt status |

Baud calculation (conceptual):

```
divider = (uart_clock_hz) / (16 * baud)   // hardware-specific formula — check TRM
```

On ESP32-S3 at 80 MHz APB and 115200 baud, the HAL computes this for you.

### Buffers in Rust

```rust
// RX interrupt buffer — static storage
static mut RX_QUEUE: heapless::Queue<u8, 256> = heapless::Queue::new();
```

Never allocate `String` per byte in `no_std` — use fixed buffers. See [02-no-std.md](./02-no-std.md).

---

## HAL Example — Serial Echo (esp-hal)

Full echo: bytes received on RX are sent back on TX.

```rust
use esp_hal::{
    gpio::Io,
    peripherals::Peripherals,
    uart::{Uart, config::Config},
};
use core::fmt::Write;

fn main() -> ! {
    let peripherals = Peripherals::take();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // 115200 8N1 — standard dev console speed
    let config = Config::default().baudrate(115_200);

    let mut uart = Uart::new(peripherals.UART0, config)
        .unwrap()
        .with_rx(io.pins.gpio44)
        .with_tx(io.pins.gpio43);

    // Optional: greeting so you know it booted
    writeln!(uart, "ESP32-S3 UART echo ready. Type something!\r\n").ok();

    loop {
        // Blocking read of one byte — simple teaching example
        if let Ok(byte) = uart.read_byte() {
            // Echo the byte back
            let _ = uart.write_byte(byte);

            // Optional: newline handling for terminal friendliness
            if byte == b'\r' {
                let _ = uart.write_byte(b'\n');
            }
        }
    }
}
```

### Interrupt-driven echo (outline)

```rust
use esp_hal::uart::{Uart, Event};

// In init: enable RX FIFO threshold interrupt
// In ISR (keep minimal — defer to queue):
//   while uart.rxfifo_count() > 0 {
//       RX_QUEUE.push(uart.read_byte()).ok();
//   }
// In main loop: drain RX_QUEUE, echo bytes
```

Prefer **Embassy** async UART for non-blocking patterns — [22-embassy.md](./22-embassy.md).

---

## Bare-Metal Sketch

```rust
// Pseudocode register sequence for UART0 init on ESP32-S3
unsafe fn uart_init_bare() {
    // 1. Enable UART0 peripheral clock in SYSTEM / PERIP_CLK_EN
    // 2. Configure GPIO43/44 as UART0 TX/RX (IO_MUX)
    // 3. Set UART_CLKDIV for 115200 baud
    // 4. Configure 8N1 in UART_CONF0
    // 5. Enable TX and RX in UART_CONF0

    // Send byte:
    // while !TX_IDLE { }
    // write(UART_FIFO, byte);

    // Receive byte:
    // while RX_FIFO_EMPTY { }
    // let b = read(UART_FIFO);
}
```

Use the **PAC** (Peripheral Access Crate) for type-safe register names — [05-embedded-architecture.md](./05-embedded-architecture.md).

---

## Step-by-Step

1. **Identify pins** from your board schematic — [boards/esp32-s3.md](./boards/esp32-s3.md).
2. **Create project** from `esp-template` or your firmware workspace.
3. **Configure UART** at 115200 8N1; flash and open serial monitor (`espflash flash --monitor`).
4. **Verify output** — boot message appears.
5. **Connect loopback** or type in terminal — confirm echo.
6. **Add line editing** (optional): backspace, echo `\r\n`.
7. **Upgrade to IRQ or DMA** when CPU usage matters.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| TX/RX swapped | Garbage or silence | Crossover: TX→RX, RX→TX |
| 5 V adapter on 3.3 V UART | Damaged chip | Use 3.3 V USB-UART |
| Wrong baud | `�` or empty | Match monitor speed to firmware |
| USB cable charge-only | No serial port | Data-capable USB cable |
| Blocking read without timeout | Hang forever | Add timeout or interrupt |
| `println!` without feature | No output | Enable `esp-println` / `defmt` |
| Newline mismatch | Jumbled terminal | Send `\r\n` for `\r` received |

---

## Debugging Tips

- **`espflash flash --monitor`** or `minicom` / `screen /dev/ttyACM0 115200`.
- Toggle an LED in RX ISR to confirm interrupts fire.
- Use a **USB-UART sniffer** or second adapter to tap TX independently.
- Log baud clock source if characters are consistently wrong — clock misconfiguration is common after sleep wake — [24-low-power.md](./24-low-power.md).
- Compare against known-good Arduino AT sketch on same pins to isolate hardware.

---

## Performance Tips

- Increase FIFO threshold before IRQ to reduce interrupt rate.
- Use [DMA](./14-dma.md) above ~1 Mbaud or for bulk transfers.
- Batch writes: `write_all(&[...])` instead of per-byte syscalls in hosted code; in `no_std`, buffer into `[u8; N]`.
- For logging, use **defmt** at 1–2 Mbaud if your probe supports it — [25-debugging.md](./25-debugging.md).

---

## Exercises

1. **Line reader:** Buffer until `\n`, then echo `You said: ...`.
2. **Command shell:** Parse `led on` / `led off` to control GPIO from [08-gpio.md](./08-gpio.md).
3. **NMEA parser:** Read GPS module on UART1; extract `$GPGGA` latitude (text parsing in `no_std`).
4. **Baud sweep:** Document minimum/maximum reliable baud for loopback on your board.
5. **Portable HAL:** Write echo using `embedded-io` traits; compile for RP2040 — [07-hal.md](./07-hal.md).

---

## References

- [ESP32-S3 TRM — UART](https://www.espressif.com/en/products/socs/esp32-s3)
- [embedded-io crate](https://docs.rs/embedded-io/)
- [esp-hal UART docs](https://docs.esp-rs.org/esp-hal/esp_hal/uart/index.html)
- [14-dma.md](./14-dma.md) — DMA-backed UART
- [25-debugging.md](./25-debugging.md) — serial monitor setup

---

*Previous: [14-dma.md](./14-dma.md) · Next: [16-spi.md](./16-spi.md)*
