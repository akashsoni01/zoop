# Firmware storage

SD card VFS adapter implementing `FileStorage` (HIL stub until SDIO mount works).

## Component in architecture

```mermaid
flowchart LR
  APP["App / portal"]
  SD["SdStorage HIL stub"]
  FAT["/sdcard FAT32"]
  APP --> SD -.-> FAT
  style SD fill:#f96,stroke:#333,stroke-width:3px
```

[architecture.md](../../architecture.md).

## Child docs

| Doc | Source |
|-----|--------|
| [mod.md](mod.md) | `mod.rs` |
| [sd.md](sd.md) | `sd.rs` |

## Status

HIL stub.
