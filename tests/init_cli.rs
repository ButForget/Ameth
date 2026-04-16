use assert_cmd::Command;
use predicates::prelude::*;
use std::error::Error;
use std::fs;
use std::path::Path;

fn assert_project_layout(project_root: &Path) {
    assert!(
        project_root.is_dir(),
        "expected project directory at {project_root:?}"
    );
    assert!(project_root.join("ideas").is_dir());
    assert!(project_root.join("ideas/abandoned").is_dir());
    assert!(project_root.join("solutions").is_dir());
    assert!(project_root.join("logs").is_dir());
    assert!(project_root.join("relevants").is_dir());
    assert!(project_root.join("code").is_dir());
    assert!(project_root.join("experiments").is_dir());
    assert!(project_root.join("ResearchQuestion.md").is_file());
    assert!(project_root.join("Ameth.toml").is_file());
}

#[test]
fn init_creates_project_in_current_dir_when_path_is_omitted() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;

    Command::cargo_bin("ameth")?
        .current_dir(temp_dir.path())
        .args(["init", "demo"])
        .assert()
        .success();

    assert_project_layout(&temp_dir.path().join("demo"));

    Ok(())
}

#[test]
fn init_creates_project_under_explicit_parent_path() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;
    let workspace = temp_dir.path().join("workspace");
    fs::create_dir(&workspace)?;

    Command::cargo_bin("ameth")?
        .current_dir(temp_dir.path())
        .args(["init", "demo", "workspace"])
        .assert()
        .success();

    assert_project_layout(&workspace.join("demo"));

    Ok(())
}

#[test]
fn init_fails_when_target_already_exists() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;
    let workspace = temp_dir.path().join("workspace");
    let project_root = workspace.join("demo");
    fs::create_dir(&workspace)?;
    fs::create_dir(&project_root)?;

    Command::cargo_bin("ameth")?
        .current_dir(temp_dir.path())
        .args(["init", "demo", "workspace"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));

    Ok(())
}

#[test]
fn init_writes_research_question_template() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;

    Command::cargo_bin("ameth")?
        .current_dir(temp_dir.path())
        .args(["init", "demo"])
        .assert()
        .success();

    let research_question_path = temp_dir.path().join("demo/ResearchQuestion.md");
    let research_question = fs::read_to_string(research_question_path)?;
    assert!(!research_question.trim().is_empty());
    assert!(research_question.contains("# Research Question"));

    Ok(())
}

#[test]
fn init_rejects_invalid_name_with_path_separator() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;

    Command::cargo_bin("ameth")?
        .current_dir(temp_dir.path())
        .args(["init", "foo/bar"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("path separator"));

    Ok(())
}

#[test]
fn init_shows_help() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("ameth")?
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage: ameth [COMMAND]"))
        .stdout(predicate::str::contains("ameth init <NAME> [PATH]").not());

    Ok(())
}
