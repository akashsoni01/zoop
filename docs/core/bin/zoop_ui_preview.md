# bin/zoop_ui_preview.rs

- **Path:** `core/src/bin/zoop_ui_preview.rs`
- **Purpose:** Renders all **UPI payment** e-Ink screens (including a live-encoded QR for Akash Soni / `akash@oksbi`) to BMP + HTML.

## Run

```bash
cargo run -p zoop-core --bin zoop-ui-preview
open target/ui-preview/index.html
```

## Exports

| File | Screen |
|------|--------|
| `01_home.bmp` | ZOOP PAY home |
| `02_qr_upi.bmp` | **UPI QR** (scan with phone to validate) |
| `03_waiting.bmp` … `16_sync.bmp` | Rest of payment flow |

## Related

[../ui-capabilities.md](../ui-capabilities.md) · [../display/qr.md](../display/qr.md)
