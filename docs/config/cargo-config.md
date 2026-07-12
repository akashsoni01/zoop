# firmware/.cargo/config.toml

- **Path:** `firmware/.cargo/config.toml`
- **Purpose:** Cargo target/linker/runner config for `xtensa-esp32s3-espidf`.
- **Key types / functions:** N/A
  - Target `xtensa-esp32s3-espidf`, linker `ldproxy`, runner `espflash flash --monitor`
  - `build-std = ["std", "panic_abort"]`
  - Env: `MCU=esp32s3`, `ESP_IDF_VERSION=v5.2.2`
- **Dependencies:** esp toolchain, espflash, ldproxy
- **Tests:** N/A
- **Status:** Active
- **Related:** [rust-toolchain.md](rust-toolchain.md), [sdkconfig.defaults.md](sdkconfig.defaults.md)
