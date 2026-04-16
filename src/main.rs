mod cli;
mod commands;
mod config;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::run(env::args_os()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(cli::Error::Clap(error)) => error.exit(),
        Err(cli::Error::Runtime(error)) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
