use colored::*;

pub struct Theme {
    pub char_bar_filled: char,
    pub char_bar_empty: char,
    pub char_bar_open: String,
    pub char_bar_close: String,
    pub threshold_usage_medium: f32,
    pub threshold_usage_high: f32,
    pub color_heading: Option<Color>,
    pub color_usage_low: Option<Color>,
    pub color_usage_medium: Option<Color>,
    pub color_usage_high: Option<Color>,
    pub color_usage_void: Option<Color>,
}

impl Theme {
    pub fn new() -> Theme {
        Theme {
            char_bar_filled: named_char::HEAVY_BOX,
            char_bar_empty: named_char::HEAVY_DOUBLE_DASH,
            char_bar_open: "".to_string(),
            char_bar_close: "".to_string(),
            threshold_usage_medium: 50.0,
            threshold_usage_high: 75.0,
            color_heading: Some(Color::Blue),
            color_usage_low: Some(Color::Green),
            color_usage_medium: Some(Color::Yellow),
            color_usage_high: Some(Color::Red),
            color_usage_void: Some(Color::Blue),
        }
    }
}

#[allow(dead_code)]
pub mod named_char {
    pub const SPACE: char = ' ';
    pub const EQUAL: char = '=';
    pub const HASHTAG: char = '#';
    pub const ASTERISK: char = '*';
    pub const LIGHT_BOX: char = '■';
    pub const HEAVY_BOX: char = '▇';
    pub const PERIOD: char = '.';
    pub const DASH: char = '-';
    pub const LONG_DASH: char = '—';
    pub const LIGHT_HORIZONTAL: char = '─';
    pub const HEAVY_HORIZONTAL: char = '━';
    pub const LIGHT_DOUBLE_DASH: char = '╌';
    pub const HEAVY_DOUBLE_DASH: char = '╍';
    pub const ELLIPSIS: char = '…';
    pub const SQUARE_BRACKET_OPEN: char = '[';
    pub const SQUARE_BRACKET_CLOSE: char = ']';
    pub const LIGHT_VERTICAL: char = '│';
    pub const LIGHT_VERTICAL_OPEN: char = '├';
    pub const LIGHT_VERTICAL_CLOSE: char = '┤';
}
