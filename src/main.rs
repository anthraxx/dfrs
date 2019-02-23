extern crate failure;

use std::fs::File;
use std::mem;

use nix::sys::statfs;

use colored::*;

mod errors;
use errors::*;

mod theme;
use theme::Theme;

mod mount;
use mount::*;

fn bar(width: usize, percentage: u8, theme: &Theme) -> String {
    let filled = (percentage as f32 / 100.0 * (width - 2) as f32).ceil() as usize;
    let fill = theme
        .char_bar_filled
        .to_string()
        .repeat(filled)
        .color(theme.color_usage_low.unwrap_or(Color::White));
    let empty = theme
        .char_bar_empty
        .to_string()
        .repeat(width - 2 - filled)
        .color(theme.color_usage_low.unwrap_or(Color::White));
    format!("[{}{}]", fill, empty)
}

fn run() -> Result<()> {
    let theme = Theme::new();
    let f = File::open("/proc/self/mounts")?;
    let _accept_minimal = vec!["/dev*"];
    let _accept_more = vec!["dev", "run", "tmpfs", "/dev*"];

    let bar_width = 22;

    let mnts = get_mounts(f)?;
    let mut mnts = mnts
        .into_iter()
        .filter(|m| {
            _accept_more.iter().any(|&x| {
                if x.ends_with("*") {
                    m.mnt_fsname.starts_with(&x[..x.len() - 1])
                } else {
                    m.mnt_fsname == x
                }
            })
        })
        .collect::<Vec<_>>();

    for mnt in &mut mnts {
        let mut stat = unsafe { mem::uninitialized() };
        let stat_opt = match statfs::statfs(&mnt.mnt_dir[..], &mut stat) {
            Ok(_) => Option::Some(stat),
            Err(_) => Option::None,
        };
        mnt.statfs = stat_opt;

        let (size, available) = match mnt.statfs {
            Some(stat) => (
                stat.f_blocks * (stat.f_frsize as u64),
                stat.f_bavail * (stat.f_frsize as u64),
            ),
            None => (0, 0),
        };

        mnt.used_percentage = 100.0 - available as f32 * 100.0 / size as f32;
        mnt.used = format!("{}", (size - available) / 1024 / 1);
        mnt.available = format!("{}", available / 1024 / 1);
        mnt.size = format!("{}", size / 1024 / 1);
    }

    let label_fsname = "Filesystem";
    let label_type = "Type";
    let label_bar = "";
    let label_used_percentage = "Used%";
    let label_used = "Used";
    let label_available = "Avail";
    let label_size = "Size";
    let label_mounted = "Mounted on";

    let fsname_width = mnts
        .iter()
        .map(|m| m.mnt_fsname.len())
        .chain(std::iter::once(label_fsname.len()))
        .max()
        .unwrap_or(label_fsname.len());

    let type_width = mnts
        .iter()
        .map(|m| m.mnt_type.len())
        .chain(std::iter::once(label_type.len()))
        .max()
        .unwrap_or(label_type.len());

    let used_width = mnts
        .iter()
        .map(|m| m.used.len())
        .chain(std::iter::once(label_used.len()))
        .max()
        .unwrap_or(label_used.len());

    let available_width = mnts
        .iter()
        .map(|m| m.available.len())
        .chain(std::iter::once(label_available.len()))
        .max()
        .unwrap_or(label_available.len());

    let size_width = mnts
        .iter()
        .map(|m| m.size.len())
        .chain(std::iter::once(label_size.len()))
        .max()
        .unwrap_or(label_size.len());

    println!(
        "{:<fsname_width$} {:<type_width$} {:<bar_width$} {:>6} {:>used_width$} {:>available_width$} {:>size_width$} {}",
        label_fsname.color(theme.color_headline.unwrap_or(Color::White)),
        label_type.color(theme.color_headline.unwrap_or(Color::White)),
        label_bar.color(theme.color_headline.unwrap_or(Color::White)),
        label_used_percentage.color(theme.color_headline.unwrap_or(Color::White)),
        label_used.color(theme.color_headline.unwrap_or(Color::White)),
        label_available.color(theme.color_headline.unwrap_or(Color::White)),
        label_size.color(theme.color_headline.unwrap_or(Color::White)),
        label_mounted.color(theme.color_headline.unwrap_or(Color::White)),
        fsname_width = fsname_width,
        type_width = type_width,
        bar_width = bar_width,
        used_width = used_width,
        available_width = available_width,
        size_width = size_width,
    );
    for mnt in mnts {
        println!(
            "{:<fsname_width$} {:<type_width$} {} {}% {:>used_width$} {:>available_width$} {:>size_width$} {}",
            mnt.mnt_fsname,
            mnt.mnt_type,
            bar(bar_width, mnt.used_percentage.ceil() as u8, &theme),
            format!("{:>5.1}", (mnt.used_percentage * 10.0).round() / 10.0)
                .color(theme.color_usage_low.unwrap_or(Color::White)),
            mnt.used
                .color(theme.color_usage_low.unwrap_or(Color::White)),
            mnt.available
                .color(theme.color_usage_low.unwrap_or(Color::White)),
            mnt.size
                .color(theme.color_usage_low.unwrap_or(Color::White)),
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
