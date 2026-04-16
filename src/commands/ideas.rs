use clap::{Args, Command, Subcommand};
use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use toml_edit::{DocumentMut, Item, Table, value};

const IDEA_TEMPLATE: &str = "## Abstract\n\n## Content\n";
const AMETH_TOML_FILE_NAME: &str = "Ameth.toml";
const AMETH_TOML_TEMPLATE: &str = "[ideas]\n";

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
    #[command(about = "Create the next idea file")]
    New,
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
        Some(IdeasCommand::New) => run_new(),
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

    let Some(id) = read_pinned_id(&project)? else {
        println!("{}", ideas_help());
        return Ok(());
    };

    show_idea(&project, id)
}

fn run_new() -> Result<(), String> {
    let project = IdeasProject::load()?;
    let next_id = next_idea_id(&project)?;
    let path = project.ideas_dir.join(idea_file_name(next_id));

    fs::write(&path, IDEA_TEMPLATE)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;

    println!("Created {}", path.display());
    Ok(())
}

fn run_list() -> Result<(), String> {
    let project = IdeasProject::load()?;
    let ideas = read_idea_directory(&project.ideas_dir)?;

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
        None => match read_pinned_id(&project)? {
            Some(id) => id,
            None => return Err("no pinned idea set".to_string()),
        },
    };

    show_idea(&project, id)
}

fn run_pin(id: u32) -> Result<(), String> {
    let project = IdeasProject::load()?;

    read_idea_markdown(&project, id)?;
    write_pinned_id(&project, id)?;

    println!("Pinned idea {}", format_idea_id(id));
    Ok(())
}

fn run_abandon(id: u32) -> Result<(), String> {
    let project = IdeasProject::load()?;
    move_idea(
        id,
        &project.ideas_dir.join(idea_file_name(id)),
        &project.abandoned_dir.join(idea_file_name(id)),
        "active",
        "abandoned",
    )?;

    println!("Abandoned idea {}", format_idea_id(id));
    Ok(())
}

fn run_restore(id: u32) -> Result<(), String> {
    let project = IdeasProject::load()?;
    move_idea(
        id,
        &project.abandoned_dir.join(idea_file_name(id)),
        &project.ideas_dir.join(idea_file_name(id)),
        "abandoned",
        "active",
    )?;

    println!("Restored idea {}", format_idea_id(id));
    Ok(())
}

fn move_idea(
    id: u32,
    source: &Path,
    destination: &Path,
    source_label: &str,
    destination_label: &str,
) -> Result<(), String> {
    if !source.is_file() {
        return Err(format!(
            "{source_label} idea {} not found",
            format_idea_id(id)
        ));
    }

    if destination.exists() {
        return Err(format!(
            "cannot move idea {} to {destination_label}: {} already exists",
            format_idea_id(id),
            destination.display()
        ));
    }

    fs::rename(source, destination).map_err(|error| {
        format!(
            "failed to move idea {} from {} to {}: {error}",
            format_idea_id(id),
            source.display(),
            destination.display()
        )
    })
}

fn show_idea(project: &IdeasProject, id: u32) -> Result<(), String> {
    let markdown = read_idea_markdown(project, id)?;

    print!("{markdown}");
    if !markdown.ends_with('\n') {
        println!();
    }

    Ok(())
}

fn read_idea_markdown(project: &IdeasProject, id: u32) -> Result<String, String> {
    let path = resolve_idea_path(project, id)?;
    let markdown = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    parse_idea_document(&markdown)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    Ok(markdown)
}

fn resolve_idea_path(project: &IdeasProject, id: u32) -> Result<PathBuf, String> {
    let file_name = idea_file_name(id);
    let active_path = project.ideas_dir.join(&file_name);
    let abandoned_path = project.abandoned_dir.join(&file_name);

    match (active_path.is_file(), abandoned_path.is_file()) {
        (true, true) => {
            eprintln!(
                "warning: idea {} exists in both ideas/ and ideas/abandoned/; showing the active idea",
                format_idea_id(id)
            );
            Ok(active_path)
        }
        (true, false) => Ok(active_path),
        (false, true) => Ok(abandoned_path),
        (false, false) => Err(format!("idea {} not found", format_idea_id(id))),
    }
}

fn read_pinned_id(project: &IdeasProject) -> Result<Option<u32>, String> {
    let document = read_ameth_toml(&project.config_path)?;
    let Some(ideas) = document.get("ideas") else {
        return Ok(None);
    };
    let Some(ideas) = ideas.as_table_like() else {
        return Err(format!(
            "invalid [ideas] table in {}",
            project.config_path.display()
        ));
    };
    let Some(pinned) = ideas.get("pinned") else {
        return Ok(None);
    };
    let Some(pinned) = pinned.as_integer() else {
        return Err(format!(
            "invalid ideas.pinned value in {}",
            project.config_path.display()
        ));
    };

    if pinned <= 0 || pinned > i64::from(u32::MAX) {
        return Err(format!(
            "invalid ideas.pinned value in {}",
            project.config_path.display()
        ));
    }

    Ok(Some(pinned as u32))
}

fn write_pinned_id(project: &IdeasProject, id: u32) -> Result<(), String> {
    let mut document = read_ameth_toml(&project.config_path)?;

    match document.get("ideas") {
        Some(item) if item.as_table_like().is_some() => {}
        Some(_) => {
            return Err(format!(
                "invalid [ideas] table in {}",
                project.config_path.display()
            ));
        }
        None => {
            document["ideas"] = Item::Table(Table::new());
        }
    }

    document["ideas"]["pinned"] = value(i64::from(id));

    fs::write(&project.config_path, document.to_string())
        .map_err(|error| format!("failed to write {}: {error}", project.config_path.display()))
}

fn read_ameth_toml(path: &Path) -> Result<DocumentMut, String> {
    if !path.exists() {
        return AMETH_TOML_TEMPLATE
            .parse::<DocumentMut>()
            .map_err(|error| format!("failed to prepare {}: {error}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("invalid Ameth config path: {}", path.display()));
    }

    let content = fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;

    content
        .parse::<DocumentMut>()
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
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
    let mut command = Command::new("ideas")
        .bin_name("ameth ideas")
        .about("Manage idea files")
        .after_help(
            "Notes:\n  Bare `ameth ideas` shows the pinned idea when one is set.\n  Bare `ameth ideas` prints this help when no pinned idea is set.",
        );
    command = IdeasCommand::augment_subcommands(command);

    command.render_long_help().to_string()
}

fn next_idea_id(project: &IdeasProject) -> Result<u32, String> {
    let active_max = read_idea_directory(&project.ideas_dir)?
        .into_iter()
        .map(|idea| idea.id)
        .max()
        .unwrap_or(0);
    let abandoned_max = read_idea_directory(&project.abandoned_dir)?
        .into_iter()
        .map(|idea| idea.id)
        .max()
        .unwrap_or(0);

    Ok(active_max.max(abandoned_max) + 1)
}

fn read_idea_directory(directory: &Path) -> Result<Vec<IdeaEntry>, String> {
    let mut ideas = Vec::new();

    for entry in fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", directory.display()))?
    {
        let entry =
            entry.map_err(|error| format!("failed to read {}: {error}", directory.display()))?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Some(id) = parse_idea_id_from_path(&path) {
            ideas.push(IdeaEntry { id, path });
        }
    }

    ideas.sort_by_key(|idea| idea.id);
    Ok(ideas)
}

fn parse_idea_id_from_path(path: &Path) -> Option<u32> {
    let file_name = path.file_name()?.to_str()?;
    let digits = file_name.strip_prefix("idea-")?.strip_suffix(".md")?;

    if digits.len() != 4 || !digits.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }

    digits.parse().ok()
}

fn idea_file_name(id: u32) -> String {
    format!("idea-{:04}.md", id)
}

fn format_idea_id(id: u32) -> String {
    format!("{:04}", id)
}

fn single_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

struct IdeasProject {
    config_path: PathBuf,
    ideas_dir: PathBuf,
    abandoned_dir: PathBuf,
}

impl IdeasProject {
    fn load() -> Result<Self, String> {
        let root = env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?;
        let config_path = root.join(AMETH_TOML_FILE_NAME);
        let ideas_dir = root.join("ideas");
        let abandoned_dir = ideas_dir.join("abandoned");
        let problem_file = ideas_dir.join("Problem.md");

        if !ideas_dir.is_dir() || !abandoned_dir.is_dir() || !problem_file.is_file() {
            return Err("current directory is not an Ameth project".to_string());
        }

        Ok(Self {
            config_path,
            ideas_dir,
            abandoned_dir,
        })
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum ProblemSection {
    Abstract,
    Goal,
    Constraints,
    OpenQuestions,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum IdeaSection {
    Abstract,
    Content,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Debug, Eq, PartialEq)]
struct ProblemDocument {
    sections: BTreeMap<ProblemSection, String>,
}

#[derive(Debug, Eq, PartialEq)]
struct IdeaDocument {
    abstract_text: String,
    content_text: String,
}

#[derive(Debug)]
struct IdeaEntry {
    id: u32,
    path: PathBuf,
}

#[cfg_attr(not(test), allow(dead_code))]
fn parse_problem_document(markdown: &str) -> Result<ProblemDocument, String> {
    let mut title_seen = false;
    let mut current_section = None;
    let mut seen_sections = BTreeSet::new();
    let mut sections = BTreeMap::new();
    let mut heading_level = None;
    let mut heading_text = String::new();

    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = Some(level);
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                let level = heading_level
                    .take()
                    .ok_or_else(|| "invalid markdown heading state".to_string())?;
                let text = heading_text.trim().to_string();

                if !title_seen {
                    if level != HeadingLevel::H1 || text != "Problem" {
                        return Err("problem file must begin with `# Problem`".to_string());
                    }

                    title_seen = true;
                    continue;
                }

                match level {
                    HeadingLevel::H1 => {
                        return Err("problem file only allows a single level-1 heading".to_string());
                    }
                    HeadingLevel::H2 => {
                        let section = parse_problem_section_name(&text)?;

                        if !seen_sections.insert(section) {
                            return Err(format!("duplicate level-2 heading `{text}`"));
                        }

                        current_section = Some(section);
                        sections.entry(section).or_insert_with(String::new);
                    }
                    HeadingLevel::H3 | HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => {
                        if current_section.is_none() {
                            return Err(
                                "content must belong to one of the fixed sections".to_string()
                            );
                        }
                    }
                }
            }
            _ => {
                if heading_level.is_some() {
                    push_event_text(&mut heading_text, &event);
                    continue;
                }

                if !title_seen {
                    if event_has_non_whitespace_text(&event) {
                        return Err("problem file must begin with `# Problem`".to_string());
                    }

                    continue;
                }

                let Some(section) = current_section else {
                    if event_has_non_whitespace_text(&event) {
                        return Err("content must belong to one of the fixed sections".to_string());
                    }

                    continue;
                };

                push_section_event_text(
                    sections.entry(section).or_insert_with(String::new),
                    &event,
                );
            }
        }
    }

    if !title_seen {
        return Err("problem file must begin with `# Problem`".to_string());
    }

    for required in [
        ProblemSection::Abstract,
        ProblemSection::Goal,
        ProblemSection::Constraints,
        ProblemSection::OpenQuestions,
    ] {
        if !seen_sections.contains(&required) {
            return Err(format!(
                "missing required level-2 heading `{}`",
                problem_section_name(required)
            ));
        }
    }

    trim_problem_sections(&mut sections);

    Ok(ProblemDocument { sections })
}

fn parse_idea_document(markdown: &str) -> Result<IdeaDocument, String> {
    let mut current_section = None;
    let mut saw_abstract = false;
    let mut saw_content = false;
    let mut heading_level = None;
    let mut heading_text = String::new();
    let mut abstract_text = String::new();
    let mut content_text = String::new();

    for event in Parser::new(markdown) {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading_level = Some(level);
                heading_text.clear();
            }
            Event::End(TagEnd::Heading(_)) => {
                let level = heading_level
                    .take()
                    .ok_or_else(|| "invalid markdown heading state".to_string())?;
                let text = heading_text.trim().to_string();

                match level {
                    HeadingLevel::H1 => {
                        return Err("idea files do not allow level-1 headings".to_string());
                    }
                    HeadingLevel::H2 => match text.as_str() {
                        "Abstract" if !saw_abstract && !saw_content => {
                            saw_abstract = true;
                            current_section = Some(IdeaSection::Abstract);
                        }
                        "Abstract" => {
                            return Err("`Abstract` must come first and appear once".to_string());
                        }
                        "Content" if saw_abstract && !saw_content => {
                            saw_content = true;
                            current_section = Some(IdeaSection::Content);
                        }
                        "Content" if !saw_abstract => {
                            return Err("`Abstract` must come first".to_string());
                        }
                        "Content" => {
                            return Err("`Content` must come second and appear once".to_string());
                        }
                        _ => {
                            return Err(format!("unknown level-2 heading `{text}`"));
                        }
                    },
                    HeadingLevel::H3 | HeadingLevel::H4 | HeadingLevel::H5 | HeadingLevel::H6 => {
                        if current_section != Some(IdeaSection::Content) {
                            return Err(
                                "nested headings are only allowed under `Content`".to_string()
                            );
                        }
                    }
                }
            }
            _ => {
                if heading_level.is_some() {
                    push_event_text(&mut heading_text, &event);
                    continue;
                }

                let Some(section) = current_section else {
                    if event_has_non_whitespace_text(&event) {
                        return Err(
                            "content must belong to either `Abstract` or `Content`".to_string()
                        );
                    }

                    continue;
                };

                match section {
                    IdeaSection::Abstract => push_section_event_text(&mut abstract_text, &event),
                    IdeaSection::Content => push_section_event_text(&mut content_text, &event),
                }
            }
        }
    }

    if !saw_abstract {
        return Err("missing required level-2 heading `Abstract`".to_string());
    }

    if !saw_content {
        return Err("missing required level-2 heading `Content`".to_string());
    }

    Ok(IdeaDocument {
        abstract_text: abstract_text.trim().to_string(),
        content_text: content_text.trim().to_string(),
    })
}

#[cfg_attr(not(test), allow(dead_code))]
fn parse_problem_section_name(name: &str) -> Result<ProblemSection, String> {
    match name {
        "Abstract" => Ok(ProblemSection::Abstract),
        "Goal" => Ok(ProblemSection::Goal),
        "Constraints" => Ok(ProblemSection::Constraints),
        "Open Questions" => Ok(ProblemSection::OpenQuestions),
        _ => Err(format!("unknown level-2 heading `{name}`")),
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn problem_section_name(section: ProblemSection) -> &'static str {
    match section {
        ProblemSection::Abstract => "Abstract",
        ProblemSection::Goal => "Goal",
        ProblemSection::Constraints => "Constraints",
        ProblemSection::OpenQuestions => "Open Questions",
    }
}

#[cfg_attr(not(test), allow(dead_code))]
fn trim_problem_sections(sections: &mut BTreeMap<ProblemSection, String>) {
    for value in sections.values_mut() {
        *value = value.trim().to_string();
    }
}

fn push_event_text(buffer: &mut String, event: &Event<'_>) {
    match event {
        Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
            buffer.push_str(text);
        }
        Event::SoftBreak | Event::HardBreak => buffer.push(' '),
        _ => {}
    }
}

fn push_section_event_text(buffer: &mut String, event: &Event<'_>) {
    match event {
        Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
            buffer.push_str(text);
        }
        Event::SoftBreak | Event::HardBreak => buffer.push('\n'),
        Event::End(TagEnd::Paragraph)
        | Event::End(TagEnd::BlockQuote(_))
        | Event::End(TagEnd::CodeBlock)
        | Event::End(TagEnd::Item)
        | Event::End(TagEnd::List(_)) => buffer.push('\n'),
        _ => {}
    }
}

fn event_has_non_whitespace_text(event: &Event<'_>) -> bool {
    match event {
        Event::Text(text) | Event::Code(text) | Event::Html(text) | Event::InlineHtml(text) => {
            !text.trim().is_empty()
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ProblemSection, parse_idea_document, parse_idea_id_from_path, parse_problem_document,
    };
    use std::collections::BTreeMap;
    use std::path::Path;

    #[test]
    fn problem_parser_accepts_required_sections_in_any_order() {
        let document = parse_problem_document(
            "# Problem\n\n## Goal\n\nReach orbit.\n\n## Abstract\n\nA short overview.\n\n## Open Questions\n\n### Feasibility\n\nCan it scale?\n\n## Constraints\n\nLow budget.\n",
        )
        .expect("problem document should parse");

        let expected = BTreeMap::from([
            (ProblemSection::Abstract, "A short overview.".to_string()),
            (ProblemSection::Goal, "Reach orbit.".to_string()),
            (ProblemSection::Constraints, "Low budget.".to_string()),
            (ProblemSection::OpenQuestions, "Can it scale?".to_string()),
        ]);

        assert_eq!(document.sections, expected);
    }

    #[test]
    fn problem_parser_rejects_unknown_level_two_heading() {
        let error = parse_problem_document(
            "# Problem\n\n## Abstract\n\nA short overview.\n\n## Risks\n\nUnknown.\n\n## Goal\n\nReach orbit.\n\n## Constraints\n\nLow budget.\n\n## Open Questions\n\nCan it scale?\n",
        )
        .expect_err("problem document should be rejected");

        assert!(error.contains("unknown level-2 heading `Risks`"));
    }

    #[test]
    fn problem_parser_rejects_duplicate_required_heading() {
        let error = parse_problem_document(
            "# Problem\n\n## Abstract\n\nA short overview.\n\n## Goal\n\nReach orbit.\n\n## Constraints\n\nLow budget.\n\n## Goal\n\nReach orbit again.\n\n## Open Questions\n\nCan it scale?\n",
        )
        .expect_err("duplicate section should be rejected");

        assert!(error.contains("duplicate level-2 heading `Goal`"));
    }

    #[test]
    fn idea_parser_accepts_required_template() {
        let document = parse_idea_document(
            "## Abstract\n\nShort summary of the idea.\n\n## Content\n\nMain idea text.\n\n### Details\n\nMore detail.\n",
        )
        .expect("idea document should parse");

        assert_eq!(document.abstract_text, "Short summary of the idea.");
        assert!(document.content_text.contains("Main idea text."));
    }

    #[test]
    fn idea_parser_rejects_extra_level_two_heading() {
        let error = parse_idea_document(
            "## Abstract\n\nShort summary of the idea.\n\n## Content\n\nMain idea text.\n\n## Notes\n\nMore detail.\n",
        )
        .expect_err("extra section should be rejected");

        assert!(error.contains("unknown level-2 heading `Notes`"));
    }

    #[test]
    fn idea_parser_rejects_nested_heading_under_abstract() {
        let error = parse_idea_document(
            "## Abstract\n\nShort summary of the idea.\n\n### Details\n\nMore detail.\n\n## Content\n\nMain idea text.\n",
        )
        .expect_err("nested heading under abstract should be rejected");

        assert!(error.contains("nested headings are only allowed under `Content`"));
    }

    #[test]
    fn idea_file_names_require_zero_padded_four_digit_ids() {
        assert_eq!(
            parse_idea_id_from_path(Path::new("ideas/idea-0001.md")),
            Some(1)
        );
        assert_eq!(parse_idea_id_from_path(Path::new("ideas/idea-1.md")), None);
        assert_eq!(
            parse_idea_id_from_path(Path::new("ideas/idea-001.md")),
            None
        );
        assert_eq!(
            parse_idea_id_from_path(Path::new("ideas/idea-000a.md")),
            None
        );
    }
}
