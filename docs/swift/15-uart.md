# Lesson 15 — UART / USART Serial Communication

**Prerequisites:** [08-gpio.md](./08-gpio.md), [09-interrupts.md](./09-interrupts.md), [14-dma.md](./14-dma.md) (optional)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md) — UART0 via USB. Also: STM32, RP2040, nRF52.

**Maturity note:** **esp32-uart-echo** in [swift-embedded-examples](https://github.com/swiftlang/swift-embedded-examples) runs on **ESP32-C6** via ESP-IDF. ESP32-S3 Swift UART via ESP-IDF follows the same pattern. Bare-metal UART on C6 uses ROM UART helpers; S3 bare-metal Swift is less mature — use ESP-IDF or host Swift serial tools for development.

---

## Theory

**UART** (Universal Asynchronous Receiver-Transmitter) sends bytes over two wires (plus ground) **without a shared clock**. Both sides agree on **baud rate** in advance.

**USART** adds optional synchronous clock mode; embedded developers usually mean async UART.

### Framing

```
Idle (high) ──┐                              ┌── Idle
              │ START │ D0..D7 │ STOP │
              └── 0 ──┴── LSB first ──┴── 1 ──┘
```

| Parameter | Typical value |
|-----------|---------------|
| Baud rate | 115200 (dev console) |
| Data bits | 8 (`8N1` = 8 data, no parity, 1 stop) |
| Parity | None / Even / Odd |
| Stop bits | 1 or 2 |

**Baud error:** Keep both sides within ~2–3% of nominal rate. Derive baud from your clock tree — [boards/esp32-s3.md](./boards/esp32-s3.md).

### Flow control (optional)

- **RTS/CTS:** Hardware handshaking against FIFO overflow.
- **XON/XOFF:** Software flow control — rare in embedded.

---

## Hardware Overview

### ESP32-S3 UART

Three UART controllers (UART0–UART2):

| UART | Typical use on DevKitC-1 |
|------|--------------------------|
| UART0 | Default console — USB-serial/JTAG bridge |
| UART1 | GPS, LoRa, external modules |
| UART2 | Free |

Features: 128-byte FIFOs, programmable baud, hardware flow control, GDMA — [14-dma.md](./14-dma.md).

### Other MCUs

| MCU | Notes |
|-----|-------|
| STM32 | Multiple USARTs; remappable AF pins |
| RP2040 | Two UARTs |
| nRF52840 | UARTE with EasyDMA |

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
     3V3 logic — NOT 5 V TTL
```

### Loopback test

```
GPIO43 (TX) ──── jumper ────► GPIO44 (RX)
```

---

## Memory & Register Notes

### ESP32-S3 UART registers (selected)

| Register | Purpose |
|----------|---------|
| `UART_CONF0` | Parity, stop bits, bit length |
| `UART_CLKDIV` | Baud divisor |
| `UART_STATUS` | TX idle, RX FIFO count |
| `UART_FIFO` | Read/write data byte |
| `UART_INT_RAW` | Interrupt status |

Baud (conceptual):

```
divider = uart_clock_hz / (16 * baud)   // check TRM for exact formula
```

### Buffers in Embedded Swift

```swift
// Fixed-size RX queue — no heap allocation in hot path
var rxQueue: FixedQueue<UInt8, 256> = FixedQueue()

struct FixedQueue<Element, Capacity: Int> {
    private var storage: [Element] = []
    private var head = 0
    private var tail = 0
    // push/pop implementation ...
}
```

Avoid `String` per byte in embedded mode — use `[UInt8]` buffers — [02-no-stdlib.md](./02-no-stdlib.md).

---

## HAL Swift Example — Serial Echo (ESP-IDF path)

Based on the **esp32-uart-echo** example pattern:

```swift
import ESPIDF

@main
struct UartEchoApp {
    static func main() {
        let uart = UART(
            port: 0,
            baud: 115_200,
            txPin: 43,
            rxPin: 44
        )

        print("ESP32-S3 UART echo ready. Type something!\r\n")

        while true {
            if let byte = uart.readByte(timeoutMs: portMAX_DELAY) {
                uart.writeByte(byte)
                if byte == 0x0D { // '\r'
                    uart.writeByte(0x0A) // '\n'
                }
            }
        }
    }
}
```

Build and flash (ESP-IDF + Embedded Swift toolchain):

```bash
idf.py build flash monitor
```

---

## Bare-Metal Swift Sketch (ESP32-C6 ROM UART pattern)

Adaptable conceptually to S3; C6 bare-metal PoC uses ROM functions:

```swift
import MMIO

let uart0 = UART0(baseAddress: 0x6000_0000) // illustrative base

func uartInit(baud: UInt32) {
    // 1. Enable UART0 peripheral clock
    // 2. Configure GPIO43/44 as UART TX/RX via IO_MUX
    // 3. Set UART_CLKDIV for 115200
    // 4. Configure 8N1 in UART_CONF0
    // 5. Enable TX and RX
    _ = baud
}

func uartWriteByte(_ byte: UInt8) {
    while !uart0.status.txidle.get() { }
    uart0.fifo.set(UInt32(byte))
}

func uartReadByte() -> UInt8 {
    while uart0.status.rxfifo_cnt.get() == 0 { }
    return UInt8(truncatingIfNeeded: uart0.fifo.get())
}
```

---

## Host Swift — Serial Monitor (macOS)

When firmware is still in C/Rust, use host Swift to talk to the board over USB serial:

```swift
import Foundation
import ORSSerial

// Using ORSSerialPort or similar — test UART protocols from macOS
let port = ORSSerialPort(path: "/dev/cu.usbmodem101")!
port.baudRate = 115_200
port.open()

port.sendData("led on\n".data(using: .utf8)!)

port.delegate = SerialDelegate { data in
    print("RX:", String(data: data, encoding: .utf8) ?? "<binary>")
}
```

Useful for HIL test harnesses — [26-testing.md](./26-testing.md).

---

## Step-by-Step

1. Identify pins from schematic — [boards/esp32-s3.md](./boards/esp32-s3.md).
2. Create project from [swift-embedded-examples](https://github.com/swiftlang/swift-embedded-examples) ESP32 template.
3. Configure UART at 115200 8N1; flash and open monitor.
4. Verify boot message on serial.
5. Loopback or type in terminal — confirm echo.
6. Add line editing (backspace, `\r\n` handling).
7. Upgrade to IRQ or DMA when CPU usage matters — [14-dma.md](./14-dma.md), [22-swift-concurrency.md](./22-swift-concurrency.md).

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| TX/RX swapped | Garbage or silence | Crossover: TX→RX, RX→TX |
| 5 V adapter on 3.3 V UART | Damaged chip | Use 3.3 V USB-UART |
| Wrong baud | Mojibake or empty | Match monitor to firmware |
| USB charge-only cable | No serial port | Use data-capable cable |
| Blocking read without timeout | Hang forever | Add timeout or interrupt |
| Newline mismatch | Jumbled terminal | Send `\r\n` for `\r` |

---

## Debugging Tips

- `idf.py monitor` or `screen /dev/ttyACM0 115200`.
- Toggle LED in RX ISR to confirm interrupts fire.
- USB-UART sniffer on TX line independently.
- After sleep wake, re-check baud clock source — [24-low-power.md](./24-low-power.md).
- Compare against known-good firmware on same pins to isolate hardware.

---

## Performance Tips

- Increase FIFO threshold before IRQ to reduce interrupt rate.
- Use [DMA](./14-dma.md) above ~1 Mbaud or for bulk transfers.
- Batch writes into `[UInt8]` buffers instead of per-byte calls.
- For logging, use structured binary logs over UART — [25-debugging.md](./25-debugging.md).

---

## Exercises

1. **Line reader:** Buffer until `\n`, echo `You said: ...`.
2. **Command shell:** Parse `led on` / `led off` to control GPIO — [08-gpio.md](./08-gpio.md).
3. **NMEA parser:** Read GPS on UART1; extract `$GPGGA` fields.
4. **Baud sweep:** Document min/max reliable loopback baud.
5. **Host HIL:** macOS Swift script sends commands, asserts responses — [26-testing.md](./26-testing.md).

---

## References

- [ESP32-S3 TRM — UART](https://www.espressif.com/en/products/socs/esp32-s3)
- [swift-embedded-examples — esp32-uart-echo](https://github.com/swiftlang/swift-embedded-examples)
- [14-dma.md](./14-dma.md)
- [25-debugging.md](./25-debugging.md)

---

*Previous: [14-dma.md](./14-dma.md) · Next: [16-spi.md](./16-spi.md)*
