# battery.rs

- **Path:** `core/src/battery.rs`
- **Purpose:** LiPo voltage→percent curve and ADC millivolt conversion for the Waveshare board divider. Used by Idle UI battery ring and low-battery warning policy. Pure math — no GPIO.

## Component in architecture

```mermaid
flowchart LR
  ADC["BatteryAdc"]
  BAT["battery"]
  APP["app / sleep"]
  UI["display/ui ring"]
  ADC --> BAT --> APP --> UI
  style BAT fill:#f96,stroke:#333,stroke-width:3px
```

## Responsibilities

- **Does:** map voltage to percent; convert ADC mV to pack voltage; average samples; low/recover thresholds on `BatteryCurve`
- **Does not:** read ADC hardware (trait / firmware)

## Key types / functions

| Item | Role |
|------|------|
| `BatteryCurve::LOW_THRESHOLD` | `15` |
| `BatteryCurve::RECOVER_THRESHOLD` | `20` |
| `battery_percent_from_voltage(v)` | curve lookup |
| `voltage_from_adc_mv(mv)` | divider scaling |
| `battery_percent_from_adc_samples(samples)` | mean mV → percent |

## Dependencies

- **Outbound:** none
- **Inbound:** `app`, `sleep` warning helpers, `firmware::power::BatteryMonitor`

## Tests

```bash
cargo test -p zoop-core battery::tests
```

## Status

Host-verified; ADC path on device is HIL stub.

## Related

[sleep.md](sleep.md), [../firmware/power/mod.md](../firmware/power/mod.md), [../architecture.md](../architecture.md)
