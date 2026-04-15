use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

pub const USAGE: &str = "ameth ideas <command>";
pub const HELP: &str = "Manage idea files.\n\nUsage:\n  ameth ideas new\n  ameth ideas list\n  ameth ideas show <id>\n  ameth ideas abandon <id>\n  ameth ideas restore <id>\n\nCommands:\n  new       Create the next idea file\n  list      List active ideas\n  show      Display an idea by ID\n  abandon   Move an active idea into ideas/abandoned/\n  restore   Move an abandoned idea back into ideas/\n";

const NEW_USAGE: &str = "ameth ideas new";
const LIST_USAGE: &str = "ameth ideas list";
const SHOW_USAGE: &str = "ameth ideas show <id>";
const ABANDON_USAGE: &str = "ameth ideas abandon <id>";
const RESTORE_USAGE: &str = "ameth ideas restore <id>";

const NEW_HELP: &str = "Create the next idea file.\n\nUsage:\n  ameth ideas new\n";
const LIST_HELP: &str = "List active ideas.\n\nUsage:\n  ameth ideas list\n";
const SHOW_HELP: &str = "Display an idea by ID.\n\nUsage:\n  ameth ideas show <id>\n";
const ABANDON_HELP: &str =
    "Move an active idea into ideas/abandoned/.\n\nUsage:\n  ameth ideas abandon <id>\n";
const RESTORE_HELP: &str =
    "Move an abandoned idea back into ideas/.\n\nUsage:\n  ameth ideas restore <id>\n";

const IDEA_TEMPLATE: &str = "## Abstract\n\n## Content\n";

pub fn run(args: Vec<OsString>) -> Result<(), String> {
    if args.is_empty() || (args.len() == 1 && is_help_flag(&args[0])) {
        print!("{HELP}");
        return Ok(());
    }

    let subcommand = args[0]
        .to_str()
        .ok_or_else(|| "subcommand must be valid UTF-8".to_string())?;

    match subcommand {
        "new" => run_new(&args[1..]),
        "list" => run_list(&args[1..]),
        "show" => run_show(&args[1..]),
        "abandon" => run_abandon(&args[1..]),
        "restore" => run_restore(&args[1..]),
        _ => Err(format!("invalid arguments\n\nUsage:\n  {USAGE}")),
    }
}

fn run_new(args: &[OsString]) -> Result<(), String> {
    if args.len() == 1 && is_help_flag(&args[0]) {
        print!("{NEW_HELP}");
        return Ok(());
    }

    if !args.is_empty() {
        return Err(format!("invalid arguments\n\nUsage:\n  {NEW_USAGE}"));
    }

    let project = IdeasProject::load()?;
    let next_id = next_idea_id(&project)?;
    let path = project.ideas_dir.join(idea_file_name(next_id));

    fs::write(&path, IDEA_TEMPLATE)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))?;

    println!("Created {}", path.display());
    Ok(())
}

fn run_list(args: &[OsString]) -> Result<(), String> {
    if args.len() == 1 && is_help_flag(&args[0]) {
        print!("{LIST_HELP}");
        return Ok(());
    }

    if !args.is_empty() {
        return Err(format!("invalid arguments\n\nUsage:\n  {LIST_USAGE}"));
    }

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

fn run_show(args: &[OsString]) -> Result<(), String> {
    if args.len() == 1 && is_help_flag(&args[0]) {
        print!("{SHOW_HELP}");
        return Ok(());
    }

    if args.len() != 1 {
        return Err(format!("invalid arguments\n\nUsage:\n  {SHOW_USAGE}"));
    }

    let project = IdeasProject::load()?;
    let id = parse_idea_id_argument(&args[0])?;
    let file_name = idea_file_name(id);
    let active_path = project.ideas_dir.join(&file_name);
    let abandoned_path = project.abandoned_dir.join(&file_name);

    let path = match (active_path.is_file(), abandoned_path.is_file()) {
        (true, true) => {
            eprintln!(
                "warning: idea {} exists in both ideas/ and ideas/abandoned/; showing the active idea",
                format_idea_id(id)
            );
            active_path
        }
        (true, false) => active_path,
        (false, true) => abandoned_path,
        (false, false) => return Err(format!("idea {} not found", format_idea_id(id))),
    };

    let markdown = fs::read_to_string(&path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    parse_idea_document(&markdown)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

    print!("{markdown}");
    if !markdown.ends_with('\n') {
        println!();
    }

    Ok(())
}

fn run_abandon(args: &[OsString]) -> Result<(), String> {
    if args.len() == 1 && is_help_flag(&args[0]) {
        print!("{ABANDON_HELP}");
        return Ok(());
    }

    if args.len() != 1 {
        return Err(format!("invalid arguments\n\nUsage:\n  {ABANDON_USAGE}"));
    }

    let project = IdeasProject::load()?;
    let id = parse_idea_id_argument(&args[0])?;
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

fn run_restore(args: &[OsString]) -> Result<(), String> {
    if args.len() == 1 && is_help_flag(&args[0]) {
        print!("{RESTORE_HELP}");
        return Ok(());
    }

    if args.len() != 1 {
        return Err(format!("invalid arguments\n\nUsage:\n  {RESTORE_USAGE}"));
    }

    let project = IdeasProject::load()?;
    let id = parse_idea_id_argument(&args[0])?;
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

fn is_help_flag(argument: &OsString) -> bool {
    argument == "--help" || argument == "-h"
}

fn parse_idea_id_argument(argument: &OsString) -> Result<u32, String> {
    let raw = argument
        .to_str()
        .ok_or_else(|| "idea id must be valid UTF-8".to_string())?;
    let id = raw
        .parse::<u32>()
        .map_err(|_| "idea id must be a positive integer".to_string())?;

    if id == 0 {
        return Err("idea id must be a positive integer".to_string());
    }

    Ok(id)
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
    ideas_dir: PathBuf,
    abandoned_dir: PathBuf,
}

impl IdeasProject {
    fn load() -> Result<Self, String> {
        let root = env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?;
        let ideas_dir = root.join("ideas");
        let abandoned_dir = ideas_dir.join("abandoned");
        let problem_file = ideas_dir.join("Problem.md");

        if !ideas_dir.is_dir() || !abandoned_dir.is_dir() || !problem_file.is_file() {
            return Err("current directory is not an Ameth project".to_string());
        }

        Ok(Self {
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
