# core/Cargo.toml

- **Path:** `core/Cargo.toml`
- **Purpose:** Manifest for `zoop-core` — host-testable library plus `zoop-sim` binary. Std-friendly deps (`thiserror`, `tempfile`) suitable for Mac/Linux CI.

## Component in architecture

```mermaid
flowchart LR
  MAN["core/Cargo.toml"]
  LIB["lib zoop-core"]
  BIN["bin zoop-sim"]
  MAN --> LIB & BIN
  style MAN fill:#f96,stroke:#333,stroke-width:3px
```

## Responsibilities

- **Does:** package metadata, lib + `[[bin]] name = zoop-sim`, dependencies
- **Does not:** embed ESP-IDF

## Key types / functions

N/A (manifest). Package `zoop-core`; bin path `src/bin/zoop_sim.rs`.

## Dependencies

- **Outbound:** `thiserror` 2, `tempfile` 3; workspace edition/license/version
- **Inbound:** firmware path dependency, CI, `zoop-sim`

## Tests

`cargo test -p zoop-core`

## Status

Build-time; crate is host-verified.

## Related

[../core/README.md](../core/README.md), [../architecture.md](../architecture.md)
