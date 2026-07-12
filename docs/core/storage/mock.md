# storage/mock.rs

- **Path:** `core/src/storage/mock.rs`
- **Purpose:** Temp-dir `FileStorage` for unit/integration tests and `zoop-sim`. Implements atomic writes and directory ops on the host filesystem.

## Component in architecture

```mermaid
flowchart LR
  TESTS["tests / zoop-sim"]
  MOCK["MockStorage"]
  FS["FileStorage"]
  TESTS --> MOCK -.->|implements| FS
  style MOCK fill:#f96,stroke:#333,stroke-width:3px
```

## Responsibilities

- **Does:** `new` (temp), `with_root`, `root`, full `FileStorage`, `dump_files`
- **Does not:** SDIO

## Key types / functions

| Item | Role |
|------|------|
| `MockStorage::new` | Temp directory |
| `with_root` / `root` | Fixed path mode |
| `dump_files` | Debug snapshot |

## Dependencies

Outbound: `FileStorage`, `error`. Inbound: nearly all storage/app tests.

## Tests

```bash
cargo test -p zoop-core storage::mock::tests
```

## Status

Host-verified.

## Related

[mod.md](mod.md), [../mock.md](../mock.md), [../../architecture.md](../../architecture.md)
