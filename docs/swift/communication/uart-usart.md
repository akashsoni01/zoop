# UART / USART — Universal Asynchronous Receiver-Transmitter

**Prerequisites:** [08-gpio.md](../08-gpio.md), [gpio.md](./gpio.md)

---

## Theory

Each byte is a **frame**:

1. **Start bit** — LOW (1 bit)
2. **Data bits** — usually 8, LSB first
3. **Parity bit** (optional) — even/odd error check
4. **Stop bit(s)** — HIGH (1 or 2 bits)

Both sides agree on **baud rate** (bits per second). Common rates: 9600, 115200, 921600.

**Full-duplex:** TX and RX are independent. Connect **TX → RX** crosswise; **GND common** is mandatory.

---

## Timing Diagram (ASCII)

115200 baud, 8N1 — bit time ≈ 8.68 µs:

```
Idle ─────────────────────────────────────────
TX (byte 0x55 = 0b01010101, LSB first)

Start  D0 D1 D2 D3 D4 D5 D6 D7  Stop
  ┐    ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐ ┌─┐
  └──┐ │ │ │ │ │ │ │ │ │ │ │ │ │ │ │ └── idle
     └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘ └─┘
  0   1   0   1   0   1   0   1   0   1
```

---

## Packet Format

UART has no native packet layer — it is a **byte stream**. Application protocols add framing:

| Layer | Example framing |
|-------|-----------------|
| Raw bytes | Debug console, ad-hoc |
| Line-oriented | NMEA (`$...*CS\r\n`), AT commands |
| Length-prefixed | `[len:u16][payload]` |
| COBS | Zero-free framing |

Standard settings: **`115200 8N1`**.

---

## Electrical Characteristics

| Interface | Voltage | Notes |
|-----------|---------|-------|
| ESP32-S3 UART | 3.3 V CMOS | Not 5 V tolerant on RX |
| USB-CDC | Via USB transceiver | Appears as `/dev/tty*` |
| RS-232 | ±3 V to ±15 V | See [rs232.md](./rs232.md) |
| RS-485 | Differential | See [rs485.md](./rs485.md) |

---

## Swift HAL Sketch

```swift
import Embedded
import ESP32Hardware

struct SerialConsole {
    var uart: UART0

    func write(_ bytes: [UInt8]) {
        for b in bytes { uart.write(b) }
    }

    func readLine(into buffer: inout [UInt8]) -> Int {
        var i = 0
        while i < buffer.count {
            let b = uart.read()
            if b == 0x0A { break }  // LF
            buffer[i] = b
            i &+= 1
        }
        return i
    }
}
```

Use a **stack buffer** (`[UInt8]`) for RX — avoid heap allocation in ISR context.

**ARC note:** Prefer `struct` wrappers for peripherals. Avoid `class` in ISRs — if you must share state, use `Unmanaged` or a lock-free ring buffer owned by one task.

---

## Bare-Metal Sketch (Register-Level)

```swift
let uartClkDiv = MMIO<UInt32>(0x6000_0000 + 0x14)  // illustrative
let uartTxFifo = MMIO<UInt32>(0x6000_0000 + 0x00)

func uartWriteByte(_ b: UInt8) {
    while uartStatus.read() & TX_FULL != 0 { }
    uartTxFifo.write(UInt32(b))
}
```

---

## Example Projects

- [usb-serial-converter.md](../projects/usb-serial-converter.md)
- [gps-tracker.md](../projects/gps-tracker.md) — NMEA over UART

---

## Common Mistakes

| Mistake | Fix |
|---------|-----|
| TX-TX, RX-RX wired | Cross TX→RX |
| Missing common GND | Tie grounds together |
| Baud mismatch | Match both sides exactly |
| Blocking read in ISR | Use ring buffer + defer to main loop |

---

## Exercises

1. Echo every received byte on UART0.
2. Parse `$GPGGA` NMEA sentences from a GPS module.
3. Implement a COBS decoder for packet framing.

---

## References

- [examples/uart-echo.md](../examples/uart-echo.md)
- [RS-232 guide](./rs232.md)

---

*Back to [Embedded Swift](../README.md)*
