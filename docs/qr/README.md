# Zoop QR — scan → string

Host-testable QR decode and debounce for the camera kit: grayscale frame → UTF-8 string.

| | |
| --- | --- |
| Plan / checklist | [`TODO_qr.md`](../../TODO_qr.md) |
| Hardware (draft) | [`hardware.md`](./hardware.md) |
| Core API | `zoop_core::qr` (`decode_grayscale`, `QrDebouncer`) |
| Host demo firmware | [`firmware-qr/`](../../firmware-qr/) |

Payment / UPI Circle flows consume the accepted string — see [`TODO_camera.md`](../../TODO_camera.md).

```bash
cargo test -p zoop-core qr::
cargo run -p zoop-firmware-qr
```
