# Lesson 26 — Testing: Host Tests, HIL, Mocking embedded-hal, CI

**Prerequisites:** [07-hal.md](./07-hal.md), [04-cargo.md](./04-cargo.md), [25-debugging.md](./25-debugging.md)

**Board focus:** Portable patterns tested on host; HIL (Hardware-In-the-Loop) optional with [ESP32-S3](./boards/esp32-s3.md).

---

## Theory

Embedded firmware benefits from **testing pyramid**:

```
        ┌─────────────┐
        │  HIL / E2E  │  Real board, slow, few
        ├─────────────┤
        │ Integration │  Host + mock HAL
        ├─────────────┤
        │  Unit tests │  Pure logic, fast, many
        └─────────────┘
```

| Test type | Runs on | Speed | Hardware |
|-----------|---------|-------|----------|
| **Host unit** | `std` + `cargo test` | ms | None |
| **Host integration** | Mock `embedded-hal` | ms | None |
| **HIL** | Target MCU | seconds | Board + probe |
| **CI** | GitHub Actions / custom | automated | Often host-only; HIL farm optional |

Rust advantage: **same driver code** compiles for host (with mocks) and target (with real HAL).

---

## Hardware Overview

HIL setup example:

```
CI Runner PC ── USB ──► ESP32-S3 DevKit
                           │
                     Test firmware listens on UART
                     asserts GPIO patterns probe reads back
```

**Knurling `defmt-test`** runs tests on-device over RTT — [25-debugging.md](./25-debugging.md).

For most projects, **host tests + periodic manual HIL** suffice.

---

## ASCII Wiring (HIL optional)

```
         ┌──────────────┐
USB ────►│ ESP32-S3     │
         │  GPIO48 ◄──┐ │  Loopback: firmware drives LED,
         │  GPIO47 ───┘ │  test jig reads via another GPIO
         └──────────────┘
```

Automated HIL often uses **USB serial protocol**: host sends `TEST RUN`, device responds `PASS/FAIL`.

---

## Memory & Register Notes

Host tests use normal heap — unlimited `Vec` for convenience.

On-target tests must use **`no_std` test harness** or external runner:

```toml
# Cargo.toml — on-target test profile
[[test]]
name = "integration"
harness = false  # custom entry via defmt-test
```

Keep test code in flash — watch size with `cargo size`.

---

## Host Unit Test — Pure logic

Extract protocol parsing from hardware:

```rust
// src/lib.rs — works on host and target
pub fn parse_command(line: &str) -> Option<Command> {
    match line.trim() {
        "led on" => Some(Command::LedOn),
        "led off" => Some(Command::LedOff),
        _ => None,
    }
}

pub enum Command {
    LedOn,
    LedOff,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_led_on() {
        assert_eq!(parse_command("led on"), Some(Command::LedOn));
    }

    #[test]
    fn rejects_unknown() {
        assert_eq!(parse_command("jump"), None);
    }
}
```

Run:

```bash
cargo test --lib
```

---

## Mocking embedded-hal on Host

Use **`embedded-hal-mock`** or custom fakes implementing traits:

```rust
// tests/mock_display.rs — runs on host (#![test])

use embedded_hal::digital::OutputPin;
use embedded_hal::spi::SpiBus;

struct MockPin {
    level: bool,
}

impl OutputPin for MockPin {
    type Error = core::convert::Infallible;
    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.level = false;
        Ok(())
    }
    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.level = true;
        Ok(())
    }
}

struct MockSpi {
    pub written: Vec<u8>,
}

impl SpiBus for MockSpi {
    type Error = core::convert::Infallible;
    fn read(&mut self, _: &mut [u8]) -> Result<(), Self::Error> { Ok(()) }
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.written.extend_from_slice(data);
        Ok(())
    }
    fn flush(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.write(write)?;
        Ok(())
    }
    fn transfer_in_place(&mut self, data: &mut [u8]) -> Result<(), Self::Error> {
        self.write(data)?;
        Ok(())
    }
}

#[test]
fn display_init_sends_reset_command() {
    let spi = MockSpi { written: Vec::new() };
    let dc = MockPin { level: true };
    let rst = MockPin { level: true };
    let cs = MockPin { level: true };

    // let mut disp = St7789::new(spi, dc, rst, cs);
    // disp.init().unwrap();
    // assert!(spi.written.contains(&0x01)); // SWRESET
}
```

Driver under test uses **`embedded-hal` traits** — no chip-specific types — [27-design-patterns.md](./27-design-patterns.md).

---

## `mockall` for trait objects (host)

```rust
#[cfg(test)]
use mockall::mock;

#[cfg(test)]
mock! {
    pub I2cBus {}
    impl embedded_hal::i2c::I2c for I2cBus {
        type Error = std::io::Error;
        fn read(&mut self, address: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
            ...
        }
        fn write(&mut self, address: u8, bytes: &[u8]) -> Result<(), Self::Error> {
            ...
        }
        fn write_read(&mut self, address: u8, prefix: &[u8], buffer: &mut [u8])
            -> Result<(), Self::Error> {
            ...
        }
        fn transaction(&mut self, address: u8, operations: &mut [embedded_hal::i2c::Operation<'_>])
            -> Result<(), Self::Error> {
            ...
        }
    }
}
```

---

## On-Target Tests (defmt-test)

```rust
// tests/on_target.rs
#![no_std]
#![no_main]

use defmt_test::defmt_test;

defmt_test::tests! {
    #[test]
    fn gpio_toggle() {
        // configure pin, toggle, read back
        assert!(true);
    }
}
```

Run via `probe-rs run` script in CI nightly job.

---

## CI Configuration (GitHub Actions)

```yaml
name: Embedded CI

on: [push, pull_request]

jobs:
  host-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --lib
      - run: cargo clippy -- -D warnings

  build-firmware:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: esp-rs/xtensa-toolchain@v1
        with:
          default: true
          buildtargets: esp32s3
      - run: cargo build --release --target xtensa-esp32s3-none-elf
      - run: cargo size --release --target xtensa-esp32s3-none-elf -- -A
```

Split **fast host tests** (every PR) from **slow HIL** (main branch/nightly).

---

## Step-by-Step

1. Move business logic to **`lib.rs`** testable on host.
2. Add **`#[cfg(test)]` module** with unit tests.
3. Introduce **`embedded-hal` traits** in driver boundaries.
4. Write **mock HAL integration tests** in `tests/`.
5. Add **CI workflow** — host test + cross-compile firmware.
6. Optional: **defmt-test** on board before releases.
7. Track **flash/RAM budget** in CI artifacts.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Tests only on target | Slow dev cycle | Host tests for logic |
| Driver tied to `esp_hal::Uart` | Won't compile on host | Generic `embedded-io` |
| Flaky HIL | USB timing | Retry + timeouts |
| CI without cross toolchain | Build fails | Install target/toolchain |
| `unwrap` in tests on host | Hides errors | `assert` / `Result` |
| No `clippy` in CI | Debt accumulates | `-D warnings` |

---

## Debugging Tips

- **`cargo test -- --nocapture`** on host for println debug.
- **`RUST_BACKTRACE=1`** for panics in host tests.
- HIL failures: capture **serial log** artifact in CI.
- Compare **mock call sequences** when hardware misbehaves.
- Use **`cargo llvm-cov`** for host coverage gaps.

---

## Performance Tips

- Keep host tests **fast** (< 30 s total) for tight feedback.
- Parallelize CI jobs: host vs each target triple.
- Cache **cargo registry** in CI.
- HIL: run **smoke subset** every commit, full suite nightly.
- Avoid sleeping in tests — inject time via trait — [27-design-patterns.md](./27-design-patterns.md).

---

## Exercises

1. **Extract parser** from UART shell; 10 host tests covering edge cases.
2. **Mock SPI:** Verify display init command sequence without hardware.
3. **CI badge:** Add workflow; README shows pass/fail.
4. **Size gate:** Fail CI if flash > 90% partition.
5. **defmt-test:** One on-target GPIO test with Knurling template.

---

## References

- [Knurling test book](https://knurling.org/book/testing.html)
- [embedded-hal-mock](https://docs.rs/embedded-hal-mock/)
- [defmt-test](https://docs.rs/defmt-test/)
- [cargo test book](https://doc.rust-lang.org/cargo/guide/tests.html)
- [27-design-patterns.md](./27-design-patterns.md)

---

*Previous: [25-debugging.md](./25-debugging.md) · Next: [27-design-patterns.md](./27-design-patterns.md)*
