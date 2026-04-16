use crate::commands::{ideas, init};
use clap::{CommandFactory, Parser, Subcommand};
use std::ffi::OsString;
use std::io::{self, Write};

use crate::cli::Error;

#[derive(Debug, Parser)]
#[command(
    name = "ameth",
    disable_help_subcommand = true,
    about = "Ameth organizes research work so humans and LLMs can recover project context with less guesswork.",
    after_help = "Notes:\n  Bare `ameth ideas` shows the pinned idea when one is set.\n  Run `ameth init --help` or `ameth ideas --help` for command-specific help."
)]
struct RootCli {
    #[command(subcommand)]
    command: Option<RootCommand>,
}

#[derive(Debug, Subcommand)]
enum RootCommand {
    #[command(about = "Initialize an Ameth project")]
    Init(init::InitArgs),
    #[command(about = "Manage idea files")]
    Ideas(ideas::IdeasArgs),
}

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), Error> {
    let cli = RootCli::try_parse_from(args).map_err(Error::Clap)?;

    if let Some(command) = cli.command {
        return match command {
            RootCommand::Init(args) => init::run(args).map_err(Error::Runtime),
            RootCommand::Ideas(args) => ideas::run(args).map_err(Error::Runtime),
        };
    }

    print_root_help().map_err(|error| Error::Runtime(format!("failed to write help: {error}")))
}

fn print_root_help() -> io::Result<()> {
    let mut command = RootCli::command();
    let mut stdout = io::stdout();
    command.write_long_help(&mut stdout)?;
    writeln!(stdout)
}
