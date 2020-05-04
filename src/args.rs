use crate::errors::*;

use structopt::StructOpt;
use structopt::clap::{AppSettings, Shell};

use std::io::stdout;

use strum_macros::EnumString;

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp, AppSettings::DeriveDisplayOrder])]
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
    /// Bypass tty detection for colors: auto, always, never
    #[structopt(short="c", group="color_group")]
    pub color_always: bool,
    /// Show inode instead of block usage
    #[structopt(short, long)]
    pub inodes: bool,
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
pub struct Completions {
    #[structopt(possible_values=&Shell::variants())]
    pub shell: Shell,
}

pub fn gen_completions(args: &Completions) -> Result<()> {
    Args::clap().gen_completions_to("dfrs", args.shell, &mut stdout());
    Ok(())
}