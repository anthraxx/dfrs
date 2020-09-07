#![deny(
    clippy::all,
    clippy::restriction,
    clippy::nursery,
    clippy::pedantic,
    clippy::cargo
)]
#![allow(clippy::print_stdout)]
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

mod util;
use util::bar;

use std::cmp;
use std::fs::File;
use std::path::{Path, PathBuf};

use nix::sys::statfs;

use env_logger::Env;

use anyhow::Result;
use colored::*;
use structopt::StructOpt;

#[inline]
fn column_width<F>(mnt: &[Mount], f: F, heading: &str) -> usize
where
    F: Fn(&Mount) -> usize,
{
    mnt.iter()
        .map(f)
        .chain(std::iter::once(heading.len()))
        .max()
        .unwrap()
}

fn display_mounts(mnts: &[Mount], theme: &Theme, delimiter: &NumberFormat, inodes_mode: bool) {
    let bar_width = 20;
    let color_heading = theme.color_heading.unwrap_or(Color::White);

    let label_fsname = "Filesystem";
    let label_type = "Type";
    let label_bar = "";
    let label_used_percentage = "Used%";
    let label_available = "Avail";
    let label_used = "Used";
    let label_capacity = if inodes_mode { "Inodes" } else { "Size" };
    let label_mounted = "Mounted on";

    let fsname_width = column_width(&mnts, |m| m.mnt_fsname.len(), label_fsname);
    let type_width = column_width(&mnts, |m| m.mnt_type.len(), label_type);
    let available_width = column_width(
        &mnts,
        |m| m.free_formatted(delimiter).len(),
        label_available,
    );
    let used_width = column_width(&mnts, |m| m.used_formatted(delimiter).len(), label_used);
    let capacity_width = column_width(
        &mnts,
        |m| m.capacity_formatted(delimiter).len(),
        label_capacity,
    );

    println!(
        "{:<fsname_width$} {:<type_width$} {:<bar_width$} {:>6} {:>used_width$} {:>available_width$} {:>capacity_width$} {}",
        label_fsname.color(color_heading),
        label_type.color(color_heading),
        label_bar.color(color_heading),
        label_used_percentage.color(color_heading),
        label_available.color(color_heading),
        label_used.color(color_heading),
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
        let usage_color = mnt.usage_color(&theme);

        let used_percentage = match mnt.used_percentage() {
            Some(percentage) => format!(
                "{:>5.1}{}",
                (percentage * 10.0).round() / 10.0,
                "%".color(Color::White)
            ),
            None => format!("{:>6}", "-"),
        }
        .color(usage_color);

        println!(
            "{:<fsname_width$} {:<type_width$} {} {} {:>used_width$} {:>available_width$} {:>size_width$} {}",
            mnt.mnt_fsname,
            mnt.mnt_type,
            bar(bar_width, mnt.used_percentage(), &theme),
            used_percentage,
            mnt.free_formatted(delimiter).color(usage_color),
            mnt.used_formatted(delimiter).color(usage_color),
            mnt.capacity_formatted(delimiter).color(usage_color),
            mnt.mnt_dir,
            fsname_width = fsname_width,
            type_width = type_width,
            used_width = used_width,
            available_width = available_width,
            size_width = capacity_width,
        );
    }
}

#[inline]
fn get_best_mount_match<'a>(path: &Path, mnts: &'a [Mount]) -> Option<&'a Mount> {
    let scores = mnts
        .iter()
        .map(|mnt| (calculate_path_match_score(path, &mnt), mnt));
    let best = scores.max_by_key(|x| x.0)?;
    Some(best.1)
}

#[inline]
fn calculate_path_match_score(path: &Path, mnt: &Mount) -> usize {
    if path.starts_with(&mnt.mnt_dir) {
        mnt.mnt_dir.len()
    } else {
        0
    }
}

#[inline]
fn cmp_by_capacity_and_dir_name(a: &Mount, b: &Mount) -> cmp::Ordering {
    u64::min(1, a.capacity)
        .cmp(&u64::min(1, b.capacity))
        .reverse()
        .then(a.mnt_dir.cmp(&b.mnt_dir))
}

#[inline]
fn mnt_matches_filter(mnt: &Mount, filter: &str) -> bool {
    if filter.ends_with('*') {
        mnt.mnt_fsname.starts_with(&filter[..filter.len() - 1])
    } else {
        mnt.mnt_fsname == filter
    }
}

#[inline]
fn calc_total(mnts: &[Mount]) -> Mount {
    let mut total = Mount::named("total".to_string());

    total.free = mnts.iter().map(|mnt| mnt.free).sum();
    total.used = mnts.iter().map(|mnt| mnt.used).sum();
    total.capacity = mnts.iter().map(|mnt| mnt.capacity).sum();

    total
}

fn run(args: Args) -> Result<()> {
    if let Some(color) = args.color {
        debug!("Bypass tty detection for colors: {:?}", color);
        match color {
            ColorOpt::Auto => {}
            ColorOpt::Always => {
                colored::control::set_override(true);
            }
            ColorOpt::Never => {
                colored::control::set_override(false);
            }
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
            let delimiter = if args.base10 {
                NumberFormat::Base10
            } else {
                NumberFormat::Base2
            };
            let mounts_to_show = if args.all {
                DisplayFilter::All
            } else if args.more {
                DisplayFilter::More
            } else {
                DisplayFilter::from_u8(args.display)
            };

            let mut mnts = get_mounts(&mounts_to_show, args.inodes, &args.paths)?;
            if args.total {
                mnts.push(calc_total(&mnts));
            }
            display_mounts(&mnts, &theme, &delimiter, args.inodes);
        }
    }

    Ok(())
}

fn get_mounts(
    mounts_to_show: &DisplayFilter,
    show_inodes: bool,
    paths: &Vec<PathBuf>,
) -> Result<Vec<Mount>> {
    let f = File::open("/proc/self/mounts")?;

    let mut mnts = parse_mounts(f)?;
    mnts.retain(|mount| {
        mounts_to_show
            .get_mnt_fsname_filter()
            .iter()
            .any(|fsname| mnt_matches_filter(mount, fsname))
    });

    for mnt in &mut mnts {
        mnt.statfs = statfs::statfs(&mnt.mnt_dir[..]).ok();

        let (capacity, free) = match mnt.statfs {
            Some(stat) => {
                if show_inodes {
                    (stat.files() as u64, stat.files_free() as u64)
                } else {
                    (
                        stat.blocks() as u64 * (stat.block_size() as u64),
                        stat.blocks_available() as u64 * (stat.block_size() as u64),
                    )
                }
            }
            None => (0, 0),
        };

        mnt.capacity = capacity;
        mnt.free = free;
        mnt.used = capacity - free;
    }

    if !paths.is_empty() {
        let mut out = Vec::new();
        for path in paths {
            let path = match path.canonicalize() {
                Ok(path) => path,
                Err(err) => {
                    eprintln!("dfrs: {}: {}", path.display(), err);
                    continue;
                }
            };

            if let Some(mnt) = get_best_mount_match(&path, &mnts) {
                out.push(mnt.clone());
            }
        }
        return Ok(out);
    }

    mnts.sort_by(cmp_by_capacity_and_dir_name);
    Ok(mnts)
}

fn main() {
    let args = Args::from_args();

    let logging = if args.verbose { "debug" } else { "info" };

    env_logger::init_from_env(Env::default().default_filter_or(logging));

    if let Err(err) = run(args) {
        eprintln!("Error: {}", err);
        for cause in err.chain().skip(1) {
            eprintln!("Because: {}", cause);
        }
        std::process::exit(1);
    }
}
