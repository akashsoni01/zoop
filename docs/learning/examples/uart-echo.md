# Example: UART Echo

**Goal:** Read bytes from USB-serial and echo them back.

**Prerequisites:** [15-uart.md](../15-uart.md), [communication/uart-usart.md](../communication/uart-usart.md)

---

## Wiring

DevKitC-1: USB-C provides CDC console — no external wiring. For external UART:

```
ESP32 TX (GPIO43) ──► RX USB adapter
ESP32 RX (GPIO44) ◄── TX USB adapter
GND ──────────────── GND
```

---

## Rust Sketch

```rust
use embedded_io::{Read, Write};

fn main() -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    let mut uart = esp_hal::uart::Uart::new(/* UART0, 115200 */);

    let mut buf = [0u8; 64];
    loop {
        if let Ok(n) = uart.read(&mut buf) {
            if n > 0 {
                let _ = uart.write_all(&buf[..n]);
            }
        }
    }
}
```

### Ownership

Single `uart` — cannot split TX/RX to two tasks without `embedded-io-async` + lock.

---

## Compile Notes

```bash
espflash flash --monitor target/xtensa-esp32s3-none-elf/release/uart-echo
```

Monitor shows echoed keystrokes.

*Next: [spi-display.md](./spi-display.md)*
