# Lesson 25 — Debugging: probe-rs, GDB, RTT, defmt, SWD/JTAG

**Prerequisites:** [04-cargo.md](./04-cargo.md), [08-gpio.md](./08-gpio.md)

**Board focus:** [ESP32-S3](./boards/esp32-s3.md) (USB-JTAG), [STM32](./boards/stm32.md) (SWD), [RP2040](./boards/rp2040.md).

---

## Theory

Embedded debugging differs from host development: no terminal, no core dumps by default, and **observability** must be designed in. A layered toolkit:

| Layer | Tool | Provides |
|-------|------|----------|
| **Logging** | `defmt`, serial `println!` | Runtime traces |
| **Interactive** | GDB + probe | Breakpoints, memory inspect |
| **Tracing** | RTT (Real-Time Transfer) | Fast log pipe without stopping CPU |
| **Physical** | SWD/JTAG | Debug wire protocol to MCU |
| **Electrical** | Logic analyzer, scope | Timing proof |

### SWD vs JTAG

| Protocol | Wires | Typical use |
|----------|-------|-------------|
| **SWD** (Serial Wire Debug) | SWDIO, SWCLK, GND, 3V3 | ARM Cortex-M default — 2 data lines |
| **JTAG** | TDI, TDO, TCK, TMS, TRST | Boundary scan, older ARM, ESP32 USB-JTAG |

**probe-rs** speaks SWD/JTAG to flash, halt, and debug.

### defmt vs println

| | `defmt` | `println!` / UART |
|---|---------|-------------------|
| Formatting | Host-side | Device-side |
| Cost | ~few bytes per log | `format!` heavy |
| Transport | RTT or UART | UART/USB |
| Timestamps | Yes with `defmt-timestamp` | Manual |

Prefer **defmt** in production firmware — [Knurling](https://knurling.org/) tooling.

---

## Hardware Overview

### ESP32-S3 DevKitC-1

Built-in **USB Serial/JTAG** — no external probe for basic GDB:

```
PC ── USB-C ──► ESP32-S3 (internal JTAG + CDC serial)
```

OpenOCD or `espflash` can attach for debugging depending on toolchain.

### STM32 Nucleo

Integrated **ST-Link** — SWD to target MCU on same PCB.

### External probe wiring (STM32 target)

```
ST-Link          Target MCU
SWDIO ─────────► SWDIO
SWCLK ─────────► SWCLK
GND   ─────────► GND
3V3   ─────────► 3V3 (sense, usually not power)
```

---

## ASCII Wiring

ESP32-S3 — single cable debugging:

```
┌─────────────┐   USB-C    ┌─────────────────┐
│ Developer PC│◄──────────►│ ESP32-S3 DevKit │
└─────────────┘            │  JTAG + UART    │
                           └─────────────────┘
```

Optional UART tap for third-party adapter — [15-uart.md](./15-uart.md):

```
GPIO43 TX ──► USB-UART RX
GPIO44 RX ◄── USB-UART TX
GND ──────── GND
```

---

## Memory & Register Notes

### RTT control block

**RTT** uses a **control block** in RAM located by the debugger:

```
RAM:
  "SEGGER RTT" magic
  Up buffer 0  → host reads logs
  Down buffer 0 → host sends commands (optional)
```

`defmt-rtt` places formatted strings in flash; RTT sends indices + arguments.

### Breakpoints and flash

Hardware breakpoints limited (2–6 on Cortex-M). **Software breakpoints** patch flash — wear not an issue for debug sessions.

### Watchpoints

Monitor memory address access — catch buffer overruns:

```
(gdb) watch *0x20001000
```

---

## Setup — defmt + RTT (ARM example)

`Cargo.toml`:

```toml
[dependencies]
defmt = "0.3"
defmt-rtt = "0.4"
panic-probe = { version = "0.3", features = ["print-defmt"] }

[[bin]]
name = "firmware"
test = false
bench = false

[profile.release]
debug = 2
```

Firmware:

```rust
use defmt::{info, warn, debug, trace};

#[entry]
fn main() -> ! {
    info!("Boot OK — firmware v{}", env!("CARGO_PKG_VERSION"));

    let mut count = 0u32;
    loop {
        count += 1;
        debug!("heartbeat {}", count);
        cortex_m::asm::delay(84_000_000); // ~1 s at 84 MHz — use timer in real code
    }
}
```

Run with probe-rs:

```bash
probe-rs run --chip STM32F411RETx target/thumbv7em-none-eabihf/debug/firmware
```

Logs appear inline in terminal.

---

## probe-rs + GDB

```bash
# Terminal 1: GDB server
probe-rs gdb --chip STM32F411RETx --port 1337

# Terminal 2: GDB client
arm-none-eabi-gdb target/thumbv7em-none-eabihf/debug/firmware
(gdb) target remote :1337
(gdb) break main
(gdb) continue
(gdb) print count
(gdb) backtrace
```

`Embed.toml` / `.cargo/config.toml` runner:

```toml
[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32F411RETx"
```

---

## ESP32-S3 — espflash monitor + OpenOCD

```bash
# Flash + serial monitor (defmt if configured)
espflash flash --monitor target/xtensa-esp32s3-none-elf/debug/app

# OpenOCD GDB (toolchain dependent)
openocd -f board/esp32s3-builtin.cfg
xtensa-esp32s3-elf-gdb target/.../debug/app
(gdb) target remote :3333
```

Check [Espressif Rust debugging chapter](https://esp-rs.github.io/book/) for current OpenOCD/GDB flow.

---

## Serial Logs (fallback)

```rust
use esp_println::println; // or uart write

println!("Sensor value: {}", reading);
```

Higher overhead but universal. Match baud in monitor — [15-uart.md](./15-uart.md).

---

## Step-by-Step

1. Add **defmt** + **panic-probe** to project.
2. Replace `println!` with `info!` / `debug!`.
3. **`cargo run`** via probe-rs — confirm RTT output.
4. Set **breakpoint** at suspicious function; inspect locals.
5. Add **GPIO debug pin** toggle around ISR — scope verification.
6. Configure **CI** to at least build debug artifacts — [26-testing.md](./26-testing.md).
7. Document **debug probe pinout** on your custom PCB.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Release profile no debug symbols | GDB `??` | `debug = 2` in profile |
| Wrong chip string | probe-rs connect fail | Exact chip name from datasheet |
| SWDIO/SWCLK swapped | No attach | Swap wires |
| Target unpowered | Voltage 0 | Power target separately |
| defmt without RTT init | Empty logs | Link `defmt-rtt` |
| Logging in tight ISR | Missed deadlines | Defer to main |
| USB hub power | Flaky debug | Direct port |

---

## Debugging Tips

- **`defmt::trace!`** only in debug builds: `#[cfg(debug_assertions)]`.
- Use **`defmt::assert!`** — prints file/line on failure without panicking strings in flash.
- **Semihosting** — slow; avoid except bring-up.
- **`cargo size -- -A`** after adding logging — watch flash growth.
- **Logic analyzer** on SPI/I²C when software logs insufficient — [16-spi.md](./16-spi.md).
- **`readelf -s`** / **`nm`** for symbol addresses.

---

## Performance Tips

- **`defmt` levels:** compile-time filter with `DEFMT_LOG=info`.
- Batch logs — avoid per-byte UART in hot loops.
- RTT up-buffer size 1–4 KB — balance latency vs drops.
- Disable debug in release shipping firmware or gate behind feature flag.
- Use **strip** + **LTO** for production; keep separate debug ELF archived.

---

## Exercises

1. **Breakpoint hunt:** Introduce off-by-one bug; locate with GDB backtrace.
2. **Watchpoint:** Overflow buffer; catch with watch on next cell.
3. **Timing pin:** Toggle GPIO at ISR entry/exit; measure on scope.
4. **ESP32-S3:** Same defmt via espflash monitor path.
5. **Compare:** Log 1000 `defmt` vs `println` messages — note flash/size diff.

---

## References

- [Knurling defmt book](https://defmt.ferrous-systems.com/)
- [probe-rs book](https://probe.rs/docs/)
- [Espressif Rust — Debugging](https://esp-rs.github.io/book/)
- [OpenOCD user guide](https://openocd.org/doc/html/index.html)
- [26-testing.md](./26-testing.md)

---

*Previous: [24-low-power.md](./24-low-power.md) · Next: [26-testing.md](./26-testing.md)*
