# UART / USART — Universal Asynchronous Receiver-Transmitter

**UART** sends bytes **asynchronously** over two wires: **TX** (transmit) and **RX** (receive). **USART** adds synchronous clock mode; embedded Rust docs often say "UART" for the common async case.

**Prerequisites:** [08-gpio.md](../08-gpio.md), [gpio.md](./gpio.md)

---

## Theory

Each byte is a **frame**:

1. **Start bit** — LOW (1 bit)
2. **Data bits** — usually 8, LSB first
3. **Parity bit** (optional) — even/odd error check
4. **Stop bit(s)** — HIGH (1 or 2 bits)

Both sides agree on **baud rate** (bits per second). Common rates: 9600, 115200, 921600. Error budget: typically ±2–3% with oversampling (16×).

**Full-duplex:** TX and RX are independent — you can send and receive simultaneously.

ESP32-S3 has multiple UART peripherals (UART0 often used for USB-CDC console). Connect **TX → RX** crosswise between devices; **GND common** is mandatory.

---

## Timing Diagram (ASCII)

115200 baud, 8N1 (8 data, no parity, 1 stop) — bit time ≈ 8.68 µs:

```
Idle ─────────────────────────────────────────
TX (byte 0x55 = 0b01010101, LSB first)

Start  D0 D1 D2 D3 D4 D5 D6 D7  Stop
  ┐    ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐
  └──┐ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ └── idle
     └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘
  0   1   0   1   0   1   0   1   0   1

Legend: LOW = 0, HIGH = 1 (idle)
Frame: |<──────── ~87 µs per byte ────────>|
```

Multi-byte stream (`"Hi\r\n"`):

```
TX  │S│H│S│i│S│\r│S│\n│
    └── each byte separated by idle HIGH ──┘
```

---

## Packet Format

UART has no native packet layer — it is a **byte stream**. Application protocols add framing:

| Layer | Example framing |
|-------|-----------------|
| Raw bytes | `defmt` RTT, ad-hoc |
| Line-oriented | NMEA (`$...*CS\r\n`), AT commands |
| Length-prefixed | `[len:u16][payload]` |
| COBS | Zero-free framing for packet boundaries |

Standard serial port settings string: **`115200 8N1`**.

---

## Electrical Characteristics

| Interface | Voltage | Notes |
|-----------|---------|-------|
| ESP32-S3 UART | 3.3 V CMOS | Not 5 V tolerant on RX |
| USB-CDC | Via USB transceiver | Appears as `/dev/tty*` |
| RS-232 | ±3 V to ±15 V | See [rs232.md](./rs232.md) — level shifters required |
| RS-485 | Differential | See [rs485.md](./rs485.md) |

| Parameter | Typical |
|-----------|---------|
| Max practical baud (ESP32-S3) | 5 Mbps (short traces) |
| Cable length @ 115200 | Several meters (TTL) |
| Idle state | HIGH (mark) |

Use **330 Ω series resistors** on TX lines in noisy environments. Always connect **GND** between boards.

---

## Rust HAL Sketch (esp-hal + embedded-io)

```rust
use core::fmt::Write;
use embedded_io::{Read, Write as IoWrite};
use esp_hal::uart::{Uart, Config};

// Ownership: `uart` owns UART0 + pins; passed by mutable reference
// to allow concurrent read/write through the same peripheral.

pub fn echo_loop(uart: &mut Uart<'static, esp_hal::Blocking>) -> ! {
    let mut buf = [0u8; 64];
    loop {
        match uart.read(&mut buf) {
            Ok(n) if n > 0 => {
                let _ = uart.write_all(&buf[..n]); // echo back
            }
            _ => {}
        }
    }
}

pub fn log_line(uart: &mut Uart<'static, esp_hal::Blocking>, msg: &str) {
    let _ = writeln!(uart, "{}", msg);
}
```

**Compile notes:**

```toml
[dependencies]
esp-hal = { version = "0.20", features = ["esp32s3", "unstable"] }
embedded-io = "0.6"
```

Flash and monitor:

```bash
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/uart-echo
```

---

## Bare-Metal Sketch (Concept)

```rust
// Pseudocode for UART FIFO read — consult PAC for exact register names
unsafe fn uart_read_byte() -> Option<u8> {
    const STATUS: *const u32 = 0x6000_0000 as *const u32; // placeholder
    const FIFO: *const u32 = 0x6000_0004 as *const u32;
    if STATUS.read_volatile() & 0x1 != 0 {
        Some((FIFO.read_volatile() & 0xFF) as u8)
    } else {
        None
    }
}
```

Use `esp-hal` or generated PAC from `esp32s3` crate for production code.

---

## Example Projects

| Link | Description |
|------|-------------|
| [examples/uart-echo.md](../examples/uart-echo.md) | Loopback console |
| [examples/gps-tracker](../projects/gps-tracker.md) | NMEA over UART |
| [projects/usb-serial-converter.md](../projects/usb-serial-converter.md) | USB-CDC bridge |
| [rs232.md](./rs232.md) | Legacy PC serial |

---

## Common Mistakes

1. **TX connected to TX** — must cross TX↔RX.
2. **Missing common ground** — garbage or no communication.
3. **Baud mismatch** — framing errors, `�` characters.
4. **5 V TTL into ESP32 RX** — use level shifter (e.g., TXS0108).
5. **Blocking read in async context** — use `embedded-io-async` or dedicated task.

---

## Exercises

1. Implement [examples/uart-echo.md](../examples/uart-echo.md) at 115200 8N1.
2. Parse `AT\r\n` commands and reply `OK\r\n`.
3. Add a simple COBS decoder for framed packets.
4. Measure UART jitter when Wi-Fi is active (ESP32-S3 coexistence).

---

## References

- [ESP32-S3 TRM — UART chapter](https://www.espressif.com/en/products/socs/esp32-s3)
- [embedded-io traits](https://docs.rs/embedded-io/latest/embedded_io/)
- [The Serial UART Guide](https://docs.rust-embedded.org/book/) (Embedded Rust Book)
- Lesson: [04-cargo.md](../04-cargo.md) (logging via serial)

---

*Prev: [gpio.md](./gpio.md) | Next: [spi.md](./spi.md)*
