//! Application layer — firmware engine + core state machine re-exports.

pub mod engine;

pub use engine::FirmwareEngine;
pub use zoop_core::sounds::SoundsPolicy;
pub use zoop_core::state::{AppState, StateMachine, Transition};
