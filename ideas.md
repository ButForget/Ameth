# Ideas Management Spec

## Scope

This document defines the current idea-management structure and the root research-question file for Ameth.

The command namespaces are `ameth ideas` and `ameth rq`.

Idea pin metadata and the default editor command live in the project root `Ameth.toml` file.

Editor config is root-level because it can be reused by multiple workflows:

```toml
editor = "nvim"
```

Use an array when the editor needs fixed arguments:

```toml
editor = ["code", "--wait"]
```

## Root Structure

An Ameth-managed research project should contain these root directories:

- `ideas/`
- `solutions/`
- `logs/`
- `relevants/`
- `code/`
- `experiments/`

Current intent:

- `ResearchQuestion.md` stores free-form background for the project.
- `ideas/` stores raw idea documents.
- `solutions/` is for more structured solution documents built from promising ideas.
- `logs/` exists as a placeholder for now.

At the project root:

- `ResearchQuestion.md`
- `ideas/`

Within `ideas/`:

- `ideas/abandoned/`
- `ideas/idea-0001.md`
- `ideas/idea-0002.md`

## Parser

Ameth should parse idea files with `pulldown-cmark`.

Idea parsing is strict about required heading names and heading levels so active and abandoned idea files can be consumed reliably by Ameth subcommands.

## Research Question File

The background file lives at the project root as `ResearchQuestion.md`.

Rules:

- The file is intentionally free-form.
- Ameth does not require any fixed headings or section names.
- `ameth rq show` prints the file as-is.
- `ameth rq edit` opens the existing file in the configured root-level editor.
- `ameth rq edit -n` creates a new file only when it is missing.
- `ameth rq edit -f -n` recreates the file even when it already exists.
- `ameth rq show` and plain `ameth rq edit` fail if the file is missing.

## Idea Files

Idea files are simple Markdown files intended for lightweight capture.

Filename format:

- `ideas/idea-0001.md`
- `ideas/idea-0002.md`
- `ideas/abandoned/idea-0001.md`

Rules:

- The numeric part is the canonical idea ID.
- IDs should be zero-padded to 4 digits.
- Active ideas live in `ideas/`.
- Abandoned ideas live in `ideas/abandoned/`.

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
- `Abstract` must come first.
- `Content` must come second.
- Both sections are required.
- Plain paragraphs are allowed under both sections.
- Nested headings are allowed only under `Content`, and they must be level 3 or deeper.
- Unknown extra level-2 headings are invalid.
- Content should belong to either `Abstract` or `Content`.

## Planned Command Behavior

Current `ameth ideas` behavior should align with these files:

- `ameth ideas new [--abs <ABSTRACT>] [--ctt <CONTENT>]` creates the next `ideas/idea-000N.md` file using the required idea template.
- When either field is omitted, `ameth ideas new` opens the root-level `editor` from `Ameth.toml` after creating the template and waits for it to exit.
- `ameth ideas list` parses active idea files and displays their IDs plus `Abstract` text.
- `ameth ideas show <id>` parses and displays the selected idea.
- `ameth ideas show` parses and displays the pinned idea.
- `ameth ideas pin <id>` records the pinned idea ID in `Ameth.toml`.
- `ameth ideas abandon <id>` moves an idea file into `ideas/abandoned/`.
- `ameth ideas restore <id>` moves an idea file back into `ideas/`.
- Bare `ameth ideas` is an alias for `ameth ideas show` when an idea is pinned; otherwise it shows ideas help.
- `ameth rq show` prints `ResearchQuestion.md`.
- `ameth rq edit` opens `ResearchQuestion.md` in the configured editor.

## Notes

- `ResearchQuestion.md` is a free-form background file at the project root.
- Idea files stay intentionally simple.
- `solutions/` and `logs/` are part of the project structure even though their workflows are not defined yet.
