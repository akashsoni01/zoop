# Lesson 14 — Direct Memory Access (DMA)

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [10-timers.md](./10-timers.md), [07-hal.md](./07-hal.md)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md) (GDMA — General Direct Memory Access). Also covers STM32, RP2040, nRF52.

---

## Theory

**Direct Memory Access (DMA)** is a hardware peripheral that moves data between memory and a peripheral (or between memory regions) **without the CPU executing load/store instructions for every byte**. The CPU configures a transfer — source address, destination address, length, and trigger — then continues other work while the DMA engine handles the copy.

Why DMA matters in embedded Rust:

- **Throughput:** UART/SPI/I²C at high baud rates or sample rates can saturate the CPU if you byte-bang.
- **Determinism:** Interrupt Service Routines (ISRs) that copy large buffers add jitter to real-time tasks.
- **Power:** The CPU can sleep while DMA runs (with careful clock gating).

Key concepts:

| Term | Meaning |
|------|---------|
| **Channel** | One independent DMA transfer engine (ESP32-S3 has many GDMA channels) |
| **Descriptor / linked list** | Chained blocks describing multiple transfers (circular mode uses this) |
| **Burst** | Number of consecutive bytes the DMA reads/writes per bus cycle |
| **Handshake** | Peripheral signals "ready" so DMA does not overrun FIFO |
| **Circular / ring buffer** | DMA wraps destination pointer when it reaches the end |

### Ownership in Rust

Rust's ownership rules apply to DMA buffers with extra constraints:

1. The buffer must remain **valid and at a fixed address** for the entire transfer (no stack buffers that go out of scope).
2. The buffer must often be **DMA-capable** (not cached incoherently, aligned per hardware rules).
3. While DMA owns the buffer, the CPU must not read/write it unless you synchronize (wait for completion or use double-buffering).

On ESP32-S3 with internal SRAM, use `static mut` or heap in a dedicated region; with **PSRAM** (Pseudo-Static RAM), check whether the peripheral supports PSRAM as a DMA source/destination — not all paths do.

---

## Hardware Overview

### ESP32-S3 GDMA

The ESP32-S3 uses **GDMA** (General DMA) with separate TX/RX channels per peripheral class:

- SPI, I²S, UART, AES, ADC, etc. each bind to GDMA channels.
- Up to 512-byte bursts depending on configuration.
- Supports **circular mode** for continuous ADC or audio streaming.

```
CPU                    GDMA Controller              Peripheral
 |                           |                          |
 |-- configure channel ----->|                          |
 |-- start transfer -------->|                          |
 |                           |--- read/write bytes ---->|
 | (continues other work)      |<--- handshake/FIFO ------|
 |<-- interrupt on complete ---|                          |
```

### STM32 DMA (reference)

Cortex-M STM32 parts use **DMA1/DMA2** with request multiplexers. Each channel maps to a peripheral via **DMAMUX**. Double-buffer mode (`DBM`) is common for ADC + timer triggers.

### RP2040 DMA (reference)

The RP2040 has **12 independent DMA channels** with **control blocks** in memory — excellent for PIO (Programmable I/O) feeding or custom protocols. See [boards/rp2040.md](./boards/rp2040.md).

---

## ASCII Wiring

DMA is internal — no external wiring. For this lesson's UART echo with DMA:

```
ESP32-S3 DevKitC-1
┌─────────────────────┐
│  GPIO43 (U0 TXD) ───┼──► USB-UART adapter RX (optional monitor)
│  GPIO44 (U0 RXD) ◄───┼─── USB-UART adapter TX
│  GND ───────────────┼─── GND
└─────────────────────┘
```

Use the built-in USB-JTAG/serial on DevKitC-1 when possible — see [25-debugging.md](./25-debugging.md).

---

## Memory & Register Notes

### ESP32-S3 GDMA registers (conceptual)

| Register / field | Purpose |
|------------------|---------|
| `CH0_CONF0` | Enable channel, memory/ peripheral address increment |
| `CH0_LINK_ADDR` | Pointer to linked-list descriptor |
| `CH0_OUT_LINK` / `IN_LINK` | Start outbound/inbound transfer |
| `CH0_INT_RAW` | Transfer complete, error flags |

Descriptors live in RAM and look like:

```
[ length | src_addr | dst_addr | next_desc_ptr ]
```

For **circular mode**, `next_desc_ptr` points back to the first descriptor or the hardware auto-wraps the write pointer within a fixed buffer bounds.

### Buffer placement (ESP32-S3)

| Region | DMA-safe? | Notes |
|--------|-----------|-------|
| Internal SRAM | Yes | Preferred for control structures |
| PSRAM | Peripheral-dependent | Audio/I²S often OK; check TRM |
| Flash (XIP) | Read-only source sometimes | Never DMA destination for writes |

Consult the [ESP32-S3 Technical Reference Manual](https://www.espressif.com/en/products/socs/esp32-s3) — search "DMA" and "Memory Mapping".

---

## HAL Example (esp-hal) — UART RX via DMA

Using `esp-hal` on ESP32-S3. Adjust versions to match your workspace.

```rust
// Cargo.toml dependencies (example):
// esp-hal = { version = "0.21", features = ["esp32s3", "async"] }

use esp_hal::{
    dma::{Dma, DmaRxBuf, DmaTransferRx},
    gpio::Io,
    peripherals::Peripherals,
    uart::{Uart, UartRx, config::Config},
};
use static_cell::StaticCell;

// DMA buffer MUST live for 'static lifetime
static RX_BUF: StaticCell<[u8; 1024]> = StaticCell::new([0u8; 1024]);

fn main() -> ! {
    let peripherals = Peripherals::take();
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // UART0 on default pins — see boards/esp32-s3.md
    let uart = Uart::new(peripherals.UART0, Config::default())
        .unwrap()
        .with_rx(io.pins.gpio44)  // U0RXD
        .with_tx(io.pins.gpio43); // U0TXD

    let (rx, _tx) = uart.split();

    // Wrap buffer for DMA — esp-hal tracks ownership until transfer completes
    let mut dma_rx_buf = DmaRxBuf::new(RX_BUF.init([0u8; 1024])).unwrap();

    let dma = Dma::new(peripherals.DMA);
    let dma_channel = dma.channel0;

    loop {
        // Start DMA receive — hardware fills buffer on incoming UART bytes
        let transfer: DmaTransferRx<'_, _, _> =
            rx.read_dma(dma_channel, &mut dma_rx_buf);

        // Wait for line idle or buffer full (API varies by version)
        // In async code you'd await; here we poll or block
        let (rx, channel, buf, received) = transfer.wait();

        // Process `received` bytes — do NOT access buf until wait() returns
        for b in received {
            // echo logic handled in lesson 15
            let _ = b;
        }

        // Re-bind for next transfer
        dma_rx_buf = buf;
        dma_channel = channel;
        // rx = rx;
    }
}
```

The HAL enforces that you cannot reuse the buffer until `wait()` or an async completion future resolves — this maps directly to Rust ownership.

---

## Bare-Metal Style (register-level sketch)

Illustrative only — prefer PAC/HAL in production:

```rust
// Conceptual: configure GDMA RX linked to UART FIFO
// Use esp32s3 PAC crate for real addresses

const RX_BUFFER: static [u8; 256] = [0; 256];

unsafe fn start_uart_rx_dma() {
    // 1. Reset GDMA channel
    // write_reg(GDMA_CONF0, CH_RESET);

    // 2. Set peripheral trigger to UART0_RX
    // write_reg(GDMA_PERI_SEL, UART0_RX_REQ);

    // 3. Set destination to RX_BUFFER.as_ptr()
    // write_reg(GDMA_DST_ADDR, RX_BUFFER.as_ptr() as u32);

    // 4. Set length and enable circular if desired
    // write_reg(GDMA_LEN, 256);
    // write_reg(GDMA_CONF0, CIRCULAR_EN | CH_EN);

    // 5. Enable transfer-complete interrupt in CPU
    // NVIC.enable(GDMA_CH0 interrupt);
}
```

See [06-register-programming.md](./06-register-programming.md) for MMIO (Memory-Mapped I/O) patterns.

---

## Step-by-Step

1. **Read the TRM** DMA chapter for your MCU — note alignment, max transfer size, and PSRAM rules.
2. **Pick a peripheral** (UART RX is the simplest first project).
3. **Allocate a `static` buffer** — size = expected burst × safety margin (often 256–4096 bytes).
4. **Configure peripheral** for DMA mode (UART FIFO threshold, SPI DMA enable bit).
5. **Configure DMA channel** — source/dest increment, data width (8/16/32-bit).
6. **Start transfer**, handle completion in ISR or async task.
7. **Verify** with logic analyzer or loopback — compare byte counts.

---

## Circular Mode

Circular DMA writes into a ring buffer continuously:

```
     write_ptr ──►
[....████████████....]
     ▲              │
     └── read_ptr ──┘
```

Rules:

- **Single producer (DMA), single consumer (CPU):** track `read_idx` and `write_idx`; never read past the writer.
- **Use power-of-two size** so index wrap is `(idx + 1) & (SIZE - 1)`.
- On ESP32-S3 I²S audio, circular mode is standard; for UART, prefer idle-line detection + one-shot DMA chunks unless streaming high throughput.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Stack-allocated DMA buffer | Random corruption, hard fault | Use `static`, `StaticCell`, or `heapless::Vec` in static |
| CPU reads buffer during DMA | Torn bytes, bad checksums | Wait for `transfer complete` or use double buffering |
| Wrong data width (16-bit on 8-bit FIFO) | Garbled data | Match peripheral FIFO width |
| Cache coherency (Cortex-A/M7 with cache) | Stale data | Clean/invalidate cache lines (not on ESP32-S3 internal SRAM) |
| PSRAM without DMA support | Silent failure or hang | Use internal SRAM or enable proper path |
| Forgetting to clear DMA IRQ flag | Immediate re-entry / stuck | Clear in ISR before re-arm |

---

## Debugging Tips

- Log **transfer length** and **error flags** via [defmt](./25-debugging.md).
- Use **GPIO pin toggle** in DMA ISR to measure latency on a scope.
- Compare **CPU copy vs DMA** throughput with a timer — see [10-timers.md](./10-timers.md).
- On STM32, enable **DMAMUX overrun** interrupts early in bring-up.
- If transfers never complete, check **peripheral DMA enable bit** (UART/SPI has a separate enable).

---

## Performance Tips

- **Double buffering:** While DMA fills buffer B, CPU processes buffer A — zero idle time.
- **Align buffers** to 4 or 16 bytes per TRM — can improve burst efficiency.
- **Batch interrupts:** Request IRQ every N bytes (FIFO threshold), not per byte.
- **Avoid `memcpy` in ISR:** DMA's job is to eliminate this — if you still copy, profile why.
- On ESP32-S3, prefer **GDMA + hardware FIFO** over bit-banging SPI for displays — see [16-spi.md](./16-spi.md).

---

## Exercises

1. **Echo with DMA:** Extend [15-uart.md](./15-uart.md) echo to receive via DMA and transmit via CPU — then flip to TX DMA.
2. **Ring buffer:** Implement a 512-byte circular UART RX ring; print statistics (overruns, bytes/sec).
3. **ADC sweep:** On STM32 or ESP32-S3 (if ADC+DMA supported in your HAL version), stream 1 kHz samples into a buffer and compute average in main loop.
4. **Compare power:** Measure current with CPU polling vs DMA + `wfi` sleep — ties to [24-low-power.md](./24-low-power.md).

---

## References

- [ESP32-S3 TRM — GDMA](https://www.espressif.com/en/products/socs/esp32-s3)
- [esp-hal DMA module docs](https://docs.esp-rs.org/esp-hal/)
- [Embedded Rust Book — Concurrency](https://docs.rust-embedded.org/book/concurrency/index.html)
- [boards/esp32-s3.md](./boards/esp32-s3.md) — memory regions
- Next: [15-uart.md](./15-uart.md)

---

*Previous: [13-dac.md](./13-dac.md) · Next: [15-uart.md](./15-uart.md)*
