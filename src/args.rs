use crate::errors::*;

use structopt::StructOpt;
use structopt::clap::{AppSettings, Shell};

use std::io::stdout;

use strum_macros::EnumString;

#[derive(Debug, StructOpt)]
#[structopt(about="Display file system space usage using graphs and colors.", global_settings = &[AppSettings::ColoredHelp, AppSettings::DeriveDisplayOrder])]
pub struct Args {
    /// Show more, use twice to show all
    #[structopt(short="a", group="display_group", parse(from_occurrences))]
    pub display: u8,
    /// Show more
    #[structopt(long, group="display_group")]
    pub more: bool,
    /// Show all
    #[structopt(long, group="display_group")]
    pub all: bool,
    /// Bypass tty detection for colors: auto, always, never
    #[structopt(long, group="color_group")]
    pub color: Option<ColorOpt>,
    /// Bypass tty detection and always show colors
    #[structopt(short="c", group="color_group")]
    pub color_always: bool,
    /// Show inode instead of block usage
    #[structopt(short, long)]
    pub inodes: bool,
    /// Print sizes in powers of 1024 (e.g., 1023M)
    #[structopt(short="h", long="human-readable", group="number_format")]
    pub base2: bool,
    /// Print sizes in powers of 1000 (e.g., 1.1G)
    #[structopt(short="H", long="si", group="number_format")]
    pub base10: bool,
    /// Verbose logging
    #[structopt(short)]
    pub verbose: bool,
    #[structopt(subcommand)]
    pub subcommand: Option<SubCommand>,
}

#[derive(Debug, StructOpt)]
pub enum SubCommand {
    /// Generate shell completions
    #[structopt(name="completions")]
    Completions(Completions),
}

#[derive(Debug, StructOpt, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ColorOpt {
    Auto,
    Always,
    Never
}

#[derive(Debug, StructOpt, EnumString)]
#[strum(serialize_all = "lowercase")]
pub enum DisplayFilter {
    Minimal,
    More,
    All,
}

impl DisplayFilter {
    pub fn from_u8(n: u8) -> DisplayFilter {
        match n {
            0 => DisplayFilter::Minimal,
            1 => DisplayFilter::More,
            _ => DisplayFilter::All
        }
    }

    pub fn get_mnt_fsname_filter(&self) -> Vec<&'static str> {
        match self {
            DisplayFilter::Minimal => vec!["/dev*"],
            DisplayFilter::More => vec!["dev", "run", "tmpfs", "/dev*"],
            DisplayFilter::All => vec!["*"],
        }
    }
}

#[derive(Debug, StructOpt)]
pub enum NumberFormat {
    Base10,
    Base2,
}

impl NumberFormat {
    pub fn get_powers_of(&self) -> f64 {
        match self {
            NumberFormat::Base10 => 1000_f64,
            NumberFormat::Base2 => 1024_f64,
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