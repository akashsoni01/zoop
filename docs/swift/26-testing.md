# Lesson 26 — Testing: Host Tests, HIL, Mocking, CI

**Prerequisites:** [07-hal.md](./07-hal.md), [04-cargo.md](./04-cargo.md), [25-debugging.md](./25-debugging.md)

**Board focus:** Portable patterns on host; HIL optional with [ESP32-S3](./boards/esp32-s3.md).

**Maturity note:** Embedded Swift testing **leans on host-side Swift Testing / XCTest** with protocol-based mocks — the same strategy as Rust `embedded-hal` mocks. On-target tests are **manual or custom UART protocols** until a Swift `defmt-test` equivalent exists.

---

## Theory

Embedded firmware benefits from a **testing pyramid**:

```
        ┌─────────────┐
        │  HIL / E2E  │  Real board, slow, few
        ├─────────────┤
        │ Integration │  Host + mock HAL
        ├─────────────┤
        │  Unit tests │  Pure logic, fast, many
        └─────────────┘
```

| Test type | Runs on | Hardware |
|-----------|---------|----------|
| **Host unit** | macOS `swift test` | None |
| **Host integration** | Mock protocols | None |
| **HIL** | Target MCU | Board + USB |
| **CI** | GitHub Actions | Usually host-only |

Swift advantage: **same driver logic** compiles for host (with mocks) and target (with real HAL) via protocols — [27-design-patterns.md](./27-design-patterns.md).

---

## Hardware Overview

HIL setup:

```
CI Runner / Mac ── USB ──► ESP32-S3 DevKit
                              │
                        Test firmware on UART
                        GPIO loopback for automated checks
```

---

## ASCII Wiring (HIL optional)

```
         ┌──────────────┐
USB ────►│ ESP32-S3     │
         │  GPIO48 ◄──┐ │  Loopback: firmware drives LED,
         │  GPIO47 ───┘ │  test reads via companion GPIO
         └──────────────┘
```

Host sends `TEST RUN` over serial; device responds `PASS` / `FAIL`.

---

## Memory & Register Notes

Host tests use normal heap. On-target tests must avoid large allocations:

```swift
#if os(macOS)
import Testing

@Test func compensationFormula() {
    let result = BME280.compensateTemperature(adc: 512_000, cal: mockCal)
    #expect(result > 20.0 && result < 30.0)
}
#endif
```

---

## Host Unit Test — Pure Logic

```swift
import Testing

struct PIDController {
    var integral: Float = 0
    let kp: Float, ki: Float, kd: Float

    mutating func update(error: Float, dt: Float) -> Float {
        integral += error * dt
        return kp * error + ki * integral
    }
}

@Test func pidProportionalOnly() {
    var pid = PIDController(kp: 1.0, ki: 0, kd: 0)
    let output = pid.update(error: 10, dt: 0.01)
    #expect(output == 10)
}
```

---

## Mock HAL — Protocol-Based Driver Test

```swift
protocol I2CBusProtocol {
    mutating func write(address: UInt8, register: UInt8, data: [UInt8]) throws
    mutating func read(address: UInt8, register: UInt8, count: Int) throws -> [UInt8]
}

struct MockI2CBus: I2CBusProtocol {
    var registers: [UInt8: UInt8] = [:]

    mutating func write(address: UInt8, register: UInt8, data: [UInt8]) throws {
        for (i, byte) in data.enumerated() {
            registers[register &+ UInt8(i)] = byte
        }
    }

    mutating func read(address: UInt8, register: UInt8, count: Int) throws -> [UInt8] {
        (0..<count).map { registers[register &+ UInt8($0)] ?? 0 }
    }
}

@Test func bme280ChipID() throws {
    var bus = MockI2CBus()
    bus.registers[0xD0] = 0x60
    var driver = BME280Driver(bus: bus, address: 0x76)
    #expect(try driver.chipID() == 0x60)
}
```

---

## HIL — Host Swift Test Runner

```swift
import Foundation

enum HILResult: String {
    case pass = "PASS"
    case fail = "FAIL"
}

func runHILTests(portPath: String) async throws {
    let port = ORSSerialPort(path: portPath)!
    port.open()

    port.sendData("TEST ALL\n".data(using: .utf8)!)

    let response = try await readLine(from: port, timeout: 30)
    guard response == HILResult.pass.rawValue else {
        throw HILError.testFailed(response)
    }
    print("HIL: all tests passed")
}
```

---

## Embedded Swift On-Target Test Firmware (sketch)

```swift
@main
struct OnTargetTests {
    static func main() {
        var passed = 0
        var failed = 0

        if testGPIOToggle() { passed += 1 } else { failed += 1 }
        if testUartLoopback() { passed += 1 } else { failed += 1 }

        if failed == 0 {
            print("PASS\r\n")
        } else {
            print("FAIL \(failed)\r\n")
        }
    }

    static func testGPIOToggle() -> Bool {
        let led = GPIO(48, mode: .output)
        led.set(high: true)
        // ... verify via loopback pin or scope
        return true
    }

    static func testUartLoopback() -> Bool { true }
}
```

---

## CI Configuration (GitHub Actions sketch)

```yaml
name: Host Tests
on: [push, pull_request]
jobs:
  test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - uses: swift-actions/setup-swift@v2
        with:
          swift-version: "6.0"
      - name: Run host tests
        run: swift test --filter HostTests
```

Add HIL job only when hardware runner available.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| Testing only on hardware | Slow iteration | Host mocks first |
| Mock diverges from HAL | Passes host, fails device | Integration test on board |
| Flaky HIL serial | Intermittent CI fail | Timeouts, reset between tests |
| `#if os` missing | Won't compile for MCU | Conditional compilation |
| Testing timing in unit tests | Flaky | Inject clock protocol |
| No CI | Regressions ship | Host tests on every PR |

---

## Debugging Tips

- Log **test name** before each on-target assertion.
- Single `TEST_NAME` UART command for bisecting HIL failures.
- Snapshot **mock register maps** from logic analyzer captures.

---

## Performance Tips

- Keep host test suite under **10 seconds** for fast feedback.
- Parallelize independent test modules.
- HIL nightly; host tests on every commit.

---

## Exercises

1. **PID unit tests:** Cover integral windup edge case.
2. **Mock I²C:** Full BME280 compensation with canned calibration.
3. **Protocol portability:** Same `BME280Driver` on host mock and ESP-IDF I2C.
4. **HIL UART:** On-target firmware responds PASS; macOS script validates.
5. **CI:** Add GitHub Actions host test job to your repo.

---

## References

- [Swift Testing documentation](https://developer.apple.com/documentation/testing)
- [27-design-patterns.md](./27-design-patterns.md)
- [25-debugging.md](./25-debugging.md)
- [swift-embedded-examples](https://github.com/swiftlang/swift-embedded-examples)

---

*Previous: [25-debugging.md](./25-debugging.md) · Next: [27-design-patterns.md](./27-design-patterns.md)*
