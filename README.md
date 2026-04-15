# Ameth

Ameth is an early-stage Rust CLI for organizing research work so humans and LLMs can understand the problem, relevant materials, code, and experiments with less guesswork.

## Status

This repository is still in very early development.

- The initial project initialization command is implemented.
- The `ideas` document format and command namespace are specified, but the commands are not implemented yet.
- `solutions/` and `logs/` are planned as part of the managed research layout.

## Why Ameth?

Research projects become hard to navigate when ideas, references, code, and experiment results are scattered or named inconsistently. That makes it harder for both people and LLMs to answer questions like:

- What problem is this project solving?
- Which ideas have already been tried?
- What references matter?
- Which experiments belong to which hypothesis?

Ameth is meant to provide a predictable project structure for research work, so context is easier to recover and easier to use.

## Intended Project Structure

The planned research layout is centered around these top-level directories:

- `ideas/`
- `solutions/`
- `logs/`
- `relevants/`
- `code/`
- `experiments/`

### `ideas/`

The `ideas/` directory stores the research problem and raw idea documents.

- `ideas/Problem.md` is the structured anchor for the research problem.
- Idea files follow a naming pattern like `idea-0001.md`.
- Abandoned ideas go under `ideas/abandoned/`.
- `Problem.md` uses fixed machine-parseable sections: `Abstract`, `Goal`, `Constraints`, and `Open Questions`.
- Idea files use fixed machine-parseable sections: `Abstract` and `Content`.
- Nested headings are allowed inside the fixed sections, but only at level 3 or deeper.

### `solutions/`

The `solutions/` directory is intended for more structured solution documents promoted from promising ideas.

### `logs/`

The `logs/` directory is reserved for research logs and currently acts as a placeholder.

The planned `ideas` workflow is specified in `ideas.md`.

## TODO

- Implement the `ameth ideas` command namespace.
- Extend project initialization to create root `solutions/` and `logs/` directories.
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
cargo run -- demo
cargo test
cargo fmt --check
```

## Current CLI

Ameth currently supports project initialization with:

```bash
ameth
ameth init <name> [path]
ameth <name> [path]
```

Behavior:

- `ameth` prints the whole-program introduction and root help.
- `ameth init <name> [path]` initializes a project.
- `<name>` becomes the new project directory name.
- `[path]` is the parent directory and defaults to `.`.
- `ameth <name> [path]` is an alias for `ameth init <name> [path]`.
- The command fails if `[path]/<name>` already exists.
- The planned `ameth ideas ...` commands are not implemented yet.

It creates this initial layout:

- `ideas/`
- `ideas/abandoned/`
- `relevants/`
- `code/`
- `experiments/`
- `ideas/Problem.md`

Planned additions such as `solutions/` and `logs/` are not created by `ameth init` yet.
