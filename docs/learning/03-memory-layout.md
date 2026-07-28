# Lesson 03: Memory Layout — Flash, RAM, and Linker Scripts

Understanding where your code and data live is essential for debugging crashes, optimizing size, and configuring peripherals. This lesson covers flash, RAM, stack, heap, and linker script sections.

**Prerequisites:** [02-no-std.md](./02-no-std.md)  
**Next:** [04-cargo.md](./04-cargo.md)  
**See also:** [06-register-programming.md](./06-register-programming.md), [glossary.md](./glossary.md)

---

## Theory

### Harvard vs Von Neumann (Simplified)

Most MCUs use a **modified Harvard architecture**: separate buses for instruction fetch and data access, but unified address space from the programmer's view.

| Memory Type | Stores | Volatile? |
|-------------|--------|-----------|
| **Flash** | Program code, constants | No — survives power-off |
| **RAM (SRAM)** | Variables, stack, heap | Yes — lost on power-off |
| **ROM** | Bootloader, chip ID, calibration | No — factory programmed |
| **MMIO** | Peripheral registers | Yes — side effects on read/write |
| **EEPROM / NVS** | User configuration | Non-volatile but slow to write |

### ELF Sections

The compiler and linker organize code into **sections**:

| Section | Purpose | Loaded To |
|---------|---------|-----------|
| `.text` | Executable code | Flash |
| `.rodata` | Read-only data (const strings, lookup tables) | Flash |
| `.data` | Initialized mutable statics | RAM (copied from flash at boot) |
| `.bss` | Zero-initialized mutable statics | RAM (zeroed at boot) |
| `.vector_table` | Interrupt vectors (ARM) | Flash (start of `.text`) |

---

## Hardware Overview

### ESP32-S3 Memory Map (Simplified)

```
Address Range          │ Region
───────────────────────┼──────────────────────────────────
0x0000_0000            │ Boot ROM (internal)
0x3C00_0000 – 0x3DFF_FFFF │ External Flash (mapped, cacheable)
0x3FC8_0000 – 0x3FD0_0000 │ SRAM (512 KB total, regions split)
0x6000_0000 – 0x600F_FFFF │ Peripheral registers (MMIO)
0x4037_0000 – 0x4039_FFFF │ Internal SRAM (IRAM — fast)
```

The ESP32-S3 can execute code from external flash via an **instruction cache**. Hot paths can be placed in **IRAM** for deterministic timing.

### STM32F411 (Comparison)

```
0x0000_0000  Flash alias (boot from flash)
0x0800_0000  Flash (512 KB – 1 MB depending on variant)
0x2000_0000  SRAM (128 KB)
0x4000_0000  AHB/APB peripherals
0xE000_0000  Cortex-M internal (NVIC, SysTick, MPU)
```

### RP2040 (Comparison)

```
0x0000_0000  Boot ROM (XIP from flash)
0x1000_0000  SRAM (264 KB)
0x4000_0000  APB peripherals (GPIO, timers, PIO)
```

---

## Wiring Diagram

Not applicable for this lesson. A logic analyzer on SWD/JTAG can help debug memory-related crashes indirectly by correlating with code execution — see [README.md](./README.md#debugging-tools).

---

## Memory & Register Explanation

### Stack

The **stack** grows downward (on ARM and Xtensa). Each function call pushes a **stack frame** containing:

- Return address
- Saved registers
- Local variables (`let x = 5;`)

```rust
fn outer() {
    let big = [0u8; 4096];  // 4 KB on stack — danger if stack is small!
    inner(&big);             // Borrow — `big` stays on stack during call
}

fn inner(data: &[u8]) {      // `data` is a borrowed slice — pointer + length on stack
    defmt::info!("len={}", data.len());
}
```

**Stack overflow** corrupts adjacent memory silently. Use `flip-link` on ARM to place stack at RAM end and trap overflows.

### Heap

The **heap** grows upward (if present). Requires an allocator:

```rust
// Only with `extern crate alloc` and global allocator configured
let v: alloc::vec::Vec<u8> = alloc::vec![0; 1024];  // Owns heap memory
// `v` dropped here — memory freed (if allocator supports free)
```

Embedded best practice: **avoid heap** in real-time paths.

### Static Storage

```rust
static COUNTER: core::sync::atomic::AtomicU32 = core::sync::atomic::AtomicU32::new(0);
// Lives in .bss (zero-init) or .data (if non-zero initial value)

const LOOKUP: [u16; 256] = [0; 256];  // Lives in .rodata (flash)
```

Statics have `'static` lifetime — valid for entire program. ISRs often share state via `static mut` (with care) or atomics.

### Linker Script Role

The **linker script** (`.ld` file) tells the linker:

1. Where flash and RAM start/end
2. Which sections go where
3. Stack/heap sizes
4. Entry point symbol

Example fragment (ARM generic):

```ld
MEMORY
{
    FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 512K
    RAM (rwx)   : ORIGIN = 0x20000000, LENGTH = 128K
}

SECTIONS
{
    .text : {
        KEEP(*(.vector_table))
        *(.text*)
        *(.rodata*)
    } > FLASH

    .data : {
        _sdata = .;
        *(.data*)
        _edata = .;
    } > RAM AT > FLASH

    .bss : {
        _sbss = .;
        *(.bss*)
        *(COMMON)
        _ebss = .;
    } > RAM
}
```

ESP32-S3 projects typically use linker scripts provided by `esp-hal` / `ldproxy` — you rarely edit them initially.

---

## HAL Implementation

Inspect memory usage after building:

```bash
cargo build --release
cargo size --release -- -A
```

Example output interpretation:

```
   text    data     bss     dec     hex filename
  142832     512    8192  151536   25030 blink
```

| Column | Section | Meaning |
|--------|---------|---------|
| text | `.text` + `.rodata` | Flash consumed by code |
| data | `.data` | RAM initialized from flash |
| bss | `.bss` | RAM zero-initialized |

Place constants in flash explicitly:

```rust
/// Stored in .rodata — does not consume RAM
static SINE_TABLE: [i16; 256] = {
    let mut table = [0i16; 256];
    // ... compile-time generation ...
    table
};
```

Force function into IRAM on ESP32 (advanced):

```rust
#[link_section = ".iram1.text"]
fn timing_critical_isr() {
    // Executed from fast internal RAM
}
```

---

## Bare-Metal Implementation

Reading linker-provided symbols from Rust:

```rust
extern "C" {
    static _sdata: u32;   // Start of .data in RAM
    static _edata: u32;   // End of .data
    static _sidata: u32;  // .data load address in flash
    static _sbss: u32;
    static _ebss: u32;
}

/// Copy .data from flash to RAM, zero .bss — called before main.
/// SAFETY: Only call once from reset handler.
unsafe fn init_memory() {
    let mut src = &_sidata as *const u32;
    let mut dst = &mut _sdata as *mut u32;
    while dst < &mut _edata as *mut u32 {
        core::ptr::write_volatile(dst, core::ptr::read_volatile(src));
        dst = dst.add(1);
        src = src.add(1);
    }

    let mut bss = &mut _sbss as *mut u32;
    while bss < &mut _ebss as *mut u32 {
        core::ptr::write_volatile(bss, 0);
        bss = bss.add(1);
    }
}
```

Startup code in `cortex-m-rt` or `esp-hal` performs equivalent operations — you rarely call this manually.

### Volatile and Memory Ordering

MMIO registers must use volatile access (see [06-register-programming.md](./06-register-programming.md)):

```rust
// Without volatile, compiler may optimize away "redundant" reads/writes
core::ptr::read_volatile(0x6000_4004 as *const u32);
```

---

## Step-by-Step Explanation

### Step 1: Build and Measure

```bash
cargo build --release
cargo size --release
```

Record baseline numbers before adding features.

### Step 2: Identify Large Consumers

```bash
cargo bloat --release -n 20   # Requires cargo-bloat install
```

Look for unexpectedly large functions or panic strings.

### Step 3: Read the Map File

```bash
cargo build --release
# Map file location depends on target — often target/*/release/*.map
```

Search for your function names to see addresses and sizes.

### Step 4: Understand Stack Size

ARM: linker script sets `_stack_start`. Estimate worst-case:

- Deepest call chain × frame size
- Largest local array on stack
- Interrupt nesting (each ISR has its own stack frame on same stack!)

Rule of thumb: start with 8 KB stack for simple apps; increase if crashing mysteriously.

### Step 5: Configure ESP32-S3 Partitions

ESP projects use a **partition table** (separate from linker script):

```
# Name,   Type, SubType, Offset,  Size
nvs,      data, nvs,     0x9000,  0x6000
app0,     app,  factory, 0x10000, 0x100000
```

The application linker script must fit within the `app0` partition size.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Large stack arrays | Random crashes | Use `static mut`, `heapless`, or heap |
| Huge `defmt` strings | Flash bloat | Short log messages; use interned strings |
| Uninitialized static read | Garbage values | Ensure `.bss` init runs (startup bug) |
| Stack + heap collision | Corruption | Reduce heap or increase RAM reservation |
| Ignoring `.data` size | RAM full at boot | Move constants to `.rodata` |
| ISR stack overflow | Crash only under interrupt load | Increase stack; reduce ISR work |

---

## Debugging Tips

1. **`cargo size` after every feature add** — catch growth early.
2. **Fill stack with pattern (`0xA5`)** — inspect in debugger how much was overwritten.
3. **`defmt` log stack pointer** in main loop (platform-specific register read).
4. **HardFault on ARM** — often null pointer or stack overflow; read `LR` and `PC` from debugger.
5. **ESP32 Guru Meditation Error** — note fault address; compare with memory map.

---

## Performance Tips

| Technique | Benefit |
|-----------|---------|
| Store lookup tables in `.rodata` | Free flash, saves RAM |
| `-C opt-level=s` or `z` | Minimize flash |
| Place ISR code in IRAM (ESP32) | Avoid cache miss jitter |
| `#[inline(never)]` on panic paths | Keep hot code compact |
| Avoid `format!` / dynamic strings | No heap, smaller `.rodata` |

---

## Exercises

### Exercise 1: Size Audit

Build the template project in debug and release. Compare `text`, `data`, `bss`. Explain each difference.

### Exercise 2: Const vs Static

Create `const TABLE: [u8; 1024]` and `static TABLE: [u8; 1024]`. Compare sizes with `cargo size`.

### Exercise 3: Stack Experiment

Create a function with `let buf = [0u8; 8192];`. Does it compile? Increase until crash or linker error. Document limit.

### Exercise 4: Memory Map Drawing

Draw the ESP32-S3 and STM32F411 memory maps from this lesson from memory. Check against datasheets.

### Exercise 5: Partition Planning

Design a partition table for an ESP32-S3 with 8 MB flash: NVS, two OTA app slots, SPIFFS storage. Estimate sizes.

---

## References

- [ESP32-S3 Technical Reference Manual — Memory Map](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [STM32F411 Reference Manual RM0383](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [RP2040 Datasheet — Section 2.1 Address Map](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
- [The Embedded Rust Book — Memory Layout](https://docs.rust-embedded.org/book/static-guarantees/index.html)
- [flip-link crate](https://github.com/knurling-rs/flip-link)

---

*Previous: [02-no-std.md](./02-no-std.md) | Next: [04-cargo.md](./04-cargo.md)*
