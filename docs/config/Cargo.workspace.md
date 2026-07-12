# Cargo.toml (workspace)

- **Path:** `Cargo.toml`
- **Purpose:** Cargo workspace root for `core` and `firmware` crates; shared package metadata and release profile.
- **Key types / functions:** N/A (manifest)
  - Members: `core`, `firmware`
  - Workspace package: edition 2021, version `0.1.0`, MIT OR Apache-2.0
  - Release: `opt-level = "s"`, `lto = true`
- **Dependencies:** Declares members only
- **Tests:** `cargo test --workspace --exclude zoop-firmware`
- **Status:** Active
- **Related:** [Cargo.core.md](Cargo.core.md), [Cargo.firmware.md](Cargo.firmware.md), [ci.md](ci.md)
