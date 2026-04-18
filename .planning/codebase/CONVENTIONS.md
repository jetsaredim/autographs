# Coding Conventions

**Analysis Date:** 2026-04-18

## Naming Patterns

**Files:**
- Markdown planning artifacts use uppercase summary-style names in `.planning/codebase/` such as `CONVENTIONS.md` and `TESTING.md`.
- Prompt artifacts use numeric prefixes plus kebab-case descriptive names, as seen in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`.
- Supporting prompt summaries use simple uppercase filenames such as `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`.
- No application source files exist under `src/`, `app/`, `tests/`, or similar directories, so source-code file naming conventions are not yet established.

**Functions:**
- Not detected. The repository does not contain implementation files with function definitions.

**Variables:**
- Not detected in code.
- Prompt and documentation text favors clear domain-oriented phrases such as "single admin account", "private image access", and "GitHub Actions". This is documentation language, not an established variable naming convention.

**Types:**
- Not detected. No typed source files are present.

## Code Style

**Formatting:**
- No formatting tool configuration is present. `eslint.config.*`, `.eslintrc*`, `.prettierrc*`, `prettier.config.*`, and `biome.json` are not present in the repository root.
- Current tracked files are short Markdown documents: `README.md`, `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`, and `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`.
- Observed Markdown style uses ATX headings, sentence-style prose, and flat bullet lists.

**Linting:**
- Not detected. No linter configuration, no package manifest, and no lint scripts are present.

## Import Organization

**Order:**
1. Not detected because no source files with imports exist.
2. Recommended future documentation standard: document import ordering once the first application module exists.
3. Treat any future convention here as an inference until real source files establish it.

**Path Aliases:**
- Not detected. No `tsconfig.json`, `jsconfig.json`, or bundler configuration is present.

## Error Handling

**Patterns:**
- No runtime error-handling code exists yet.
- Documentation does show a habit of calling out operational constraints and missing prerequisites explicitly in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` and `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`.
- Inference: future project documentation is likely expected to distinguish between automated steps and manual prerequisites because the prompt repeatedly requires that separation.

## Logging

**Framework:** Not detected

**Patterns:**
- No logging implementation exists.
- No documented logging standard exists outside the prompt's general requirement for operator-facing documentation.

## Comments

**When to Comment:**
- Inline code comments are not observable because no code exists.
- Documentation is currently the primary communication mechanism. The main conventions are:
- State scope explicitly, as seen in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`.
- Separate requirements, implementation guidance, output expectations, and verification steps into dedicated sections.
- Use bullets for constraints and numbered lists for execution order.

**JSDoc/TSDoc:**
- Not detected.

## Function Design

**Size:** Not detected

**Parameters:** Not detected

**Return Values:** Not detected

## Module Design

**Exports:**
- Not detected. No modules are present.

**Barrel Files:**
- Not detected.

## Documentation Habits

- `README.md` currently contains only the project title `# autographs`, so repository-level onboarding conventions are not established there yet.
- The strongest documented pattern is prompt-driven planning in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`, which uses XML-style sections such as `<objective>`, `<requirements>`, and `<verification>` to structure instructions.
- `.prompts/001-autograph-gallery-bootstrap-do/` is the main source of product and implementation intent in the current repository state. Treat `README.md` as a placeholder and the prompt directory as the authoritative description of expected architecture, delivery flow, and verification scope until implementation files exist.
- `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md` follows a compact status-report format with stable sections: `One-liner`, `Version`, `Key Findings`, `Files Created`, `Decisions Needed`, `Blockers`, and `Next Step`.
- Inference: when adding future operational or planning docs, reuse the explicit sectioning and status-summary style already present in `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`.

## Gaps

- No application code exists yet, so language-level conventions for naming, imports, typing, errors, and module boundaries are not established.
- No repository automation exists yet for formatting, linting, or documentation validation.
- No `.editorconfig` or equivalent shared editor settings are present.
- No contribution guide or engineering handbook exists beyond the prompt artifacts in `.prompts/`.
- `README.md` does not currently document local setup, commands, coding standards, or review expectations; that intent currently lives in `.prompts/001-autograph-gallery-bootstrap-do/`.

## Prescriptive Guidance For Future Work

- Treat everything above that references implementation style as "not yet established" until real source files land.
- When the first executable code is added, document the actual conventions in this file rather than copying generic JavaScript or TypeScript norms.
- Reuse the repository's existing documentation strengths:
- Prefer explicit section headers.
- Write constraints and assumptions plainly.
- Distinguish current state from recommendations.
- Keep naming descriptive and consistent with the domain language already used in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`.
- When deciding whether a new behavior is "intended," consult `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` first because it is the repo's authoritative implementation brief at this stage.

---

*Convention analysis: 2026-04-18*
