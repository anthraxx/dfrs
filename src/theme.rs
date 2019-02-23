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
            char_bar_filled: '█',
            char_bar_empty: '─',
            color_headline: Some(Color::Blue),
            color_usage_low: Some(Color::Green),
            color_usage_medium: Some(Color::Yellow),
            color_usage_critical: Some(Color::Red),
        }
    }
}
