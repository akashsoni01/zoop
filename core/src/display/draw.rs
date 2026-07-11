//! 200×200 monochrome framebuffer — ports `draw.cpp` primitives.

pub const WIDTH: u16 = 200;
pub const HEIGHT: u16 = 200;
pub const BYTES: usize = (WIDTH as usize * HEIGHT as usize) / 8;

pub const BLACK: u8 = 0;
pub const WHITE: u8 = 1;

/// Set one pixel (MSB-first within each byte, row-major).
pub fn set_pixel(buf: &mut [u8], x: i32, y: i32, color: u8) {
    if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
        return;
    }
    let idx = (y as usize) * (WIDTH as usize / 8) + (x as usize / 8);
    let bit = 7 - (x as u8 % 8);
    if color == WHITE {
        buf[idx] |= 1 << bit;
    } else {
        buf[idx] &= !(1 << bit);
    }
}

pub fn get_pixel(buf: &[u8], x: i32, y: i32) -> u8 {
    if x < 0 || y < 0 || x >= WIDTH as i32 || y >= HEIGHT as i32 {
        return WHITE;
    }
    let idx = (y as usize) * (WIDTH as usize / 8) + (x as usize / 8);
    let bit = 7 - (x as u8 % 8);
    if buf[idx] & (1 << bit) != 0 {
        WHITE
    } else {
        BLACK
    }
}

pub fn fill_rect(buf: &mut [u8], x: i32, y: i32, w: i32, h: i32, color: u8) {
    for dy in 0..h {
        for dx in 0..w {
            set_pixel(buf, x + dx, y + dy, color);
        }
    }
}

pub fn hline(buf: &mut [u8], x: i32, y: i32, w: i32, color: u8) {
    for dx in 0..w {
        set_pixel(buf, x + dx, y, color);
    }
}

pub fn vline(buf: &mut [u8], x: i32, y: i32, h: i32, color: u8) {
    for dy in 0..h {
        set_pixel(buf, x, y + dy, color);
    }
}

pub fn line(buf: &mut [u8], x0: i32, y0: i32, x1: i32, y1: i32, color: u8) {
    let mut x = x0;
    let mut y = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        set_pixel(buf, x, y, color);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

pub fn fill_circle(buf: &mut [u8], cx: i32, cy: i32, r: i32, color: u8) {
    for y in -r..=r {
        for x in -r..=r {
            if x * x + y * y <= r * r {
                set_pixel(buf, cx + x, cy + y, color);
            }
        }
    }
}

pub fn stroke_circle(buf: &mut [u8], cx: i32, cy: i32, r: i32, thickness: i32, color: u8) {
    for t in 0..thickness {
        let rr = r - t;
        for angle in 0..360 {
            let rad = angle as f32 * std::f32::consts::PI / 180.0;
            let x = cx + (rr as f32 * rad.cos()).round() as i32;
            let y = cy + (rr as f32 * rad.sin()).round() as i32;
            set_pixel(buf, x, y, color);
        }
    }
}

/// Minimal 5×7 bitmap font (ASCII 32–90).
const FONT_W: i32 = 5;

fn glyph(c: char) -> [u8; 7] {
    match c {
        'A' => [0x7C, 0x12, 0x11, 0x12, 0x7C, 0, 0],
        'B' => [0x7F, 0x49, 0x49, 0x49, 0x36, 0, 0],
        'C' => [0x3E, 0x41, 0x41, 0x41, 0x22, 0, 0],
        'D' => [0x7F, 0x41, 0x41, 0x41, 0x3E, 0, 0],
        'E' => [0x7F, 0x49, 0x49, 0x49, 0x41, 0, 0],
        'F' => [0x7F, 0x09, 0x09, 0x09, 0x01, 0, 0],
        'G' => [0x3E, 0x41, 0x49, 0x49, 0x3A, 0, 0],
        'H' => [0x7F, 0x08, 0x08, 0x08, 0x7F, 0, 0],
        'I' => [0x41, 0x41, 0x7F, 0x41, 0x41, 0, 0],
        'L' => [0x7F, 0x40, 0x40, 0x40, 0x40, 0, 0],
        'M' => [0x7F, 0x02, 0x04, 0x02, 0x7F, 0, 0],
        'N' => [0x7F, 0x04, 0x08, 0x10, 0x7F, 0, 0],
        'O' => [0x3E, 0x41, 0x41, 0x41, 0x3E, 0, 0],
        'P' => [0x7F, 0x09, 0x09, 0x09, 0x06, 0, 0],
        'R' => [0x7F, 0x09, 0x19, 0x29, 0x46, 0, 0],
        'S' => [0x26, 0x49, 0x49, 0x49, 0x32, 0, 0],
        'T' => [0x01, 0x01, 0x7F, 0x01, 0x01, 0, 0],
        'U' => [0x3F, 0x40, 0x40, 0x40, 0x3F, 0, 0],
        'V' => [0x0F, 0x30, 0x40, 0x30, 0x0F, 0, 0],
        'W' => [0x7F, 0x20, 0x10, 0x20, 0x7F, 0, 0],
        'Y' => [0x07, 0x08, 0x70, 0x08, 0x07, 0, 0],
        '0'..='9' => {
            let d = c as u8 - b'0';
            match d {
                0 => [0x3E, 0x51, 0x49, 0x45, 0x3E, 0, 0],
                1 => [0x00, 0x42, 0x7F, 0x40, 0x00, 0, 0],
                2 => [0x62, 0x51, 0x49, 0x49, 0x46, 0, 0],
                3 => [0x22, 0x41, 0x49, 0x49, 0x36, 0, 0],
                4 => [0x18, 0x14, 0x12, 0x7F, 0x10, 0, 0],
                5 => [0x27, 0x45, 0x45, 0x45, 0x39, 0, 0],
                6 => [0x3C, 0x4A, 0x49, 0x49, 0x30, 0, 0],
                7 => [0x01, 0x71, 0x09, 0x05, 0x03, 0, 0],
                8 => [0x36, 0x49, 0x49, 0x49, 0x36, 0, 0],
                _ => [0x06, 0x49, 0x49, 0x49, 0x3E, 0, 0],
            }
        }
        '.' => [0, 0, 0x60, 0x60, 0, 0, 0],
        ':' => [0, 0x36, 0x36, 0, 0, 0, 0],
        '-' => [0x08, 0x08, 0x08, 0x08, 0x08, 0, 0],
        '%' => [0x23, 0x13, 0x08, 0x64, 0x62, 0, 0],
        '#' => [0x14, 0x7F, 0x14, 0x7F, 0x14, 0, 0],
        ' ' => [0; 7],
        _ => [0x7F, 0x41, 0x41, 0x41, 0x7F, 0, 0],
    }
}

pub fn text_width(s: &str, scale: i32) -> i32 {
    let chars = s.chars().count() as i32;
    if chars == 0 {
        return 0;
    }
    chars * (FONT_W + 1) * scale - scale
}

pub fn draw_str(buf: &mut [u8], x: i32, y: i32, s: &str, scale: i32, color: u8) {
    let mut cx = x;
    for ch in s.chars() {
        let g = glyph(ch);
        for (row, &bits) in g.iter().enumerate() {
            for col in 0..FONT_W {
                if bits & (1 << (4 - col)) != 0 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            set_pixel(
                                buf,
                                cx + col * scale + sx,
                                y + row as i32 * scale + sy,
                                color,
                            );
                        }
                    }
                }
            }
        }
        cx += (FONT_W + 1) * scale;
    }
}

pub fn draw_str_centered(buf: &mut [u8], cx: i32, y: i32, s: &str, scale: i32, color: u8) {
    let w = text_width(s, scale);
    draw_str(buf, cx - w / 2, y, s, scale, color);
}

pub fn draw_header(buf: &mut [u8], title: &str, right: Option<&str>) {
    fill_rect(buf, 0, 0, WIDTH as i32, 28, BLACK);
    draw_str_centered(buf, (WIDTH / 2) as i32, 10, title, 1, WHITE);
    if let Some(r) = right {
        let rw = text_width(r, 1);
        draw_str(buf, WIDTH as i32 - 8 - rw, 10, r, 1, WHITE);
    }
}

pub fn draw_hints(buf: &mut [u8], rec_label: &str, pwr_label: &str) {
    hline(buf, 0, 179, WIDTH as i32, BLACK);
    fill_rect(buf, 0, 180, WIDTH as i32, 20, WHITE);
    draw_str(buf, 8, 186, rec_label, 1, BLACK);
    let rw = text_width(pwr_label, 1);
    draw_str(buf, WIDTH as i32 - 8 - rw, 186, pwr_label, 1, BLACK);
}

pub fn draw_battery_ring(buf: &mut [u8], cx: i32, cy: i32, percent: u8) {
    stroke_circle(buf, cx, cy, 18, 2, BLACK);
    let filled = (percent as i32 * 360 / 100).min(360);
    for angle in 0..filled {
        let rad = (angle - 90) as f32 * std::f32::consts::PI / 180.0;
        let x = cx + (16.0 * rad.cos()).round() as i32;
        let y = cy + (16.0 * rad.sin()).round() as i32;
        set_pixel(buf, x, y, BLACK);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framebuffer_dimensions() {
        let mut buf = vec![0xFF; BYTES];
        fill_rect(&mut buf, 0, 0, 200, 200, BLACK);
        assert_eq!(get_pixel(&buf, 0, 0), BLACK);
        assert_eq!(get_pixel(&buf, 199, 199), BLACK);
    }

    #[test]
    fn draw_str_changes_buffer() {
        let a = vec![0xFF; BYTES];
        let mut b = a.clone();
        draw_str(&mut b, 10, 10, "HI", 1, BLACK);
        assert_ne!(a, b);
    }

    #[test]
    fn header_bar_is_black_top_strip() {
        let mut buf = vec![0xFF; BYTES];
        draw_header(&mut buf, "MENU", None);
        assert_eq!(get_pixel(&buf, 100, 5), WHITE);
        assert_eq!(get_pixel(&buf, 100, 25), WHITE);
    }
}
