//! Decode a QR payload from a grayscale frame buffer.

use thiserror::Error;

/// Maximum accepted UTF-8 payload length (QR Version 40 binary capacity).
pub const MAX_PAYLOAD_BYTES: usize = 2_953;

/// Successful decode of one QR symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeResult {
    pub payload: String,
    pub version: Option<u8>,
    pub ecc_level: Option<u8>,
}

/// Errors from [`decode_grayscale`].
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DecodeError {
    #[error("no QR code found in frame")]
    NotFound,
    #[error("QR found but too blurry or corrupt to decode")]
    TooBlurry,
    #[error("decoded payload is not valid UTF-8")]
    InvalidUtf8,
    #[error("decoded payload exceeds {MAX_PAYLOAD_BYTES} bytes")]
    TooLarge,
    #[error("invalid frame: {0}")]
    InvalidFrame(&'static str),
}

/// Decode the first QR code in a grayscale (`0` = black, `255` = white) buffer.
pub fn decode_grayscale(
    width: usize,
    height: usize,
    pixels: &[u8],
) -> Result<DecodeResult, DecodeError> {
    if width == 0 || height == 0 {
        return Err(DecodeError::InvalidFrame("width and height must be non-zero"));
    }
    let expected = width
        .checked_mul(height)
        .ok_or(DecodeError::InvalidFrame("width*height overflow"))?;
    if pixels.len() != expected {
        return Err(DecodeError::InvalidFrame(
            "pixels length does not match width*height",
        ));
    }

    let mut img = rqrr::PreparedImage::prepare_from_greyscale(width, height, |x, y| {
        pixels[y * width + x]
    });
    let grids = img.detect_grids();
    if grids.is_empty() {
        return Err(DecodeError::NotFound);
    }

    let (meta, content) = grids[0].decode().map_err(|e| match e {
        rqrr::DeQRError::EncodingError => DecodeError::InvalidUtf8,
        _ => DecodeError::TooBlurry,
    })?;

    if content.len() > MAX_PAYLOAD_BYTES {
        return Err(DecodeError::TooLarge);
    }

    Ok(DecodeResult {
        payload: content,
        version: u8::try_from(meta.version.0).ok(),
        ecc_level: u8::try_from(meta.ecc_level).ok(),
    })
}

#[cfg(test)]
fn render_qr_grayscale(payload: &str, module_px: usize, quiet: usize) -> (usize, Vec<u8>) {
    let code = qrcode::QrCode::new(payload.as_bytes()).expect("encode QR");
    let modules = code.width();
    let total = modules + quiet * 2;
    let size = total * module_px;
    let mut pixels = vec![255u8; size * size];
    for y in 0..modules {
        for x in 0..modules {
            if code[(x, y)] == qrcode::Color::Dark {
                let ox = (x + quiet) * module_px;
                let oy = (y + quiet) * module_px;
                for dy in 0..module_px {
                    for dx in 0..module_px {
                        pixels[(oy + dy) * size + (ox + dx)] = 0;
                    }
                }
            }
        }
    }
    (size, pixels)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_plain_text() {
        let payload = "hello zoop";
        let (size, pixels) = render_qr_grayscale(payload, 4, 4);
        let got = decode_grayscale(size, size, &pixels).expect("decode");
        assert_eq!(got.payload, payload);
        assert!(got.version.is_some());
    }

    #[test]
    fn decodes_url() {
        let payload = "https://example.com/pay?id=42";
        let (size, pixels) = render_qr_grayscale(payload, 4, 4);
        let got = decode_grayscale(size, size, &pixels).expect("decode");
        assert_eq!(got.payload, payload);
    }

    #[test]
    fn decodes_upi_uri_raw_string() {
        let payload = "upi://pay?pa=merchant@oksbi&am=200.00&cu=INR";
        let (size, pixels) = render_qr_grayscale(payload, 4, 4);
        let got = decode_grayscale(size, size, &pixels).expect("decode");
        assert_eq!(got.payload, payload);
        assert!(got.payload.starts_with("upi://"));
    }

    #[test]
    fn decodes_long_string() {
        let payload: String = (0..200).map(|i| char::from(b'a' + (i % 26) as u8)).collect();
        let (size, pixels) = render_qr_grayscale(&payload, 3, 4);
        let got = decode_grayscale(size, size, &pixels).expect("decode");
        assert_eq!(got.payload, payload);
    }

    #[test]
    fn empty_frame_not_found() {
        let pixels = vec![255u8; 64 * 64];
        let err = decode_grayscale(64, 64, &pixels).unwrap_err();
        assert_eq!(err, DecodeError::NotFound);
    }

    #[test]
    fn bad_dimensions() {
        assert_eq!(
            decode_grayscale(0, 10, &[]).unwrap_err(),
            DecodeError::InvalidFrame("width and height must be non-zero")
        );
        assert_eq!(
            decode_grayscale(10, 0, &[]).unwrap_err(),
            DecodeError::InvalidFrame("width and height must be non-zero")
        );
        let pixels = vec![0u8; 10];
        assert_eq!(
            decode_grayscale(4, 4, &pixels).unwrap_err(),
            DecodeError::InvalidFrame("pixels length does not match width*height")
        );
    }
}
