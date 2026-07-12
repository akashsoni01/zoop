//! QR code rendering onto the 200×200 1-bit e-Paper framebuffer.
//!
//! ## Why this works on e-Ink IoT
//! - QR is inherently 1-bit (black modules / white quiet zone)
//! - No grayscale or anti-aliasing needed
//! - Encode once → paint modules as filled rectangles → flush panel
//! - Prefer ECC level M/Q for dusty/scratched screens; keep payload short

use crate::display::draw::{fill_rect, BLACK, HEIGHT, WIDTH};
use crate::error::{CoreError, CoreResult};
use qrcode::QrCode;

/// Max square when chrome (header/hints) is present.
pub const QR_MAX_PX: i32 = 140;

/// Near full-panel QR for the home collect screen (minimal chrome).
pub const QR_FULL_PX: i32 = 196;

/// Encode `payload` and draw a centered QR under `top_y` (uses [`QR_MAX_PX`]).
pub fn draw_qr_centered(buf: &mut [u8], payload: &str, top_y: i32) -> CoreResult<usize> {
    draw_qr_sized(buf, payload, top_y, QR_MAX_PX)
}

/// Encode `payload` and fill as much of the panel as possible (home screen).
pub fn draw_qr_fullscreen(buf: &mut [u8], payload: &str) -> CoreResult<usize> {
    let code = QrCode::new(payload.as_bytes())
        .map_err(|_| CoreError::Other("QR encode failed (payload too long?)".into()))?;
    let modules = code.width();
    let quiet = 2usize;
    let total = modules + quiet * 2;
    let avail = QR_FULL_PX.min(WIDTH as i32).min(HEIGHT as i32);
    let scale = (avail / total as i32).max(1);
    let qr_px = scale * total as i32;
    let origin_x = (WIDTH as i32 - qr_px) / 2;
    let origin_y = (HEIGHT as i32 - qr_px) / 2;
    draw_qr_at(buf, &code, modules, quiet, scale, origin_x, origin_y)
}

fn draw_qr_sized(buf: &mut [u8], payload: &str, top_y: i32, max_px: i32) -> CoreResult<usize> {
    let code = QrCode::new(payload.as_bytes())
        .map_err(|_| CoreError::Other("QR encode failed (payload too long?)".into()))?;
    let modules = code.width();
    let quiet = 2usize;
    let total = modules + quiet * 2;
    let avail = max_px.min(WIDTH as i32 - 4);
    let scale = (avail / total as i32).max(1);
    let qr_px = scale * total as i32;
    let origin_x = (WIDTH as i32 - qr_px) / 2;
    let origin_y = top_y.max(0).min(HEIGHT as i32 - qr_px);
    draw_qr_at(buf, &code, modules, quiet, scale, origin_x, origin_y)
}

fn draw_qr_at(
    buf: &mut [u8],
    code: &QrCode,
    modules: usize,
    quiet: usize,
    scale: i32,
    origin_x: i32,
    origin_y: i32,
) -> CoreResult<usize> {
    let total = modules + quiet * 2;
    let qr_px = scale * total as i32;
    fill_rect(
        buf,
        origin_x,
        origin_y,
        qr_px,
        qr_px,
        crate::display::draw::WHITE,
    );

    for y in 0..modules {
        for x in 0..modules {
            if code[(x, y)] == qrcode::Color::Dark {
                let px = origin_x + (x + quiet) as i32 * scale;
                let py = origin_y + (y + quiet) as i32 * scale;
                fill_rect(buf, px, py, scale, scale, BLACK);
            }
        }
    }
    Ok(modules)
}

/// Estimate whether a payload will fit reasonably on 200×200 at full-panel size.
pub fn qr_fits_panel(payload: &str) -> bool {
    QrCode::new(payload.as_bytes())
        .map(|c| {
            let m = c.width() + 4; // quiet
            let scale = QR_FULL_PX / m as i32;
            scale >= 2
        })
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::draw::{get_pixel, BYTES, WHITE};
    use crate::upi::build_upi_uri;

    #[test]
    fn draws_upi_qr_with_dark_modules() {
        let mut buf = vec![0xFF; BYTES];
        let uri = build_upi_uri("akash@oksbi", "Akash Soni", "100.00", "Zoop");
        assert!(qr_fits_panel(&uri));
        let modules = draw_qr_fullscreen(&mut buf, &uri).expect("qr");
        assert!(modules >= 21);
        let mut dark = 0;
        for y in 20..180 {
            for x in 20..180 {
                if get_pixel(&buf, x, y) == BLACK {
                    dark += 1;
                }
            }
        }
        assert!(dark > 100, "expected QR ink, got {dark} dark px");
        assert_eq!(get_pixel(&buf, 0, 0), WHITE);
    }
}
