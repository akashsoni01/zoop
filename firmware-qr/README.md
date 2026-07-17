# zoop-firmware-qr

Host-only demo for **QR scan → string** (no ESP-IDF yet). Uses `zoop_core::qr::decode_grayscale` on a rendered fixture.

See [`docs/qr/README.md`](../docs/qr/README.md) and [`TODO_qr.md`](../TODO_qr.md).

## Build / run

```bash
cargo run -p zoop-firmware-qr
```

Expected stdout (JSON line):

```text
{"ok":true,"payload":"upi://pay?pa=merchant@oksbi&am=200.00","len":37}
```

## Layout

| Path | Role |
| --- | --- |
| `src/main.rs` | Encode fixture → decode → print JSON |
| `src/board/pins.rs` | Draft GPIO constants (OLED / buttons / I²S) |
