# Example: Raw Flash Storage

**Goal:** Read/write NVS or raw partition for calibration data.

**Prerequisites:** [03-memory-layout.md](../03-memory-layout.md)

---

## Rust Sketch

```rust
use esp_storage::FlashStorage;

fn save_calibration(offset: u32, data: &[u8; 256]) {
    let mut flash = FlashStorage::new();
    flash.erase_sector(offset).unwrap();
    flash.write(offset, data).unwrap();
}

// Wear leveling: don't erase every write — use NVS or append-only log
```

Flash **must be erased** before write (sector granularity 4 KB typical).

*Next: [eeprom.md](./eeprom.md)*
