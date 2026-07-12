# display/draw.rs

- **Path:** `firmware/src/display/draw.rs`
- **Purpose:** Re-export of `zoop_core::display::draw::*` so firmware code can import drawing primitives from the local module path.

## Component in architecture

```mermaid
flowchart LR
  FW["firmware/display/draw"]
  CORE["zoop_core::display::draw"]
  FW --> CORE
  style FW fill:#f96,stroke:#333,stroke-width:3px
```

## Responsibilities

- **Does:** re-export core draw API
- **Does not:** add device-specific drawing

## Key types / functions

Same as [../../core/display/draw.md](../../core/display/draw.md).

## Dependencies

- **Outbound:** `zoop_core::display::draw`
- **Inbound:** firmware UI callers (via re-export path)

## Tests

Covered by core `display::draw::tests`.

## Status

Host-verified logic via core; this file is build-time glue.

## Related

[ui.md](ui.md), [../../architecture.md](../../architecture.md)
