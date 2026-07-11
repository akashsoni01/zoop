use thiserror::Error;

pub type CoreResult<T> = Result<T, CoreError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("storage error: {0}")]
    Storage(String),
    #[error("invalid index line: {0}")]
    InvalidIndexLine(String),
    #[error("invalid tag: {0}")]
    InvalidTag(String),
    #[error("tag not found: {0}")]
    TagNotFound(String),
    #[error("tag limit reached ({0})")]
    TagLimitReached(usize),
    #[error("note not found: {0}")]
    NoteNotFound(i32),
    #[error("invalid WAV header: {0}")]
    InvalidWav(String),
    #[error("invalid state transition: {from:?} + {event:?}")]
    InvalidTransition {
        from: crate::state::AppState,
        event: crate::state::Transition,
    },
    #[error("invalid meta: {0}")]
    InvalidMeta(String),
}
