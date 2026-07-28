# Project: USB Keyboard

HID keyboard from matrix or single-key input — [usb-hid.md](../examples/usb-hid.md).

---

## BOM

ESP32-S3 DevKitC-1 (native USB), 4×4 keypad or push buttons.

---

## Wiring

```
GPIO matrix rows/cols ── keypad
USB-C to host PC
```

---

## Firmware Plan

1. [usb-hid.md](../examples/usb-hid.md) descriptor setup
2. Matrix scan or [button.md](../examples/button.md)
3. Keymap → `KeyboardReport`
4. `usb.poll()` in main loop — [usb.md](../communication/usb.md)

---

## Testing

- [ ] Enumerates on Linux/macOS/Windows
- [ ] Typed characters appear in text editor

---

## Extensions

- Macro keys (multi-key sequences)
- Layer switching (Fn key)

**Lesson:** [19-usb.md](../19-usb.md)
