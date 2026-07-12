# .github/workflows/ci.yml

- **Path:** `.github/workflows/ci.yml`
- **Purpose:** CI pipeline for host-verified Zoop logic. Job `core` runs workspace tests excluding firmware, plus `fmt` and `clippy -D warnings`. Firmware Xtensa job is commented pending esp toolchain on runners.

## Component in architecture

```mermaid
flowchart LR
  CI["GitHub Actions"]
  TEST["cargo test --workspace --exclude zoop-firmware"]
  CORE["zoop-core"]
  CI --> TEST --> CORE
  style CI fill:#f96,stroke:#333,stroke-width:3px
```

## Responsibilities

- **Does:** gate PRs on host tests/format/clippy
- **Does not:** flash hardware or build esp-idf firmware (yet)

## Key types / functions

N/A (YAML). Triggers: push/PR to `main`/`master`. Steps: checkout@v4, `dtolnay/rust-toolchain@stable`, test/fmt/clippy with `--exclude zoop-firmware`.

## Dependencies

- **Outbound:** cargo workspace (core)
- **Inbound:** GitHub Actions runners

## Tests

Workflow invokes:

```bash
cargo test --workspace --exclude zoop-firmware
cargo fmt --all -- --check
cargo clippy --workspace --exclude zoop-firmware -- -D warnings
```

## Status

Build-time / CI.

## Related

[../core/README.md](../core/README.md), [../architecture.md](../architecture.md)
