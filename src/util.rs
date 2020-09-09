use crate::theme::Theme;

use colored::*;
use std::cmp;

pub fn format_count(num: f64, delimiter: f64) -> String {
    let units = ["B", "k", "M", "G", "T", "P", "E", "Z", "Y"];
    if num < 1_f64 {
        return format!("{}", num);
    }
    let exponent = cmp::min(
        (num.ln() / delimiter.ln()).floor() as i32,
        (units.len() - 1) as i32,
    );
    let pretty_bytes = format!("{:.*}", 1, num / delimiter.powi(exponent));
    let unit = units[exponent as usize];
    format!("{}{}", pretty_bytes, unit)
}

pub fn bar(width: usize, percentage: Option<f32>, theme: &Theme) -> String {
    let fill_len_total = (percentage.unwrap_or(0.0) as f32 / 100.0 * width as f32).ceil() as usize;
    let fill_len_low = std::cmp::min(
        fill_len_total,
        (width as f32 * theme.threshold_usage_medium / 100.0).ceil() as usize,
    );
    let fill_len_medium = std::cmp::min(
        fill_len_total,
        (width as f32 * theme.threshold_usage_high / 100.0).ceil() as usize,
    ) - fill_len_low;
    let fill_len_high = fill_len_total - fill_len_low - fill_len_medium;

    let color_empty = match percentage {
        Some(_) => theme.color_usage_low,
        None => theme.color_usage_void,
    }
    .unwrap_or(Color::Green);

    let fill_low = theme
        .char_bar_filled
        .to_string()
        .repeat(fill_len_low)
        .color(theme.color_usage_low.unwrap_or(Color::Green));
    let fill_medium = theme
        .char_bar_filled
        .to_string()
        .repeat(fill_len_medium)
        .color(theme.color_usage_medium.unwrap_or(Color::Yellow));
    let fill_high = theme
        .char_bar_filled
        .to_string()
        .repeat(fill_len_high)
        .color(theme.color_usage_high.unwrap_or(Color::Red));
    let empty = theme
        .char_bar_empty
        .to_string()
        .repeat(width - fill_len_total)
        .color(color_empty);

    format!(
        "{}{}{}{}{}{}",
        theme.char_bar_open, fill_low, fill_medium, fill_high, empty, theme.char_bar_close
    )
}

pub fn lvm_alias(device: &str) -> Option<String> {
    if !device.starts_with("/dev/mapper/") {
        return None;
    }
    let device = &device["/dev/mapper/".len()..].replace("--", "$$");
    if !device.contains("-") {
        return None;
    }
    let mut it = device.splitn(2, "-");
    let vg = it.next().unwrap_or("");
    let lv = it.next().unwrap_or("");
    Some(format!("/dev/{}/{}", vg, lv).replace("$$", "-"))
}
