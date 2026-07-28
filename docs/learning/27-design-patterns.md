# Lesson 27 — Design Patterns: State Machines, Type-State, BSP, Driver Crates

**Prerequisites:** [05-embedded-architecture.md](./05-embedded-architecture.md), [07-hal.md](./07-hal.md), [22-embassy.md](./22-embassy.md) or [23-rtic.md](./23-rtic.md)

**Board focus:** Patterns portable across [ESP32-S3](./boards/esp32-s3.md), [STM32](./boards/stm32.md), [RP2040](./boards/rp2040.md).

---

## Theory

Production embedded Rust organizes code into **layers** and **patterns** that leverage the type system to eliminate invalid states at compile time.

### Layer cake (review)

```
Application  ── business logic, state machines
Driver crates ── device-specific (BME280, ST7789), generic over embedded-hal
BSP ── Board Support Package: pin aliases, clock init for one PCB
HAL ── peripheral drivers (esp-hal, embassy-stm32)
PAC ── register definitions
Hardware
```

See [05-embedded-architecture.md](./05-embedded-architecture.md).

### Pattern overview

| Pattern | Purpose |
|---------|---------|
| **State machine** | Explicit transitions — no impossible modes |
| **Type-state** | Encode state in type parameters — API enforces order |
| **BSP** | Map `LED = pin 48` once for your board |
| **Driver crate** | Reusable sensor/display logic, no pin knowledge |
| **Newtype** | Wrap raw units (MHz, Celsius) |

---

## Hardware Overview

Patterns are software architecture — hardware is abstracted behind traits. Example system:

```
┌─────────────────────────────────────┐
│ App: ChargerStateMachine            │
├─────────────────────────────────────┤
│ Drivers: Bme280<I2C>, St7789<SPI>   │
├─────────────────────────────────────┤
│ BSP: esp32s3_devkitc pins           │
├─────────────────────────────────────┤
│ HAL: esp-hal                        │
└─────────────────────────────────────┘
```

---

## ASCII Wiring

No new wiring — organize existing [08-gpio.md](./08-gpio.md) / [17-i2c.md](./17-i2c.md) projects into crates:

```
firmware/
  Cargo.toml
  src/
    main.rs      ← thin: init BSP, spawn tasks
    app.rs       ← state machine
  bsp/
    lib.rs       ← pin definitions for ESP32-S3 DevKitC-1
  drivers/
    bme280.rs    ← generic I2C driver
```

---

## Memory & Register Notes

State machines store **current state enum** — typically one byte:

```rust
pub enum ChargerState {
    Idle,
    Precharge,
    ConstantCurrent,
    ConstantVoltage,
    Fault,
}
```

**Type-state** avoids storing enum by consuming `self`:

```rust
pub struct Uninitialized;
pub struct Ready;
pub struct Sensor<S> { inner: ..., _state: PhantomData<S> }
```

Zero runtime cost — compiled away.

Place **BSP statics** in flash/RAM once; drivers hold references `&mut impl I2c`.

---

## State Machine Pattern

```rust
pub enum AppEvent {
    ButtonPress,
    TimerTick,
    SensorReady(f32),
    Error,
}

pub struct App {
    state: AppState,
}

enum AppState {
    Sleeping,
    Sampling { deadline: u32 },
    Transmitting,
    Fault { code: u8 },
}

impl App {
    pub fn handle(&mut self, event: AppEvent) -> Action {
        use AppState::*;
        match (&self.state, event) {
            (Sleeping, ButtonPress) => {
                self.state = Sampling { deadline: millis() + 1000 };
                Action::StartSensor
            }
            (Sampling { .. }, SensorReady(v)) if v > 50.0 => {
                self.state = Fault { code: 1 };
                Action::Alarm
            }
            (Sampling { .. }, SensorReady(_)) => {
                self.state = Transmitting;
                Action::SendPacket
            }
            (Transmitting, TimerTick) => {
                self.state = Sleeping;
                Action::EnterSleep
            }
            (_, Error) => {
                self.state = Fault { code: 255 };
                Action::Alarm
            }
            _ => Action::None,
        }
    }
}

pub enum Action {
    None,
    StartSensor,
    SendPacket,
    EnterSleep,
    Alarm,
}
```

Test transitions on host — [26-testing.md](./26-testing.md).

---

## Type-State Pattern

```rust
use core::marker::PhantomData;

pub struct Uninit;
pub struct Calibrated;
pub struct Running;

pub struct Imu<S = Uninit> {
    i2c_addr: u8,
    _state: PhantomData<S>,
}

impl Imu<Uninit> {
    pub fn new(addr: u8) -> Self {
        Self { i2c_addr: addr, _state: PhantomData }
    }

    pub fn calibrate<I2C: embedded_hal::i2c::I2c>(
        self,
        i2c: &mut I2C,
    ) -> Result<Imu<Calibrated>, I2C::Error> {
        // write calibration regs...
        Ok(Imu { i2c_addr: self.i2c_addr, _state: PhantomData })
    }
}

impl Imu<Calibrated> {
    pub fn start(self) -> Imu<Running> {
        Imu { i2c_addr: self.i2c_addr, _state: PhantomData }
    }
}

impl Imu<Running> {
    pub fn read_sample<I2C: embedded_hal::i2c::I2c>(
        &mut self,
        i2c: &mut I2C,
    ) -> Result<[i16; 3], I2C::Error> {
        // Only Running can read — compile error if skipped calibrate()
        Ok([0, 0, 0])
    }
}
```

Usage:

```rust
let imu = Imu::new(0x68);
let imu = imu.calibrate(&mut i2c).unwrap();
let imu = imu.start();
let sample = imu.read_sample(&mut i2c).unwrap();
// Imu::new(0x68).read_sample(...) // WON'T COMPILE
```

---

## BSP Pattern

`bsp/esp32s3_devkitc.rs`:

```rust
use esp_hal::gpio::{GpioPin, Output};

pub struct Board {
    pub led: Output<'static>,
    pub i2c_sda: GpioPin<8>,
    pub i2c_scl: GpioPin<9>,
}

impl Board {
    pub fn init(peripherals: esp_hal::peripherals::Peripherals) -> Self {
        let io = esp_hal::gpio::Io::new(peripherals.GPIO, peripherals.IO_MUX);
        Self {
            led: Output::new(io.pins.gpio48, esp_hal::gpio::Level::Low),
            i2c_sda: io.pins.gpio8,
            i2c_scl: io.pins.gpio9,
        }
    }
}
```

`main.rs` stays thin:

```rust
let bsp = Board::init(peripherals);
let i2c = I2c::new(/* bsp.i2c_sda, bsp.i2c_scl */);
let mut sensor = Bme280::new(&mut i2c, 0x76);
```

Swap board → change BSP only — drivers unchanged.

---

## Driver Crate Pattern

```rust
// drivers/bme280/src/lib.rs
pub struct Bme280<I2C> {
    i2c: I2C,
    addr: u8,
}

impl<I2C, E> Bme280<I2C>
where
    I2C: embedded_hal::i2c::I2c<Error = E>,
{
    pub fn new(i2c: I2C, addr: u8) -> Self {
        Self { i2c, addr }
    }

    pub fn init(&mut self) -> Result<(), E> {
        self.write_reg(0xF4, 0x93) // ctrl_meas — example
    }

    fn write_reg(&mut self, reg: u8, val: u8) -> Result<(), E> {
        self.i2c.write(self.addr, &[reg, val])
    }

    pub fn read_temperature(&mut self) -> Result<i32, E> {
        let mut buf = [0u8; 3];
        self.i2c.write_read(self.addr, &[0xFA], &mut buf)?;
        // compensate per datasheet...
        Ok(2500) // 25.00 °C × 100
    }
}
```

Publish to crates.io or use as workspace member.

---

## Async State Machine (Embassy)

```rust
#[embassy_executor::task]
async fn app_task(mut ctx: AppContext) {
    let mut state = State::Idle;
    loop {
        match state {
            State::Idle => {
                ctx.button_pressed.wait().await;
                state = State::Running;
            }
            State::Running => {
                Timer::after_secs(1).await;
                do_work().await;
                state = State::Idle;
            }
        }
    }
}
```

Combine with `embassy-sync::Signal` for events — [22-embassy.md](./22-embassy.md).

---

## Step-by-Step

1. **Identify states** in your app — draw transition diagram on paper.
2. Extract **sensor/display** code to generic driver crate.
3. Create **BSP module** for your dev kit pin map.
4. Refactor `main` to **init + spawn** only.
5. Add **type-state** to one driver init path.
6. Write **host tests** for state transitions — [26-testing.md](./26-testing.md).
7. Document public API with **`///` rustdoc**.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| God `main.rs` | Untestable spaghetti | Layer crates |
| Driver imports `esp_hal` | Not portable | Use embedded-hal |
| State enum + invalid combos | Impossible states at runtime | Type-state or separate enums per mode |
| BSP in driver | Wrong pins on new board | BSP only in board crate |
| Over-engineered type-state | Unreadable generics | Balance — enums often enough |
| Shared mutable static | UB | RTIC/Mutex/channel |

---

## Debugging Tips

- **`defmt` state transitions:** `info!("state {:?} -> {:?}", old, new)`.
- Compile errors from type-state **are documentation** — read message carefully.
- Diagram state machine; verify every edge has handler.
- `cargo doc --open` for driver crate API review.

---

## Performance Tips

- Enums compile to efficient jumps — no OOP vtable unless using trait objects.
- **`impl Trait`** in drivers avoids monomorphization bloat vs generics in app only.
- **`#[inline]`** sparingly on hot one-liner accessors.
- Place **const config** in flash (`const CONFIG: ...`).
- Avoid **`Box<dyn Error>`** on `no_std` — use `enum Error`.

---

## Exercises

1. **Draw + implement** traffic light FSM (Red/Yellow/Green) with GPIO.
2. **Type-state UART:** `Unconfigured` → `Configured<Baud115200>` before `write`.
3. **Split crate:** Move BME280 to `drivers/`; BSP + main in firmware; host test driver.
4. **RTIC integration:** App FSM in software task; ISR sends events via queue — [23-rtic.md](./23-rtic.md).
5. **Publish:** `cargo publish --dry-run` driver crate checklist.

---

## References

- [Embedded Rust Book — Design Patterns](https://docs.rust-embedded.org/book/patterns/index.html)
- [Rust Embedded patterns repo](https://rust-embedded.github.io/patterns/)
- [Typestate pattern in Rust](https://cliffle.com/blog/rust-typestate/)
- [05-embedded-architecture.md](./05-embedded-architecture.md)
- [26-testing.md](./26-testing.md)

---

*Previous: [26-testing.md](./26-testing.md) · [README.md](./README.md)*
