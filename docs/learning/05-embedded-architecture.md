# Lesson 05: Embedded Architecture — MCU, PAC, HAL, and BSP

Firmware is organized in layers from silicon registers to application logic. Understanding **PAC → HAL → BSP** helps you choose the right abstraction level and write portable code.

**Prerequisites:** [04-cargo.md](./04-cargo.md)  
**Next:** [06-register-programming.md](./06-register-programming.md)  
**See also:** [07-hal.md](./07-hal.md), [08-gpio.md](./08-gpio.md)

---

## Theory

### MCU vs MPU

| Term | Meaning | Examples |
|------|---------|----------|
| **MCU** (Microcontroller Unit) | CPU + RAM + Flash + peripherals on one chip | ESP32-S3, STM32F411, RP2040 |
| **MPU** (Microprocessor Unit) | CPU only; needs external RAM/flash | Application processors in Linux SBCs |

This curriculum focuses on **MCUs** running bare-metal or lightweight RTOS firmware.

### Firmware Layer Stack

```
┌─────────────────────────────────────────┐
│  Application (your business logic)       │
├─────────────────────────────────────────┤
│  Drivers (sensor chips, displays)        │  ← use embedded-hal traits
├─────────────────────────────────────────┤
│  BSP (Board Support Package)             │  ← pin aliases, clock config
├─────────────────────────────────────────┤
│  HAL (Hardware Abstraction Layer)        │  ← esp-hal, stm32f4xx-hal
├─────────────────────────────────────────┤
│  PAC (Peripheral Access Crate)           │  ← auto-generated registers
├─────────────────────────────────────────┤
│  Silicon (hardware registers)            │
└─────────────────────────────────────────┘
```

| Layer | Responsibility | Portable? |
|-------|----------------|-----------|
| **PAC** | Raw register access, one crate per chip family | No |
| **HAL** | Safe wrappers, type-state APIs | Per chip family |
| **BSP** | Maps DevKit pins to human names (`LED`, `Button`) | Per board |
| **Drivers** | Chip drivers (BME280, SSD1306) | Yes (via embedded-hal) |
| **App** | Your logic | Yes (if lower layers abstracted) |

### Interrupts and Exceptions

**Exceptions** are CPU-internal events (hard fault, undefined instruction).  
**Interrupts** are peripheral-generated signals routed through the **NVIC** (Nested Vectored Interrupt Controller) on ARM, or the **Interrupt Matrix** on ESP32.

```
Peripheral (GPIO, Timer, UART)
        │ IRQ line
        ▼
NVIC / Interrupt Matrix  ← priority, enable/mask
        │
        ▼
CPU saves context → ISR (Interrupt Service Routine)
        │
        ▼
Return to main code
```

See [09-interrupts.md](./09-interrupts.md) for ISR patterns.

### Clocks

Peripherals need clock signals. Typical boot sequence:

1. Start from internal RC oscillator (fast but inaccurate)
2. Enable external crystal (**HSE** / **XTAL**) if present
3. Configure **PLL** to multiply to target CPU frequency
4. Set bus prescalers (AHB, APB on ARM; APB on ESP32)

Wrong clock config → UART baud rate wrong, timers drift, USB fails.

```rust
// ESP32-S3 — HAL hides complexity
let clocks = ClockControl::boot_defaults(system.clock_control).freeze();
// `clocks` is consumed by peripherals — ownership ensures init order
```

---

## Hardware Overview

### ESP32-S3 Block Diagram (Simplified)

```
┌──────────┐    ┌─────────┐    ┌─────────────────────────────┐
│ Xtensa   │◄──►│ Cache   │◄──►│ External Flash (code/data)  │
│ Dual Core│    └─────────┘    └─────────────────────────────┘
└────┬─────┘
     │ AHB
     ├── GPIO (45 pins)
     ├── UART, SPI, I2C
     ├── Timers, LEDC (PWM)
     ├── ADC, I2S
     ├── USB OTG
     └── Wi-Fi / BLE (radio)
```

### STM32F411 (Comparison)

Single Cortex-M4F @ 100 MHz, FPU, no radio. Simpler power domains. **ST-Link** on Nucleo boards provides SWD debug.

### RP2040 (Comparison)

Dual Cortex-M0+ @ 125 MHz, unique **PIO** (Programmable I/O) state machines for custom protocols. Excellent for teaching — simpler interrupt model than ESP32.

---

## Wiring Diagram

Not applicable at architecture level. BSP layer documents board-specific wiring — see [08-gpio.md](./08-gpio.md).

Example BSP pin mapping (conceptual):

```
ESP32-S3 DevKitC-1 (common defaults)
─────────────────────────────────────
GPIO2  ──► On-board LED (active high)
GPIO0  ──► Boot button (active low, internal pull-up)
GPIO43 ──► UART0 TX (USB serial)
GPIO44 ──► UART0 RX
3V3    ──► Breadboard power rail
GND    ──► Common ground
```

---

## Memory & Register Explanation

Each peripheral occupies a range of addresses in the **memory map**. The PAC exposes these as typed register blocks:

```rust
// Conceptual PAC usage (generated from SVD)
let gpio = pac::GPIO;
// gpio.out_w1ts is a register at a fixed address
// Writing 1 to bit N sets GPIO N high (ESP32 "write 1 to set" pattern)
```

PAC types are **zero-sized or thin wrappers** — they don't own hardware, they represent addresses. The HAL adds ownership and safety.

---

## HAL Implementation

ESP32-S3 initialization pattern:

```rust
use esp_hal::{
    clock::ClockControl,
    gpio::{Io, Level, Output},
    peripherals::Peripherals,
};

pub fn init_led() -> Output<'static, esp_hal::gpio::GpioPin<'static, 2>> {
    // `Peripherals::take()` — once per boot, returns owned peripheral structs
    let peripherals = Peripherals::take();

    // Split SYSTEM for clock — destructuring moves sub-peripherals out
    let system = peripherals.SYSTEM.split();
    let clocks = ClockControl::boot_defaults(system.clock_control).freeze();

    // `Io` owns GPIO mux — pins borrowed from it become Output/Input
    let io = Io::new(peripherals.GPIO, peripherals.IO_MUX);

    // Type-state: Output pin configured as push-pull output
    Output::new(io.pins.gpio2, Level::Low)
    // Returned `Output` owns the pin configuration until dropped
}
```

STM32 equivalent:

```rust
use stm32f4xx_hal as hal;

let dp = hal::pac::Peripherals::take().unwrap();
let rcc = dp.RCC.constrain();
let clocks = rcc.cfgr.use_hse(8.MHz()).sysclk(84.MHz()).freeze();
let gpioa = dp.GPIOA.split();
let mut led = gpioa.pa5.into_push_pull_output();
```

---

## Bare-Metal Implementation

Without HAL — direct PAC/register access:

```rust
/// Minimal ESP32-S3 GPIO2 output enable + set high.
/// Prefer HAL in production — shown for understanding layers.
fn bare_init_gpio2() {
    const GPIO_ENABLE1: *mut u32 = 0x6000_4024 as *mut u32;
    const GPIO_OUT1_SET: *mut u32 = 0x6000_4014 as *mut u32;
    const IO_MUX_GPIO2: *mut u32 = 0x6000_9090 as *mut u32;

    unsafe {
        // Configure pin function as GPIO
        write_volatile(IO_MUX_GPIO2, 1);
        // Enable output on GPIO2 (bit 2 of enable1 register for pins 32+)
        // Note: GPIO2 is in enable register 0 — address differs; check TRM
        let enable0: *mut u32 = 0x6000_4020 as *mut u32;
        write_volatile(enable0, read_volatile(enable0) | (1 << 2));
        write_volatile(GPIO_OUT1_SET, 0); // GPIO2 uses OUT1 reg on some docs — verify TRM
    }
}

unsafe fn write_volatile(addr: *mut u32, val: u32) {
    core::ptr::write_volatile(addr, val);
}
unsafe fn read_volatile(addr: *const u32) -> u32 {
    core::ptr::read_volatile(addr)
}
```

Always verify register addresses in the **Technical Reference Manual** — the HAL/PAC is authoritative.

---

## Step-by-Step Explanation

### Step 1: Identify Layers in Your Project

Open your template firmware. Find:

- Where `Peripherals::take()` is called (HAL)
- Whether a separate `board` crate exists (BSP)
- Whether any `pac::` paths appear (direct PAC)

### Step 2: Trace GPIO from App to Register

Follow a `led.set_high()` call:

1. **App** calls HAL method
2. **HAL** validates pin mode, writes PAC register
3. **PAC** generates volatile write to MMIO address

### Step 3: Understand Type-State

HAL uses Rust types to enforce init order:

```rust
// STM32 pattern — pin type changes after configuration
let pin = gpioa.pa5;                    // Pin identifier
let output = pin.into_push_pull_output(); // Consumes pin, returns Output
// Cannot call into_push_pull_output twice — compile error
```

### Step 4: Add a BSP Layer

```rust
// board/src/lib.rs
pub mod pins {
    pub type LedPin = esp_hal::gpio::GpioPin<'static, 2>;
    pub const LED: u8 = 2;
    pub const BUTTON: u8 = 0;
}

pub fn init() -> board::Led {
    // Clock + GPIO setup
}
```

### Step 5: Write Portable Driver

```rust
use embedded_hal::digital::OutputPin;

pub fn blink<P: OutputPin>(led: &mut P, times: u32) -> Result<(), P::Error> {
    for _ in 0..times {
        led.set_high()?;
        // delay...
        led.set_low()?;
    }
    Ok(())
}
```

---

## Common Mistakes

| Mistake | Consequence | Fix |
|---------|-------------|-----|
| Skipping clock init | Peripherals hang or behave randomly | Always init clocks first |
| Calling `Peripherals::take()` twice | Panic | Single init function |
| Mixing PAC and HAL on same peripheral | Conflicting config | Use one layer per peripheral |
| Wrong BSP pin for board revision | Silent wrong behavior | Check board silkscreen vs docs |
| Heavy work in ISR | Missed deadlines, jitter | Defer to main loop |
| Ignoring errata sheet | Mystery bugs on specific silicon rev | Read chip errata |

---

## Debugging Tips

1. **Start at HAL layer** — only drop to PAC when HAL lacks feature.
2. **Compare with working example** from chip manufacturer's repo.
3. **Use `defmt` in init** — log each init step to find where boot stops.
4. **Read `Peripheral::ptr()` in debugger** — verify register values match expected.
5. **Check chip feature in Cargo.toml** — wrong chip feature = wrong PAC.

---

## Performance Tips

| Tip | Rationale |
|-----|-----------|
| Init peripherals once at boot | Avoid repeated enable/disable overhead |
| Cache `clocks` and pass references | Prevents re-configuring PLL |
| Use PAC only in hot paths if HAL is too slow | Rare — profile first |
| Enable only used peripheral clocks (STM32) | Saves power |
| ESP32: place Wi-Fi code in separate task | Isolates timing-critical code |

---

## Exercises

### Exercise 1: Layer Diagram

Draw the call stack for `button.read()` from app to register on your target chip.

### Exercise 2: BSP Crate

Create a `board` crate exporting `init_led()` and `init_button()` for your DevKit.

### Exercise 3: Clock Investigation

Log or print configured CPU frequency. Change PLL settings (if supported) and observe timer behavior.

### Exercise 4: PAC Peek

Using PAC or `cargo readmem`, read GPIO direction register after HAL init. Confirm bit matches pin.

### Exercise 5: Portable Blink

Write `blink_generic<P: OutputPin>(pin: &mut P)` in a driver crate. Use from two different board BSPs.

---

## References

- [The Embedded Rust Book — Peripheral Access Crates](https://docs.rust-embedded.org/book/peripherals/index.html)
- [ESP32-S3 Technical Reference Manual](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [STM32F4 HAL Documentation](https://docs.rs/stm32f4xx-hal/)
- [RP2040 Datasheet](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf)
- [embedded-hal project](https://github.com/rust-embedded/embedded-hal)

---

*Previous: [04-cargo.md](./04-cargo.md) | Next: [06-register-programming.md](./06-register-programming.md)*
