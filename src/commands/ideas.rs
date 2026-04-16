mod document;
mod project;

use self::document::{idea_template, parse_idea_document};
use self::project::{IdeasProject, format_idea_id};
use clap::{Args, Command as ClapCommand, Subcommand};
use std::fs;

#[derive(Args, Debug)]
#[command(
    about = "Manage idea files",
    after_help = "Notes:\n  Bare `ameth ideas` shows the pinned idea when one is set.\n  Bare `ameth ideas` prints this help when no pinned idea is set."
)]
pub struct IdeasArgs {
    #[command(subcommand)]
    command: Option<IdeasCommand>,
}

#[derive(Debug, Subcommand)]
enum IdeasCommand {
    #[command(
        about = "Create the next idea file",
        override_usage = "ameth ideas new [--abs <ABSTRACT>] [--ctt <CONTENT>]",
        after_help = "When either field is omitted, Ameth opens the root-level `editor` configured in Ameth.toml."
    )]
    New(NewArgs),
    #[command(about = "List active ideas")]
    List,
    #[command(about = "Display an idea by ID, or the pinned idea when no ID is given")]
    Show(ShowArgs),
    #[command(about = "Pin an idea ID for quick access")]
    Pin(IdeaIdArgs),
    #[command(about = "Move an active idea into ideas/abandoned/")]
    Abandon(IdeaIdArgs),
    #[command(about = "Move an abandoned idea back into ideas/")]
    Restore(IdeaIdArgs),
}

#[derive(Args, Debug)]
struct NewArgs {
    #[arg(long = "abs", value_name = "ABSTRACT")]
    abstract_text: Option<String>,

    #[arg(long = "ctt", value_name = "CONTENT")]
    content_text: Option<String>,
}

#[derive(Args, Debug)]
struct ShowArgs {
    #[arg(value_name = "ID", value_parser = parse_idea_id)]
    id: Option<u32>,
}

#[derive(Args, Debug)]
struct IdeaIdArgs {
    #[arg(value_name = "ID", value_parser = parse_idea_id)]
    id: u32,
}

pub fn run(args: IdeasArgs) -> Result<(), String> {
    match args.command {
        None => run_default(),
        Some(IdeasCommand::New(args)) => run_new(args),
        Some(IdeasCommand::List) => run_list(),
        Some(IdeasCommand::Show(args)) => run_show(args.id),
        Some(IdeasCommand::Pin(args)) => run_pin(args.id),
        Some(IdeasCommand::Abandon(args)) => run_abandon(args.id),
        Some(IdeasCommand::Restore(args)) => run_restore(args.id),
    }
}

fn run_default() -> Result<(), String> {
    let Ok(project) = IdeasProject::load() else {
        println!("{}", ideas_help());
        return Ok(());
    };

    let Some(id) = project.read_pinned_id()? else {
        println!("{}", ideas_help());
        return Ok(());
    };

    show_idea(&project, id)
}

fn run_new(args: NewArgs) -> Result<(), String> {
    let project = IdeasProject::load()?;
    let next_id = project.next_idea_id()?;
    let path = project.active_idea_path(next_id);
    let should_open_editor = args.abstract_text.is_none() || args.content_text.is_none();
    let template = idea_template(args.abstract_text.as_deref(), args.content_text.as_deref());

    fs::write(&path, template)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;

    println!("Created {}", path.display());

    if should_open_editor {
        project.open_configured_editor(&path)?;
    }

    Ok(())
}

fn run_list() -> Result<(), String> {
    let project = IdeasProject::load()?;
    let ideas = project.active_ideas()?;

    if ideas.is_empty() {
        println!("No active ideas found.");
        return Ok(());
    }

    for idea in ideas {
        let markdown = fs::read_to_string(&idea.path)
            .map_err(|error| format!("failed to read {}: {error}", idea.path.display()))?;
        let document = parse_idea_document(&markdown)
            .map_err(|error| format!("failed to parse {}: {error}", idea.path.display()))?;
        println!(
            "{}  {}",
            format_idea_id(idea.id),
            single_line(&document.abstract_text)
        );
    }

    Ok(())
}

fn run_show(id: Option<u32>) -> Result<(), String> {
    let project = IdeasProject::load()?;
    let id = match id {
        Some(id) => id,
        None => match project.read_pinned_id()? {
            Some(id) => id,
            None => return Err("no pinned idea set".to_string()),
        },
    };

    show_idea(&project, id)
}

fn run_pin(id: u32) -> Result<(), String> {
    let project = IdeasProject::load()?;

    project.read_idea_markdown(id)?;
    project.write_pinned_id(id)?;

    println!("Pinned idea {}", format_idea_id(id));
    Ok(())
}

fn run_abandon(id: u32) -> Result<(), String> {
    let project = IdeasProject::load()?;
    project.abandon_idea(id)?;

    println!("Abandoned idea {}", format_idea_id(id));
    Ok(())
}

fn run_restore(id: u32) -> Result<(), String> {
    let project = IdeasProject::load()?;
    project.restore_idea(id)?;

    println!("Restored idea {}", format_idea_id(id));
    Ok(())
}

fn show_idea(project: &IdeasProject, id: u32) -> Result<(), String> {
    let markdown = project.read_idea_markdown(id)?;

    print!("{markdown}");
    if !markdown.ends_with('\n') {
        println!();
    }

    Ok(())
}

fn parse_idea_id(raw: &str) -> Result<u32, String> {
    let id = raw
        .parse::<u32>()
        .map_err(|_| "idea id must be a positive integer".to_string())?;

    if id == 0 {
        return Err("idea id must be a positive integer".to_string());
    }

    Ok(id)
}

fn ideas_help() -> String {
    let mut command = ClapCommand::new("ideas")
        .bin_name("ameth ideas")
        .about("Manage idea files")
        .after_help(
            "Notes:\n  Bare `ameth ideas` shows the pinned idea when one is set.\n  Bare `ameth ideas` prints this help when no pinned idea is set.",
        );
    command = IdeasCommand::augment_subcommands(command);

    command.render_long_help().to_string()
}

fn single_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
