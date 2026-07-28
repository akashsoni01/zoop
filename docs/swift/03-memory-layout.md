# Lesson 03: Memory Layout for Embedded Swift

Understanding where your code and data live is essential for debugging crashes, optimizing flash usage, and sharing data with interrupt service routines (ISRs). This lesson covers flash, RAM, stack, static storage, and linker scripts for Embedded Swift firmware.

**Prerequisites:** [02-embedded-swift.md](./02-embedded-swift.md)  
**Next:** [04-swiftpm.md](./04-swiftpm.md)  
**See also:** [06-register-programming.md](./06-register-programming.md), [glossary.md](./glossary.md)

---

## Theory

### Memory Map Overview

A typical Cortex-M or ESP32 MCU address space:

```
Address          Region              Contents
─────────────────────────────────────────────────
0x0000_0000      Flash (mapped)      .text, .rodata
0x2000_0000      SRAM                .data, .bss, heap, stack
0x4000_0000+     Peripherals (MMIO)  GPIO, UART, timers
0xE000_0000+     Cortex-M core       NVIC, SysTick, debug
```

Each MCU family differs — always read the datasheet **memory map** chapter.

### ELF Sections

| Section | Loaded to | Purpose |
|---------|-----------|---------|
| `.text` | Flash | Executable code |
| `.rodata` | Flash | Constants, string literals, vtables (if any) |
| `.data` | RAM (copy from flash at boot) | Initialized globals |
| `.bss` | RAM (zeroed at boot) | Uninitialized globals |
| `.stack` | RAM (top of stack region) | Local variables, call frames |

Swift-specific considerations:

- **`StaticString`** → `.rodata` (no heap)
- **Global `var`** → `.data` or `.bss`
- **Struct locals** → stack
- **Class instances** → heap (avoid on MCU)

### Stack vs Heap

```
High address
┌─────────────────┐
│     Stack       │  ← grows downward
│       ↓         │
│                 │
│       ↑         │
│     Heap        │  ← grows upward (if enabled)
├─────────────────┤
│     .bss        │
│     .data       │
Low address
```

**Stack overflow** is the most common RAM failure — a recursive function or large local array corrupts adjacent memory.

Embedded Swift best practice: **disable heap**, size stack generously in linker script, measure with fill patterns.

### Static Lifetimes and ISRs

ISRs may access data that outlives any function call — use `static var` or global storage:

```swift
// Shared between main loop and ISR — must be safe concurrent access
static var buttonPressed: Bool = false
static var tickCount: UInt32 = 0
```

Prefer atomic operations or critical sections when both ISR and main read/write. See [09-interrupts.md](./09-interrupts.md).

---

## Hardware Overview

### ESP32-S3 Memory

| Region | Size | Notes |
|--------|------|-------|
| Internal SRAM | 512 KB | DMA-capable regions differ |
| External Flash | 8–16 MB | Memory-mapped for code |
| RTC SRAM | 16 KB | Survives deep sleep |
| MMIO | Per map | GPIO at ~0x6000_0000 |

### STM32F411 (Comparison)

| Region | Size |
|--------|------|
| Flash | 512 KB |
| SRAM | 128 KB |
| CCM | 64 KB (not all peripherals DMA-accessible) |

### RP2040 (Comparison)

| Region | Size |
|--------|------|
| Flash (external) | 2–16 MB |
| SRAM | 264 KB |

---

## Wiring Diagram

Not applicable — memory layout is software/linker focused. Hardware connection is unchanged from [README.md](./README.md).

---

## Memory & Register Explanation

### Linker Script Essentials

Linker scripts define where sections land. Simplified excerpt:

```ld
MEMORY
{
    FLASH (rx)  : ORIGIN = 0x08000000, LENGTH = 512K
    RAM (rwx)   : ORIGIN = 0x20000000, LENGTH = 128K
}

SECTIONS
{
    .text : { *(.text*) *(.rodata*) } > FLASH
    .data : { *(.data*) } > RAM AT > FLASH
    .bss  : { *(.bss*) *(COMMON) } > RAM
    .stack (NOLOAD) : {
        . = ALIGN(8);
        _stack_top = .;
        . = . + 0x4000;   /* 16 KB stack */
    } > RAM
}
```

Symbols like `_stack_top` are referenced from startup code.

### Reading Size Output

After building firmware:

```bash
# GNU size (ARM GCC toolchain)
arm-none-eabi-size firmware.elf

# LLVM size
llvm-size firmware.elf
```

Example output:

```
   text    data     bss     dec     hex filename
  45678     892    2048   48618    bdea firmware.elf
```

| Column | Meaning |
|--------|---------|
| text | Flash code + rodata |
| data | Initialized RAM (also occupies flash) |
| bss | Zero-init RAM |

### MMIO vs RAM

MMIO registers are **not** memory you allocate — they are fixed addresses:

```swift
// This pointer does NOT consume RAM for storage — it names an address
let gpio = UnsafeMutablePointer<UInt32>(bitPattern: 0x6000_4000)!
```

Distinguish **variables** (consume RAM) from **addresses** (constants in code).

---

## HAL-Style Swift Implementation

Track memory usage in a diagnostics module:

```swift
struct MemoryReport {
    let textBytes: UInt32
    let dataBytes: UInt32
    let bssBytes: UInt32
    let stackSize: UInt32

    var totalRam: UInt32 { dataBytes + bssBytes + stackSize }

    func log() {
        log("Flash text: ")
        logUInt(textBytes)
        log(" RAM data+bss: ")
        logUInt(dataBytes + bssBytes)
    }
}

// Linker symbols imported from C/assembly
@_silgen_name("_etext") extern var etext: UInt8
@_silgen_name("_sdata") extern var sdata: UInt8
@_silgen_name("_ebss") extern var ebss: UInt8

func buildMemoryReport() -> MemoryReport {
    let textSize = UInt32(UInt(bitPattern: &sdata) - UInt(bitPattern: &etext))
    let dataSize = UInt32(0)  // compute from linker symbols
    let bssSize = UInt32(UInt(bitPattern: &ebss) - UInt(bitPattern: &sdata))
    return MemoryReport(
        textBytes: textSize,
        dataBytes: dataSize,
        bssBytes: bssSize,
        stackSize: 16 * 1024
    )
}
```

Exact symbol names depend on toolchain — consult your project's linker script.

---

## Bare-Metal / MMIO Swift Notes

Stack canary pattern for overflow detection during development:

```swift
const stackCanary: UInt32 = 0xDEADBEEF
var stackCanaryCheck: UInt32 = stackCanary

func checkStack() -> Bool {
    return stackCanaryCheck == stackCanary
}

// Call periodically from main loop
if !checkStack() {
    fatalError("Stack overflow detected")
}
```

Place canary at stack boundary via linker script for production-grade detection.

Fixed buffer in `.bss`:

```swift
// 256-byte UART RX buffer — lives in .bss, no heap
var uartRxBuffer: (
    UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
    UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
    UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8,
    UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8, UInt8
) = (0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0)
// ... extend to 256 bytes or use UnsafeMutableBufferPointer to static storage
```

---

## Step-by-Step Explanation

### Step 1: Build and Run `size`

Record baseline numbers for your blink project.

### Step 2: Locate Linker Script

Find `linker.ld` or `memory.ld` in your board support package. Identify FLASH and RAM origins.

### Step 3: Map Swift Constructs to Sections

| Code | Section |
|------|---------|
| `func foo()` | `.text` |
| `let x = 42` at global | `.data` |
| `var counter = 0` global | `.data` |
| `var buffer: [UInt8; 64]` static | `.bss` |
| `let msg: StaticString = "hi"` | `.rodata` |
| `var x` inside `func` | stack |

### Step 4: Estimate Stack Depth

Nested calls and large locals consume stack. Rule of thumb: start with 4–8 KB, increase if crashes are mysterious.

### Step 5: Avoid Heap

If you must allocate, use a fixed pool allocator in a static buffer — never `malloc` without bounds.

### Step 6: Align ISR Shared Data

Place shared statics in proper memory for DMA (ESP32: internal vs external RAM rules).

### Step 7: Re-measure After Changes

Every new feature — re-run size and note flash/RAM delta.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Large stack arrays | Random crashes | Move to static or reduce size |
| Unbounded recursion | Stack overflow | Use iterative algorithms |
| Global mutable without sync | ISR corruption | Atomics or critical sections |
| Ignoring `.data` flash cost | Flash full | Prefer zero-init `.bss` |
| DMA from flash buffer | Garbage data | Place buffer in DMA-capable RAM |
| Wrong linker script | Boot loop | Match exact chip variant |

---

## Debugging Tips

1. **Fill stack with pattern** (0xA5) — inspect unused portion in debugger.
2. **Watch `_estack` / `_stack_top`** — SP must stay below limit.
3. **Map file** — `nm` or linker `-Map=` output shows symbol addresses.
4. **Compare before/after** — bisect flash growth.
5. **LLDB `memory read`** — inspect RAM regions during halt.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| `const` / static for lookup tables | Keep in flash `.rodata` |
| Place hot loop in IRAM (ESP32) | Linker section attribute |
| Minimize `.data` | Default globals to zero (`.bss`) |
| Pack structs | `#pragma pack` or Swift `@_packed` if available |
| Avoid floating point on M0+ | Software float is slow and large |

---

## Exercises

### Exercise 1: Size Baseline

Document text/data/bss for your blink firmware. Screenshot or paste into lab notebook.

### Exercise 2: Section Map

Draw a diagram of your chip's memory map from the datasheet. Label where `.text` and stack live.

### Exercise 3: Static Buffer

Replace a function-local `[UInt8; 256]` with a static buffer. Compare stack usage (conceptually).

### Exercise 4: Linker Symbol

Import one linker symbol and print computed section size over UART.

### Exercise 5: Stack Canary

Implement a simple stack overflow check and trigger it deliberately to verify detection.

---

## References

- [ESP32-S3 TRM — Memory Map](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [STM32F411 Reference Manual RM0383](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [RP2040 Datasheet — Memory Map](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
- [Lesson 04 — SwiftPM](./04-swiftpm.md)
- [Embedded Rust memory equivalent](../learning/03-memory-layout.md)

---

*Previous: [02-embedded-swift.md](./02-embedded-swift.md) | Next: [04-swiftpm.md](./04-swiftpm.md)*
