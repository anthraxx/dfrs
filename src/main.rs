#![deny(clippy::nursery, clippy::cargo)]
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
use util::try_print;

use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use nix::sys::statfs;

use env_logger::Env;

use crate::mount::Mount;
use crate::util::format_percentage;
use anyhow::Result;
use colored::*;
use std::io::{stdout, Write};
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

fn display_mounts(
    mnts: &[Mount],
    theme: &Theme,
    delimiter: &NumberFormat,
    inodes_mode: bool,
    no_aliases: bool,
) {
    let color_heading = theme.color_heading.unwrap_or(Color::White);

    let fsname_func = if no_aliases {
        Mount::fsname
    } else {
        Mount::fsname_aliased
    };

    let fsname_width = column_width(
        mnts,
        |m| fsname_func(m).len(),
        ColumnType::Filesystem.label(inodes_mode),
    );
    let type_width = column_width(
        mnts,
        |m| m.mnt_type.len(),
        ColumnType::Type.label(inodes_mode),
    );
    let available_width = column_width(
        mnts,
        |m| m.free_formatted(delimiter).len(),
        ColumnType::Available.label(inodes_mode),
    );
    let used_width = column_width(
        mnts,
        |m| m.used_formatted(delimiter).len(),
        ColumnType::Used.label(inodes_mode),
    );
    let capacity_width = column_width(
        mnts,
        |m| m.capacity_formatted(delimiter).len(),
        ColumnType::Capacity.label(inodes_mode),
    );
    let mounted_width = column_width(
        mnts,
        |m| m.mnt_dir.len(),
        ColumnType::MountedOn.label(inodes_mode),
    );

    let print_heading_left_func = |column: &ColumnType, width: usize| -> String {
        format!(
            "{:<width$} ",
            column.label(inodes_mode).color(color_heading),
            width = width,
        )
    };

    let print_heading_right_func = |column: &ColumnType, width: usize| -> String {
        format!(
            "{:>width$} ",
            column.label(inodes_mode).color(color_heading),
            width = width,
        )
    };

    let mut line = String::new();
    for column in &theme.columns {
        match column {
            ColumnType::Filesystem => {
                line.push_str(print_heading_left_func(column, fsname_width).as_str());
            }
            ColumnType::Type => {
                line.push_str(print_heading_left_func(column, type_width).as_str());
            }
            ColumnType::Bar => {
                line.push_str(print_heading_left_func(column, theme.bar_width).as_str());
            }
            ColumnType::Used => {
                line.push_str(print_heading_right_func(column, used_width).as_str());
            }
            ColumnType::UsedPercentage => {
                line.push_str(print_heading_right_func(column, 6).as_str());
            }
            ColumnType::Available => {
                line.push_str(print_heading_right_func(column, available_width).as_str());
            }
            ColumnType::AvailablePercentage => {
                line.push_str(print_heading_right_func(column, 6).as_str());
            }
            ColumnType::Capacity => {
                line.push_str(print_heading_right_func(column, capacity_width).as_str());
            }
            ColumnType::MountedOn => {
                line.push_str(print_heading_left_func(column, mounted_width).as_str());
            }
        }
    }
    if try_println!("{}", line.trim_end()).is_err() {
        return;
    }

    for mnt in mnts {
        let usage_color = mnt.usage_color(theme);

        let used_percentage = format_percentage(mnt.used_percentage()).color(usage_color);
        let available_percentage = format_percentage(mnt.free_percentage()).color(usage_color);

        line.clear();
        for column in &theme.columns {
            match column {
                ColumnType::Filesystem => {
                    line.push_str(
                        format!("{:<width$} ", fsname_func(mnt), width = fsname_width).as_str(),
                    );
                }
                ColumnType::Type => {
                    line.push_str(
                        format!("{:<width$} ", mnt.mnt_type, width = type_width).as_str(),
                    );
                }
                ColumnType::Bar => {
                    line.push_str(
                        format!(
                            "{:<width$} ",
                            bar(theme.bar_width, mnt.used_percentage(), theme),
                            width = theme.bar_width
                        )
                        .as_str(),
                    );
                }
                ColumnType::Used => {
                    line.push_str(
                        format!(
                            "{:>width$} ",
                            mnt.used_formatted(delimiter).color(usage_color),
                            width = used_width
                        )
                        .as_str(),
                    );
                }
                ColumnType::UsedPercentage => {
                    line.push_str(format!("{} ", used_percentage).as_str());
                }
                ColumnType::Available => {
                    line.push_str(
                        format!(
                            "{:>width$} ",
                            mnt.free_formatted(delimiter).color(usage_color),
                            width = available_width
                        )
                        .as_str(),
                    );
                }
                ColumnType::AvailablePercentage => {
                    line.push_str(format!("{} ", available_percentage).as_str());
                }
                ColumnType::Capacity => {
                    line.push_str(
                        format!(
                            "{:>width$} ",
                            mnt.capacity_formatted(delimiter).color(usage_color),
                            width = capacity_width
                        )
                        .as_str(),
                    );
                }
                ColumnType::MountedOn => {
                    line.push_str(
                        format!("{:<width$} ", mnt.mnt_dir, width = mounted_width).as_str(),
                    );
                }
            }
        }
        if try_println!("{}", line.trim_end()).is_err() {
            return;
        }
    }
    if stdout().flush().is_err() {}
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
            let mut theme = Theme::new();
            theme.columns = args.columns;

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

            let mut mnts = get_mounts(
                &mounts_to_show,
                args.inodes,
                &args.paths,
                &args.mounts,
                args.local,
            )?;
            if args.total {
                mnts.push(util::calc_total(&mnts));
            }
            display_mounts(&mnts, &theme, &delimiter, args.inodes, args.no_aliases);
        }
    }

    Ok(())
}

fn get_mounts(
    mounts_to_show: &DisplayFilter,
    show_inodes: bool,
    paths: &[PathBuf],
    mounts: &Path,
    local_only: bool,
) -> Result<Vec<Mount>> {
    let f = File::open(mounts)?;

    let mut mnts = parse_mounts(f)?;
    mnts.retain(|mount| {
        mounts_to_show
            .get_mnt_fsname_filter()
            .iter()
            .any(|fsname| util::mnt_matches_filter(mount, fsname))
    });
    if local_only {
        mnts.retain(Mount::is_local);
    }

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

            if let Some(mnt) = util::get_best_mount_match(&path, &mnts) {
                out.push(mnt.clone());
            }
        }
        return Ok(out);
    }

    mnts.sort_by(util::cmp_by_capacity_and_dir_name);
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
