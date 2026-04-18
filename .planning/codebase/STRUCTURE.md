# Codebase Structure

**Analysis Date:** 2026-04-18

## Directory Layout

```text
autographs/
├── .planning/                         # Generated repository intelligence and planning artifacts
│   └── codebase/                     # Current-state mapping documents
├── .prompts/                         # Reusable execution prompts and summaries
│   └── 001-autograph-gallery-bootstrap-do/
│       ├── 001-autograph-gallery-bootstrap-do.md   # Primary build prompt
│       ├── SUMMARY.md                             # Prompt summary
│       └── completed/                            # Marker/output directory for prompt workflow
└── README.md                          # Minimal repository identifier
```

## Directory Purposes

**`.planning/`:**
- Purpose: Hold generated planning and analysis outputs.
- Contains: Markdown reference documents under `.planning/codebase/`.
- Key files: `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/STRUCTURE.md`

**`.planning/codebase/`:**
- Purpose: Store codebase mapping documents consumed by later planning/execution flows.
- Contains: Repository analysis in uppercase Markdown files.
- Key files: `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/STRUCTURE.md`

**`.prompts/`:**
- Purpose: Hold reusable prompts that describe future implementation work.
- Contains: Numbered prompt directories.
- Key files: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`

**`.prompts/001-autograph-gallery-bootstrap-do/`:**
- Purpose: Serve as the main source of product and implementation intent for the repository's planned autograph-gallery build.
- Contains: The authoritative project prompt, `SUMMARY.md`, and a `completed/` subdirectory.
- Key files: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`, `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`

## Key File Locations

**Entry Points:**
- `README.md`: Minimal repository landing file.
- `.prompts/001-autograph-gallery-bootstrap-do/`: Main source of project intent in the current repository.
- `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`: Primary actionable artifact in the current repository.
- `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`: Quick-reference summary for the prompt.

**Configuration:**
- Not detected. No package manager, framework, lint, test, container, or infrastructure configuration files are present at the repository root.

**Core Logic:**
- Not detected. The repository contains no runtime implementation files.

**Testing:**
- Not detected. No test directories, test files, or test runner configuration files are present.

## Naming Conventions

**Files:**
- Prompt files use numeric prefixes plus descriptive kebab-case names, for example `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`.
- Summary files use uppercase `SUMMARY.md`.
- Codebase mapping files use uppercase names, for example `.planning/codebase/ARCHITECTURE.md` and `.planning/codebase/STRUCTURE.md`.

**Directories:**
- Workflow/prompt directories use numeric prefixes with kebab-case identifiers, for example `.prompts/001-autograph-gallery-bootstrap-do/`.
- Planning output is grouped by concern under `.planning/codebase/`.

## Where to Add New Code

**New Feature:**
- Primary code: Not established yet. The prompt in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` suggests a future `app/` or framework-standard Next.js layout, but that structure does not exist yet.
- Tests: Not established yet. Add a test location only after the runtime stack is created.

**New Component/Module:**
- Implementation: Follow the selected framework's default layout once the first implementation pass creates it. Do not infer an existing convention from the current repo because none is present.

**Utilities:**
- Shared helpers: Not established. No shared utility directory exists today.

## Special Directories

**`.prompts/`:**
- Purpose: Stores prompt artifacts that describe work to be done.
- Generated: No
- Committed: Yes

**`.planning/`:**
- Purpose: Stores generated analysis/planning documents for agent workflows.
- Generated: Yes
- Committed: Yes

**`.prompts/001-autograph-gallery-bootstrap-do/completed/`:**
- Purpose: Workflow support directory associated with the prompt artifact.
- Generated: Likely yes
- Committed: Yes

## Current Layout Guidance

- Treat the repository as planning-first until real implementation directories are added.
- Treat `.prompts/001-autograph-gallery-bootstrap-do/` as the main source of product and implementation intent until runtime code exists.
- Use `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` as the authoritative source for the intended first runtime layout, but document newly created directories only after they actually exist.
- Avoid assuming conventions such as `src/`, `app/`, `infra/`, `.github/workflows/`, or `db/` until those paths are introduced by implementation work.

---

*Structure analysis: 2026-04-18*
