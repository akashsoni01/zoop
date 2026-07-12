# Firmware audio

ES8311 codec adapter implementing `zoop_core::io::Audio` (HIL stub).

## Component in architecture

```mermaid
flowchart LR
  APP["App / record"]
  AUD["Es8311Audio HIL stub"]
  HW["ES8311 + I2S"]
  APP --> AUD -.-> HW
  style AUD fill:#f96,stroke:#333,stroke-width:3px
```

[architecture.md](../../architecture.md).

## Child docs

| Doc | Source |
|-----|--------|
| [mod.md](mod.md) | `audio/mod.rs` |
| [es8311.md](es8311.md) | `audio/es8311.rs` |

## Status

HIL stub.
