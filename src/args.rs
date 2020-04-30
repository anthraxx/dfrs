use crate::errors::*;

use structopt::StructOpt;
use structopt::clap::{AppSettings, Shell};

use std::io::stdout;

#[derive(Debug, StructOpt)]
#[structopt(global_settings = &[AppSettings::ColoredHelp])]
pub struct Args {
    /// Bypass tty detection and always use colors
    #[structopt(long, global=true)]
    pub color: bool,
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

#[derive(Debug, StructOpt)]
pub struct Completions {
    #[structopt(possible_values=&Shell::variants())]
    pub shell: Shell,
}

pub fn gen_completions(args: &Completions) -> Result<()> {
    Args::clap().gen_completions_to("dfrs", args.shell, &mut stdout());
    Ok(())
}