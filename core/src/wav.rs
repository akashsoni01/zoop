//! 16 kHz mono 16-bit PCM WAV header — ports `record.cpp` header rewrite.

use crate::error::{CoreError, CoreResult};

pub const SAMPLE_RATE: u32 = 16_000;
pub const CHANNELS: u16 = 1;
pub const BITS_PER_SAMPLE: u16 = 16;
pub const BYTES_PER_SAMPLE: u16 = 2;
pub const HEADER_LEN: usize = 44;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WavHeader {
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub data_bytes: u32,
}

impl WavHeader {
    pub fn new_mono_pcm(data_bytes: u32) -> Self {
        Self {
            sample_rate: SAMPLE_RATE,
            channels: CHANNELS,
            bits_per_sample: BITS_PER_SAMPLE,
            data_bytes,
        }
    }

    pub fn byte_rate(&self) -> u32 {
        self.sample_rate * self.channels as u32 * self.bits_per_sample as u32 / 8
    }

    pub fn block_align(&self) -> u16 {
        self.channels * self.bits_per_sample / 8
    }

    pub fn riff_chunk_size(&self) -> u32 {
        self.data_bytes + 36
    }

    pub fn total_file_bytes(&self) -> u32 {
        HEADER_LEN as u32 + self.data_bytes
    }
}

/// Build a 44-byte little-endian WAV header for mono PCM at 16 kHz.
pub fn build_wav_header(data_bytes: u32) -> [u8; HEADER_LEN] {
    let h = WavHeader::new_mono_pcm(data_bytes);
    let mut buf = [0u8; HEADER_LEN];
    let mut off = 0;

    write_fourcc(&mut buf, &mut off, b"RIFF");
    write_u32_le(&mut buf, &mut off, h.riff_chunk_size());
    write_fourcc(&mut buf, &mut off, b"WAVE");
    write_fourcc(&mut buf, &mut off, b"fmt ");
    write_u32_le(&mut buf, &mut off, 16); // fmt chunk size
    write_u16_le(&mut buf, &mut off, 1); // PCM
    write_u16_le(&mut buf, &mut off, h.channels);
    write_u32_le(&mut buf, &mut off, h.sample_rate);
    write_u32_le(&mut buf, &mut off, h.byte_rate());
    write_u16_le(&mut buf, &mut off, h.block_align());
    write_u16_le(&mut buf, &mut off, h.bits_per_sample);
    write_fourcc(&mut buf, &mut off, b"data");
    write_u32_le(&mut buf, &mut off, h.data_bytes);
    buf
}

pub fn parse_wav_header(bytes: &[u8]) -> CoreResult<WavHeader> {
    if bytes.len() < HEADER_LEN {
        return Err(CoreError::InvalidWav("header too short".into()));
    }
    if &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(CoreError::InvalidWav("missing RIFF/WAVE".into()));
    }
    if &bytes[12..16] != b"fmt " {
        return Err(CoreError::InvalidWav("missing fmt chunk".into()));
    }
    let audio_format = read_u16_le(bytes, 20)?;
    if audio_format != 1 {
        return Err(CoreError::InvalidWav(format!(
            "unsupported format {audio_format}"
        )));
    }
    let channels = read_u16_le(bytes, 22)?;
    let sample_rate = read_u32_le(bytes, 24)?;
    let bits_per_sample = read_u16_le(bytes, 34)?;
    if &bytes[36..40] != b"data" {
        return Err(CoreError::InvalidWav("missing data chunk".into()));
    }
    let data_bytes = read_u32_le(bytes, 40)?;
    Ok(WavHeader {
        sample_rate,
        channels,
        bits_per_sample,
        data_bytes,
    })
}

fn write_fourcc(buf: &mut [u8], off: &mut usize, tag: &[u8; 4]) {
    buf[*off..*off + 4].copy_from_slice(tag);
    *off += 4;
}

fn write_u16_le(buf: &mut [u8], off: &mut usize, v: u16) {
    buf[*off..*off + 2].copy_from_slice(&v.to_le_bytes());
    *off += 2;
}

fn write_u32_le(buf: &mut [u8], off: &mut usize, v: u32) {
    buf[*off..*off + 4].copy_from_slice(&v.to_le_bytes());
    *off += 4;
}

fn read_u16_le(bytes: &[u8], pos: usize) -> CoreResult<u16> {
    let slice = bytes
        .get(pos..pos + 2)
        .ok_or_else(|| CoreError::InvalidWav("truncated u16".into()))?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32_le(bytes: &[u8], pos: usize) -> CoreResult<u32> {
    let slice = bytes
        .get(pos..pos + 4)
        .ok_or_else(|| CoreError::InvalidWav("truncated u32".into()))?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_parse_roundtrip() {
        let data_bytes = 32000;
        let header = build_wav_header(data_bytes);
        let parsed = parse_wav_header(&header).expect("parse");
        assert_eq!(parsed.sample_rate, SAMPLE_RATE);
        assert_eq!(parsed.channels, 1);
        assert_eq!(parsed.bits_per_sample, 16);
        assert_eq!(parsed.data_bytes, data_bytes);
    }

    #[test]
    fn header_matches_record_cpp_layout() {
        let data_bytes = 16000u32;
        let h = build_wav_header(data_bytes);
        assert_eq!(&h[0..4], b"RIFF");
        assert_eq!(
            u32::from_le_bytes(h[4..8].try_into().unwrap()),
            data_bytes + 36
        );
        assert_eq!(&h[36..40], b"data");
        assert_eq!(
            u32::from_le_bytes(h[40..44].try_into().unwrap()),
            data_bytes
        );
    }

    #[test]
    fn rejects_short_buffer() {
        assert!(parse_wav_header(&[0u8; 10]).is_err());
    }
}
