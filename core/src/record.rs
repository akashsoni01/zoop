//! WAV recording flow — ports `record.cpp` without ES8311.

use crate::error::{CoreError, CoreResult};
use crate::io::Audio;
use crate::paths::note_path;
use crate::storage::{add_to_index, write_note_meta, FileStorage, IndexStore};
use crate::wav::{build_wav_header, HEADER_LEN, SAMPLE_RATE};

pub const MIN_MONO_BYTES: usize = 1000;
pub const MIN_RECORD_MS: u64 = 500;

/// Result of a completed recording attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordOutcome {
    Success { num: i32, mono_bytes: usize },
    TooShort,
    WriteFailed(String),
}

/// Stream PCM from audio into a WAV on storage; rewrite header on stop.
pub struct RecordSession {
    pub num: i32,
    pub path: String,
    pub mono_bytes: usize,
    pub started_ms: u64,
    active: bool,
}

impl RecordSession {
    pub fn start(num: i32) -> Self {
        Self {
            num,
            path: note_path(num, "wav"),
            mono_bytes: 0,
            started_ms: 0,
            active: false,
        }
    }

    pub fn begin(
        &mut self,
        audio: &mut impl Audio,
        storage: &impl FileStorage,
        started_ms: u64,
    ) -> CoreResult<()> {
        self.started_ms = started_ms;
        self.active = true;
        self.mono_bytes = 0;
        audio.set_volume(0);
        audio.start_record()?;
        let placeholder = build_wav_header(0);
        storage.write_bytes(&self.path, &placeholder)?;
        Ok(())
    }

    pub fn pump(&mut self, audio: &mut impl Audio, storage: &impl FileStorage) -> CoreResult<()> {
        if !self.active {
            return Ok(());
        }
        let mut chunk = [0u8; 512];
        let n = audio.read_record_chunk(&mut chunk)?;
        if n == 0 {
            return Ok(());
        }
        self.mono_bytes += n;
        append_pcm(storage, &self.path, &chunk[..n])?;
        Ok(())
    }

    pub fn stop(
        &mut self,
        audio: &mut impl Audio,
        storage: &impl FileStorage,
        now_ms: u64,
    ) -> RecordOutcome {
        if !self.active {
            return RecordOutcome::TooShort;
        }
        self.active = false;
        audio.stop_record();
        audio.set_volume(85);

        let elapsed = now_ms.saturating_sub(self.started_ms);
        if elapsed < MIN_RECORD_MS || self.mono_bytes <= MIN_MONO_BYTES {
            let _ = storage.remove(&self.path);
            return RecordOutcome::TooShort;
        }

        let header = build_wav_header(self.mono_bytes as u32);
        if rewrite_header(storage, &self.path, &header).is_err() {
            return RecordOutcome::WriteFailed("header rewrite".into());
        }

        RecordOutcome::Success {
            num: self.num,
            mono_bytes: self.mono_bytes,
        }
    }
}

fn append_pcm(storage: &impl FileStorage, path: &str, pcm: &[u8]) -> CoreResult<()> {
    let mut existing = storage.read_bytes(path)?.unwrap_or_default();
    if existing.len() < HEADER_LEN {
        existing.resize(HEADER_LEN, 0);
    }
    existing.extend_from_slice(pcm);
    storage.write_bytes(path, &existing)
}

fn rewrite_header(
    storage: &impl FileStorage,
    path: &str,
    header: &[u8; HEADER_LEN],
) -> CoreResult<()> {
    let mut file = storage
        .read_bytes(path)?
        .ok_or_else(|| CoreError::Storage("wav missing".into()))?;
    if file.len() < HEADER_LEN {
        return Err(CoreError::InvalidWav("file too short".into()));
    }
    file[..HEADER_LEN].copy_from_slice(header);
    storage.write_bytes(path, &file)
}

/// Finalize a new note: index row + meta with UTC timestamp.
pub fn finalize_new_note<S: FileStorage>(
    storage: &S,
    index: &mut IndexStore,
    num: i32,
    tag: &str,
    created_utc: &str,
) -> CoreResult<()> {
    write_note_meta(storage, index, num, Some(tag), Some(created_utc))?;
    add_to_index(storage, index, num, tag, false)
}

/// Duration in milliseconds from mono byte count @ 16 kHz 16-bit.
pub fn duration_ms_from_mono_bytes(bytes: usize) -> u64 {
    (bytes as u64 * 1000) / (SAMPLE_RATE as u64 * 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::MockAudio;
    use crate::storage::MockStorage;
    use crate::wav::parse_wav_header;

    #[test]
    fn record_rejects_short_clip() {
        let storage = MockStorage::new().expect("storage");
        let mut audio = MockAudio::new();
        let mut session = RecordSession::start(1);
        session.begin(&mut audio, &storage, 0).expect("begin");
        session.mono_bytes = 500;
        let out = session.stop(&mut audio, &storage, 600);
        assert_eq!(out, RecordOutcome::TooShort);
    }

    #[test]
    fn record_writes_valid_wav_header() {
        let storage = MockStorage::new().expect("storage");
        let mut audio = MockAudio::new();
        let mut session = RecordSession::start(2);
        session.begin(&mut audio, &storage, 0).expect("begin");
        for _ in 0..4 {
            session.pump(&mut audio, &storage).expect("pump");
        }
        let out = session.stop(&mut audio, &storage, 1000);
        match out {
            RecordOutcome::Success { num, .. } => assert_eq!(num, 2),
            other => panic!("expected success, got {other:?}"),
        }
        let bytes = storage
            .read_bytes(&session.path)
            .expect("read")
            .expect("file");
        let hdr = parse_wav_header(&bytes).expect("hdr");
        assert!(hdr.data_bytes > MIN_MONO_BYTES as u32);
    }
}
