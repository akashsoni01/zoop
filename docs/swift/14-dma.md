# Lesson 14 — Direct Memory Access (DMA)

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [10-timers.md](./10-timers.md), [07-hal.md](./07-hal.md)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md) (GDMA). Also covers STM32, RP2040, nRF52.

**Maturity note:** Embedded Swift DMA examples are **early-stage**. ESP32-C6 bare-metal Swift has UART/GPIO/SPI PoCs; ESP32-S3 Swift via ESP-IDF is the most practical path today. Where on-MCU Swift is not ready, this lesson shows **illustrative Embedded Swift sketches** plus **host Swift** patterns for buffer management you can test on macOS.

---

## Theory

**Direct Memory Access (DMA)** moves data between memory and a peripheral (or between memory regions) **without the CPU executing a load/store for every byte**. The CPU configures source, destination, length, and trigger — then continues other work while the DMA engine copies.

Why DMA matters in embedded firmware:

- **Throughput:** High baud UART, SPI displays, and ADC streaming saturate the CPU without DMA.
- **Determinism:** Large buffer copies in ISRs add jitter to real-time tasks.
- **Power:** The CPU can sleep while DMA runs (with careful clock gating).

| Term | Meaning |
|------|---------|
| **Channel** | One independent DMA transfer engine |
| **Descriptor / linked list** | Chained blocks for multi-buffer or circular mode |
| **Burst** | Consecutive bytes per bus cycle |
| **Handshake** | Peripheral signals FIFO ready |
| **Circular / ring buffer** | DMA wraps write pointer at buffer end |

### Ownership in Embedded Swift

Embedded Swift has no garbage collector. DMA buffers need the same discipline as Rust or C:

1. Buffer must remain **valid at a fixed address** for the entire transfer — no stack locals that go out of scope.
2. Buffer must be **DMA-capable** (alignment, cache coherency rules per MCU).
3. While DMA owns the buffer, the CPU must not read/write it unless synchronized (completion IRQ or double-buffering).

On ESP32-S3, prefer **internal SRAM** for control structures; **PSRAM** support is peripheral-dependent — see [boards/esp32-s3.md](./boards/esp32-s3.md).

---

## Hardware Overview

### ESP32-S3 GDMA

```
CPU                    GDMA Controller              Peripheral
 |                           |                          |
 |-- configure channel ----->|                          |
 |-- start transfer -------->|                          |
 |                           |--- read/write bytes ---->|
 | (continues other work)      |<--- handshake/FIFO ------|
 |<-- interrupt on complete ---|                          |
```

- SPI, I²S, UART, AES, ADC bind to GDMA channels.
- Supports **circular mode** for audio/ADC streaming.

### Other MCUs

| MCU | Notes |
|-----|-------|
| STM32 | DMA1/DMA2 + DMAMUX; double-buffer common for ADC |
| RP2040 | 12 channels with control blocks in memory — see [boards/rp2040.md](./boards/rp2040.md) |
| nRF52840 | EasyDMA on UARTE, SPIM, TWIM |

---

## ASCII Wiring

DMA is internal — no external wiring. UART echo with DMA uses the same pins as [15-uart.md](./15-uart.md):

```
ESP32-S3 DevKitC-1
┌─────────────────────┐
│  GPIO43 (U0 TXD) ───┼──► USB-UART adapter RX (optional monitor)
│  GPIO44 (U0 RXD) ◄───┼─── USB-UART adapter TX
│  GND ───────────────┼─── GND
└─────────────────────┘
```

Use built-in USB-JTAG/serial on DevKitC-1 when possible — [25-debugging.md](./25-debugging.md).

---

## Memory & Register Notes

### ESP32-S3 GDMA registers (conceptual)

| Register / field | Purpose |
|------------------|---------|
| `CH0_CONF0` | Enable channel, address increment |
| `CH0_LINK_ADDR` | Linked-list descriptor pointer |
| `CH0_OUT_LINK` / `IN_LINK` | Start outbound/inbound transfer |
| `CH0_INT_RAW` | Transfer complete, error flags |

Descriptor layout in RAM:

```
[ length | src_addr | dst_addr | next_desc_ptr ]
```

### Buffer placement (ESP32-S3)

| Region | DMA-safe? | Notes |
|--------|-----------|-------|
| Internal SRAM | Yes | Preferred |
| PSRAM | Peripheral-dependent | Check TRM |
| Flash (XIP) | Read-only source sometimes | Never DMA write destination |

Consult the [ESP32-S3 Technical Reference Manual](https://www.espressif.com/en/products/socs/esp32-s3).

---

## HAL Swift Example — UART RX via DMA (ESP-IDF path)

**Status:** Production-ready path is **Embedded Swift + ESP-IDF C HAL** until a pure Swift HAL matures on S3.

```swift
// Package.swift — Embedded Swift + ESP-IDF component
// Target: ESP32-S3 via ESP-IDF v5.x / v6.x

import ESPIDF

// Static DMA buffer — must live for entire program lifetime
@Section(".bss")
var rxBuffer: [UInt8] = Array(repeating: 0, count: 1024)

@main
struct UartDmaApp {
    static func main() {
        // ESP-IDF UART + GDMA init (C bindings via ESPIDF module)
        let uart = UART(port: 0, baud: 115_200, txPin: 43, rxPin: 44)
        let dma = GDMARxChannel(channel: 0, peripheral: .uart0)

        dma.configure(
            destination: &rxBuffer,
            length: rxBuffer.count,
            circular: false
        )

        uart.enableDmaRx(channel: dma)

        while true {
            if let received = dma.waitForCompletion(timeoutMs: 1000) {
                processBytes(rxBuffer.prefix(received))
                dma.rearm(destination: &rxBuffer)
            }
        }
    }

    static func processBytes(_ data: ArraySlice<UInt8>) {
        for byte in data { _ = byte }
    }
}
```

> **Honest status:** The `ESPIDF`/`GDMARxChannel` types above are **illustrative** — wire them to actual ESP-IDF `uart_driver_install` + `uart_dma` APIs from your project's C bridge. See [swift-embedded-examples](https://github.com/swiftlang/swift-embedded-examples).

---

## Bare-Metal Swift Sketch (swift-mmio)

Illustrative register-level access on ESP32-C6 (RISC-V bare-metal PoC exists; S3 follows same GDMA concepts):

```swift
import MMIO

// Generated from SVD via swift-svd / esp32s3 SVD files
let gdma = GDMA(baseAddress: 0x6003_F000)

// Static buffer — required for DMA lifetime
var rxBuffer: [UInt8] = Array(repeating: 0, count: 256)

func startUartRxDma() {
    // 1. Reset GDMA channel
    gdma.channel0.conf0.ch_reset.set(true)

    // 2. Set peripheral trigger to UART0_RX
    gdma.peri_sel.uart0_rx.set(1)

    // 3. Set destination address
    withUnsafePointer(to: &rxBuffer) { ptr in
        gdma.channel0.dst_addr.set(UInt32(bitPattern: Int32(Int(bitPattern: ptr))))
    }

    // 4. Set length, enable channel
    gdma.channel0.len.set(UInt32(rxBuffer.count))
    gdma.channel0.conf0.circular_en.set(false)
    gdma.channel0.conf0.ch_en.set(true)
}
```

Prefer **swift-mmio** + SVD-generated types over raw addresses — [06-register-programming.md](./06-register-programming.md).

---

## Host Swift — Ring Buffer Pattern (testable on macOS)

When MCU DMA is not yet wired, model the producer/consumer contract on the host:

```swift
struct DmaRingBuffer {
    private var storage: [UInt8]
    private var writeIndex: Int = 0
    private var readIndex: Int = 0
    let capacity: Int

    init(capacity: Int) {
        precondition(capacity > 0 && capacity.isPowerOfTwo)
        self.capacity = capacity
        self.storage = Array(repeating: 0, count: capacity)
    }

    /// Called from "DMA ISR" side — single producer
    mutating func dmaWrite(_ byte: UInt8) -> Bool {
        let next = (writeIndex + 1) & (capacity - 1)
        guard next != readIndex else { return false } // overrun
        storage[writeIndex] = byte
        writeIndex = next
        return true
    }

    /// Called from main/task side — single consumer
    mutating func read() -> UInt8? {
        guard readIndex != writeIndex else { return nil }
        let byte = storage[readIndex]
        readIndex = (readIndex + 1) & (capacity - 1)
        return byte
    }
}
```

Unit-test overrun handling before porting to hardware — [26-testing.md](./26-testing.md).

---

## Step-by-Step

1. Read the TRM DMA chapter — note alignment, max transfer size, PSRAM rules.
2. Pick a peripheral (UART RX is simplest).
3. Allocate a **static** buffer (256–4096 bytes typical).
4. Configure peripheral for DMA mode (FIFO threshold, SPI DMA enable).
5. Configure DMA channel — source/dest increment, data width.
6. Start transfer; handle completion in ISR or async task — [22-swift-concurrency.md](./22-swift-concurrency.md).
7. Verify with logic analyzer or loopback.

---

## Circular Mode

```
     write_ptr ──►
[....████████████....]
     ▲              │
     └── read_ptr ──┘
```

Rules:

- **Single producer (DMA), single consumer (CPU):** track indices; never read past the writer.
- **Power-of-two size** for cheap wrap: `(idx + 1) & (SIZE - 1)`.
- On ESP32-S3 I²S, circular mode is standard; for UART, prefer idle-line detection + one-shot chunks unless streaming high throughput.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Stack-allocated DMA buffer | Corruption, hard fault | Use `static` / global / `@Section` storage |
| CPU reads buffer during DMA | Torn bytes, bad checksums | Wait for transfer-complete or double-buffer |
| Wrong data width (16-bit on 8-bit FIFO) | Garbled data | Match peripheral FIFO width |
| Cache coherency (Cortex-M7) | Stale data | Clean/invalidate cache lines |
| PSRAM without DMA support | Hang or silent failure | Use internal SRAM |
| Forgetting to clear DMA IRQ | Stuck ISR | Clear flag before re-arm |

---

## Debugging Tips

- Log **transfer length** and **error flags** over UART — [25-debugging.md](./25-debugging.md).
- Toggle a GPIO in the DMA ISR; measure latency on a scope.
- Compare CPU copy vs DMA throughput with a timer — [10-timers.md](./10-timers.md).
- On STM32, enable **DMAMUX overrun** interrupts early in bring-up.
- If transfers never complete, check the peripheral **DMA enable bit**.

---

## Performance Tips

- **Double buffering:** DMA fills B while CPU processes A.
- **Align buffers** to 4 or 16 bytes per TRM.
- **Batch interrupts:** IRQ on FIFO threshold, not per byte.
- **Avoid `memcpy` in ISR** — that's what DMA is for.
- On ESP32-S3, prefer **GDMA + hardware FIFO** for SPI displays — [16-spi.md](./16-spi.md).

---

## Exercises

1. **Echo with DMA:** Extend [15-uart.md](./15-uart.md) echo — RX via DMA, TX via CPU; then flip to TX DMA.
2. **Ring buffer:** Implement a 512-byte circular UART RX ring; print overruns and bytes/sec.
3. **Host unit test:** Test `DmaRingBuffer` on macOS with simulated DMA writes.
4. **ADC sweep:** Stream 1 kHz samples via DMA (ESP-IDF path on S3) and compute average in main loop.
5. **Power compare:** Measure current with CPU polling vs DMA + light sleep — [24-low-power.md](./24-low-power.md).

---

## References

- [ESP32-S3 TRM — GDMA](https://www.espressif.com/en/products/socs/esp32-s3)
- [swift-embedded-examples](https://github.com/swiftlang/swift-embedded-examples)
- [swift-mmio](https://github.com/apple/swift-mmio)
- [Embedded Swift documentation](https://github.com/swiftlang/swift/tree/main/docs/EmbeddedSwift)
- [boards/esp32-s3.md](./boards/esp32-s3.md)
- Next: [15-uart.md](./15-uart.md)

---

*Previous: [13-dac.md](./13-dac.md) · Next: [15-uart.md](./15-uart.md)*
