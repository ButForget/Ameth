# Ameth

Ameth is an early-stage Rust CLI for organizing research work so humans and LLMs can understand the problem, relevant materials, code, and experiments with less guesswork.

## Status

This repository is still in very early development.

- Project initialization is implemented.
- The `ideas` and `rq` command namespaces are implemented.
- `solutions/` and `logs/` are now created as part of the managed project layout.

## Why Ameth?

Research projects become hard to navigate when ideas, references, code, and experiment results are scattered or named inconsistently. That makes it harder for both people and LLMs to answer questions like:

- What problem is this project solving?
- Which ideas have already been tried?
- What references matter?
- Which experiments belong to which hypothesis?

Ameth is meant to provide a predictable project structure for research work, so context is easier to recover and easier to use.

## Intended Project Structure

The planned research layout is centered around these top-level directories:

- `ResearchQuestion.md`
- `ideas/`
- `solutions/`
- `logs/`
- `relevants/`
- `code/`
- `experiments/`

### `ideas/`

The `ideas/` directory stores raw idea documents.

- Idea files follow a naming pattern like `idea-0001.md`.
- Abandoned ideas go under `ideas/abandoned/`.
- `Ameth.toml` stores project metadata including the root editor command and pinned idea ID.
- Idea files use fixed machine-parseable sections: `Abstract` and `Content`.
- Nested headings are allowed inside the fixed sections, but only at level 3 or deeper.

### `ResearchQuestion.md`

`ResearchQuestion.md` lives at the project root.

- It is a free-form background file for humans and LLMs.
- Ameth does not enforce any heading structure or markdown schema for it.
- `ameth rq show` prints it as-is.
- `ameth rq edit` opens it with the root-level `editor` from `Ameth.toml`.

### `solutions/`

The `solutions/` directory is intended for more structured solution documents promoted from promising ideas.

### `logs/`

The `logs/` directory is reserved for research logs and currently acts as a placeholder.

The planned `ideas` workflow is specified in `ideas.md`.

## TODO

- Define the `solutions/` document workflow in more detail.
- Keep `logs/` as a placeholder until its workflow is designed.

## Current Repository Layout

This repo currently contains:

- `Cargo.toml` for the single Rust package
- `src/main.rs` as the thin entrypoint
- `src/cli/` for root CLI dispatch and whole-program help
- `src/commands/` for per-command parsing, help text, and execution
- `PROJECT_STATE.md` for a brief project status note
- `ideas.md` for the planned ideas-management format and command behavior

## Development

### Prerequisites

- Rust toolchain with Cargo installed

### Useful Commands

```bash
cargo run -- init demo
cargo test
cargo fmt --check
```

## Current CLI

Ameth currently supports project initialization, idea management, and root research-question management with:

```bash
ameth
ameth init <name> [path]
ameth ideas [command]
ameth rq [command]
```

Behavior:

- `ameth` prints the whole-program introduction and root help.
- `ameth init <name> [path]` initializes a project.
- `<name>` becomes the new project directory name.
- `[path]` is the parent directory and defaults to `.`.
- The command fails if `[path]/<name>` already exists.
- `ameth init` creates `Ameth.toml` for project metadata.
- `ameth ideas new [--abs <ABSTRACT>] [--ctt <CONTENT>]` creates the next `idea-000N.md` file.
- If either idea field is omitted, `ameth ideas new` opens the root-level `editor` configured in `Ameth.toml` after writing the template.
- `editor = "nvim"` configures a simple editor command; `editor = ["code", "--wait"]` configures an editor plus fixed arguments.
- `ameth ideas list` lists active ideas and their abstract text.
- `ameth ideas show <id>` shows an active or abandoned idea.
- `ameth ideas show` shows the pinned idea.
- `ameth ideas pin <id>` records the pinned idea in `Ameth.toml`.
- `ameth ideas abandon <id>` moves an idea into `ideas/abandoned/`.
- `ameth ideas restore <id>` moves an idea back into `ideas/`.
- Bare `ameth ideas` shows the pinned idea when one is set, and otherwise prints ideas help.
- `ameth rq show` prints the root `ResearchQuestion.md` file.
- `ameth rq edit` opens the existing root `ResearchQuestion.md` file.
- `ameth rq edit -n` creates `ResearchQuestion.md` when it is missing, then opens it.
- `ameth rq edit -f -n` recreates `ResearchQuestion.md` even when it already exists, then opens it.
- `ameth rq show` and `ameth rq edit` fail when `ResearchQuestion.md` is missing.

It creates this layout:

- `ideas/`
- `ideas/abandoned/`
- `solutions/`
- `logs/`
- `relevants/`
- `code/`
- `experiments/`
- `Ameth.toml`
- `ResearchQuestion.md`
