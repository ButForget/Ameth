# Ameth

Ameth is a Rust CLI for organizing research work so humans and LLMs can recover project context with less guesswork.

## Current Status

- `ameth init <name> [path]` is implemented.
- `ameth config <key> <value>` is implemented.
- `ameth ideas` supports `new`, `list`, `show`, `pin`, `abandon`, and `restore`.
- `ameth rq` supports `show` and `edit`.
- `solutions/` and `logs/` are created as part of the managed layout, but their workflows are still intentionally undefined.

## Why Ameth?

Research projects become hard to navigate when ideas, references, code, and experiment results are scattered or named inconsistently. That makes it harder for both people and LLMs to answer questions like:

- What problem is this project solving?
- Which ideas have already been tried?
- What references matter?
- Which experiments belong to which hypothesis?

Ameth provides a predictable project structure so project context is easier to recover and easier to use.

## Current Project Model

`ameth init demo` creates a project root like this:

```text
demo/
  Ameth.toml
  ResearchQuestion.md
  ideas/
    abandoned/
  solutions/
  logs/
  relevants/
  code/
  experiments/
```

The `ideas` and `rq` commands operate on the current working directory, so they should be run from an initialized Ameth project root.

### Root Files

`Ameth.toml` stores project metadata. Ameth currently uses:

```toml
editor = "nvim"

[ideas]
pinned = 4
```

Use an array when the editor needs fixed arguments:

```toml
editor = ["code", "--wait"]
```

- `editor` is required for interactive `ameth ideas new` and `ameth rq edit`.
- `[ideas].pinned` stores the pinned idea ID for `ameth ideas show` and bare `ameth ideas`.
- `ameth config <key> <value>` updates `Ameth.toml` and accepts dotted keys like `ideas.pinned`.

`ResearchQuestion.md` lives at the project root.

- It is a free-form background file for humans and LLMs.
- Ameth does not enforce any heading structure or markdown schema for it.
- `ameth init` and `ameth rq edit -n` create it with the initial template `# Research Question`.

### Idea Files

Idea files are lightweight Markdown documents stored under `ideas/`.

- Active ideas live in `ideas/`.
- Abandoned ideas live in `ideas/abandoned/`.
- Filenames use zero-padded four-digit IDs such as `idea-0001.md`.

Required template:

```md
## Abstract

Short summary of the idea.

## Content

Main idea text.

### Optional Subheading

More detail.
```

Rules:

- The only allowed level-2 headings are `Abstract` and `Content`.
- `Abstract` must come first and appear once.
- `Content` must come second and appear once.
- Level-1 headings are not allowed.
- Nested headings are allowed only under `Content`, and they must be level 3 or deeper.
- Content outside `Abstract` or `Content` is invalid.

## Current CLI

Top-level commands:

```bash
ameth
ameth init <name> [path]
ameth config <key> <value>
ameth ideas [command]
ameth rq [command]
```

Behavior:

- `ameth` prints the whole-program introduction and root help.
- `ameth init <name> [path]` initializes a new project directory.
- `<name>` becomes the new project directory name.
- `[path]` is the parent directory and defaults to `.`.
- `ameth init` fails if `[path]/<name>` already exists.
- `ameth init` creates `Ameth.toml`, `ResearchQuestion.md`, and the managed directory layout.
- `ameth config <key> <value>` updates `Ameth.toml` in the current project root.
- `ameth config` parses `<value>` as TOML when possible and otherwise stores it as a string.
- Dotted keys like `ideas.pinned` update nested config tables.

### `ameth ideas`

- `ameth ideas new [--abs <ABSTRACT>] [--ctt <CONTENT>]` creates the next `idea-000N.md` file.
- If either field is omitted, `ameth ideas new` writes the template, opens the root-level `editor` from `Ameth.toml`, and waits for it to exit.
- `ameth ideas list` lists active ideas and displays each ID plus the `Abstract` text on one line.
- `ameth ideas show <id>` displays an active or abandoned idea.
- `ameth ideas show` displays the pinned idea.
- `ameth ideas pin <id>` records the pinned idea ID in `Ameth.toml`.
- `ameth ideas abandon <id>` moves an active idea into `ideas/abandoned/`.
- `ameth ideas restore <id>` moves an abandoned idea back into `ideas/`.
- Bare `ameth ideas` shows the pinned idea when one is set. Otherwise it prints ideas help.

### `ameth rq`

- Bare `ameth rq` prints research-question help.
- `ameth rq show` prints `ResearchQuestion.md` as-is.
- `ameth rq edit` opens the existing `ResearchQuestion.md` file in the configured editor.
- `ameth rq edit -n` creates `ResearchQuestion.md` when it is missing, then opens it.
- `ameth rq edit -f -n` recreates `ResearchQuestion.md` even when it already exists, then opens it.
- `ameth rq show` and plain `ameth rq edit` fail when `ResearchQuestion.md` is missing.

## Relevant Source Layout

- `src/main.rs` is the thin binary entrypoint.
- `src/cli/` contains root CLI dispatch and whole-program help.
- `src/commands/init.rs` implements project initialization.
- `src/commands/config.rs` implements config updates.
- `src/commands/ideas.rs` implements idea subcommand parsing and execution.
- `src/commands/ideas/document.rs` defines the idea template and strict idea parser.
- `src/commands/ideas/project.rs` handles idea-project file operations and pinned-id persistence.
- `src/commands/rq.rs` implements research-question display and editor-driven updates.
- `src/config.rs` loads and saves `Ameth.toml`.
- `tests/init_cli.rs`, `tests/root_cli.rs`, and `tests/ideas_cli.rs` exercise the CLI end to end.

## Development

### Prerequisites

- Rust toolchain with Cargo installed

### Useful Commands

```bash
cargo run --
cargo run -- init demo
cargo run -- ideas --help
cargo run -- rq --help
cargo test
cargo fmt --check
```

## Not Yet Defined

- `solutions/` exists in initialized projects, but its document workflow is not defined yet.
- `logs/` exists in initialized projects, but its workflow is still a placeholder.
