use structopt::clap::{AppSettings, Shell};
use structopt::StructOpt;

use std::io::stdout;

use anyhow::Result;
use std::path::PathBuf;
use strum_macros::EnumString;

#[derive(Debug, StructOpt)]
#[structopt(about="Display file system space usage using graphs and colors.", global_settings = &[AppSettings::ColoredHelp, AppSettings::DeriveDisplayOrder])]
pub struct Args {
    /// Show more, use twice to show all
    #[structopt(short = "a", group = "display_group", parse(from_occurrences))]
    pub display: u8,
    /// Show more
    #[structopt(long, group = "display_group")]
    pub more: bool,
    /// Show all
    #[structopt(long, group = "display_group")]
    pub all: bool,
    /// Bypass tty detection for colors: auto, always, never
    #[structopt(long, group = "color_group")]
    pub color: Option<ColorOpt>,
    /// Bypass tty detection and always show colors
    #[structopt(short = "c", group = "color_group")]
    pub color_always: bool,
    /// Show inode instead of block usage
    #[structopt(short, long)]
    pub inodes: bool,
    /// Print sizes in powers of 1024 (e.g., 1023M)
    #[structopt(short = "h", long = "human-readable", group = "number_format")]
    pub base2: bool,
    /// Print sizes in powers of 1000 (e.g., 1.1G)
    #[structopt(short = "H", long = "si", group = "number_format")]
    pub base10: bool,
    /// Produce and show a grand total
    #[structopt(long)]
    pub total: bool,
    /// Limit listing to local file systems
    #[structopt(short, long)]
    pub local: bool,
    /// Do not resolve file system shorthand aliases (e.g., LVM)
    #[structopt(long)]
    pub no_aliases: bool,
    /// File to get mount information from
    #[structopt(long, parse(from_os_str), default_value = "/proc/self/mounts")]
    pub mounts: PathBuf,
    /// Verbose logging
    #[structopt(short)]
    pub verbose: bool,
    #[structopt(parse(from_os_str))]
    pub paths: Vec<PathBuf>,
    /// Display columns as comma separated list
    #[structopt(long, use_delimiter = true, default_value = "filesystem,type,bar,used_percentage,available,used,capacity,mounted_on")]
    pub columns: Vec<ColumnType>,
    #[structopt(subcommand)]
    pub subcommand: Option<SubCommand>,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    /// Generate shell completions
    #[structopt(name = "completions")]
    Completions(Completions),
}

#[derive(Debug, StructOpt, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ColorOpt {
    Auto,
    Always,
    Never,
}

#[derive(Debug, StructOpt, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum DisplayFilter {
    Minimal,
    More,
    All,
}

impl DisplayFilter {
    pub const fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::Minimal,
            1 => Self::More,
            _ => Self::All,
        }
    }

    pub fn get_mnt_fsname_filter(&self) -> Vec<&'static str> {
        match self {
            Self::Minimal => vec!["/dev*", "storage"],
            Self::More => vec!["dev", "run", "tmpfs", "/dev*", "storage"],
            Self::All => vec!["*"],
        }
    }
}

#[derive(Debug, StructOpt)]
pub enum NumberFormat {
    Base10,
    Base2,
}

impl NumberFormat {
    pub const fn get_powers_of(&self) -> f64 {
        match self {
            Self::Base10 => 1000_f64,
            Self::Base2 => 1024_f64,
        }
    }
}

#[derive(Debug, StructOpt, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ColumnType {
    Filesystem,
    Type,
    Bar,
    Used,
    UsedPercentage,
    Available,
    AvailablePercentage,
    Capacity,
    MountedOn,
}

impl ColumnType {
    pub const fn label(&self, inodes_mode: bool) -> &str {
        match self {
            Self::Filesystem => "Filesystem",
            Self::Type => "Type",
            Self::Bar => "",
            Self::Used => "Used",
            Self::UsedPercentage => "Used%",
            Self::Available => "Avail",
            Self::AvailablePercentage => "Avail%",
            Self::Capacity => {
                if inodes_mode {
                    "Inodes"
                } else {
                    "Size"
                }
            }
            Self::MountedOn => "Mounted on",
        }
    }
}

#[derive(Debug, StructOpt)]
pub struct Completions {
    #[structopt(possible_values=&Shell::variants())]
    pub shell: Shell,
}

pub fn gen_completions(args: &Completions) -> Result<()> {
    Args::clap().gen_completions_to("dfrs", args.shell, &mut stdout());
    Ok(())
}
