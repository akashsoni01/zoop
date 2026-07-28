# Lesson 04: Swift Package Manager (SwiftPM) for Embedded

**Swift Package Manager (SwiftPM)** is Swift's built-in build system and dependency manager — the equivalent of Rust's **Cargo**. For embedded targets, you configure `Package.swift`, define cross-compilation triples, and link board-specific linker scripts.

**Prerequisites:** [02-embedded-swift.md](./02-embedded-swift.md)  
**Next:** [05-embedded-architecture.md](./05-embedded-architecture.md)  
**See also:** [03-memory-layout.md](./03-memory-layout.md), [glossary.md](./glossary.md)

---

## Theory

### SwiftPM Concepts

| SwiftPM | Cargo equivalent | Purpose |
|---------|------------------|---------|
| `Package.swift` | `Cargo.toml` | Manifest — name, targets, dependencies |
| Target | Crate / package target | Unit of compilation |
| Product | Binary or library product | What consumers build |
| Dependency | `[dependencies]` | External packages (Git URL or path) |
| `swift build` | `cargo build` | Compile |
| `swift test` | `cargo test` | Run tests on host |

### Package Layout

```
MyFirmware/
├── Package.swift           # Manifest
├── Package.resolved        # Locked dependency versions
├── Sources/
│   ├── App/
│   │   └── main.swift      # @main entry (or BlinkApp.swift)
│   ├── BoardSupport/
│   │   └── ESP32S3.swift   # BSP
│   └── HAL/
│       └── Protocols.swift # Shared protocols
├── Tests/
│   └── HALTests/
│       └── GPIOTests.swift # Host-only unit tests
└── linker/
    └── esp32s3.ld          # Linker script
```

### Package.swift Structure

```swift
// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "BlinkFirmware",
    platforms: [
        .macOS(.v13)   // Host tests; embedded uses custom triple
    ],
    products: [
        .executable(name: "BlinkFirmware", targets: ["App"]),
        .library(name: "EmbeddedHAL", targets: ["HAL"]),
    ],
    dependencies: [
        // .package(url: "https://github.com/example/ESP32PAC.git", from: "0.1.0"),
    ],
    targets: [
        .executableTarget(
            name: "App",
            dependencies: ["BoardSupport", "HAL"],
            swiftSettings: embeddedSwiftSettings
        ),
        .target(
            name: "BoardSupport",
            dependencies: ["HAL"],
            linkerSettings: [
                .linkedLibrary("c"),
                .unsafeFlags([
                    "-T", "linker/esp32s3.ld",
                ])
            ]
        ),
        .target(
            name: "HAL",
            dependencies: []
        ),
        .testTarget(
            name: "HALTests",
            dependencies: ["HAL"]
        ),
    ]
)

/// Compiler flags for freestanding Embedded Swift
let embeddedSwiftSettings: [SwiftSetting] = [
    .unsafeFlags([
        "-Xfrontend", "-disable-objc-interaction",
        "-Xfrontend", "-disable-clang-spi",
        "-Xfrontend", "-enable-experimental-feature",
        "-Xfrontend", "Embedded",
        "-wmo",                    // Whole module optimization
        "-Osize",                  // Size optimization
    ])
]
```

> **Note:** Exact flags evolve with Swift releases. Always cross-check with swift-embedded-examples for your toolchain version.

### Cross-Compilation

Build for a non-host architecture using `--triple`:

```bash
# ARM Cortex-M4 (STM32)
swift build -c release \
  --triple armv7em-none-eabi \
  --sdk /path/to/arm-none-eabi-sdk

# ESP32-S3 (when toolchain supports)
swift build -c release \
  --triple xtensa-esp32s3-none-elf \
  --sdk /path/to/xtensa-sdk
```

You typically need:

1. **Cross-compiler SDK** (LLVM/GCC bare-metal)
2. **Linker script** for your chip
3. **Startup assembly** (reset handler, vector table)
4. **Swift runtime stubs** (minimal or none)

### Host Tests + Cross-Build Pattern

```
┌─────────────────────────────────────────┐
│  swift test (macOS)                     │
│  Tests/HALTests — mock GPIO, no hardware│
└─────────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────────┐
│  swift build --triple arm-none-eabi     │
│  Sources/App — real MMIO via BSP        │
└─────────────────────────────────────────┘
```

Keep hardware-independent logic in `HAL` target — test on Mac, flash from `App`.

---

## Hardware Overview

SwiftPM itself is host-side tooling. Your **Package.swift** selects which board binary is produced via triple and linker flags.

| Board | Typical Triple | Linker Script |
|-------|----------------|---------------|
| STM32F411 | `armv7em-none-eabihf` | `stm32f411xe.ld` |
| RP2040 | `thumbv6m-none-eabi` | `rp2040.ld` |
| ESP32-S3 | `xtensa-esp32s3-none-elf` | `esp32s3.ld` |

---

## Wiring Diagram

Not applicable — build system lesson. Hardware unchanged.

---

## Memory & Register Explanation

SwiftPM passes linker flags that determine section placement:

```swift
linkerSettings: [
    .unsafeFlags([
        "-T", "linker/esp32s3.ld",
        "-Wl,-Map=firmware.map",   // Generate map file
    ])
]
```

The generated `firmware.map` file lists every symbol and section size — essential for [03-memory-layout.md](./03-memory-layout.md) audits.

Build artifacts land in:

```
.build/<triple>/release/
├── BlinkFirmware          # ELF executable
├── BlinkFirmware.bin      # Raw flash image (via objcopy)
└── ...
```

Convert ELF to flashable binary:

```bash
llvm-objcopy -O binary \
  .build/release/BlinkFirmware \
  firmware.bin
```

---

## HAL-Style Swift Implementation

Separate portable HAL from board BSP using SwiftPM targets:

**Sources/HAL/DigitalPin.swift:**

```swift
public protocol DigitalOutputPin {
    mutating func setHigh() -> Result<Void, GPIOError>
    mutating func setLow() -> Result<Void, GPIOError>
}

public enum GPIOError: Error {
    case invalidPin
    case hardwareFault
}
```

**Sources/BoardSupport/ESP32OutputPin.swift:**

```swift
import HAL

public struct ESP32OutputPin: DigitalOutputPin {
    let pinNumber: UInt8

    public mutating func setHigh() -> Result<Void, GPIOError> {
        gpioSet(pin: pinNumber, level: true)
        return .success(())
    }

    public mutating func setLow() -> Result<Void, GPIOError> {
        gpioSet(pin: pinNumber, level: false)
        return .success(())
    }
}

// Low-level C or MMIO call
func gpioSet(pin: UInt8, level: Bool) { /* ... */ }
```

**Sources/App/main.swift:**

```swift
import HAL
import BoardSupport

@main
struct App {
    static func main() {
        var led = ESP32OutputPin(pinNumber: 2)
        while true {
            _ = led.setHigh()
            delay(ms: 500)
            _ = led.setLow()
            delay(ms: 500)
        }
    }
}
```

---

## Bare-Metal / MMIO Swift Notes

Some projects use a **C startup file** linked alongside Swift:

```swift
// Package.swift — include C startup
.target(
    name: "BoardSupport",
    dependencies: ["HAL"],
    sources: ["Swift/", "C/startup.c", "C/vectors.s"],
    linkerSettings: [ /* ... */ ]
)
```

Swift calls into C for reset handler and syscall stubs:

```swift
@_silgen_name("uart_init")
func uartInit(baud: UInt32)

@_silgen_name("write")
func cWrite(_ fd: Int32, _ buf: UnsafeRawPointer, _ count: Int) -> Int
```

---

## Step-by-Step Explanation

### Step 1: Create Package

```bash
mkdir BlinkFirmware && cd BlinkFirmware
swift package init --type executable
```

### Step 2: Add HAL Target

Edit `Package.swift` — split into `App`, `HAL`, `BoardSupport` targets.

### Step 3: Add Embedded Swift Flags

Apply experimental Embedded feature flags per your toolchain docs.

### Step 4: Add Linker Script

Place `linker/esp32s3.ld` and reference with `-T` in `linkerSettings`.

### Step 5: Build for Host (Tests)

```bash
swift test
```

### Step 6: Cross-Build

```bash
swift build -c release --triple <your-triple>
```

### Step 7: Flash

Use board-specific tool on output binary.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Missing linker script | Link error or boot loop | Add `-T linker.ld` |
| Host flags on embedded target | Wrong code generation | Separate swiftSettings per target |
| `unsafeFlags` abuse | Fragile builds | Pin toolchain version |
| Testing only on device | Slow iteration | Add host unit tests |
| Forgetting `Package.resolved` in CI | Non-reproducible builds | Commit lockfile |
| Wrong product type | Missing @main | Use executableTarget for firmware |

---

## Debugging Tips

1. **`swift package describe`** — inspect resolved graph.
2. **Verbose build** — `swift build -v` shows exact compiler invocations.
3. **Map file** — `-Wl,-Map=` reveals unexpected symbols.
4. **Dry-run host build** — validate Swift syntax before cross-compile.
5. **Pin Swift version** — `.swift-version` file or toolchain file in repo.

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| `-Osize` release | Smaller flash footprint |
| `-wmo` | Whole module optimization across files |
| Split HAL to separate module | Faster incremental host tests |
| Strip debug symbols for release | `strip` or linker `--strip-all` |
| LTO if supported | Link-time optimization |

---

## Exercises

### Exercise 1: Three-Target Package

Create `App`, `HAL`, and `BoardSupport` targets. `App` depends on both; tests depend only on `HAL`.

### Exercise 2: Host Mock Test

Implement `MockOutputPin: DigitalOutputPin` and test a blink state machine on macOS.

### Exercise 3: Map File

Enable linker map output and find the ten largest symbols.

### Exercise 4: Cross-Compile Script

Write a shell script `build-firmware.sh` wrapping triple, SDK path, and objcopy.

### Exercise 5: Compare with Cargo

If you know Rust, document five SwiftPM vs Cargo differences in a table.

---

## References

- [Swift Package Manager Documentation](https://www.swift.org/documentation/package-manager/)
- [PackageDescription API](https://docs.swift.org/package-manager/PackageDescription/index.html)
- [swift-embedded-examples Package.swift files](https://github.com/apple/swift-embedded-examples)
- [Lesson 05 — Architecture](./05-embedded-architecture.md)
- [Embedded Rust Cargo equivalent](../learning/04-cargo.md)

---

*Previous: [03-memory-layout.md](./03-memory-layout.md) | Next: [05-embedded-architecture.md](./05-embedded-architecture.md)*
