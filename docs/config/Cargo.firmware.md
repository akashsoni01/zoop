# firmware/Cargo.toml

- **Path:** `firmware/Cargo.toml`
- **Purpose:** Manifest for `zoop-firmware` ESP32-S3 binary using esp-idf-svc.
- **Key types / functions:** N/A
  - Binary `zoop-firmware` → `src/main.rs`
  - Deps: `log`, `esp-idf-svc`, `zoop-core`
  - Build-deps: `embuild`, `toml`, `zoop-core`
  - Metadata: ESP-IDF `v5.2.2`, `sdkconfig.defaults`
- **Dependencies:** Path dep on `../core`
- **Tests:** Excluded from CI workspace tests; build locally with esp toolchain
- **Status:** Compiles with BSP stubs; HIL pending
- **Related:** [../firmware/README.md](../firmware/README.md), [build.md](build.md)
