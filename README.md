# Ameth

Ameth is an early-stage Rust CLI for organizing research work so humans and LLMs can understand the problem, relevant materials, code, and experiments with less guesswork.

## Status

This repository is still in very early development.

- The initial project initialization command is implemented.
- The idea-management flow beyond project bootstrapping is still in development.

## Why Ameth?

Research projects become hard to navigate when ideas, references, code, and experiment results are scattered or named inconsistently. That makes it harder for both people and LLMs to answer questions like:

- What problem is this project solving?
- Which ideas have already been tried?
- What references matter?
- Which experiments belong to which hypothesis?

Ameth is meant to provide a predictable project structure for research work, so context is easier to recover and easier to use.

## Intended Project Structure

The current design is centered around a few top-level directories:

- `ideas/`
- `relevants/`
- `code/`
- `experiments/`

### `ideas/`

The `ideas/` directory is intended to capture the research problem and candidate solutions.

- `Problem.md` describes the main problem the research focuses on.
- Idea files follow a naming pattern like `idea-<index>-<time>.md`.
- Abandoned ideas go under `ideas/abandoned/`.

## Current Repository Layout

This repo currently contains:

- `Cargo.toml` for the single Rust package
- `src/main.rs` as the thin entrypoint
- `src/cli/` for root CLI dispatch and whole-program help
- `src/commands/` for per-command parsing, help text, and execution
- `PROJECT_STATE.md` for a brief project status note

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

It creates this initial layout:

- `ideas/`
- `ideas/abandoned/`
- `relevants/`
- `code/`
- `experiments/`
- `ideas/Problem.md`
