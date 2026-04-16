The repository currently implements these command surfaces:

- `ameth init <name> [path]`
- `ameth config <key> <value>`
- `ameth ideas`
- `ameth rq`

Current behavior snapshot:

- `ameth init` creates a new project root with `Ameth.toml`, `ResearchQuestion.md`, `ideas/`, `ideas/abandoned/`, `solutions/`, `logs/`, `relevants/`, `code/`, and `experiments/`.
- `ResearchQuestion.md` starts as a free-form file with the initial template `# Research Question`.
- `Ameth.toml` stores root-level editor configuration plus idea metadata such as `[ideas].pinned`.
- `ameth config` updates `Ameth.toml` using dotted keys and TOML-aware values.
- `ameth ideas new` creates the next zero-padded idea file and opens the configured editor when either `--abs` or `--ctt` is omitted.
- `ameth ideas list` shows active ideas and their abstract text.
- `ameth ideas show` displays an idea by ID or the pinned idea when no ID is given.
- `ameth ideas pin` records the pinned idea ID in `Ameth.toml`.
- `ameth ideas abandon` and `ameth ideas restore` move idea files between `ideas/` and `ideas/abandoned/`.
- `ameth rq show` prints the root research-question file.
- `ameth rq edit` opens the existing file in the configured editor.
- `ameth rq edit -n` creates a missing file before opening it.
- `ameth rq edit -f -n` recreates the file before opening it.

Still intentionally undefined:

- The `solutions/` workflow.
- The `logs/` workflow.
