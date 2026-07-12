# Integration tests (`core/tests`)

Workspace/host integration tests for offline UX flows.

| File | Doc |
|------|-----|
| `integration_offline.rs` | [integration_offline.md](integration_offline.md) |
| `zoop_sim_flow.rs` | [zoop_sim_flow.md](zoop_sim_flow.md) |

```bash
cargo test -p zoop-core --test integration_offline --test zoop_sim_flow
```
