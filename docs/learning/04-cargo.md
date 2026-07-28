# Lesson 04: Cargo for Embedded Projects

**Cargo** is Rust's build system and package manager. Embedded projects extend standard Cargo with cross-compilation targets, feature flags, linker configuration, and custom runners for flashing.

**Prerequisites:** [02-no-std.md](./02-no-std.md), [03-memory-layout.md](./03-memory-layout.md)  
**Next:** [05-embedded-architecture.md](./05-embedded-architecture.md)  
**See also:** [07-hal.md](./07-hal.md), [README.md](./README.md)

---

## Theory

### Cargo Workspace Concepts

| Concept | Embedded Usage |
|---------|----------------|
| **Package** | One firmware binary or one driver library |
| **Target triple** | Architecture-OS-ABI (e.g., `thumbv7em-none-eabihf`) |
| **Features** | Enable chip-specific HAL code at compile time |
| **Profiles** | `dev`, `release`, custom `release-opt-size` |
| **build.rs** | Generate linker scripts, configure flags |
| **config.toml** | Default target, runner, rustflags |

### Why Not Build on Host?

Your laptop is `x86_64-unknown-linux-gnu` (or similar). An ESP32-S3 is `xtensa-esp32s3-none-elf`. The compiler must **cross-compile** — produce machine code for a different architecture.

---

## Hardware Overview

Each board maps to a target triple:

| Board | Target Triple | Toolchain |
|-------|---------------|-----------|
| ESP32-S3 | `xtensa-esp32s3-none-elf` | espup (custom rustc fork) |
| STM32F411 | `thumbv7em-none-eabihf` | rustup std library |
| RP2040 | `thumbv6m-none-eabi` | rustup std library |

The triple encodes:

- **Architecture:** `xtensa`, `thumbv7em`, `thumbv6m`
- **Vendor:** often omitted or `none`
- **OS:** `none` (bare metal)
- **ABI:** `eabi`, `eabihf` (hard float)

---

## Wiring Diagram

Not applicable. Cargo runs on your host PC; the output binary is flashed to the MCU via USB.

---

## Memory & Register Explanation

Cargo controls **where** code lands via linker flags in `.cargo/config.toml`:

```toml
[target.xtensa-esp32s3-none-elf]
runner = "espflash flash --monitor"
rustflags = [
    "-C", "link-arg=-Tlinkall.x",  # Linker script name (provided by esp-hal)
]
```

The linker script (not in Cargo.toml directly) defines flash/RAM regions — see [03-memory-layout.md](./03-memory-layout.md).

---

## HAL Implementation

Typical ESP32-S3 `Cargo.toml`:

```toml
[package]
name = "sensor-node"
version = "0.1.0"
edition = "2021"

[dependencies]
esp-hal = { version = "0.21", features = ["esp32s3", "defmt"] }
esp-backtrace = { version = "0.14", features = ["esp32s3", "defmt"] }
defmt = "0.3"
defmt-rtt = "0.4"
embedded-hal = "1.0"
panic-halt = "0.2"

[features]
default = ["defmt"]
# Feature flags for optional hardware
bme280 = []

[profile.dev]
opt-level = 1   # Faster dev builds, some optimization

[profile.release]
codegen-units = 1
debug = 2       # Keep debug symbols for probe/debug
lto = true
opt-level = "s" # Optimize for size
panic = "abort"

[profile.release-opt-size]
inherits = "release"
opt-level = "z"   # Aggressive size optimization
```

`.cargo/config.toml`:

```toml
[build]
target = "xtensa-esp32s3-none-elf"

[target.xtensa-esp32s3-none-elf]
runner = "espflash flash --monitor"

[env]
DEFMT_LOG = "info"
```

Build and run:

```bash
cargo run --release   # Builds, flashes, opens monitor via runner
```

### STM32 Example (Comparison)

```toml
[dependencies]
stm32f4xx-hal = { version = "0.22", features = ["stm32f411", "defmt"] }
cortex-m-rt = "0.7"
cortex-m = "0.7"
```

```toml
# .cargo/config.toml
[build]
target = "thumbv7em-none-eabihf"

[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32F411RETx"
rustflags = ["-C", "link-arg=-Tlink.x"]
```

---

## Bare-Metal Implementation

### Minimal Cargo.toml (Library Driver)

A portable driver crate — no binary target:

```toml
[package]
name = "my-sensor-driver"
version = "0.1.0"
edition = "2021"

[dependencies]
embedded-hal = "1.0"
embedded-hal-async = { version = "1.0", optional = true }
defmt = { version = "0.3", optional = true }

[features]
default = []
async = ["embedded-hal-async"]
defmt = ["dep:defmt"]

[lints.rust]
unsafe_code = "warn"
```

Consumer firmware enables features:

```toml
my-sensor-driver = { path = "../my-sensor-driver", features = ["defmt"] }
```

### build.rs Example

Generate config or validate environment at compile time:

```rust
// build.rs
use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("cargo:rustc-cfg=target_{}", target.replace('-', "_"));

    if target.contains("esp32s3") {
        println!("cargo:rerun-if-changed=memory.ld");
    }
}
```

Use in firmware:

```rust
#[cfg(target_arch = "xtensa")]
fn platform_delay() { /* ... */ }

#[cfg(target_arch = "arm")]
fn platform_delay() { /* ... */ }
```

---

## Step-by-Step Explanation

### Step 1: Install Target

```bash
# ARM
rustup target add thumbv7em-none-eabihf

# ESP — via espup, not rustup
espup install && source ~/export-esp.sh
```

### Step 2: Configure Default Target

Create `.cargo/config.toml` so `cargo build` cross-compiles automatically.

### Step 3: Set Runner for `cargo run`

The **runner** flashes and optionally starts a debug session after build.

### Step 4: Feature Flags for Hardware Variants

```rust
// In your crate — chip-specific module selection
#[cfg(feature = "esp32s3")]
mod platform {
    pub use esp_hal::delay::Delay as PlatformDelay;
}

#[cfg(feature = "rp2040")]
mod platform {
    pub use rp2040_hal::Timer as PlatformDelay;
}
```

Enable in firmware `Cargo.toml`:

```toml
my-board-support = { path = "board", features = ["esp32s3"] }
```

### Step 5: Custom Profiles

```toml
[profile.dev]
opt-level = 1   # Dev builds big but fast to compile

[profile.bench]
inherits = "release"
debug = true
```

### Step 6: Workspace Layout (Multi-Crate)

```
firmware/
├── Cargo.toml          # Workspace root
├── app/
│   ├── Cargo.toml      # Binary firmware
│   └── src/main.rs
├── board/
│   ├── Cargo.toml      # BSP — pin definitions
│   └── src/lib.rs
└── drivers/
    └── bme280/
        ├── Cargo.toml  # Portable sensor driver
        └── src/lib.rs
```

Root `Cargo.toml`:

```toml
[workspace]
members = ["app", "board", "drivers/bme280"]
resolver = "2"
```

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Wrong default target | Host binary or link errors | Set `.cargo/config.toml` |
| Missing runner | "Can't run binary" | Add `espflash` or `probe-rs` runner |
| Feature not enabled | Missing peripheral code | Add chip feature to HAL dep |
| `std` pulled via dependency | Link failure | `default-features = false` |
| Debug profile too slow on device | Timing bugs only in dev | Test with `--release` |
| Forgetting `panic = "abort"` | Huge binary, link errors | Set in release profile |

---

## Debugging Tips

1. **`cargo tree -e features`** — see which features are enabled.
2. **`cargo build -vv`** — verbose: see exact rustc and linker commands.
3. **`RUSTFLAGS="-C link-dead-code"`** — temporarily keep all symbols for analysis.
4. **`cargo clean`** after target/toolchain changes.
5. **Verify active target:** `rustc --print cfg --target thumbv7em-none-eabihf`.

---

## Performance Tips

| Setting | Effect |
|---------|--------|
| `lto = true` | Smaller, faster code; slower builds |
| `codegen-units = 1` | Better optimization |
| `opt-level = "z"` | Smallest flash |
| `opt-level = 3` | Fastest execution; larger flash |
| `debug = 2` in release | Keep symbols for profiling |

---

## Exercises

### Exercise 1: Dual-Target Config

Configure a project to build for both `thumbv6m-none-eabi` and `thumbv7em-none-eabihf` using `--target` flag. Document differences in output size.

### Exercise 2: Custom Profile

Create `release-fast` profile with `opt-level = 3`. Compare code size vs default release.

### Exercise 3: Feature Flag Driver

Write a driver crate with optional `defmt` feature. Build with and without; compare size.

### Exercise 4: build.rs CFG

Use `build.rs` to embed git commit hash as `env!("GIT_HASH")` in firmware.

### Exercise 5: Workspace

Split a blink project into `app` + `board` crates in a workspace. Board crate exports pin constants.

---

## References

- [Cargo Book — Configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- [Cargo Book — Profiles](https://doc.rust-lang.org/cargo/reference/profiles.html)
- [The Embedded Rust Book — Cargo Config](https://docs.rust-embedded.org/book/intro/install.html)
- [esp-template repository](https://github.com/esp-rs/esp-template)
- [probe-rs cargo-embed config](https://probe.rs/docs/tools/cargo-embed/)

---

*Previous: [03-memory-layout.md](./03-memory-layout.md) | Next: [05-embedded-architecture.md](./05-embedded-architecture.md)*
