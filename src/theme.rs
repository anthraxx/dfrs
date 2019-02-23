use colored::*;

pub struct Theme {
    pub char_bar_filled: char,
    pub char_bar_empty: char,
    pub color_headline: Option<Color>,
    pub color_usage_low: Option<Color>,
    pub color_usage_medium: Option<Color>,
    pub color_usage_critical: Option<Color>,
}

impl Theme {
    pub fn new() -> Theme {
        Theme {
            char_bar_filled: named_char::LIGHT_BOX,
            char_bar_empty: named_char::HEAVY_DOUBLE_DASH,
            color_headline: Some(Color::Blue),
            color_usage_low: Some(Color::Green),
            color_usage_medium: Some(Color::Yellow),
            color_usage_critical: Some(Color::Red),
        }
    }
}

#[allow(dead_code)]
pub mod named_char {
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
}
