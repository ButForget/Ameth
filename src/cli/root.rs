use crate::commands::{
    ideas,
    init::{self, Invocation},
};
use std::ffi::OsString;

pub const USAGE: &str =
    "ameth\nameth init <name> [path]\nameth <name> [path]\nameth ideas <command>";
pub const HELP: &str = "Ameth organizes research work so humans and LLMs can recover project context with less guesswork.\n\nUsage:\n  ameth\n  ameth init <name> [path]\n  ameth <name> [path]\n  ameth ideas <command>\n\nCommands:\n  init   Initialize an Ameth project\n  ideas  Manage idea files\n\nNotes:\n  `ameth <name> [path]` is an alias for `ameth init <name> [path]`.\n  Run `ameth init --help` or `ameth ideas --help` for command-specific help.\n";

pub fn run(args: impl IntoIterator<Item = OsString>) -> Result<(), String> {
    let mut args = args.into_iter();
    let _program_name = args.next();
    let arguments: Vec<OsString> = args.collect();

    dispatch(arguments)
}

fn dispatch(arguments: Vec<OsString>) -> Result<(), String> {
    if arguments.is_empty() {
        print!("{HELP}");
        return Ok(());
    }

    if is_help_flag(&arguments[0]) {
        if arguments.len() == 1 {
            print!("{HELP}");
            return Ok(());
        }

        return Err(format!("invalid arguments\n\nUsage:\n  {USAGE}"));
    }

    if arguments.first().and_then(|argument| argument.to_str()) == Some("init") {
        return init::run(
            arguments.into_iter().skip(1).collect(),
            Invocation::Explicit,
        );
    }

    if arguments.first().and_then(|argument| argument.to_str()) == Some("ideas") {
        return ideas::run(arguments.into_iter().skip(1).collect());
    }

    init::run(arguments, Invocation::Alias)
}

fn is_help_flag(argument: &OsString) -> bool {
    argument == "--help" || argument == "-h"
}
