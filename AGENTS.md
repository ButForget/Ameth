# AGENTS.md

## Current State
- The repo now implements the initial project bootstrap command: `ameth <name> [path]`.
- `README.md` and `PROJECT_STATE.md` describe this current scope, while idea-management features beyond initialization are still planned.

## Repo Shape
- Single-package Rust repo. Root `Cargo.toml` defines one crate, `ameth`, on edition `2024`.
- `src/main.rs` is the thin binary entrypoint.
- Root CLI dispatch and whole-program help live under `src/cli/`.
- Each subcommand lives in its own file under `src/commands/`.
- Integration tests live in `tests/init_cli.rs` and exercise the binary end to end.
- There is no workspace, no library crate, and no checked-in CI/lint/codegen config.

## Verified Commands
- `cargo test`: runs the CLI integration tests for project initialization and currently passes.
- `cargo fmt --check`: formatting check currently passes.
- `cargo run --`: prints the whole-program help.
- `cargo run -- demo`: creates a new `demo/` Ameth project in the current directory.
- `cargo run -- init demo`: creates a new `demo/` Ameth project in the current directory.

## Practical Guidance
- If you add functionality described in the README, treat it as new implementation work, not behavior you can rely on already existing.
- Keep `src/main.rs` minimal. Top-level wiring, subcommand registration, and I/O are allowed there, but subcommand business logic is not.
- Each subcommand should live in its own file and own its own usage/help text, parse logic, and execution logic.
- `ameth <name> [path]` is the default alias for `ameth init <name> [path]`.
- Bare `ameth` should print the introduction and root help for the whole program.
- Do not read files under `tests/` before a verification failure. Implement from the user request and source first, then inspect test code only after the code or test run fails.
- Keep `AGENTS.md` aligned with code/config, not with planned product docs.
