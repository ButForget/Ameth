# Ameth

Ameth is an early-stage Rust CLI for organizing research work so humans and LLMs can understand the problem, relevant materials, code, and experiments with less guesswork.

## Status

This repository is still in very early development.

- The current codebase is a minimal Rust binary stub.
- The CLI described below is the intended direction, not fully implemented behavior.
- Today, `cargo run` only runs a placeholder program from `src/main.rs`.

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
- `src/main.rs` as the only entrypoint
- `PROJECT_STATE.md` for a brief project status note

## Development

### Prerequisites

- Rust toolchain with Cargo installed

### Useful Commands

```bash
cargo run
cargo test
cargo fmt --check
```

## Planned CLI

The README originally described a command like:

```bash
ameth <path>
```

That interface is not implemented yet. If it gets added, it should be treated as new functionality rather than existing behavior.

