# Lesson 27 — Design Patterns: State Machines, Type-State, BSP, Driver Crates

**Prerequisites:** [05-embedded-architecture.md](./05-embedded-architecture.md), [07-hal.md](./07-hal.md), [22-swift-concurrency.md](./22-swift-concurrency.md) or [23-realtime-patterns.md](./23-realtime-patterns.md)

**Board focus:** Patterns portable across [ESP32-S3](./boards/esp32-s3.md), [STM32](./boards/stm32.md), [RP2040](./boards/rp2040.md).

---

## Theory

Production embedded Swift organizes code into **layers** and **patterns** that eliminate invalid states at compile time where possible.

### Layer cake

```
Application  ── business logic, state machines
Driver crates ── BME280, ST7789 — generic over protocols
BSP ── Board Support Package: pin aliases, clock init
HAL ── ESP-IDF bindings, swift-mmio, platform init
Hardware
```

### Pattern overview

| Pattern | Purpose |
|---------|---------|
| **State machine** | Explicit transitions — no impossible modes |
| **Type-state** | Encode state in generic parameters |
| **BSP** | Map `ledPin = 48` once for your board |
| **Driver crate** | Reusable sensor logic, no pin knowledge |
| **Newtype** | Wrap raw units (MHz, Celsius) |

---

## Hardware Overview

Patterns are software — hardware abstracted behind protocols:

```
┌─────────────────────────────────────┐
│ App: ChargerStateMachine            │
├─────────────────────────────────────┤
│ Drivers: BME280<I2C>, ST7789<SPI>   │
├─────────────────────────────────────┤
│ BSP: ESP32S3DevKitC pins            │
├─────────────────────────────────────┤
│ HAL: ESP-IDF / swift-mmio           │
└─────────────────────────────────────┘
```

---

## ASCII Wiring

No new wiring — organize existing projects:

```
firmware/
  Package.swift
  Sources/
    App/main.swift       ← thin: init BSP, run app
    App/ChargerApp.swift ← state machine
  Sources/BSP/
    ESP32S3DevKitC.swift ← pin definitions
  Sources/Drivers/
    BME280.swift         ← generic I2C driver
```

---

## Memory & Register Notes

State machines store **current state enum** — typically one byte:

```swift
enum ChargerState: UInt8 {
    case idle
    case preconditioning
    case constantCurrent
    case constantVoltage
    case complete
    case fault
}
```

Type-state uses **phantom types** — zero runtime cost:

```swift
struct Uninitialized { }
struct Ready { }
struct Running { }

struct Motor<State> {
    // State-specific methods only available on correct type
}

extension Motor where State == Uninitialized {
    func configure() -> Motor<Ready> { Motor<Ready>() }
}

extension Motor where State == Ready {
    func start() -> Motor<Running> { Motor<Running>() }
}
```

---

## BSP Example — ESP32-S3 DevKitC-1

```swift
enum ESP32S3DevKitC {
    static let ledPin: UInt8 = 48
    static let buttonPin: UInt8 = 0
    static let uartTxPin: UInt8 = 43
    static let uartRxPin: UInt8 = 44
    static let i2cSdaPin: UInt8 = 8
    static let i2cSclPin: UInt8 = 9
    static let spiMosiPin: UInt8 = 11
    static let spiSclkPin: UInt8 = 12
    static let spiCsPin: UInt8 = 10
}
```

Swap BSP file for STM32 Nucleo — drivers unchanged.

---

## Driver Crate — Generic BME280

```swift
protocol I2CBusProtocol {
    mutating func readRegister(address: UInt8, reg: UInt8) throws -> UInt8
    mutating func readRegisters(address: UInt8, reg: UInt8, count: Int) throws -> [UInt8]
}

struct BME280<Bus: I2CBusProtocol> {
    var bus: Bus
    let address: UInt8

    mutating func chipID() throws -> UInt8 {
        try bus.readRegister(address: address, reg: 0xD0)
    }

    mutating func readTemperatureC() throws -> Float {
        let raw = try bus.readRegisters(address: address, reg: 0xFA, count: 3)
        let adc = (Int32(raw[0]) << 12) | (Int32(raw[1]) << 4) | (Int32(raw[2]) >> 4)
        return compensate(adc: adc)
    }

    private func compensate(adc: Int32) -> Float {
        // Datasheet formulas — pure, testable on host
        _ = adc
        return 25.0
    }
}
```

---

## State Machine — Smart Switch

```swift
enum SwitchEvent {
    case buttonPress
    case buttonRelease
    case timerExpired
    case faultDetected
}

struct SmartSwitchStateMachine {
    private(set) var state: ChargerState = .idle
    private var holdTimer: UInt32 = 0

    mutating func handle(_ event: SwitchEvent, nowMs: UInt32) -> Bool {
        switch (state, event) {
        case (.idle, .buttonPress):
            state = .constantCurrent
            holdTimer = nowMs + 3000
            return true // relay ON
        case (.constantCurrent, .timerExpired) where nowMs >= holdTimer:
            state = .complete
            return false // relay OFF
        case (_, .faultDetected):
            state = .fault
            return false
        default:
            return false
        }
    }
}
```

Test transitions on host with Swift Testing — [26-testing.md](./26-testing.md).

---

## Application Entry — Thin main

```swift
import BSP
import Drivers

@main
struct FirmwareApp {
    static func main() {
        var i2c = ESPIDFI2CBus(sda: ESP32S3DevKitC.i2cSdaPin,
                               scl: ESP32S3DevKitC.i2cSclPin)
        var sensor = BME280(bus: i2c, address: 0x76)

        var sm = SmartSwitchStateMachine()
        let button = GPIO(ESP32S3DevKitC.buttonPin, mode: .input, pull: .up)
        let relay = GPIO(ESP32S3DevKitC.ledPin, mode: .output) // stand-in for relay

        while true {
            if !button.read() {
                let on = sm.handle(.buttonPress, nowMs: millis())
                relay.set(high: on)
            }
            delay(ms: 10)
        }
    }
}
```

---

## Host Swift — Same Driver in iOS Companion

```swift
import Foundation

// Shared package: Drivers/BME280.swift compiled for iOS
class SensorViewModel: ObservableObject {
    @Published var temperature: Float = 0
    private var bleCentral: BleCentralManager

    func refresh() async {
        // Fetch from BLE NUS — MCU runs same BME280 driver
        if let data = await bleCentral.readTemperature() {
            temperature = Float(data[0])
        }
    }
}
```

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Pins hardcoded in drivers | Won't port boards | BSP layer |
| God struct `main` | Untestable | Split app/state/drivers |
| Missing invalid transitions | Impossible states at runtime | State machine enum |
| Protocol with 50 methods | Mock pain | Split protocols by capability |
| Duplicate logic host/device | Drift | Shared Swift package |
| Type-state everywhere | Over-engineering | Use where API order matters |

---

## Debugging Tips

- Log **state transitions** with `(from, event, to)` tuples.
- Host unit test every transition table row.
- Static assert pin conflicts at BSP compile time where possible.

---

## Performance Tips

- **`@inlinable`** small hot-path helpers in drivers.
- State enum dispatches compile to jump tables — fast.
- Avoid heap in state machine — value types only.

---

## Exercises

1. **BSP swap:** Run BME280 demo on ESP32-S3 and STM32 BSP files.
2. **Type-state UART:** `Uart<Unconfigured>` → `Configured` → `Active` API.
3. **State machine tests:** Cover all transitions + invalid event ignored.
4. **Package structure:** Extract Drivers to Swift package; test on macOS.
5. **iOS companion:** Display values from MCU using shared decode types.

---

## References

- [05-embedded-architecture.md](./05-embedded-architecture.md)
- [26-testing.md](./26-testing.md)
- [Swift API Design Guidelines](https://swift.org/documentation/api-design-guidelines/)
- [boards/esp32-s3.md](./boards/esp32-s3.md)

---

*Previous: [26-testing.md](./26-testing.md)*
