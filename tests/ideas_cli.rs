use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const PROBLEM_TEMPLATE: &str =
    "# Problem\n\n## Abstract\n\n## Goal\n\n## Constraints\n\n## Open Questions\n";
const IDEA_TEMPLATE: &str = "## Abstract\n\n## Content\n";
const AMETH_TOML_TEMPLATE: &str = "[ideas]\n";

#[test]
fn init_creates_the_full_ideas_project_layout() {
    let (_workspace, project_root) = init_project("demo");

    assert!(project_root.join("ideas").is_dir());
    assert!(project_root.join("ideas/abandoned").is_dir());
    assert!(project_root.join("solutions").is_dir());
    assert!(project_root.join("logs").is_dir());
    assert!(project_root.join("relevants").is_dir());
    assert!(project_root.join("code").is_dir());
    assert!(project_root.join("experiments").is_dir());
    assert_eq!(
        fs::read_to_string(project_root.join("Ameth.toml")).expect("config file should exist"),
        AMETH_TOML_TEMPLATE
    );
    assert_eq!(
        fs::read_to_string(project_root.join("ideas/Problem.md"))
            .expect("problem file should exist"),
        PROBLEM_TEMPLATE
    );
}

#[test]
fn ideas_new_creates_the_first_idea_template() {
    let (_workspace, project_root) = init_project("demo");

    command_in(&project_root)
        .args(["ideas", "new"])
        .assert()
        .success()
        .stdout(predicate::str::contains("idea-0001.md"))
        .stderr(predicate::str::is_empty());

    assert_eq!(
        fs::read_to_string(project_root.join("ideas/idea-0001.md"))
            .expect("idea file should exist"),
        IDEA_TEMPLATE
    );
}

#[test]
fn ideas_new_uses_the_max_id_from_active_and_abandoned_ideas() {
    let (_workspace, project_root) = init_project("demo");

    write_active_idea(
        &project_root,
        2,
        idea_markdown("Active summary", "Active content"),
    );
    write_abandoned_idea(
        &project_root,
        7,
        idea_markdown("Abandoned summary", "Abandoned content"),
    );

    command_in(&project_root)
        .args(["ideas", "new"])
        .assert()
        .success()
        .stdout(predicate::str::contains("idea-0008.md"));

    assert!(project_root.join("ideas/idea-0008.md").is_file());
    assert!(!project_root.join("ideas/idea-0003.md").exists());
}

#[test]
fn ideas_new_fails_outside_an_ameth_project() {
    let workspace = TempDir::new().expect("temporary directory should be created");

    command_in(workspace.path())
        .args(["ideas", "new"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: current directory is not an Ameth project",
        ));
}

#[test]
fn ideas_list_shows_active_ideas_and_excludes_abandoned_ones() {
    let (_workspace, project_root) = init_project("demo");

    write_active_idea(
        &project_root,
        1,
        idea_markdown("Alpha summary", "Alpha content"),
    );
    write_active_idea(
        &project_root,
        2,
        idea_markdown("Beta summary", "Beta content"),
    );
    write_abandoned_idea(
        &project_root,
        3,
        idea_markdown("Gamma summary", "Gamma content"),
    );

    command_in(&project_root)
        .args(["ideas", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0001  Alpha summary"))
        .stdout(predicate::str::contains("0002  Beta summary"))
        .stdout(predicate::str::contains("0003").not())
        .stderr(predicate::str::is_empty());
}

#[test]
fn ideas_list_reports_when_no_active_ideas_exist() {
    let (_workspace, project_root) = init_project("demo");

    command_in(&project_root)
        .args(["ideas", "list"])
        .assert()
        .success()
        .stdout("No active ideas found.\n")
        .stderr(predicate::str::is_empty());
}

#[test]
fn ideas_list_rejects_malformed_active_idea_files() {
    let (_workspace, project_root) = init_project("demo");

    fs::write(
        project_root.join("ideas/idea-0001.md"),
        "## Abstract\n\nOnly an abstract section.\n",
    )
    .expect("malformed idea file should be written");

    command_in(&project_root)
        .args(["ideas", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error: failed to parse"))
        .stderr(predicate::str::contains("idea-0001.md"));
}

#[test]
fn ideas_show_reads_active_and_abandoned_ideas() {
    let (_workspace, project_root) = init_project("demo");

    write_active_idea(
        &project_root,
        1,
        idea_markdown("Active summary", "Active content"),
    );
    write_abandoned_idea(
        &project_root,
        2,
        idea_markdown("Abandoned summary", "Abandoned content"),
    );

    command_in(&project_root)
        .args(["ideas", "show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active summary"))
        .stdout(predicate::str::contains("Active content"))
        .stderr(predicate::str::is_empty());

    command_in(&project_root)
        .args(["ideas", "show", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Abandoned summary"))
        .stdout(predicate::str::contains("Abandoned content"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn ideas_show_without_an_id_uses_the_pinned_idea() {
    let (_workspace, project_root) = init_project("demo");

    write_active_idea(
        &project_root,
        4,
        idea_markdown("Pinned summary", "Pinned content"),
    );

    command_in(&project_root)
        .args(["ideas", "pin", "4"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pinned idea 0004"));

    command_in(&project_root)
        .args(["ideas", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pinned summary"))
        .stdout(predicate::str::contains("Pinned content"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn bare_ideas_uses_the_pinned_idea() {
    let (_workspace, project_root) = init_project("demo");

    write_active_idea(
        &project_root,
        5,
        idea_markdown("Bare summary", "Bare content"),
    );

    command_in(&project_root)
        .args(["ideas", "pin", "5"])
        .assert()
        .success();

    command_in(&project_root)
        .args(["ideas"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Bare summary"))
        .stdout(predicate::str::contains("Bare content"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn bare_ideas_shows_help_when_no_pin_exists() {
    let (_workspace, project_root) = init_project("demo");

    command_in(&project_root)
        .args(["ideas"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage idea files."))
        .stdout(predicate::str::contains("ameth ideas pin <id>"))
        .stderr(predicate::str::is_empty());
}

#[test]
fn ideas_show_without_an_id_fails_when_no_pin_exists() {
    let (_workspace, project_root) = init_project("demo");

    command_in(&project_root)
        .args(["ideas", "show"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error: no pinned idea set"));
}

#[test]
fn ideas_pin_persists_an_active_idea_id() {
    let (_workspace, project_root) = init_project("demo");

    write_active_idea(
        &project_root,
        1,
        idea_markdown("Pin active summary", "Pin active content"),
    );

    command_in(&project_root)
        .args(["ideas", "pin", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pinned idea 0001"))
        .stderr(predicate::str::is_empty());

    assert!(
        fs::read_to_string(project_root.join("Ameth.toml"))
            .expect("config file should exist")
            .contains("pinned = 1")
    );
}

#[test]
fn ideas_pin_accepts_an_abandoned_idea_id() {
    let (_workspace, project_root) = init_project("demo");

    write_abandoned_idea(
        &project_root,
        2,
        idea_markdown("Pin abandoned summary", "Pin abandoned content"),
    );

    command_in(&project_root)
        .args(["ideas", "pin", "2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pinned idea 0002"));

    assert!(
        fs::read_to_string(project_root.join("Ameth.toml"))
            .expect("config file should exist")
            .contains("pinned = 2")
    );
}

#[test]
fn ideas_pin_preserves_existing_ameth_toml_fields() {
    let (_workspace, project_root) = init_project("demo");

    write_active_idea(
        &project_root,
        3,
        idea_markdown("Preserve summary", "Preserve content"),
    );
    fs::write(
        project_root.join("Ameth.toml"),
        "name = \"demo\"\n\n[ideas]\n",
    )
    .expect("config file should be overwritten for the test");

    command_in(&project_root)
        .args(["ideas", "pin", "3"])
        .assert()
        .success();

    let config =
        fs::read_to_string(project_root.join("Ameth.toml")).expect("config file should exist");
    assert!(config.contains("name = \"demo\""));
    assert!(config.contains("pinned = 3"));
}

#[test]
fn ideas_pin_fails_for_missing_ids() {
    let (_workspace, project_root) = init_project("demo");

    command_in(&project_root)
        .args(["ideas", "pin", "9"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error: idea 0009 not found"));
}

#[test]
fn ideas_pin_fails_for_invalid_ids() {
    let (_workspace, project_root) = init_project("demo");

    command_in(&project_root)
        .args(["ideas", "pin", "abc"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: idea id must be a positive integer",
        ));
}

#[test]
fn ideas_pin_fails_outside_an_ameth_project() {
    let workspace = TempDir::new().expect("temporary directory should be created");

    command_in(workspace.path())
        .args(["ideas", "pin", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: current directory is not an Ameth project",
        ));
}

#[test]
fn stale_pinned_idea_ids_fail_cleanly() {
    let (_workspace, project_root) = init_project("demo");

    fs::write(project_root.join("Ameth.toml"), "[ideas]\npinned = 42\n")
        .expect("config file should be updated");

    command_in(&project_root)
        .args(["ideas", "show"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error: idea 0042 not found"));
}

#[test]
fn ideas_show_prefers_active_ideas_and_warns_when_ids_are_duplicated() {
    let (_workspace, project_root) = init_project("demo");

    write_active_idea(
        &project_root,
        7,
        idea_markdown("Active duplicate summary", "Active duplicate content"),
    );
    write_abandoned_idea(
        &project_root,
        7,
        idea_markdown("Abandoned duplicate summary", "Abandoned duplicate content"),
    );

    command_in(&project_root)
        .args(["ideas", "show", "7"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active duplicate summary"))
        .stdout(predicate::str::contains("Abandoned duplicate summary").not())
        .stderr(predicate::str::contains(
            "warning: idea 0007 exists in both",
        ));
}

#[test]
fn pinned_show_prefers_active_ideas_and_warns_when_ids_are_duplicated() {
    let (_workspace, project_root) = init_project("demo");

    write_active_idea(
        &project_root,
        7,
        idea_markdown("Pinned active summary", "Pinned active content"),
    );
    write_abandoned_idea(
        &project_root,
        7,
        idea_markdown("Pinned abandoned summary", "Pinned abandoned content"),
    );

    command_in(&project_root)
        .args(["ideas", "pin", "7"])
        .assert()
        .success();

    command_in(&project_root)
        .args(["ideas", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Pinned active summary"))
        .stdout(predicate::str::contains("Pinned abandoned summary").not())
        .stderr(predicate::str::contains(
            "warning: idea 0007 exists in both",
        ));
}

#[test]
fn ideas_show_rejects_invalid_ids() {
    let (_workspace, project_root) = init_project("demo");

    command_in(&project_root)
        .args(["ideas", "show", "abc"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: idea id must be a positive integer",
        ));
}

#[test]
fn ideas_abandon_and_restore_move_files_without_changing_content() {
    let (_workspace, project_root) = init_project("demo");
    let markdown = idea_markdown("Original summary", "Original content");

    write_active_idea(&project_root, 1, &markdown);

    command_in(&project_root)
        .args(["ideas", "abandon", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Abandoned idea 0001"));

    assert!(!project_root.join("ideas/idea-0001.md").exists());
    assert_eq!(
        fs::read_to_string(project_root.join("ideas/abandoned/idea-0001.md"))
            .expect("abandoned idea should exist"),
        markdown
    );

    command_in(&project_root)
        .args(["ideas", "show", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Original summary"))
        .stdout(predicate::str::contains("Original content"));

    command_in(&project_root)
        .args(["ideas", "restore", "1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Restored idea 0001"));

    assert!(!project_root.join("ideas/abandoned/idea-0001.md").exists());
    assert_eq!(
        fs::read_to_string(project_root.join("ideas/idea-0001.md"))
            .expect("restored idea should exist"),
        markdown
    );
}

#[test]
fn ideas_abandon_fails_when_the_abandoned_destination_already_exists() {
    let (_workspace, project_root) = init_project("demo");

    write_active_idea(
        &project_root,
        1,
        idea_markdown("Active summary", "Active content"),
    );
    write_abandoned_idea(
        &project_root,
        1,
        idea_markdown("Abandoned summary", "Abandoned content"),
    );

    command_in(&project_root)
        .args(["ideas", "abandon", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: cannot move idea 0001 to abandoned",
        ));
}

#[test]
fn ideas_restore_fails_when_the_active_destination_already_exists() {
    let (_workspace, project_root) = init_project("demo");

    write_active_idea(
        &project_root,
        1,
        idea_markdown("Active summary", "Active content"),
    );
    write_abandoned_idea(
        &project_root,
        1,
        idea_markdown("Abandoned summary", "Abandoned content"),
    );

    command_in(&project_root)
        .args(["ideas", "restore", "1"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "error: cannot move idea 0001 to active",
        ));
}

fn init_project(name: &str) -> (TempDir, PathBuf) {
    let workspace = TempDir::new().expect("temporary directory should be created");

    command_in(workspace.path())
        .args(["init", name])
        .assert()
        .success();

    let project_root = workspace.path().join(name);
    (workspace, project_root)
}

fn command_in(directory: &Path) -> Command {
    let mut command = Command::cargo_bin("ameth").expect("ameth binary should build");
    command.current_dir(directory);
    command
}

fn write_active_idea(project_root: &Path, id: u32, markdown: impl AsRef<str>) {
    fs::write(
        project_root.join(format!("ideas/idea-{id:04}.md")),
        markdown.as_ref(),
    )
    .expect("active idea should be written");
}

fn write_abandoned_idea(project_root: &Path, id: u32, markdown: impl AsRef<str>) {
    fs::write(
        project_root.join(format!("ideas/abandoned/idea-{id:04}.md")),
        markdown.as_ref(),
    )
    .expect("abandoned idea should be written");
}

fn idea_markdown(summary: &str, content: &str) -> String {
    format!("## Abstract\n\n{summary}\n\n## Content\n\n{content}\n")
}
