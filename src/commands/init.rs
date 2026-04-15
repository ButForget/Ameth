use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

pub const USAGE: &str = "ameth init <name> [path]";
pub const ALIAS_USAGE: &str = "ameth <name> [path]";
pub const HELP: &str = "Initialize an Ameth project.\n\nUsage:\n  ameth init <name> [path]\n  ameth <name> [path]\n\nArguments:\n  <name>  Project directory name\n  [path]  Parent directory for the project (defaults to `.`)\n";

const PROBLEM_TEMPLATE: &str =
    "# Problem\n\n## Abstract\n\n## Goal\n\n## Constraints\n\n## Open Questions\n";
const AMETH_TOML_TEMPLATE: &str = "[ideas]\n";

pub enum Invocation {
    Explicit,
    Alias,
}

pub struct InitCommand {
    name: String,
    parent: PathBuf,
}

enum ParsedInitCommand {
    Help,
    Run(InitCommand),
}

pub fn run(args: Vec<OsString>, invocation: Invocation) -> Result<(), String> {
    match parse(args, invocation)? {
        ParsedInitCommand::Help => {
            print!("{HELP}");
            Ok(())
        }
        ParsedInitCommand::Run(command) => execute(&command),
    }
}

fn parse(args: Vec<OsString>, invocation: Invocation) -> Result<ParsedInitCommand, String> {
    if args.len() == 1 && is_help_flag(&args[0]) {
        return Ok(ParsedInitCommand::Help);
    }

    if args.is_empty() {
        return Err(format!(
            "missing required arguments\n\nUsage: {}",
            usage_for(&invocation)
        ));
    }

    if args.len() > 2 {
        return Err(format!(
            "invalid arguments\n\nUsage: {}",
            usage_for(&invocation)
        ));
    }

    let name = validate_project_name(&args[0])?.to_owned();
    let parent = args
        .get(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    Ok(ParsedInitCommand::Run(InitCommand { name, parent }))
}

fn is_help_flag(argument: &OsString) -> bool {
    argument == "--help" || argument == "-h"
}

fn usage_for(invocation: &Invocation) -> &'static str {
    match invocation {
        Invocation::Explicit => USAGE,
        Invocation::Alias => ALIAS_USAGE,
    }
}

fn validate_project_name(name: &OsString) -> Result<&str, String> {
    let name = name
        .to_str()
        .ok_or_else(|| "project name must be valid UTF-8".to_string())?;

    if name.is_empty() {
        return Err("project name cannot be empty".to_string());
    }

    if name == "." || name == ".." {
        return Err("project name must be a single directory name".to_string());
    }

    if name.chars().any(std::path::is_separator) {
        return Err("project name cannot contain a path separator".to_string());
    }

    Ok(name)
}

fn execute(command: &InitCommand) -> Result<(), String> {
    if !command.parent.exists() {
        return Err(format!(
            "parent path does not exist: {}",
            command.parent.display()
        ));
    }

    if !command.parent.is_dir() {
        return Err(format!(
            "parent path is not a directory: {}",
            command.parent.display()
        ));
    }

    let project_root = command.parent.join(&command.name);

    if project_root.exists() {
        return Err(format!("target already exists: {}", project_root.display()));
    }

    fs::create_dir(&project_root).map_err(|error| format_create_error(&project_root, error))?;
    fs::create_dir_all(project_root.join("ideas/abandoned"))
        .map_err(|error| format_create_error(&project_root.join("ideas/abandoned"), error))?;
    fs::create_dir(project_root.join("solutions"))
        .map_err(|error| format_create_error(&project_root.join("solutions"), error))?;
    fs::create_dir(project_root.join("logs"))
        .map_err(|error| format_create_error(&project_root.join("logs"), error))?;
    fs::create_dir(project_root.join("relevants"))
        .map_err(|error| format_create_error(&project_root.join("relevants"), error))?;
    fs::create_dir(project_root.join("code"))
        .map_err(|error| format_create_error(&project_root.join("code"), error))?;
    fs::create_dir(project_root.join("experiments"))
        .map_err(|error| format_create_error(&project_root.join("experiments"), error))?;
    fs::write(project_root.join("ideas/Problem.md"), PROBLEM_TEMPLATE)
        .map_err(|error| format_write_error(&project_root.join("ideas/Problem.md"), error))?;
    fs::write(project_root.join("Ameth.toml"), AMETH_TOML_TEMPLATE)
        .map_err(|error| format_write_error(&project_root.join("Ameth.toml"), error))?;

    println!("Initialized Ameth project at {}", project_root.display());
    Ok(())
}

fn format_create_error(path: &Path, error: std::io::Error) -> String {
    format!("failed to create {}: {error}", path.display())
}

fn format_write_error(path: &Path, error: std::io::Error) -> String {
    format!("failed to write {}: {error}", path.display())
}
