extern crate anyhow;
extern crate strum;
extern crate strum_macros;

use args::*;
mod args;

mod errors;
use errors::*;

mod theme;
use theme::Theme;

mod mount;
use mount::*;

use std::cmp;
use std::fs::File;

use nix::sys::statfs;

use env_logger::Env;

use colored::*;
use structopt::StructOpt;
use anyhow::Result;

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

fn bar(width: usize, percentage: Option<f32>, theme: &Theme) -> String {
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
        None => theme.color_usage_void
    }.unwrap_or(Color::Green);

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

#[inline]
fn column_width(mnt: &[&MountEntry], f: &dyn Fn(&&MountEntry) -> usize, heading: &str) -> usize {
    mnt.iter()
        .map(f)
        .chain(std::iter::once(heading.len()))
        .max()
        .unwrap()
}

fn display_mounts(mnts: &[&MountEntry], theme: &Theme, inodes_mode: bool) {
    let bar_width = 20;
    let color_heading = theme.color_heading.unwrap_or(Color::White);

    let label_fsname = "Filesystem";
    let label_type = "Type";
    let label_bar = "";
    let label_used_percentage = "Used%";
    let label_used = "Used";
    let label_available = "Avail";
    let label_capacity = if inodes_mode { "Inodes" } else { "Size" };
    let label_mounted = "Mounted on";

    let fsname_width = column_width(&mnts, &|m: &&MountEntry| m.mnt_fsname.len(), label_fsname);
    let type_width = column_width(&mnts, &|m: &&MountEntry| m.mnt_type.len(), label_type);
    let used_width = column_width(&mnts, &|m: &&MountEntry| m.used_formatted.len(), label_used);
    let available_width = column_width(&mnts, &|m: &&MountEntry| m.free_formatted.len(), label_available);
    let capacity_width = column_width(&mnts, &|m: &&MountEntry| m.capacity_formatted.len(), label_capacity);

    println!(
        "{:<fsname_width$} {:<type_width$} {:<bar_width$} {:>6} {:>used_width$} {:>available_width$} {:>capacity_width$} {}",
        label_fsname.color(color_heading),
        label_type.color(color_heading),
        label_bar.color(color_heading),
        label_used_percentage.color(color_heading),
        label_used.color(color_heading),
        label_available.color(color_heading),
        label_capacity.color(color_heading),
        label_mounted.color(color_heading),
        fsname_width = fsname_width,
        type_width = type_width,
        bar_width = bar_width,
        used_width = used_width,
        available_width = available_width,
        capacity_width = capacity_width,
    );
    for mnt in mnts {
        let color_usage = match mnt.used_percentage {
            Some(p) if p >= theme.threshold_usage_high => theme.color_usage_high,
            Some(p) if p >= theme.threshold_usage_medium => theme.color_usage_medium,
            Some(_) => theme.color_usage_low,
            _ => theme.color_usage_void,
        }.unwrap_or(Color::White);

        let used_percentage = match mnt.used_percentage {
            Some(percentage) => format!("{:>5.1}{}", (percentage * 10.0).round() / 10.0, "%".color(Color::White)),
            None => format!("{:>6}", "-")
        }.color(color_usage);

        println!(
            "{:<fsname_width$} {:<type_width$} {} {} {:>used_width$} {:>available_width$} {:>size_width$} {}",
            mnt.mnt_fsname,
            mnt.mnt_type,
            bar(bar_width, mnt.used_percentage, &theme),
            used_percentage,
            mnt.used_formatted.color(color_usage),
            mnt.free_formatted.color(color_usage),
            mnt.capacity_formatted.color(color_usage),
            mnt.mnt_dir,
            fsname_width = fsname_width,
            type_width = type_width,
            used_width = used_width,
            available_width = available_width,
            size_width = capacity_width,
        );
    }
}

fn run(args: Args) -> Result<()> {
    if let Some(color) = args.color {
        debug!("Bypass tty detection for colors: {:?}", color);
        match color {
            ColorOpt::Auto => {},
            ColorOpt::Always => {
                colored::control::set_override(true);
            },
            ColorOpt::Never => {
                colored::control::set_override(false);
            },
        }
    }

    if args.color_always {
        debug!("Bypass tty detection for colors: always");
        colored::control::set_override(true);
    }

    match args.subcommand {
        Some(SubCommand::Completions(completions)) => args::gen_completions(&completions)?,
        _ => {
            let theme = Theme::new();
            let f = File::open("/proc/self/mounts")?;
            let delimiter = if args.base10 { NumberFormat::Base10 } else { NumberFormat::Base2 };

            let mounts_to_show = if args.all {
                DisplayFilter::All
            } else if args.more {
                DisplayFilter::More
            } else {
                DisplayFilter::from_u8(args.display)
            };

            let mnts = get_mounts(f)?;
            let mut mnts:Vec<MountEntry> = mnts
                .into_iter()
                .filter(|mount| {
                    mounts_to_show.get_mnt_fsname_filter().iter().any(|&fsname| {
                        if fsname.ends_with('*') {
                            mount.mnt_fsname.starts_with(&fsname[..fsname.len() - 1])
                        } else {
                            mount.mnt_fsname == fsname
                        }
                    })
                })
                .collect::<Vec<_>>();

            for mnt in &mut mnts {
                mnt.statfs = statfs::statfs(&mnt.mnt_dir[..]).ok();

                let (capacity, free) = match mnt.statfs {
                    Some(stat) => {
                        if args.inodes {
                            (stat.files(), stat.files_free())
                        } else {
                            (
                                stat.blocks() * (stat.block_size() as u64),
                                stat.blocks_available() * (stat.block_size() as u64)
                            )
                        }
                    },
                    None => (0, 0),
                };

                mnt.capacity = capacity;
                mnt.free = free;
                mnt.used = capacity - free;

                mnt.capacity_formatted = format_count(mnt.capacity as f64, delimiter.get_powers_of());
                mnt.free_formatted = format_count(mnt.free as f64, delimiter.get_powers_of());
                mnt.used_formatted = format_count(mnt.used as f64, delimiter.get_powers_of());

                if capacity > 0 {
                    mnt.used_percentage = Some(100.0 - free as f32 * 100.0 / capacity as f32);
                }
            }

            if args.paths.is_empty() {
                mnts.sort_by(|a, b| {
                    u64::min(1, a.capacity)
                        .cmp(&u64::min(1, b.capacity))
                        .reverse()
                        .then(a.mnt_dir.cmp(&b.mnt_dir))
                });
                let mnts: &[&MountEntry] = &mnts.iter().collect::<Vec<_>>();
                display_mounts(&mnts, &theme, args.inodes);
            } else {
                let mnts: &[&MountEntry] = &args.paths.iter().filter_map(|path| {
                    path.canonicalize().map_err(|error|
                        eprintln!("dfrs: {}: {}", path.clone().into_os_string().to_string_lossy(), error)
                    ).ok()
                }).map(|path| {
                    debug!("{:?}", path);
                    mnts.iter().map(|mnt| {
                        (if path.starts_with(&mnt.mnt_dir) { mnt.mnt_dir.len() } else { 0 }, mnt)
                    }).max_by_key(|x| x.0).map(|x| x.1)
                }).filter_map(|opt| opt).collect::<Vec<_>>();

                display_mounts(&mnts, &theme, args.inodes);
            }
        }
    }

    Ok(())
}

fn main() {
    let args = Args::from_args();

    let logging = if args.verbose {
        "debug"
    } else {
        "info"
    };

    env_logger::init_from_env(Env::default()
        .default_filter_or(logging));

    if let Err(err) = run(args) {
        eprintln!("Error: {}", err);
        for cause in err.chain().skip(1) {
            eprintln!("Because: {}", cause);
        }
        std::process::exit(1);
    }
}
