use crate::config::{AMETH_TOML_FILE_NAME, AmethConfig};
use clap::{Args, Command as ClapCommand, Subcommand};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const RESEARCH_QUESTION_FILE_NAME: &str = "ResearchQuestion.md";
const RESEARCH_QUESTION_TEMPLATE: &str = "# Research Question\n\n";

#[derive(Args, Debug)]
#[command(about = "Manage the root research question file")]
pub struct RqArgs {
    #[command(subcommand)]
    command: Option<RqCommand>,
}

#[derive(Debug, Subcommand)]
enum RqCommand {
    #[command(about = "Show the root research question file")]
    Show,
    #[command(about = "Edit the root research question file")]
    Edit(EditArgs),
}

#[derive(Args, Debug)]
struct EditArgs {
    #[arg(short = 'n', long = "new")]
    new: bool,

    #[arg(short = 'f', long = "force", requires = "new")]
    force: bool,
}

pub fn run(args: RqArgs) -> Result<(), String> {
    match args.command {
        None => {
            println!("{}", rq_help());
            Ok(())
        }
        Some(RqCommand::Show) => run_show(),
        Some(RqCommand::Edit(args)) => run_edit(args),
    }
}

fn run_show() -> Result<(), String> {
    let project = RqProject::load()?;
    let markdown = project.read_markdown()?;

    print!("{markdown}");
    if !markdown.ends_with('\n') {
        println!();
    }

    Ok(())
}

fn run_edit(args: EditArgs) -> Result<(), String> {
    let project = RqProject::load()?;

    if args.new {
        if project.research_question_path.exists() && !args.force {
            return Err(format!("{} already exists", RESEARCH_QUESTION_FILE_NAME));
        }

        let (program, editor_args) = project.configured_editor()?;
        project.create_new_file(args.force)?;
        return project.open_editor(&program, &editor_args);
    }

    project.ensure_file_exists()?;
    let (program, editor_args) = project.configured_editor()?;
    project.open_editor(&program, &editor_args)
}

fn rq_help() -> String {
    let mut command = ClapCommand::new("rq")
        .bin_name("ameth rq")
        .about("Manage the root research question file");
    command = RqCommand::augment_subcommands(command);

    command.render_long_help().to_string()
}

struct RqProject {
    config_path: PathBuf,
    research_question_path: PathBuf,
}

impl RqProject {
    fn load() -> Result<Self, String> {
        let root = env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?;
        let config_path = root.join(AMETH_TOML_FILE_NAME);

        if !config_path.is_file() {
            return Err("current directory is not an Ameth project".to_string());
        }

        Ok(Self {
            config_path,
            research_question_path: root.join(RESEARCH_QUESTION_FILE_NAME),
        })
    }

    fn read_markdown(&self) -> Result<String, String> {
        self.ensure_file_exists()?;
        fs::read_to_string(&self.research_question_path).map_err(|error| {
            format!(
                "failed to read {}: {error}",
                self.research_question_path.display()
            )
        })
    }

    fn create_new_file(&self, force: bool) -> Result<(), String> {
        if self.research_question_path.exists() && !force {
            return Err(format!("{} already exists", RESEARCH_QUESTION_FILE_NAME));
        }

        fs::write(&self.research_question_path, RESEARCH_QUESTION_TEMPLATE).map_err(|error| {
            format!(
                "failed to write {}: {error}",
                self.research_question_path.display()
            )
        })
    }

    fn ensure_file_exists(&self) -> Result<(), String> {
        if !self.research_question_path.is_file() {
            return Err(format!("{} not found", RESEARCH_QUESTION_FILE_NAME));
        }

        Ok(())
    }

    fn configured_editor(&self) -> Result<(String, Vec<String>), String> {
        let config = AmethConfig::load_or_default(&self.config_path)?;
        let (program, args) = config.editor_command().ok_or_else(|| {
            format!(
                "missing root-level `editor` in {}; configure it before using interactive `ameth rq edit`",
                self.config_path.display()
            )
        })?;

        Ok((program.to_string(), args.to_vec()))
    }

    fn open_editor(&self, program: &str, args: &[String]) -> Result<(), String> {
        let status = Command::new(program)
            .args(args)
            .arg(&self.research_question_path)
            .status()
            .map_err(|error| {
                format!(
                    "failed to launch editor `{program}` for {}: {error}",
                    self.research_question_path.display()
                )
            })?;

        if !status.success() {
            return Err(format!(
                "editor `{program}` exited unsuccessfully: {status}"
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::EditArgs;
    use clap::Args;

    #[test]
    fn force_requires_new() {
        let command = EditArgs::augment_args(clap::Command::new("edit"));
        let error = command
            .try_get_matches_from(["edit", "--force"])
            .expect_err("`--force` without `--new` should be rejected");

        assert_eq!(
            error.kind(),
            clap::error::ErrorKind::MissingRequiredArgument
        );
    }
}
