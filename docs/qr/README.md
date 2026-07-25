# Zoop QR — scan → string

Host-testable QR decode, debounce, OLED UI frames, and scan state machine for the camera kit: grayscale frame → UTF-8 string → OLED screens.

| | |
| --- | --- |
| Plan / checklist | [`TODO_qr.md`](../../TODO_qr.md) |
| Hardware (canonical) | [`../../physical-components/hardware_spec.md`](../../physical-components/hardware_spec.md) |
| Hardware (short) | [`hardware.md`](./hardware.md) |
| Core API | `zoop_core::qr` (`decode_grayscale`, `QrDebouncer`) |
| OLED UI | `zoop_core::display::oled` (**1.54″** panel, 128×64 framebuffer + screens) |
| State machine | `zoop_core::state_qr` (`QrAppState`, `QrStateMachine`) |
| Host demo firmware | [`firmware-qr/`](../../firmware-qr/) |

Payment / UPI Circle flows consume the accepted string — see [`TODO_camera.md`](../../TODO_camera.md).

```bash
cargo test -p zoop-core qr::
cargo test -p zoop-core state_qr
cargo test -p zoop-core oled
cargo run -p zoop-firmware-qr
```
