# Lesson 06: Register Programming — MMIO, Volatile, and PAC

Every peripheral on an MCU is controlled through **registers** — fixed memory addresses that read or write hardware state. This lesson covers **MMIO** (Memory-Mapped I/O), volatile access, PAC-generated APIs, and bitfield manipulation.

**Prerequisites:** [05-embedded-architecture.md](./05-embedded-architecture.md)  
**Next:** [07-hal.md](./07-hal.md)  
**See also:** [03-memory-layout.md](./03-memory-layout.md), [08-gpio.md](./08-gpio.md)

---

## Theory

### What is MMIO?

**MMIO (Memory-Mapped I/O)** maps peripheral control registers into the CPU's address space. Reading address `0x6000_4020` on ESP32-S3 might read the GPIO output enable register — not RAM.

| Access Type | Behavior |
|-------------|----------|
| Normal RAM read/write | Compiler may reorder, cache, elide "redundant" ops |
| Volatile MMIO read/write | Always emitted; side effects preserved |
| Read-only register | Write ignored or fault |
| Write-only register | Read returns garbage or 0 |
| Read-to-clear | Reading clears status bits |

### PAC (Peripheral Access Crate)

PACs are **auto-generated** from vendor **SVD (System View Description)** XML files using tools like `svd2rust`:

```
vendor.svd  →  svd2rust  →  pac crate (esp32s3, stm32f4, rp2040_pac)
```

PAC provides:

- Typed register structs at correct addresses
- Named fields with bit ranges
- `read()`, `write()`, `modify()` methods

---

## Hardware Overview

### ESP32-S3 GPIO Registers (Subset)

| Register | Offset | Purpose |
|----------|--------|---------|
| `GPIO_OUT` | +0x04 | Current output levels |
| `GPIO_OUT_W1TS` | +0x10 | Write 1 to **set** bit high |
| `GPIO_OUT_W1TC` | +0x14 | Write 1 to **clear** bit low |
| `GPIO_ENABLE` | +0x20 | Output enable per pin |
| `GPIO_IN` | +0x3C | Input levels |

Base address: `0x6000_4000` (GPIO peripheral).

**W1TS/W1TC pattern:** Avoids read-modify-write races in concurrent code (ISR + main both toggling different bits).

### STM32 GPIO (Comparison)

STM32 uses classic set/reset registers:

| Register | Purpose |
|----------|---------|
| `MODER` | Input / output / alternate function mode |
| `OTYPER` | Push-pull vs open-drain |
| `OSPEEDR` | Slew rate |
| `PUPDR` | Pull-up / pull-down |
| `IDR` | Input data |
| `ODR` | Output data |
| `BSRR` | Bit set/reset (atomic) |

### RP2040 IO BANK (Comparison)

RP2040 groups 30 GPIO pins with **`IO`** and **`PADS`** banks. Control split across `iobank0` and `padsbank0` PAC modules.

---

## Wiring Diagram

Register programming is invisible on the breadboard, but you must know which **pin number** maps to which **register bit**:

```
ESP32-S3 DevKitC-1 — LED on GPIO2
──────────────────────────────────

    ESP32-S3                LED Circuit
  ┌──────────┐
  │      GPIO2├──────────[ 220Ω ]────(>|)──── GND
  │          │
  │       3V3│  (not used for output drive — LED sinks from GPIO)
  │       GND├────────────────────────────── common GND
  └──────────┘

Register bit: GPIO_OUT bit 2 (pin 2)
Enable bit:   GPIO_ENABLE bit 2
```

Always check your board schematic — some DevKits use GPIO48 for RGB LED.

---

## Memory & Register Explanation

### Bitfield Example: STM32 GPIO MODER

Each pin uses 2 bits in `MODER`:

```
MODER[1:0]   for pin 0
MODER[3:2]   for pin 1
...
00 = Input
01 = General purpose output
10 = Alternate function
11 = Analog
```

For pin 5 (PA5 LED on Nucleo):

```rust
// PAC modify — safe read-modify-write
gpioa.moder.modify(|r, w| unsafe {
    w.moder5().bits(0b01)  // Output mode
});
```

### Volatile Semantics in Rust

```rust
use core::ptr::{read_volatile, write_volatile};

let reg = 0x6000_4010 as *mut u32;

unsafe {
    // Set bit 2 — write-only register pattern on ESP32
    write_volatile(reg, 1 << 2);

    // Read input register
    let inputs = read_volatile(0x6000_403C as *const u32);
    let button_pressed = (inputs & (1 << 0)) == 0;  // Active low
}
```

**Why `unsafe`?** The compiler cannot prove these addresses are valid or that aliasing is safe — you assert hardware knowledge.

### Read-Modify-Write Hazard

```rust
// DANGEROUS if ISR also modifies same register:
let val = read_volatile(reg);
write_volatile(reg, val | (1 << 5));  // ISR may interleave here

// BETTER on ESP32: use W1TS/W1TC
write_volatile(OUT_W1TS, 1 << 5);  // Atomic set
write_volatile(OUT_W1TC, 1 << 5);  // Atomic clear
```

---

## HAL Implementation

HAL wraps PAC with safe APIs:

```rust
use esp_hal::gpio::{Io, Level, Output, Input, Pull};

fn setup_gpio(peripherals: esp_hal::peripherals::Peripherals) {
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // `Output::new` consumes the pin identifier — exclusive ownership
    let mut led = Output::new(io.pins.gpio2, Level::Low);

    // `Input::new` with pull-up for button
    let button = Input::new(io.pins.gpio0, Pull::Up);

    led.set_high().unwrap();
    let pressed = button.is_low().unwrap();  // Active low when pressed
}
```

Under the hood, `set_high()` writes to `GPIO_OUT_W1TS` via PAC.

STM32 HAL:

```rust
let mut led = gpioa.pa5.into_push_pull_output();
led.set_high();
let button = gpioc.pc13.into_pull_up_input();
let pressed = button.is_low();
```

---

## Bare-Metal Implementation

### Raw Pointer GPIO Toggle (ESP32-S3 GPIO2)

```rust
const GPIO_BASE: u32 = 0x6000_4000;
const GPIO_ENABLE: u32 = GPIO_BASE + 0x20;
const GPIO_OUT_W1TS: u32 = GPIO_BASE + 0x10;
const GPIO_OUT_W1TC: u32 = GPIO_BASE + 0x14;
const IO_MUX_BASE: u32 = 0x6000_9000;
const IO_MUX_GPIO2: u32 = IO_MUX_BASE + 0x90;  // 0x4 × pin for some layouts

/// Configure GPIO2 as output and blink once.
/// SAFETY: Caller ensures exclusive access to GPIO hardware.
pub unsafe fn bare_blink_once() {
    // Step 1: Set IO_MUX to GPIO function (MCU_FUNC_SEL = 1)
    core::ptr::write_volatile(IO_MUX_GPIO2 as *mut u32, 1);

    // Step 2: Enable output
    let enable = core::ptr::read_volatile(GPIO_ENABLE as *const u32);
    core::ptr::write_volatile(GPIO_ENABLE as *mut u32, enable | (1 << 2));

    // Step 3: Set high then low using atomic W1TS/W1TC
    core::ptr::write_volatile(GPIO_OUT_W1TS as *mut u32, 1 << 2);
    busy_wait();
    core::ptr::write_volatile(GPIO_OUT_W1TC as *mut u32, 1 << 2);
}

fn busy_wait() {
    for _ in 0..500_000 {
        core::hint::spin_loop();
    }
}
```

### PAC-Based (Preferred Bare-Metal)

```rust
use esp32s3::GPIO;

fn pac_blink() {
    // Access PAC singleton — zero-cost wrapper around address
    GPIO.out_w1ts().write(|w| w.out_data().bits(1 << 2));
    busy_wait();
    GPIO.out_w1tc().write(|w| w.out_data().bits(1 << 2));
}
```

PAC `modify` for read-modify-write fields:

```rust
GPIO.enable().modify(|r, w| unsafe {
    w.enable().bits(r.enable().bits() | (1 << 2))
});
```

---

## Step-by-Step Explanation

### Step 1: Find Register Map

Open the chip **Technical Reference Manual**. Locate peripheral chapter (GPIO). Note base address and register offsets.

### Step 2: Identify Pin-to-Bit Mapping

GPIO pin number usually equals bit index (with exceptions for pins > 31 on 32-bit registers — use second register bank).

### Step 3: Configure Pin Function (Mux)

Before GPIO works, the **IO mux** must route the pin to GPIO (not UART/SPI alternate function).

### Step 4: Set Direction

Enable output or configure input mode with appropriate pull resistor.

### Step 5: Read or Write Data

Use W1TS/W1TC (ESP32) or BSRR (STM32) for atomic bit operations.

### Step 6: Verify with Debugger or LED

If LED doesn't light, check:

- Correct pin number
- Output enable set
- LED polarity (active high vs low)
- Current-limiting resistor present

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Non-volatile access | Optimized-away writes | Use `read_volatile`/`write_volatile` or PAC |
| Wrong register for pin > 31 | Some bits never change | Use OUT1, ENABLE1 registers |
| Forgot IO mux | Pin stuck or wrong signal | Configure function select first |
| Read-modify-write race | Random bit flips | Use set/clear registers |
| Assuming write persists | Pin toggles briefly | Check if register is write-only pulse |
| Floating input | Random button reads | Enable internal or external pull-up |

---

## Debugging Tips

1. **Read back registers** after write — confirm enable bits set.
2. **Use logic analyzer** on pin — faster than guessing.
3. **`probe-rs readmem 0x60004020`** — inspect register from CLI.
4. **Compare PAC vs manual address** — catches typos.
5. **Check reset value** in TRM — some pins have default pull-ups.

---

## Performance Tips

| Technique | When |
|-----------|------|
| W1TS/W1TC instead of RMW | Any multi-context GPIO |
| Batch writes via `OUT` register | Multiple pins change simultaneously |
| Direct PAC in ISR | When HAL overhead matters (nanoseconds) |
| Cache register addresses in `const` | Avoid repeated address math |
| Avoid debug reads in hot loop | MMIO reads stall pipeline |

---

## Exercises

### Exercise 1: Register Map

List all GPIO registers for your chip with offsets and one-line descriptions.

### Exercise 2: PAC Toggle

Implement blink using only PAC calls (no HAL GPIO API).

### Exercise 3: Input Read

Read a button via `GPIO_IN` register; log state via `defmt`.

### Exercise 4: Bit Field

Extract 3-bit field from a configuration register using shift and mask.

### Exercise 5: Race Demonstration

Write RMW toggle in main + ISR; observe glitching. Fix with W1TS/W1TC.

---

## References

- [ESP32-S3 TRM — GPIO Chapter](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [svd2rust documentation](https://docs.rs/svd2rust/)
- [STM32F411 Reference Manual — GPIO](https://www.st.com/resource/en/reference_manual/rm0383-stm32f411-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [The Embedded Rust Book — Registers](https://docs.rust-embedded.org/book/start/registers/index.html)
- [volatile crate](https://docs.rs/volatile/)

---

*Previous: [05-embedded-architecture.md](./05-embedded-architecture.md) | Next: [07-hal.md](./07-hal.md)*
