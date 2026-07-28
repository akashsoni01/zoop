# Lesson 22 — Swift Concurrency (async/await, Task, Actors)

**Prerequisites:** [09-interrupts.md](./09-interrupts.md), [02-no-stdlib.md](./02-no-stdlib.md), basic Swift concurrency

**Board focus:** [ESP32-S3](./boards/esp32-s3.md) via ESP-IDF. Host Swift patterns for development.

**Maturity note:** Swift Concurrency is the **Embassy equivalent** for structured async I/O. On **Embedded Swift + ESP-IDF**, async/await integrates with FreeRTOS tasks and event loops. On **bare-metal Swift**, there is no runtime executor yet — use cooperative state machines or poll loops. **Host Swift** (macOS/iOS) is the best place to learn async patterns you later port to MCU event-driven code.

---

## Theory

**Async** programming lets firmware **wait for events without blocking the CPU** in a spin loop:

```swift
// Blocking — wastes CPU
while !flagReady { }

// Async — executor runs other work
await eventReady()
```

### Swift Concurrency building blocks

| Component | Role |
|-----------|------|
| `async`/`await` | Suspend until event; resume later |
| `Task` | Structured concurrent work unit |
| `Actor` | Serialized access to mutable state |
| `AsyncStream` | Async sequence of events |
| `CheckedContinuation` | Bridge callback APIs to async |

### Executor model (embedded)

On ESP-IDF + Embedded Swift:

```
FreeRTOS tasks / ESP event loop
        ↓
Swift async tasks (integrated via continuation)
        ↓
Hardware IRQ → post event → resume awaiting Task
```

No full Swift runtime on bare-metal — **cooperative** scheduling only.

### vs Real-Time Patterns

| Approach | Best for |
|----------|----------|
| **Swift Concurrency** | Wi-Fi, HTTP, BLE, multi-I/O — [20-wifi.md](./20-wifi.md) |
| **Interrupt-driven patterns** | Hard latency, ISR work — [23-realtime-patterns.md](./23-realtime-patterns.md) |
| **Blocking + IRQ** | Simple blink, UART echo |

---

## Hardware Overview

Concurrency is software architecture — hardware events wake async tasks:

| Platform | Integration |
|----------|-------------|
| ESP32-S3 + ESP-IDF | FreeRTOS + `esp_event` loop |
| STM32 + host tests | Mock executor on macOS |
| Bare-metal C6 | Poll loop; no Task scheduler yet |

---

## ASCII Wiring

No special wiring — async LED blink:

```
GPIO48 ──► LED ──► GND   (ESP32-S3 DevKitC-1 onboard LED)
```

---

## Memory & Register Notes

Each `Task` has stack cost. On ESP32, prefer **few long-lived tasks** over thousands of ephemeral tasks:

```swift
// Good: one supervisor task
Task {
    await runSensorLoop()
}

// Avoid on MCU: unbounded Task spawning in hot path
```

`Actor` state lives in heap/static depending on allocation — minimize actor count in embedded.

---

## HAL Swift Example — Async Sensor Loop (ESP-IDF)

```swift
import ESPIDF

actor SensorState {
    private(set) var lastReading: Float = 0

    func update(_ value: Float) {
        lastReading = value
    }
}

@main
struct AsyncSensorApp {
    static func main() async {
        let i2c = I2CBus(port: 0, sda: 8, scl: 9)
        let sensor = BME280(i2c: i2c)
        let state = SensorState()

        // Parallel tasks
        async let sampling = sampleLoop(sensor: sensor, state: state)
        async let reporting = reportLoop(state: state)

        _ = await (sampling, reporting)
    }

    static func sampleLoop(sensor: BME280, state: SensorState) async {
        while true {
            let temp = sensor.readTemperatureC()
            await state.update(temp)
            try? await Task.sleep(for: .milliseconds(100))
        }
    }

    static func reportLoop(state: SensorState) async {
        while true {
            try? await Task.sleep(for: .seconds(5))
            let temp = await state.lastReading
            print("Temp: \(temp) C")
        }
    }
}
```

---

## Bare-Metal Swift — Cooperative Async (illustrative)

Without runtime executor, model async with explicit state machine:

```swift
enum BlinkPhase {
    case idle
    case ledOn(until: UInt32)
    case ledOff(until: UInt32)
}

struct CooperativeScheduler {
    var phase: BlinkPhase = .idle
    var nowMs: UInt32 = 0

    mutating func tick(nowMs: UInt32, led: GPIO) {
        self.nowMs = nowMs
        switch phase {
        case .idle:
            led.set(high: true)
            phase = .ledOn(until: nowMs + 500)
        case .ledOn(let until) where nowMs >= until:
            led.set(high: false)
            phase = .ledOff(until: nowMs + 500)
        case .ledOff(let until) where nowMs >= until:
            phase = .idle
        default:
            break
        }
    }
}
```

This is the bare-metal equivalent until Embedded Swift gains a `no_std` executor.

---

## Host Swift — AsyncStream for UART Events

Bridge callback serial port to async sequence on macOS:

```swift
import Foundation

func uartLines(path: String) -> AsyncStream<String> {
    AsyncStream { continuation in
        let port = ORSSerialPort(path: path)!
        port.open()
        port.receiveDataCallback = { data in
            if let line = String(data: data, encoding: .utf8) {
                continuation.yield(line)
            }
        }
        continuation.onTermination = { _ in port.close() }
    }
}

// Usage
for await line in uartLines(path: "/dev/cu.usbmodem101") {
    print("Board said: \(line)")
}
```

---

## Bridging C Callbacks — CheckedContinuation

ESP-IDF Wi-Fi events are callback-based; wrap for async/await:

```swift
func connectWiFi(ssid: String, password: String) async throws {
    try await withCheckedThrowingContinuation { (continuation: CheckedContinuation<Void, Error>) in
        ESPWiFi.connect(ssid: ssid, password: password) { result in
            switch result {
            case .success: continuation.resume()
            case .failure(let error): continuation.resume(throwing: error)
            }
        }
    }
}
```

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| `await` in ISR | Crash / undefined behavior | Resume tasks from ISR via event, never await |
| Unbounded `Task` creation | Stack overflow | Pool tasks; reuse |
| Data race on shared var | Corruption | Use `actor` or IRQ-safe queue |
| Blocking `Task.sleep` in ISR path | Jitter | Sleep only in task context |
| Actor deadlock | Hang | Avoid synchronous cross-actor calls in loops |
| Assuming bare-metal has executor | Link errors | Use cooperative state machine |

---

## Debugging Tips

- Log **task transitions** over UART with timestamps.
- On host, use **Swift Concurrency Instruments** (Xcode) before porting to MCU.
- Count **stack high-water mark** in FreeRTOS for each task.
- Verify `Task` cancellation on shutdown paths.

---

## Performance Tips

- Prefer **AsyncStream** over polling loops for event sources.
- Batch sensor reads; don't spawn per-sample tasks.
- Use **`nonisolated`** carefully — default to actor isolation on shared hardware state.
- Combine with DMA completion callbacks via continuation — [14-dma.md](./14-dma.md).

---

## Exercises

1. **Async blink:** Two tasks — one toggles LED, one prints uptime every second.
2. **Actor counter:** Button ISR increments via queue; actor processes in task.
3. **Wi-Fi connect:** Wrap ESP-IDF callback in `CheckedContinuation`.
4. **Host AsyncStream:** macOS app streams serial lines to SwiftUI view.
5. **Cooperative FSM:** Rewrite async blink as bare-metal state machine on C6.

---

## References

- [Swift Concurrency documentation](https://docs.swift.org/swift-book/documentation/the-swift-programming-language/concurrency/)
- [Embedded Swift docs](https://github.com/swiftlang/swift/tree/main/docs/EmbeddedSwift)
- [ESP-IDF FreeRTOS](https://docs.espressif.com/projects/esp-idf/en/latest/esp32s3/api-reference/system/freertos.html)
- [23-realtime-patterns.md](./23-realtime-patterns.md)

---

*Previous: [21-bluetooth.md](./21-bluetooth.md) · Next: [23-realtime-patterns.md](./23-realtime-patterns.md)*
