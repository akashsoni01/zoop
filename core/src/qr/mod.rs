//! QR scan → string (decode + debounce).
//! Encode/draw for e-Paper collect lives in [`crate::display::qr`].

mod debounce;
mod decode;

pub use debounce::{DebounceOutcome, QrDebouncer};
pub use decode::{decode_grayscale, DecodeError, DecodeResult, MAX_PAYLOAD_BYTES};
