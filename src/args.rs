use crate::errors::*;

use structopt::StructOpt;
use structopt::clap::{AppSettings, Shell};

use std::io::stdout;

use strum_macros::EnumString;

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Bypass tty detection for colors: auto, always, never
    #[structopt(long)]
    pub color: Option<ColorOpt>,
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

#[derive(Debug, StructOpt)]
pub struct Completions {
    #[structopt(possible_values=&Shell::variants())]
    pub shell: Shell,
}

pub fn gen_completions(args: &Completions) -> Result<()> {
    Args::clap().gen_completions_to("dfrs", args.shell, &mut stdout());
    Ok(())
}