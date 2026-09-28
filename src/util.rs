//! Small, side-effect-free helpers shared by discovery and table rendering.
//!
//! These functions own the low-level details that are easy to get subtly
//! wrong in a terminal utility: unit scaling, percentage formatting, bounded
//! bars, mount-path matching, deterministic ordering, and overflow-safe
//! totals.

use crate::mount::Mount;
use crate::theme::Theme;

use colored::*;
use std::cmp;
use std::fmt;
use std::io::{self, stdout, Write};
use std::path::Path;

/// Format a byte or inode count using a decimal or binary unit delimiter.
pub fn format_count(num: f64, delimiter: f64) -> String {
    let units = ["B", "k", "M", "G", "T", "P", "E", "Z", "Y"];
    if num < 1_f64 {
        return format!("{}", num);
    }
    let exponent = cmp::min(num.log(delimiter).floor() as i32, (units.len() - 1) as i32);
    let pretty_bytes = format!("{:.*}", 1, num / delimiter.powi(exponent));
    let unit = units[exponent as usize];
    format!("{}{}", pretty_bytes, unit)
}

#[inline]
/// Format an optional percentage for a six-character table column.
pub fn format_percentage(percentage: Option<f32>) -> String {
    percentage.map_or_else(
        || format!("{:>6}", "-"),
        |percentage| {
            format!(
                "{:>5.1}{}",
                (percentage * 10.0).round() / 10.0,
                "%".color(Color::White)
            )
        },
    )
}

/// Render a bounded colored usage bar for the configured terminal theme.
pub fn bar(width: usize, percentage: Option<f32>, theme: &Theme) -> String {
    let normalized_percentage = percentage
        .filter(|value| value.is_finite())
        .map(|value| value.clamp(0.0, 100.0));
    let fill_len_total = (normalized_percentage.unwrap_or(0.0) / 100.0 * width as f32)
        .ceil()
        .min(width as f32) as usize;
    let fill_len_low = std::cmp::min(
        fill_len_total,
        (width as f32 * theme.threshold_usage_medium / 100.0).ceil() as usize,
    );
    let fill_len_medium = std::cmp::min(
        fill_len_total,
        (width as f32 * theme.threshold_usage_high / 100.0).ceil() as usize,
    ) - fill_len_low;
    let fill_len_high = fill_len_total.saturating_sub(fill_len_low + fill_len_medium);

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
        .repeat(width.saturating_sub(fill_len_total))
        .color(color_empty);

    format!(
        "{}{}{}{}{}{}",
        theme.char_bar_open, fill_low, fill_medium, fill_high, empty, theme.char_bar_close
    )
}

/// Convert an escaped device-mapper name into a shorter `/dev/VG/LV` label.
pub fn lvm_alias(device: &str) -> Option<String> {
    if !device.starts_with("/dev/mapper/") {
        return None;
    }
    let device = &device["/dev/mapper/".len()..].replace("--", "$$");
    if !device.contains('-') {
        return None;
    }
    let mut it = device.splitn(2, '-');
    let vg = it.next().unwrap_or("");
    let lv = it.next().unwrap_or("");
    Some(format!("/dev/{}/{}", vg, lv).replace("$$", "-"))
}

#[inline]
/// Return the deepest mount whose directory is a path prefix.
pub fn get_best_mount_match<'a>(path: &Path, mnts: &'a [Mount]) -> Option<&'a Mount> {
    let scores = mnts
        .iter()
        .map(|mnt| (calculate_path_match_score(path, mnt), mnt));
    let best = scores.filter(|x| x.0 > 0).max_by_key(|x| x.0)?;
    Some(best.1)
}

#[inline]
/// Score a mount by the length of its directory path prefix.
pub fn calculate_path_match_score(path: &Path, mnt: &Mount) -> usize {
    if path.starts_with(Path::new(&mnt.mnt_dir)) {
        mnt.mnt_dir.len()
    } else {
        0
    }
}

#[inline]
/// Order records by inspectability, then by mount directory.
pub fn cmp_by_capacity_and_dir_name(a: &Mount, b: &Mount) -> cmp::Ordering {
    a.capacity
        .min(1)
        .cmp(&b.capacity.min(1))
        .reverse()
        .then(a.mnt_dir.cmp(&b.mnt_dir))
}

#[inline]
/// Match an exact filesystem source or a source prefix ending in `*`.
pub fn mnt_matches_filter(mnt: &Mount, filter: &str) -> bool {
    filter.strip_suffix('*').map_or_else(
        || mnt.mnt_fsname == filter,
        |start| mnt.mnt_fsname.starts_with(start),
    )
}

#[inline]
/// Sum a report's counters without allowing a `u64` wraparound.
pub fn calc_total(mnts: &[Mount]) -> Mount {
    let mut total = Mount::named("total".to_string());

    total.free = mnts
        .iter()
        .fold(0_u64, |sum, mnt| sum.saturating_add(mnt.free));
    total.used = mnts
        .iter()
        .fold(0_u64, |sum, mnt| sum.saturating_add(mnt.used));
    total.capacity = mnts
        .iter()
        .fold(0_u64, |sum, mnt| sum.saturating_add(mnt.capacity));

    total
}

#[inline]
/// Write formatted output while allowing a broken pipe to stop the report.
pub fn try_print(args: fmt::Arguments) -> io::Result<()> {
    stdout().write_fmt(args)
}

#[macro_export]
macro_rules! try_print {
    ($($arg:tt)*) => ($crate::try_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! try_println {
    ($fmt:expr) => (try_print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (try_print!(concat!($fmt, "\n"), $($arg)*));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn format_count_zero() {
        let s = format_count(0.0, 1024.0);
        assert_eq!(s, "0");
    }

    #[test]
    fn format_count_bytes() {
        let s = format_count(12.0, 1024.0);
        assert_eq!(s, "12.0B");
    }

    #[test]
    fn format_count_megabyte() {
        let s = format_count(12693000.0, 1024.0);
        assert_eq!(s, "12.1M");
    }

    #[test]
    fn format_count_very_large() {
        let s = format_count(2535301200456458802993406410752.0, 1024.0);
        assert_eq!(s, "2097152.0Y");
    }

    #[test]
    fn format_percentage_zero() {
        let s = format_percentage(Option::Some(0f32));
        assert_eq!(s, format!("  0.0{}", "%".color(Color::White)));
    }

    #[test]
    fn format_percentage_fraction() {
        let s = format_percentage(Option::Some(0.333_333_34_f32));
        assert_eq!(s, format!("  0.3{}", "%".color(Color::White)));
    }

    #[test]
    fn format_percentage_hundred() {
        let s = format_percentage(Option::Some(100f32));
        assert_eq!(s, format!("100.0{}", "%".color(Color::White)));
    }

    #[test]
    fn format_percentage_none() {
        let s = format_percentage(Option::None);
        assert_eq!(s, "     -");
    }

    #[test]
    fn lvm_alias_none() {
        let s = lvm_alias("/dev/mapper/crypto");
        assert_eq!(s, None);
    }

    #[test]
    fn lvm_alias_simple() {
        let s = lvm_alias("/dev/mapper/crypto-foo");
        assert_eq!(s, Some("/dev/crypto/foo".to_string()));
    }

    #[test]
    fn lvm_alias_two_dashes() {
        let s = lvm_alias("/dev/mapper/crypto--foo");
        assert_eq!(s, None);
    }

    #[test]
    fn lvm_alias_three_dashes() {
        let s = lvm_alias("/dev/mapper/crypto---foo");
        assert_eq!(s, Some("/dev/crypto-/foo".to_string()));
    }

    #[test]
    fn lvm_alias_four_dashes() {
        let s = lvm_alias("/dev/mapper/crypto----foo");
        assert_eq!(s, None);
    }

    #[test]
    fn lvm_alias_five_dashes() {
        let s = lvm_alias("/dev/mapper/crypto-----foo");
        assert_eq!(s, Some("/dev/crypto--/foo".to_string()));
    }

    #[test]
    fn get_best_mount_match_simple() {
        let mut mnt1 = Mount::named("foo".into());
        mnt1.mnt_dir = "/a".to_string();
        let mut mnt2 = Mount::named("bar".into());
        mnt2.mnt_dir = "/a/b".to_string();
        let mut mnt3 = Mount::named("fizz".into());
        mnt3.mnt_dir = "/a/b/c".to_string();
        let mut mnt4 = Mount::named("buzz".into());
        mnt4.mnt_dir = "/a/b/c/d".to_string();

        let mnts = &[mnt1, mnt2, mnt3, mnt4];
        let matched = get_best_mount_match(&PathBuf::from("/a/b/c"), mnts).unwrap();
        assert_eq!(matched.mnt_dir, "/a/b/c");
    }

    #[test]
    fn calculate_path_match_score_simple() {
        let mut mnt1 = Mount::named("foo".into());
        mnt1.mnt_dir = "/a/s/d".to_string();
        let score = calculate_path_match_score(&PathBuf::from("/a/s/d/f"), &mnt1);
        assert_eq!(score, 6);
    }

    #[test]
    fn cmp_by_capacity_and_dir_name_equal() {
        let mnt1 = Mount::named("foo".into());
        let mnt2 = Mount::named("bar".into());

        let ord = cmp_by_capacity_and_dir_name(&mnt1, &mnt2);
        assert_eq!(ord, cmp::Ordering::Equal);
    }

    #[test]
    fn cmp_by_capacity_and_dir_name_greater_capacity_greater_name() {
        let mut mnt1 = Mount::named("foo".into());
        mnt1.capacity = 123;
        mnt1.mnt_dir = "/b".to_string();
        let mut mnt2 = Mount::named("bar".into());
        mnt2.capacity = 64;
        mnt2.mnt_dir = "/a".to_string();

        let ord = cmp_by_capacity_and_dir_name(&mnt1, &mnt2);
        assert_eq!(ord, cmp::Ordering::Greater);
    }

    #[test]
    fn cmp_by_capacity_and_dir_name_smaller_capacity_greater_name() {
        let mut mnt1 = Mount::named("foo".into());
        mnt1.capacity = 64;
        mnt1.mnt_dir = "/b".to_string();
        let mut mnt2 = Mount::named("bar".into());
        mnt2.capacity = 123;
        mnt2.mnt_dir = "/a".to_string();

        let ord = cmp_by_capacity_and_dir_name(&mnt1, &mnt2);
        assert_eq!(ord, cmp::Ordering::Greater);
    }

    #[test]
    fn cmp_by_capacity_and_dir_name_greater_capacity_smaller_name() {
        let mut mnt1 = Mount::named("foo".into());
        mnt1.capacity = 123;
        mnt1.mnt_dir = "/a".to_string();
        let mut mnt2 = Mount::named("bar".into());
        mnt2.capacity = 64;
        mnt2.mnt_dir = "/b".to_string();

        let ord = cmp_by_capacity_and_dir_name(&mnt1, &mnt2);
        assert_eq!(ord, cmp::Ordering::Less);
    }

    #[test]
    fn cmp_by_capacity_and_dir_name_smaller_capacity_smaller_name() {
        let mut mnt1 = Mount::named("foo".into());
        mnt1.capacity = 64;
        mnt1.mnt_dir = "/a".to_string();
        let mut mnt2 = Mount::named("bar".into());
        mnt2.capacity = 123;
        mnt2.mnt_dir = "/b".to_string();

        let ord = cmp_by_capacity_and_dir_name(&mnt1, &mnt2);
        assert_eq!(ord, cmp::Ordering::Less);
    }

    #[test]
    fn cmp_by_capacity_and_dir_name_equal_capacity_greater_name() {
        let mut mnt1 = Mount::named("foo".into());
        mnt1.capacity = 123;
        mnt1.mnt_dir = "/b".to_string();
        let mut mnt2 = Mount::named("bar".into());
        mnt2.capacity = 123;
        mnt2.mnt_dir = "/a".to_string();

        let ord = cmp_by_capacity_and_dir_name(&mnt1, &mnt2);
        assert_eq!(ord, cmp::Ordering::Greater);
    }

    #[test]
    fn cmp_by_capacity_and_dir_name_greater_capacity_equal_name() {
        let mut mnt1 = Mount::named("foo".into());
        mnt1.capacity = 123;
        mnt1.mnt_dir = "/a".to_string();
        let mut mnt2 = Mount::named("bar".into());
        mnt2.capacity = 64;
        mnt2.mnt_dir = "/a".to_string();

        let ord = cmp_by_capacity_and_dir_name(&mnt1, &mnt2);
        assert_eq!(ord, cmp::Ordering::Equal);
    }

    #[test]
    fn calc_total_simple() {
        let mut mnt1 = Mount::named("foo".into());
        mnt1.free = 123;
        mnt1.used = 456;
        mnt1.capacity = 123 + 456;
        let mut mnt2 = Mount::named("bar".into());
        mnt2.free = 678;
        mnt2.used = 9123;
        mnt2.capacity = 678 + 9123;
        let mut mnt3 = Mount::named("fizz".into());
        mnt3.free = 4567;
        mnt3.used = 0;
        mnt3.capacity = 4567;
        let mut mnt4 = Mount::named("buzz".into());
        mnt4.free = 0;
        mnt4.used = 890123;
        mnt4.capacity = 890123;

        let total = calc_total(&[mnt1, mnt2, mnt3, mnt4]);
        assert_eq!(total.mnt_fsname, "total");
        assert_eq!(total.free, 5368);
        assert_eq!(total.used, 899702);
        assert_eq!(total.capacity, 5368 + 899702);
    }
}
