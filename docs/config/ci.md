# .github/workflows/ci.yml

- **Path:** `.github/workflows/ci.yml`
- **Purpose:** GitHub Actions CI — host tests, fmt, clippy for `zoop-core` (firmware excluded).
- **Key types / functions:** N/A (workflow)
  - Triggers: push to `main`/`master`, pull requests
  - Job `core`: `cargo test --workspace --exclude zoop-firmware`, `cargo fmt --check`, `cargo clippy ... -D warnings`
  - Firmware xtensa job commented (run locally after `espup install`)
- **Dependencies:** `actions/checkout@v4`, `dtolnay/rust-toolchain@stable`
- **Tests:** This workflow *is* the CI test runner
- **Status:** Active
- **Related:** [Cargo.workspace.md](Cargo.workspace.md), [../core/README.md](../core/README.md)
