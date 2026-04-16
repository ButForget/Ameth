# Ideas Management Spec

## Scope

This document defines the planned idea-management structure and file formats for Ameth.

The command namespace is `ameth ideas`.

Idea pin metadata lives in the project root `Ameth.toml` file.

## Root Structure

An Ameth-managed research project should contain these root directories:

- `ideas/`
- `solutions/`
- `logs/`
- `relevants/`
- `code/`
- `experiments/`

Current intent:

- `ideas/` stores the research problem and raw idea documents.
- `solutions/` is for more structured solution documents built from promising ideas.
- `logs/` exists as a placeholder for now.

Within `ideas/`:

- `ideas/Problem.md`
- `ideas/abandoned/`
- `ideas/idea-0001.md`
- `ideas/idea-0002.md`

## Parser

Ameth should parse `ideas/Problem.md` and idea files with `pulldown-cmark`.

The parser should be strict about required heading names and heading levels so these files can be consumed reliably by Ameth subcommands.

## Problem File

The problem file stays at `ideas/Problem.md`.

Required template:

```md
# Problem

## Abstract

## Goal

## Constraints

## Open Questions
```

Rules:

- The file must begin with `# Problem`.
- The level-2 headings are fixed: `Abstract`, `Goal`, `Constraints`, `Open Questions`.
- These level-2 headings are the machine-parseable section boundaries.
- Free text is allowed inside each section.
- Nested headings are allowed inside each section, but they must be level 3 or deeper.
- Unknown level-2 headings are invalid.
- Content should belong to one of the fixed sections.

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

Initial `ameth ideas` behavior should align with these files:

- `ameth ideas new` creates the next `ideas/idea-000N.md` file using the required idea template.
- `ameth ideas list` parses active idea files and displays their IDs plus `Abstract` text.
- `ameth ideas show <id>` parses and displays the selected idea.
- `ameth ideas show` parses and displays the pinned idea.
- `ameth ideas pin <id>` records the pinned idea ID in `Ameth.toml`.
- `ameth ideas abandon <id>` moves an idea file into `ideas/abandoned/`.
- `ameth ideas restore <id>` moves an idea file back into `ideas/`.
- Bare `ameth ideas` is an alias for `ameth ideas show` when an idea is pinned; otherwise it shows ideas help.

## Notes

- `Problem.md` is the structured anchor for the research problem.
- Idea files stay intentionally simple.
- `solutions/` and `logs/` are part of the project structure even though their workflows are not defined yet.
