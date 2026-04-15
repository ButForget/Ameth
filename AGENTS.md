# AGENTS.md

## Current State
- Trust executable sources over `README.md`: the README describes a planned research-management CLI, but the current codebase is only a minimal Rust binary stub.
- `PROJECT_STATE.md` is accurate but brief: the repo is still in early development around the idea-management concept; most README-described behavior is not implemented yet.

## Repo Shape
- Single-package Rust repo. Root `Cargo.toml` defines one crate, `ameth`, on edition `2024`.
- Only code entrypoint is `src/main.rs`.
- There is no workspace, no library crate, no tests beyond Cargo's default harness, and no checked-in CI/lint/codegen config.

## Verified Commands
- `cargo test`: current best verification step. It passes today and effectively acts as a compile smoke test because there are `0` tests.
- `cargo fmt --check`: formatting check currently passes.
- `cargo run`: runs the current stub binary. Do not assume the README usage `ameth <path>` exists yet; argument parsing is not implemented in `src/main.rs`.

## Practical Guidance
- If you add functionality described in the README, treat it as new implementation work, not behavior you can rely on already existing.
- Keep `AGENTS.md` aligned with code/config, not with planned product docs.
