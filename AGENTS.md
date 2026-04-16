# AGENTS.md

## Current State
- The repo now implements project bootstrap with `ameth init <name> [path]`, the `ameth ideas` command namespace, and the top-level `ameth rq` command namespace.
- `ameth init` now creates the full planned root layout, including `solutions/`, `logs/`, and `Ameth.toml`.
- `ameth init` also creates root `ResearchQuestion.md` as a free-form background file.
- Idea management currently supports `new`, `list`, `show`, `pin`, `abandon`, and `restore`; `new` accepts `--abs`/`--ctt` and opens the root-level `editor` from `Ameth.toml` when either field is omitted.
- `ameth rq` currently supports `show` and `edit`; `edit` accepts `-n/--new` and `-f/--force` for controlled creation and recreation of `ResearchQuestion.md`.

## Repo Shape
- Single-package Rust repo. Root `Cargo.toml` defines one crate, `ameth`, on edition `2024`.
- `src/main.rs` is the thin binary entrypoint.
- Root CLI dispatch and whole-program help live under `src/cli/`.
- Each subcommand lives in its own file under `src/commands/`.
- Integration tests live in `tests/init_cli.rs`, `tests/root_cli.rs`, and `tests/ideas_cli.rs` and exercise the binary end to end.
- There is no workspace, no library crate, and no checked-in CI/lint/codegen config.

## Verified Commands
- `cargo test`: runs the CLI integration tests for project initialization and currently passes.
- `cargo fmt --check`: formatting check currently passes.
- `cargo run --`: prints the whole-program help.
- `cargo run -- init demo`: creates a new `demo/` Ameth project in the current directory.
- `cargo run -- ideas --help`: prints the ideas command help.
- `cargo run -- rq --help`: prints the research-question command help.
- `cargo run -- ideas new --abs "summary" --ctt "details"`: creates the next idea file without opening an editor in an initialized project.
- `cargo run -- ideas pin 1`: pins an existing idea in an initialized project.

## Practical Guidance
- Keep `src/main.rs` minimal. Top-level wiring, subcommand registration, and I/O are allowed there, but subcommand business logic is not.
- Each subcommand should live in its own file and own its own usage/help text, parse logic, and execution logic.
- Bare `ameth` should print the introduction and root help for the whole program.
- `ameth ideas` owns idea-file creation, listing, display, pinning, and archive/restore moves under `ideas/`.
- `ameth rq` owns root `ResearchQuestion.md` display and editor-driven updates.
- Interactive `ameth ideas new` requires a root-level `editor` setting in `Ameth.toml`; use both `--abs` and `--ctt` for non-interactive creation.
- Do not read files under `tests/` before a verification failure. Implement from the user request and source first, then inspect test code only after the code or test run fails.
- Keep `AGENTS.md` aligned with code/config, not with planned product docs.
