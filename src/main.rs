extern crate failure;

use std::cmp;
use std::fs::File;

use nix::sys::statfs;

use colored::*;

mod errors;
use errors::*;

mod theme;
use theme::Theme;

mod mount;
use mount::*;

pub fn convert_bytes(num: f64) -> String {
    let units = ["B", "k", "M", "G", "T", "P", "E", "Z", "Y"];
    if num < 1_f64 {
        return format!("{}{}", num, units[0]);
    }
    let delimiter = 1024_f64;
    let exponent = cmp::min(
        (num.ln() / delimiter.ln()).floor() as i32,
        (units.len() - 1) as i32,
    );
    let pretty_bytes = format!("{:.*}", 1, num / delimiter.powi(exponent));
    let unit = units[exponent as usize];
    format!("{}{}", pretty_bytes, unit)
}

fn bar(width: usize, percentage: u8, theme: &Theme) -> String {
    let fill_len_total = (percentage as f32 / 100.0 * width as f32).ceil() as usize;
    let fill_len_low = std::cmp::min(
        fill_len_total,
        (width as f32 * theme.threshold_usage_medium / 100.0).ceil() as usize,
    );
    let fill_len_medium = std::cmp::min(
        fill_len_total,
        (width as f32 * theme.threshold_usage_high / 100.0).ceil() as usize,
    ) - fill_len_low;
    let fill_len_high = fill_len_total - fill_len_low - fill_len_medium;

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
        .color(theme.color_usage_low.unwrap_or(Color::White));

    format!(
        "{}{}{}{}{}{}",
        theme.char_bar_open, fill_low, fill_medium, fill_high, empty, theme.char_bar_close
    )
}

#[inline]
fn column_width(mnt: &Vec<MountEntry>, f: &dyn Fn(&MountEntry) -> usize, heading: &str) -> usize {
    mnt.iter()
        .map(f)
        .chain(std::iter::once(heading.len()))
        .max()
        .unwrap()
}

fn run() -> Result<()> {
    let theme = Theme::new();
    let f = File::open("/proc/self/mounts")?;
    let _accept_minimal = vec!["/dev*"];
    let _accept_more = vec!["dev", "run", "tmpfs", "/dev*"];
    let _accept_all = vec!["*"];
    let mounts_to_show = _accept_more;

    let bar_width = 20;

    let mnts = get_mounts(f)?;
    let mut mnts = mnts
        .into_iter()
        .filter(|m| {
            mounts_to_show.iter().any(|&x| {
                if x.ends_with("*") {
                    m.mnt_fsname.starts_with(&x[..x.len() - 1])
                } else {
                    m.mnt_fsname == x.to_string()
                }
            })
        })
        .collect::<Vec<_>>();

    for mnt in &mut mnts {
        let stat_opt = match statfs::statfs(&mnt.mnt_dir[..]) {
            Ok(stat) => Option::Some(stat),
            Err(_) => Option::None,
        };
        mnt.statfs = stat_opt;

        let (size, available) = match mnt.statfs {
            Some(stat) => (
                stat.blocks() * (stat.block_size() as u64),
                stat.blocks_available() * (stat.block_size() as u64),
            ),
            None => (0, 0),
        };

        mnt.used_percentage = 100.0 - available as f32 * 100.0 / std::cmp::max(size, 1) as f32;
        mnt.used = convert_bytes((size - available) as f64);
        mnt.available = convert_bytes(available as f64);
        mnt.size = convert_bytes(size as f64);
    }

    let color_heading = theme.color_heading.unwrap_or(Color::White);

    let label_fsname = "Filesystem";
    let label_type = "Type";
    let label_bar = "";
    let label_used_percentage = "Used%";
    let label_used = "Used";
    let label_available = "Avail";
    let label_size = "Size";
    let label_mounted = "Mounted on";

    let fsname_width = column_width(&mnts, &|m: &MountEntry| m.mnt_fsname.len(), label_fsname);
    let type_width = column_width(&mnts, &|m: &MountEntry| m.mnt_type.len(), label_type);
    let used_width = column_width(&mnts, &|m: &MountEntry| m.used.len(), label_used);
    let available_width = column_width(&mnts, &|m: &MountEntry| m.available.len(), label_available);
    let size_width = column_width(&mnts, &|m: &MountEntry| m.size.len(), label_size);

    println!(
        "{:<fsname_width$} {:<type_width$} {:<bar_width$} {:>6} {:>used_width$} {:>available_width$} {:>size_width$} {}",
        label_fsname.color(color_heading),
        label_type.color(color_heading),
        label_bar.color(color_heading),
        label_used_percentage.color(color_heading),
        label_used.color(color_heading),
        label_available.color(color_heading),
        label_size.color(color_heading),
        label_mounted.color(color_heading),
        fsname_width = fsname_width,
        type_width = type_width,
        bar_width = bar_width,
        used_width = used_width,
        available_width = available_width,
        size_width = size_width,
    );
    for mnt in mnts {
        let color_usage = match mnt.used_percentage {
            p if p >= theme.threshold_usage_high => theme.color_usage_high,
            p if p >= theme.threshold_usage_medium => theme.color_usage_medium,
            _ => theme.color_usage_low,
        }
        .unwrap_or(Color::White);
        println!(
            "{:<fsname_width$} {:<type_width$} {} {}% {:>used_width$} {:>available_width$} {:>size_width$} {}",
            mnt.mnt_fsname,
            mnt.mnt_type,
            bar(bar_width, mnt.used_percentage.ceil() as u8, &theme),
            format!("{:>5.1}", (mnt.used_percentage * 10.0).round() / 10.0).color(color_usage),
            mnt.used.color(color_usage),
            mnt.available.color(color_usage),
            mnt.size.color(color_usage),
            mnt.mnt_dir,
            fsname_width = fsname_width,
            type_width = type_width,
            used_width = used_width,
            available_width = available_width,
            size_width = size_width,
        );
    }

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {}", err);
        for cause in err.iter_chain().skip(1) {
            eprintln!("Because: {}", cause);
        }
        std::process::exit(1);
    }
}
