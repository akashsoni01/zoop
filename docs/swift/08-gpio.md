# Lesson 08: GPIO — Digital Input and Output

**GPIO (General Purpose Input/Output)** pins are the foundation of embedded interaction — LEDs, buttons, relays, and digital sensors all start here.

**Prerequisites:** [07-hal.md](./07-hal.md)  
**Next:** [09-interrupts.md](./09-interrupts.md)  
**See also:** [06-register-programming.md](./06-register-programming.md), [11-pwm.md](./11-pwm.md)

---

## Theory

### GPIO Modes

| Mode | Direction | Use Case |
|------|-----------|----------|
| **Output push-pull** | Out | Drive LED high/low |
| **Output open-drain** | Out | I²C, shared bus pull-up |
| **Input floating** | In | External pull resistor required |
| **Input pull-up** | In | Button to GND (active low) |
| **Input pull-down** | In | Button to VCC (active high) |
| **Analog** | — | ADC input ([12-adc.md](./12-adc.md)) |
| **Alternate function** | — | UART, SPI hardware peripheral |

### Electrical Basics

- **Active high:** LED on when pin is HIGH (3.3 V)
- **Active low:** LED on when pin is LOW (common on STM32 Nucleo)
- **Logic levels:** ESP32/STM32/RP2040 are **3.3 V** — do not connect 5 V signals directly

Current through LED: `I = (Vgpio - Vf_led) / R`. Target ~5–10 mA.

### Swift Value Semantics for Pin State

Model pin configuration as immutable structs; mutate output through HAL methods:

```swift
struct LedState {
    var isOn: Bool
}

// Pin hardware owned by HAL wrapper
var led = ESP32GpioPin(pin: 2)
_ = led.setHigh()
```

---

## Hardware Overview

### ESP32-S3 DevKitC-1

- Built-in RGB LED may be on GPIO48 (WS2812) or separate GPIO2 LED depending on revision
- Boot button: GPIO0 (strapping pin — avoid holding at boot)
- 45 GPIO pins, not all exposed on DevKit headers

### STM32 Nucleo-F411RE (Comparison)

- User LED: PA5 (active high)
- User button: PC13 (active low, internal pull-up on some boards)

### RP2040 Pico (Comparison)

- On-board LED: GPIO25 (active low on Pico W variant differs)
- 30 multi-function GPIO pins

---

## Wiring Diagram

### LED + Button Circuit (Breadboard)

```
                    ESP32-S3 DevKit
                  ┌─────────────────┐
                  │                 │
    3V3 ──────────┤ 3V3             │
                  │                 │
                  │ GPIO2 ────┬─────┼──► To LED anode (+)
                  │           │     │
                  │           [ 220Ω ]     (current limit)
                  │           │     │
                  │           └──(>|)────┤ GND (cathode)
                  │                 │
                  │ GPIO0 ────┬─────┤ (input, internal pull-up enabled in software)
                  │           │     │
                  │          [SW]     │ tactile button
                  │           │     │
    GND ──────────┤ GND ──────┴─────┤
                  └─────────────────┘

Button wiring (active low):
  GPIO0 ──── button ──── GND
  (internal pull-up keeps pin HIGH when open)
  (pressed → pin pulled LOW)
```

### Multi-LED Example

```
GPIO2 ──[220Ω]──(LED1)── GND
GPIO4 ──[220Ω]──(LED2)── GND
GPIO5 ──[220Ω]──(LED3)── GND
```

Always share common GND between board and breadboard.

---

## Memory & Register Explanation

GPIO state lives in MMIO registers — see [06-register-programming.md](./06-register-programming.md).

| Operation | ESP32-S3 Register | STM32 Register |
|-----------|-------------------|----------------|
| Set output high | `GPIO_OUT_W1TS` | `BSRR` low half |
| Set output low | `GPIO_OUT_W1TC` | `BSRR` high half |
| Read input | `GPIO_IN` | `IDR` |
| Enable output | `GPIO_ENABLE` | `MODER` |

HAL configures these once; application code calls `setHigh()` / `isLow()`.

---

## HAL-Style Swift Implementation

Complete ESP32-S3 blink + button toggle:

```swift
import EmbeddedHAL
import ESP32HAL

@main
struct BlinkButtonApp {
    static func main() {
        guard boardInit() else {
            fatalError("Board init failed")
        }

        var led = ESP32GpioOutputPin(pin: 2, initial: .low)
        let button = ESP32GpioInputPin(pin: 0, pull: .up)
        var delay = BusyLoopDelay(cpuHz: 240_000_000)

        var ledState = false

        while true {
            if case .success(true) = button.isLow() {
                delay.delayMs(50)  // Debounce

                if case .success(true) = button.isLow() {
                    ledState.toggle()
                    if ledState {
                        _ = led.setHigh()
                        log("LED ON")
                    } else {
                        _ = led.setLow()
                        log("LED OFF")
                    }
                    // Wait for release
                    while case .success(true) = button.isLow() {
                        delay.delayMs(10)
                    }
                }
            }
            delay.delayMs(10)
        }
    }
}
```

STM32 equivalent using same protocols:

```swift
var led = Stm32GpioOutputPin(port: .a, pin: 5)
let button = Stm32GpioInputPin(port: .c, pin: 13, pull: .up)

while true {
    if case .success(true) = button.isLow() {
        _ = led.toggle()
        // debounce...
    }
}
```

---

## Bare-Metal / MMIO Swift Notes

```swift
enum GPIOAddr {
    static let base: UInt = 0x6000_4000
    static let enable: UInt = base + 0x20
    static let outSet: UInt = base + 0x10
    static let outClear: UInt = base + 0x14
    static let `in`: UInt = base + 0x3C
    static let ioMux: UInt = 0x6000_9000
}

let PIN_LED: UInt8 = 2
let PIN_BTN: UInt8 = 0
let MASK_LED: UInt32 = 1 << PIN_LED
let MASK_BTN: UInt32 = 1 << PIN_BTN

func reg(_ addr: UInt) -> UnsafeMutablePointer<UInt32> {
    UnsafeMutablePointer(bitPattern: addr)!
}

func gpioInit() {
    // Mux pins as GPIO
    reg(GPIOAddr.ioMux + UInt(PIN_LED) * 4).pointee = 1
    reg(GPIOAddr.ioMux + UInt(PIN_BTN) * 4).pointee = 1

    // Enable output on LED
    reg(GPIOAddr.enable).pointee |= MASK_LED
    // Input: clear enable bit
    reg(GPIOAddr.enable).pointee &= ~MASK_BTN
}

func ledOn()  { reg(GPIOAddr.outSet).pointee = MASK_LED }
func ledOff() { reg(GPIOAddr.outClear).pointee = MASK_LED }

func buttonPressed() -> Bool {
    (reg(GPIOAddr.in).pointee & MASK_BTN) == 0
}

@main
struct BareGpioApp {
    static func main() {
        gpioInit()
        var on = false
        while true {
            if buttonPressed() {
                on.toggle()
                if on { ledOn() } else { ledOff() }
                while buttonPressed() { busyWait() }
            }
            busyWait()
        }
    }
}
```

---

## Step-by-Step Explanation

### Step 1: Identify Pins

Check board schematic. Note LED polarity and button strapping constraints (GPIO0 on ESP32 affects boot mode).

### Step 2: Wire Breadboard

Connect LED with resistor. Connect button between GPIO and GND. Verify GND common.

### Step 3: Initialize Clocks and GPIO HAL

Follow BSP init — clocks before GPIO.

### Step 4: Configure Output Pin

Set initial level (usually LOW to avoid LED flash at boot).

### Step 5: Configure Input with Pull-Up

Internal pull-up (~45 kΩ on ESP32) usually sufficient for buttons.

### Step 6: Poll Button in Loop

Simplest approach — works for learning; interrupts improve responsiveness ([09-interrupts.md](./09-interrupts.md)).

### Step 7: Toggle LED on Press

Add software debounce (20–50 ms) to filter mechanical bounce.

---

## Common Mistakes

| Mistake | Symptom | Fix |
|---------|---------|-----|
| No current-limiting resistor | Dead LED or damaged GPIO | Add 220 Ω–1 kΩ |
| Wrong pin number | Nothing happens | Check schematic |
| Floating input | Erratic button reads | Enable pull-up/down |
| Active low vs high confusion | Inverted behavior | Match `isLow()` / `isHigh()` |
| GPIO0 held at boot (ESP32) | Boot failure | Release button before reset |
| 5 V on 3.3 V pin | Damaged MCU | Level shifter or 3.3 V logic |

---

## Debugging Tips

1. **Toggle pin with multimeter** — verify voltage changes.
2. **Slow blink on boot** — confirms code runs before button logic.
3. **UART log raw pin read** — see bounce noise.
4. **Swap LED orientation** — won't light if reversed.
5. **Logic analyzer** — visualize bounce duration (typically 1–20 ms).

---

## Performance Tips

| Tip | Detail |
|-----|--------|
| Use interrupts for buttons | Frees CPU ([09-interrupts.md](./09-interrupts.md)) |
| Batch GPIO via port writes | Toggle multiple pins in one register write |
| Avoid polling in tight loop | Add small delay or WFI sleep |
| Direct register access in ISR | Only if profiling shows HAL too slow |

---

## Exercises

### Exercise 1: Blink Rate

Modify blink to 2 Hz. Measure with phone camera slow-mo or logic analyzer.

### Exercise 2: Three LEDs

Control three LEDs with three GPIO pins; chase pattern.

### Exercise 3: Hold-to-Dim Preview

Button held > 1 s turns LED on; short press toggles (prepares for [11-pwm.md](./11-pwm.md)).

### Exercise 4: Bare-Metal Port

Rewrite HAL blink in bare-metal MMIO style.

### Exercise 5: Portable Driver

Write `ButtonLed` struct using `DigitalInputPin` + `DigitalOutputPin`; test with mock on macOS.

---

## References

- [ESP32-S3 TRM — GPIO](https://www.espressif.com/sites/default/files/documentation/esp32-s3_technical_reference_manual_en.pdf)
- [Lesson 06 — Register Programming](./06-register-programming.md)
- [Lesson 07 — HAL Protocols](./07-hal.md)
- [STM32 Nucleo User Manual UM1724](https://www.st.com/resource/en/user_manual/um1724-stm32-nucleo64-boards-mb1136-stmicroelectronics.pdf)
- [Embedded Rust GPIO lesson](../learning/08-gpio.md)

---

*Previous: [07-hal.md](./07-hal.md) | Next: [09-interrupts.md](./09-interrupts.md)*
