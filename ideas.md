# Ideas and Research Question Spec

## Scope

This document describes the current Ameth behavior for project config, idea management, and the root research-question file.

The command namespaces are `ameth config`, `ameth ideas`, and `ameth rq`.

These commands operate on the current working directory and expect to run from an initialized Ameth project root.

## Root Layout

An initialized Ameth project contains this managed root shape:

```text
<project>/
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

Current intent:

- `ResearchQuestion.md` stores free-form background for the project.
- `ideas/` stores raw idea documents.
- `solutions/` is reserved for more structured solution documents promoted from promising ideas.
- `logs/` exists as a placeholder for now.

## `Ameth.toml`

Idea pin metadata and the default editor command live in the project root `Ameth.toml` file.

Ameth currently uses:

```toml
editor = "nvim"

[ideas]
pinned = 4
```

Use an array when the editor needs fixed arguments:

```toml
editor = ["code", "--wait"]
```

Rules:

- `editor` must be either a string or an array of strings.
- The first editor entry is the program name.
- Interactive `ameth ideas new` and `ameth rq edit` require `editor` to be configured.
- `[ideas].pinned` stores the pinned idea ID as a positive integer.
- `ameth config <key> <value>` updates `Ameth.toml`, accepts dotted keys like `ideas.pinned`, parses TOML values when possible, and otherwise stores the raw value as a string.

## Research Question File

The background file lives at the project root as `ResearchQuestion.md`.

Rules:

- The file is intentionally free-form.
- Ameth does not require any fixed headings or section names.
- `ameth init` creates it with the template `# Research Question`.
- `ameth rq show` prints the file as-is.
- `ameth rq edit` opens the existing file in the configured root-level editor.
- `ameth rq edit -n` creates a new file only when it is missing, then opens it.
- `ameth rq edit -f -n` recreates the file even when it already exists, then opens it.
- `ameth rq show` and plain `ameth rq edit` fail if the file is missing.

## Idea Files

Idea files are simple Markdown files intended for lightweight capture.

Filename format:

- `ideas/idea-0001.md`
- `ideas/idea-0002.md`
- `ideas/abandoned/idea-0001.md`

Rules:

- The numeric part is the canonical idea ID.
- IDs are stored in filenames as zero-padded four-digit numbers.
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

Parser rules:

- Level-1 headings are not allowed.
- The only allowed level-2 headings are `Abstract` and `Content`.
- `Abstract` must come first.
- `Content` must come second.
- Both sections are required.
- Plain paragraphs are allowed under both sections.
- Nested headings are allowed only under `Content`, and they must be level 3 or deeper.
- Unknown extra level-2 headings are invalid.
- Content must belong to either `Abstract` or `Content`.

## Current Command Behavior

### `ameth config`

- `ameth config <key> <value>` updates `Ameth.toml` in the current project root.
- Dotted keys like `ideas.pinned` update nested config tables.
- `<value>` is parsed as TOML when possible, so arrays like `["code", "--wait"]` and integers like `4` are stored with their TOML types.
- Values that are not valid TOML literals are stored as strings.

### `ameth ideas`

- `ameth ideas new [--abs <ABSTRACT>] [--ctt <CONTENT>]` creates the next `ideas/idea-000N.md` file using the required idea template.
- When either field is omitted, `ameth ideas new` opens the root-level `editor` from `Ameth.toml` after creating the template and waits for it to exit.
- `ameth ideas list` parses active idea files and displays their IDs plus single-line `Abstract` text.
- `ameth ideas show <id>` parses and displays the selected active or abandoned idea.
- `ameth ideas show` parses and displays the pinned idea.
- `ameth ideas pin <id>` records the pinned idea ID in `Ameth.toml`.
- `ameth ideas abandon <id>` moves an active idea file into `ideas/abandoned/`.
- `ameth ideas restore <id>` moves an abandoned idea file back into `ideas/`.
- Bare `ameth ideas` shows the pinned idea when one is set. Otherwise it shows ideas help.

### `ameth rq`

- Bare `ameth rq` shows help.
- `ameth rq show` prints `ResearchQuestion.md`.
- `ameth rq edit` opens `ResearchQuestion.md` in the configured editor.
- `ameth rq edit -n` creates the file when it is missing, then opens it.
- `ameth rq edit -f -n` recreates the file, then opens it.

## Notes

- `ResearchQuestion.md` is intentionally free-form.
- Idea files stay intentionally simple but are parsed strictly.
- `solutions/` and `logs/` are part of the initialized project structure even though their workflows are not defined yet.
