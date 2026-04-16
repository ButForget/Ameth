use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

#[test]
fn root_help_lists_config_command() {
    ameth_command()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("config"));
}

#[test]
fn config_sets_editor_string_values() {
    let (_temp_dir, project_root) = init_project();

    ameth_command()
        .current_dir(&project_root)
        .args(["config", "editor", "nvim"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated Ameth.toml: editor"));

    let config = read_config(&project_root);
    assert_eq!(
        config.get("editor").and_then(toml::Value::as_str),
        Some("nvim")
    );
}

#[test]
fn config_parses_toml_array_values() {
    let (_temp_dir, project_root) = init_project();

    ameth_command()
        .current_dir(&project_root)
        .args(["config", "editor", "[\"code\", \"--wait\"]"])
        .assert()
        .success();

    let config = read_config(&project_root);
    let editor = config
        .get("editor")
        .and_then(toml::Value::as_array)
        .expect("editor should be stored as an array");

    assert_eq!(editor.len(), 2);
    assert_eq!(editor[0].as_str(), Some("code"));
    assert_eq!(editor[1].as_str(), Some("--wait"));
}

#[test]
fn config_sets_dotted_keys() {
    let (_temp_dir, project_root) = init_project();

    ameth_command()
        .current_dir(&project_root)
        .args(["config", "ideas.pinned", "4"])
        .assert()
        .success();

    let config = read_config(&project_root);
    let ideas = config
        .get("ideas")
        .and_then(toml::Value::as_table)
        .expect("ideas should be stored as a table");

    assert_eq!(
        ideas.get("pinned").and_then(toml::Value::as_integer),
        Some(4)
    );
}

#[test]
fn config_rejects_invalid_pinned_values() {
    let (_temp_dir, project_root) = init_project();

    ameth_command()
        .current_dir(&project_root)
        .args(["config", "ideas.pinned", "0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid ideas.pinned value"));
}

#[test]
fn config_rejects_invalid_reserved_nested_keys() {
    let (_temp_dir, project_root) = init_project();

    ameth_command()
        .current_dir(&project_root)
        .args(["config", "editor.foo", "bar"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid root-level `editor`"));
}

#[test]
fn config_requires_an_ameth_project() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");

    ameth_command()
        .current_dir(temp_dir.path())
        .args(["config", "editor", "nvim"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "current directory is not an Ameth project",
        ));
}

fn init_project() -> (TempDir, PathBuf) {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");

    ameth_command()
        .current_dir(temp_dir.path())
        .args(["init", "demo"])
        .assert()
        .success();

    let project_root = temp_dir.path().join("demo");
    (temp_dir, project_root)
}

fn read_config(project_root: &Path) -> toml::Table {
    let config_path = project_root.join("Ameth.toml");
    let content = fs::read_to_string(&config_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", config_path.display()));

    toml::from_str(&content)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", config_path.display()))
}

fn ameth_command() -> Command {
    Command::cargo_bin("ameth").expect("ameth binary should build")
}
