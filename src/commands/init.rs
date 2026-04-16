use clap::Args;
use std::fs;
use std::path::{Path, PathBuf};

const PROBLEM_TEMPLATE: &str =
    "# Problem\n\n## Abstract\n\n## Goal\n\n## Constraints\n\n## Open Questions\n";
const AMETH_TOML_TEMPLATE: &str = "[ideas]\n";

#[derive(Args, Debug)]
#[command(about = "Initialize an Ameth project")]
pub struct InitArgs {
    #[arg(value_name = "NAME", value_parser = parse_project_name)]
    pub name: String,

    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
}

pub struct InitCommand {
    name: String,
    parent: PathBuf,
}

pub fn run(args: InitArgs) -> Result<(), String> {
    let command = InitCommand {
        name: args.name,
        parent: args.path.unwrap_or_else(|| PathBuf::from(".")),
    };

    execute(&command)
}

pub fn parse_project_name(name: &str) -> Result<String, String> {
    if name.is_empty() {
        return Err("project name cannot be empty".to_string());
    }

    if name == "." || name == ".." {
        return Err("project name must be a single directory name".to_string());
    }

    if name.chars().any(std::path::is_separator) {
        return Err("project name cannot contain a path separator".to_string());
    }

    Ok(name.to_owned())
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
