use super::document::parse_idea_document;
use crate::config::{AMETH_TOML_FILE_NAME, AmethConfig};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(super) struct IdeasProject {
    config_path: PathBuf,
    ideas_dir: PathBuf,
    abandoned_dir: PathBuf,
}

impl IdeasProject {
    pub(super) fn load() -> Result<Self, String> {
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

    pub(super) fn active_idea_path(&self, id: u32) -> PathBuf {
        self.ideas_dir.join(idea_file_name(id))
    }

    pub(super) fn next_idea_id(&self) -> Result<u32, String> {
        let active_max = read_idea_directory(&self.ideas_dir)?
            .into_iter()
            .map(|idea| idea.id)
            .max()
            .unwrap_or(0);
        let abandoned_max = read_idea_directory(&self.abandoned_dir)?
            .into_iter()
            .map(|idea| idea.id)
            .max()
            .unwrap_or(0);

        Ok(active_max.max(abandoned_max) + 1)
    }

    pub(super) fn open_configured_editor(&self, path: &Path) -> Result<(), String> {
        let config = AmethConfig::load_or_default(&self.config_path)?;
        let (program, args) = config.editor_command().ok_or_else(|| {
            format!(
                "missing root-level `editor` in {}; configure it before using interactive `ameth ideas new`",
                self.config_path.display()
            )
        })?;

        let status = Command::new(program)
            .args(args)
            .arg(path)
            .status()
            .map_err(|error| {
                format!(
                    "failed to launch editor `{program}` for {}: {error}",
                    path.display()
                )
            })?;

        if !status.success() {
            return Err(format!(
                "editor `{program}` exited unsuccessfully: {status}"
            ));
        }

        Ok(())
    }

    pub(super) fn active_ideas(&self) -> Result<Vec<IdeaEntry>, String> {
        read_idea_directory(&self.ideas_dir)
    }

    pub(super) fn read_idea_markdown(&self, id: u32) -> Result<String, String> {
        let path = resolve_idea_path(self, id)?;
        let markdown = fs::read_to_string(&path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        parse_idea_document(&markdown)
            .map_err(|error| format!("failed to parse {}: {error}", path.display()))?;

        Ok(markdown)
    }

    pub(super) fn read_pinned_id(&self) -> Result<Option<u32>, String> {
        let config = AmethConfig::load_or_default(&self.config_path)?;
        Ok(config.pinned_id())
    }

    pub(super) fn write_pinned_id(&self, id: u32) -> Result<(), String> {
        let mut config = AmethConfig::load_or_default(&self.config_path)?;
        config.set_pinned_id(id);
        config.save(&self.config_path)
    }

    pub(super) fn abandon_idea(&self, id: u32) -> Result<(), String> {
        move_idea(
            id,
            &self.active_idea_path(id),
            &self.abandoned_dir.join(idea_file_name(id)),
            "active",
            "abandoned",
        )
    }

    pub(super) fn restore_idea(&self, id: u32) -> Result<(), String> {
        move_idea(
            id,
            &self.abandoned_dir.join(idea_file_name(id)),
            &self.active_idea_path(id),
            "abandoned",
            "active",
        )
    }
}

#[derive(Debug)]
pub(super) struct IdeaEntry {
    pub(super) id: u32,
    pub(super) path: PathBuf,
}

pub(super) fn format_idea_id(id: u32) -> String {
    format!("{:04}", id)
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

#[cfg(test)]
mod tests {
    use super::parse_idea_id_from_path;
    use std::path::Path;

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
