# Core crate (`zoop-core`)

Host-testable application logic for Zoop — ports `pala_note` behavior without ESP-IDF dependencies.

## Layout

| Area | Docs |
|------|------|
| Crate root | [lib.md](lib.md), [app.md](app.md), [state.md](state.md), [error.md](error.md), [io.md](io.md), [mock.md](mock.md) |
| Input / power | [buttons.md](buttons.md), [battery.md](battery.md), [power.md](power.md), [sleep.md](sleep.md), [sounds.md](sounds.md) |
| Media | [record.md](record.md), [wav.md](wav.md), [paths.md](paths.md) |
| Time / STT | [time.md](time.md), [transcribe.md](transcribe.md), [whisper_parse.md](whisper_parse.md), [portal_fmt.md](portal_fmt.md) |
| Display | [display/](display/) |
| Network | [network/](network/) |
| Storage | [storage/](storage/) |
| Bin / tests | [bin/zoop_sim.md](bin/zoop_sim.md), [tests/](tests/) |

## How to test

```bash
cargo test -p zoop-core
# or from workspace root:
cargo test --workspace --exclude zoop-firmware
```

## Status

**Host-verified** — unit + integration tests pass on CI. Firmware consumes these APIs via BSP trait adapters.
