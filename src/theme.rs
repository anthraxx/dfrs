//! Terminal rendering theme and glyph choices.
//!
//! The theme is intentionally code-defined because this release exposes a
//! stable CLI rather than a user configuration file. Keeping thresholds and
//! colors together makes the renderer's visual contract easy to inspect in
//! the generated RustDoc.

use crate::args::ColumnType;
use colored::*;

/// Visual policy used by the table renderer.
pub struct Theme {
    /// Glyph used for filled bar cells.
    pub char_bar_filled: char,
    /// Glyph used for empty bar cells.
    pub char_bar_empty: char,
    /// Optional opening decoration for the bar.
    pub char_bar_open: String,
    /// Optional closing decoration for the bar.
    pub char_bar_close: String,
    /// Percentage at which usage becomes medium severity.
    pub threshold_usage_medium: f32,
    /// Percentage at which usage becomes high severity.
    pub threshold_usage_high: f32,
    /// Heading color, if color output is enabled.
    pub color_heading: Option<Color>,
    /// Low-usage color.
    pub color_usage_low: Option<Color>,
    /// Medium-usage color.
    pub color_usage_medium: Option<Color>,
    /// High-usage color.
    pub color_usage_high: Option<Color>,
    /// Color for records without a usable percentage.
    pub color_usage_void: Option<Color>,
    /// Number of cells in a usage bar.
    pub bar_width: usize,
    /// Columns rendered from left to right.
    pub columns: Vec<ColumnType>,
}

impl Theme {
    /// Construct the default dfrs color and column policy.
    pub fn new() -> Self {
        Self {
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
            bar_width: 20,
            columns: vec![
                ColumnType::Filesystem,
                ColumnType::Type,
                ColumnType::Bar,
                ColumnType::UsedPercentage,
                ColumnType::Available,
                ColumnType::Used,
                ColumnType::Capacity,
                ColumnType::MountedOn,
            ],
        }
    }
}

#[allow(dead_code)]
/// Named Unicode glyphs used by the terminal theme.
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
