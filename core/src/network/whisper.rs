//! Whisper upload with pluggable HTTP — host-tested via mock server.

use crate::error::CoreResult;
use crate::paths::note_path;
use crate::storage::{update_index_has_text, FileStorage, IndexStore};
use crate::transcribe::{parse_transcription_response, TranscriptionConfig};
use crate::whisper_parse::parse_whisper_text;

pub const CHUNK_SIZE: usize = 4096;
pub const MAX_RETRIES: u32 = 3;
pub const RETRY_DELAY_MS: u64 = 3000;
pub const REQUEST_TIMEOUT_MS: u64 = 90_000;

/// Minimal HTTP client for transcription upload (host + firmware adapter).
pub trait HttpClient {
    fn post_multipart_wav(
        &mut self,
        config: &TranscriptionConfig,
        wav_bytes: &[u8],
        filename: &str,
    ) -> CoreResult<String>;
}

/// Transcribe one note WAV; writes `.txt` and updates index on success.
pub fn transcribe_note<S: FileStorage>(
    storage: &S,
    index: &mut IndexStore,
    config: &TranscriptionConfig,
    note_num: i32,
    http: &mut impl HttpClient,
) -> CoreResult<bool> {
    for attempt in 0..MAX_RETRIES {
        match transcribe_once(storage, index, config, note_num, http) {
            Ok(true) => return Ok(true),
            Ok(false) | Err(_) => {
                if attempt + 1 < MAX_RETRIES {
                    std::thread::sleep(std::time::Duration::from_millis(RETRY_DELAY_MS));
                }
            }
        }
    }
    Ok(false)
}

fn transcribe_once<S: FileStorage>(
    storage: &S,
    index: &mut IndexStore,
    config: &TranscriptionConfig,
    note_num: i32,
    http: &mut impl HttpClient,
) -> CoreResult<bool> {
    let wav_path = note_path(note_num, "wav");
    let wav = storage
        .read_bytes(&wav_path)?
        .ok_or_else(|| crate::error::CoreError::Storage("wav missing".into()))?;

    let body = http.post_multipart_wav(config, &wav, "note.wav")?;
    let text = parse_transcription_response(&body)
        .or_else(|| parse_whisper_text(&body))
        .unwrap_or_default();
    if text.is_empty() {
        return Ok(false);
    }

    let txt_path = note_path(note_num, "txt");
    storage.write_string(&txt_path, &text)?;
    update_index_has_text(storage, index, note_num)?;
    Ok(true)
}

/// Notes lacking transcripts.
pub fn pending_transcription(index: &IndexStore) -> Vec<i32> {
    index
        .entries()
        .iter()
        .filter(|e| !e.has_text)
        .map(|e| e.num)
        .collect()
}

/// Transcribe all pending notes with progress callback.
pub fn transcribe_all<S: FileStorage>(
    storage: &S,
    index: &mut IndexStore,
    config: &TranscriptionConfig,
    http: &mut impl HttpClient,
    mut on_progress: impl FnMut(usize, usize),
) -> CoreResult<usize> {
    let pending: Vec<i32> = pending_transcription(index);
    let total = pending.len();
    let mut done = 0usize;
    for (i, num) in pending.iter().enumerate() {
        on_progress(i, total);
        if transcribe_note(storage, index, config, *num, http)? {
            done += 1;
        }
    }
    on_progress(total, total);
    Ok(done)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::index::add_to_index;
    use crate::storage::MockStorage;
    use crate::transcribe::TranscriptionConfig;

    struct MockHttp;

    impl HttpClient for MockHttp {
        fn post_multipart_wav(
            &mut self,
            _config: &TranscriptionConfig,
            _wav_bytes: &[u8],
            _filename: &str,
        ) -> CoreResult<String> {
            Ok(r#"{"text":"Hello from mock Whisper"}"#.to_string())
        }
    }

    #[test]
    fn transcribe_writes_txt_and_updates_index() {
        let storage = MockStorage::new().expect("storage");
        let mut index = IndexStore::new();
        add_to_index(&storage, &mut index, 1, "Work", false).expect("add");
        storage
            .write_bytes(&note_path(1, "wav"), &[0u8; 2048])
            .expect("wav");

        let config = TranscriptionConfig::from_secrets("openai", "sk-test", "", None).expect("cfg");
        let mut http = MockHttp;
        assert!(transcribe_note(&storage, &mut index, &config, 1, &mut http).expect("tx"));
        assert!(index.find(1).unwrap().has_text);
        let txt = storage
            .read_to_string(&note_path(1, "txt"))
            .expect("read")
            .expect("txt");
        assert!(txt.contains("Hello from mock Whisper"));
    }

    #[test]
    fn pending_lists_untranscribed_only() {
        let storage = MockStorage::new().unwrap();
        let mut index = IndexStore::new();
        add_to_index(&storage, &mut index, 1, "A", false).unwrap();
        add_to_index(&storage, &mut index, 2, "B", true).unwrap();
        assert_eq!(pending_transcription(&index), vec![1]);
    }
}
