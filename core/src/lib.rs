//! Host-testable core logic for Zoop — ports `pala_note` behavior without ESP dependencies.

pub mod battery;
pub mod error;
pub mod paths;
pub mod portal_fmt;
pub mod state;
pub mod storage;
pub mod wav;
pub mod whisper_parse;

pub use battery::{battery_percent_from_voltage, BatteryCurve};
pub use error::{CoreError, CoreResult};
pub use paths::*;
pub use portal_fmt::{format_export_text, html_escape, ExportNote};
pub use state::{AppState, ButtonEvent, StateMachine, Transition};
pub use storage::{
    add_custom_tag, add_to_index, delete_note, delete_tag, load_index, load_tags, next_note_number,
    read_note_meta_value, save_index, save_tag, save_tags, update_index_has_text, write_note_meta,
    FileStorage, IndexStore, MockStorage, NoteEntry, TagStore,
};
pub use wav::{parse_wav_header, WavHeader, SAMPLE_RATE};
pub use whisper_parse::parse_whisper_text;
