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

/// Max square size reserved for QR on the Zoop panel (leaves room for labels).
pub const QR_MAX_PX: i32 = 140;

/// Encode `payload` and draw a centered QR into `buf`.
///
/// Returns module count (width in modules).
pub fn draw_qr_centered(buf: &mut [u8], payload: &str, top_y: i32) -> CoreResult<usize> {
    let code = QrCode::new(payload.as_bytes())
        .map_err(|_| CoreError::Other("QR encode failed (payload too long?)".into()))?;
    let modules = code.width();
    draw_qr_modules(buf, &code, modules, top_y)
}

fn draw_qr_modules(
    buf: &mut [u8],
    code: &QrCode,
    modules: usize,
    top_y: i32,
) -> CoreResult<usize> {
    // Quiet zone: 2 modules on each side (e-Ink scanners tolerate 2–4)
    let quiet = 2usize;
    let total = modules + quiet * 2;
    let avail = QR_MAX_PX.min(WIDTH as i32 - 16);
    let scale = (avail / total as i32).max(1);
    let qr_px = scale * total as i32;
    let origin_x = (WIDTH as i32 - qr_px) / 2;
    let origin_y = top_y.max(0).min(HEIGHT as i32 - qr_px);

    // White quiet zone (already white from clear_screen; reinforce)
    fill_rect(buf, origin_x, origin_y, qr_px, qr_px, crate::display::draw::WHITE);

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

/// Estimate whether a payload will fit reasonably on 200×200.
pub fn qr_fits_panel(payload: &str) -> bool {
    QrCode::new(payload.as_bytes())
        .map(|c| {
            let m = c.width() + 4; // quiet
            let scale = QR_MAX_PX / m as i32;
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
        let modules = draw_qr_centered(&mut buf, &uri, 36).expect("qr");
        assert!(modules >= 21);
        // Center region should have some black after encode
        let mut dark = 0;
        for y in 40..160 {
            for x in 40..160 {
                if get_pixel(&buf, x, y) == BLACK {
                    dark += 1;
                }
            }
        }
        assert!(dark > 100, "expected QR ink, got {dark} dark px");
        assert_eq!(get_pixel(&buf, 0, 0), WHITE);
    }
}
