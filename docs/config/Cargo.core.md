# core/Cargo.toml

- **Path:** `core/Cargo.toml`
- **Purpose:** Manifest for `zoop-core` — host-testable library + `zoop-sim` binary.
- **Key types / functions:** N/A
  - Package name `zoop-core`
  - Binary `zoop-sim` → `src/bin/zoop_sim.rs`
  - Deps: `thiserror`, `tempfile`
- **Dependencies:** Workspace version/edition/license
- **Tests:** Default `cargo test -p zoop-core`
- **Status:** Host-verified on CI
- **Related:** [../core/README.md](../core/README.md), [Cargo.workspace.md](Cargo.workspace.md)
