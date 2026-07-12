# error.rs

- **Path:** `core/src/error.rs`
- **Purpose:** Shared error type and result alias for core operations.
- **Key types / functions:**
  - `CoreResult<T>`
  - `CoreError` — `Storage`, `InvalidIndexLine`, `InvalidTag`, `TagNotFound`, `TagLimitReached`, `NoteNotFound`, `InvalidWav`, `InvalidTransition`, `InvalidMeta`
- **Dependencies:** `thiserror`, `state::{AppState, Transition}` (for invalid transitions)
- **Tests:** Exercised indirectly via storage/state tests
- **Status:** Host-verified
- **Related:** [lib.md](lib.md), [state.md](state.md), [storage/README.md](storage/README.md)
