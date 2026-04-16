use assert_cmd::Command;
use predicates::prelude::*;
use std::error::Error;

#[test]
fn root_help_prints_program_introduction() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("ameth")?
        .assert()
        .success()
        .stdout(predicate::str::contains("Ameth organizes research work"))
        .stdout(predicate::str::contains("Usage: ameth [COMMAND]"))
        .stdout(predicate::str::contains("rq"))
        .stdout(predicate::str::contains("ameth <name> [path]").not());

    Ok(())
}

#[test]
fn bare_project_name_is_rejected_as_an_unknown_subcommand() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;

    Command::cargo_bin("ameth")?
        .current_dir(temp_dir.path())
        .arg("demo")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand 'demo'"));

    Ok(())
}

#[test]
fn explicit_init_subcommand_creates_project() -> Result<(), Box<dyn Error>> {
    let temp_dir = tempfile::tempdir()?;

    Command::cargo_bin("ameth")?
        .current_dir(temp_dir.path())
        .args(["init", "demo"])
        .assert()
        .success();

    assert!(temp_dir.path().join("demo/ResearchQuestion.md").is_file());

    Ok(())
}

#[test]
fn init_help_is_command_specific() -> Result<(), Box<dyn Error>> {
    Command::cargo_bin("ameth")?
        .args(["init", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Initialize an Ameth project"))
        .stdout(predicate::str::contains("Usage: ameth init <NAME> [PATH]"));

    Ok(())
}
