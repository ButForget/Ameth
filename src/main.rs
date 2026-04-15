mod cli;
mod commands;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::run(env::args_os()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}
