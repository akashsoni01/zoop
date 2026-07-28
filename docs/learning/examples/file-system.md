# Example: File System (LittleFS)

**Goal:** Store logs and config files on internal flash partition.

**Prerequisites:** [04-cargo.md](../04-cargo.md)

---

## Rust Sketch

```rust
use littlefs2::{File, Filesystem, OpenOptions};

fn append_log(fs: &Filesystem, msg: &str) {
    let mut f = File::open(fs, "log.txt", OpenOptions::new().append(true)).unwrap();
    f.write_all(msg.as_bytes()).unwrap();
}

// Ownership: Filesystem mounted once — use Mutex for multi-task access
static FS: Mutex<CriticalSectionRawMutex, Option<Filesystem>> = Mutex::new(None);
```

Partition table in `partitions.csv` — separate from app slot for OTA.

*Next: [flash-storage.md](./flash-storage.md)*
