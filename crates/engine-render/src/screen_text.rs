pub const CELL: f32 = 5.0;
pub const SCALE: f32 = 2.0;
pub const GAP: f32 = 1.0;
pub const LINE_HEIGHT: f32 = CELL * SCALE + 10.0;
pub const CHAR_WIDTH: f32 = (CELL + GAP) * SCALE;

/// MC widget button height in source pixels (matches `button.png`).
pub const WIDGET_BUTTON_H: f32 = 20.0;
/// MC font height inside a 20px-tall button.
pub const WIDGET_FONT_H: f32 = 8.0;
pub const WIDGET_CHAR_W: f32 = 6.0;

pub fn widget_glyph_pixel(widget_scale: f32) -> f32 {
    (WIDGET_FONT_H / 7.0) * widget_scale
}

pub fn widget_char_width(widget_scale: f32) -> f32 {
    WIDGET_CHAR_W * widget_scale
}

pub fn widget_line_height(widget_scale: f32) -> f32 {
    WIDGET_FONT_H * widget_scale
}

pub fn widget_text_width(text: &str, widget_scale: f32) -> f32 {
    text.chars().count() as f32 * widget_char_width(widget_scale)
}

pub fn widget_centered_x(text: &str, rect_x: f32, rect_w: f32, widget_scale: f32) -> f32 {
    rect_x + (rect_w - widget_text_width(text, widget_scale)).max(0.0) * 0.5
}

pub fn widget_centered_y(rect_y: f32, rect_h: f32, widget_scale: f32) -> f32 {
    rect_y + (rect_h - widget_line_height(widget_scale)).max(0.0) * 0.5
}

pub fn char_width(scale: f32) -> f32 {
    widget_char_width(scale)
}

pub fn line_height(scale: f32) -> f32 {
    widget_line_height(scale)
}

pub fn text_width(text: &str) -> f32 {
    text.chars().count() as f32 * CHAR_WIDTH
}

pub fn text_width_scaled(text: &str, scale: f32) -> f32 {
    text.chars().count() as f32 * char_width(scale)
}

pub fn centered_x(text: &str, rect_x: f32, rect_w: f32) -> f32 {
    centered_x_scaled(text, rect_x, rect_w, 1.0)
}

pub fn centered_y(rect_y: f32, rect_h: f32) -> f32 {
    centered_y_scaled(rect_y, rect_h, 1.0)
}

pub fn centered_x_scaled(text: &str, rect_x: f32, rect_w: f32, scale: f32) -> f32 {
    rect_x + (rect_w - text_width_scaled(text, scale)).max(0.0) * 0.5
}

pub fn centered_y_scaled(rect_y: f32, rect_h: f32, scale: f32) -> f32 {
    rect_y + (rect_h - line_height(scale)).max(0.0) * 0.5
}

pub fn glyph_pixel_scale(scale: f32) -> f32 {
    widget_glyph_pixel(scale)
}

pub fn glyph_rows(ch: char) -> Option<[u8; 7]> {
    match ch.to_ascii_uppercase() {
        '0' => Some([0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110]),
        '1' => Some([0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
        '2' => Some([0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111]),
        '3' => Some([0b11110, 0b00001, 0b00010, 0b00110, 0b00001, 0b10001, 0b01110]),
        '4' => Some([0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010]),
        '5' => Some([0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110]),
        '6' => Some([0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110]),
        '7' => Some([0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000]),
        '8' => Some([0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110]),
        '9' => Some([0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100]),
        '-' => Some([0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000]),
        '.' => Some([0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b01100, 0b01100]),
        ':' => Some([0b00000, 0b01100, 0b01100, 0b00000, 0b01100, 0b01100, 0b00000]),
        '/' => Some([0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b00000, 0b00000]),
        'A' => Some([0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
        'B' => Some([0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110]),
        'C' => Some([0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110]),
        'D' => Some([0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110]),
        'E' => Some([0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111]),
        'F' => Some([0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000]),
        'G' => Some([0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01110]),
        'H' => Some([0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001]),
        'I' => Some([0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
        'J' => Some([0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100]),
        'K' => Some([0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001]),
        'L' => Some([0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111]),
        'M' => Some([0b10001, 0b11011, 0b10101, 0b10001, 0b10001, 0b10001, 0b10001]),
        'N' => Some([0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001]),
        'O' => Some([0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
        'P' => Some([0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000]),
        'R' => Some([0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001]),
        'S' => Some([0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110]),
        'T' => Some([0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100]),
        'U' => Some([0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
        'V' => Some([0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100]),
        'W' => Some([0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b11011, 0b10001]),
        'X' => Some([0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001]),
        'Y' => Some([0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100]),
        'Z' => Some([0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111]),
        _ => None,
    }
}
