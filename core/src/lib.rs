//! Host-testable core logic for Zoop — ports `pala_note` behavior without ESP dependencies.

pub mod app;
pub mod battery;
pub mod buttons;
pub mod display;
pub mod error;
pub mod io;
pub mod mock;
pub mod network;
pub mod paths;
pub mod portal_fmt;
pub mod power;
pub mod record;
pub mod sleep;
pub mod sounds;
pub mod state;
pub mod storage;
pub mod transcribe;
pub mod wav;
pub mod whisper_parse;

pub use app::App;
pub use battery::{battery_percent_from_voltage, BatteryCurve};
pub use buttons::ButtonEngine;
pub use error::{CoreError, CoreResult};
pub use io::*;
pub use paths::*;
pub use portal_fmt::{format_export_text, html_escape, portal_css, url_decode_simple, ExportNote};
pub use power::{power_on_sequence, power_sleep_sequence};
pub use record::{RecordOutcome, RecordSession};
pub use sleep::{ActivityTimer, WakeCause};
pub use sounds::SoundsPolicy;
pub use state::{AppState, ButtonEvent, StateMachine, Transition};
pub use storage::{
    add_custom_tag, add_to_index, delete_note, delete_tag, load_index, load_tags, next_note_number,
    read_note_meta_value, save_index, save_tag, save_tags, update_index_has_text, write_note_meta,
    FileStorage, IndexStore, MockStorage, NoteEntry, TagStore,
};
pub use wav::{parse_wav_header, WavHeader, SAMPLE_RATE};
pub use transcribe::{
    parse_transcription_response, TranscribeError, TranscriptionConfig, TranscriptionProvider,
    CURSOR_HOST, DEFAULT_BOUNDARY, DEFAULT_MODEL, OPENAI_HOST, TRANSCRIPTION_PATH,
};
pub use whisper_parse::parse_whisper_text;
pub use network::portal::{handle_portal_request, serve_portal, HttpRequest, HttpResponse};
pub use network::whisper::{transcribe_all, transcribe_note, HttpClient};
pub use network::wifi::{advance_wifi_connect, WifiConnectPhase, WifiMode};
