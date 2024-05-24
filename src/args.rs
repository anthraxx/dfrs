#![allow(clippy::use_self)]

use std::io::stdout;

use clap::CommandFactory;
use clap::{ArgAction, Args as ClapArgs, Parser, Subcommand, ValueEnum, ValueHint};

use clap_complete::{generate, Shell};

use lazy_static::lazy_static;
use std::path::PathBuf;
use strum_macros::{Display, EnumString, VariantNames};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[command(disable_help_flag = true)]
pub struct Args {
    /// Show more, use twice to show all
    #[arg(short = 'a', group = "display_group", action = ArgAction::Count, )]
    pub display: u8,
    /// Show more
    #[arg(long, group = "display_group")]
    pub more: bool,
    /// Show all
    #[arg(long, group = "display_group")]
    pub all: bool,
    /// Bypass tty detection for colors
    #[arg(long, group = "color_group")]
    pub color: Option<ColorOpt>,
    /// Bypass tty detection and always show colors
    #[arg(short = 'c', group = "color_group")]
    pub color_always: bool,
    /// Show inode instead of block usage
    #[arg(short, long)]
    pub inodes: bool,
    /// Print sizes in powers of 1024 (e.g., 1023M)
    #[arg(short = 'h', long = "human-readable", group = "number_format")]
    pub base2: bool,
    /// Print sizes in powers of 1000 (e.g., 1.1G)
    #[arg(short = 'H', long = "si", group = "number_format")]
    pub base10: bool,
    /// Produce and show a grand total
    #[arg(long)]
    pub total: bool,
    /// Limit listing to local file systems
    #[arg(short, long)]
    pub local: bool,
    /// Do not resolve file system shorthand aliases (e.g., LVM)
    #[arg(long)]
    pub no_aliases: bool,
    /// File to get mount information from
    #[arg(long, value_hint = ValueHint::FilePath, default_value = "/proc/self/mounts", value_name = "FILE")]
    pub mounts: PathBuf,
    /// Verbose logging
    #[arg(short)]
    pub verbose: bool,
    #[arg(value_hint = ValueHint::AnyPath)]
    pub paths: Vec<PathBuf>,
    /// Display columns as comma separated list
    #[arg(long, use_value_delimiter = true, default_value = &**COLUMNS_OPT_DEFAULT_VALUE)]
    pub columns: Vec<ColumnType>,
    /// Print help information
    #[arg(long, action = ArgAction::Help, global = true)]
    pub help: Option<bool>,
    #[command(subcommand)]
    pub subcommand: Option<SubCommand>,
}

#[derive(Debug, Subcommand)]
pub enum SubCommand {
    /// Generate shell completions
    #[clap(name = "completions")]
    Completions(Completions),
}

#[derive(Debug, Clone, ValueEnum, Display, EnumString, VariantNames)]
#[strum(serialize_all = "lowercase")]
pub enum ColorOpt {
    Auto,
    Always,
    Never,
}

#[derive(Debug, Clone, ValueEnum, EnumString)]
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

#[derive(Debug)]
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

#[derive(Debug, Clone, Display, ValueEnum, EnumString, VariantNames)]
#[strum(serialize_all = "snake_case")]
#[clap(rename_all = "snake_case")]
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

lazy_static! {
    static ref COLUMNS_OPT_DEFAULT_VALUE: String = [
        ColumnType::Filesystem,
        ColumnType::Type,
        ColumnType::Bar,
        ColumnType::UsedPercentage,
        ColumnType::Available,
        ColumnType::Used,
        ColumnType::Capacity,
        ColumnType::MountedOn
    ]
    .iter()
    .map(|e| e.to_string())
    .collect::<Vec<String>>()
    .join(",");
}

#[derive(Debug, ClapArgs)]
pub struct Completions {
    pub shell: Shell,
}

pub fn gen_completions(completions: &Completions) {
    let mut cmd = Args::command();
    let bin_name = cmd.get_name().to_string();
    generate(completions.shell, &mut cmd, &bin_name, &mut stdout());
}
