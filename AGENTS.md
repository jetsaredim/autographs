<!-- GSD:project-start source:PROJECT.md -->
## Project

**Autographs**

Autographs is a production-lean personal autograph collection website where you can publish your own signed memorabilia for anonymous public browsing. The first release pairs a single self-hosted `Next.js` application with private OCI Object Storage for images and Oracle Autonomous Database Free for metadata, while also establishing the OCI bootstrap, CI/CD, and operator guidance needed to run the collection as a real, durable personal project.

**Core Value:** A collector can reliably browse and manage a high-quality autograph catalog where private images and useful metadata stay connected end to end.

### Constraints

- **Tech stack**: Use a single `Next.js` full-stack application for v1 — keeps implementation and operations simpler than a split-service design.
- **Cloud**: Prefer OCI Always Free services wherever feasible — the product should be realistic for a fresh low-cost tenancy.
- **Database**: Prefer Oracle Autonomous Database Free — the prompt explicitly selects it unless implementation friction forces a justified fallback.
- **Storage**: Keep autograph images private in OCI Object Storage — access should be centralized through the app rather than direct public buckets.
- **Delivery**: Auto-deploy from GitHub Actions on merge to `main` — CI/CD is part of project bootstrap, not optional polish.
- **Operations**: One developer should be able to understand and run the system — avoid enterprise sprawl and multi-service complexity.
- **Scope**: v1 must stay narrow — no staging environment, no bulk import, no public accounts, and no advanced search platform, but multi-image items and edit history are in scope because they matter directly for managing a personal collection well.
- **Security**: Use least-privilege OCI access and explicit secret handling — routine deploy workflows should not rely on tenancy-wide admin power.
<!-- GSD:project-end -->

<!-- GSD:stack-start source:codebase/STACK.md -->
## Technology Stack

## Languages
- Markdown - The only implementation-adjacent content present is documentation and prompt text in `README.md`, `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`, and `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`.
- Not detected - No application source files, infrastructure source files, or test files are present under the repository root.
## Runtime
- Not detected - No runtime declaration files such as `package.json`, `pyproject.toml`, `requirements.txt`, `Cargo.toml`, or `go.mod` are present in the repository root scan.
- Not detected - No package manager manifest or lockfile was found.
- Lockfile: missing
## Frameworks
- Not detected - No framework configuration or source tree exists today.
- Not detected - No test runner configuration or test files were found.
- Not detected - No build tool configuration files such as `tsconfig.json`, `next.config.*`, Dockerfiles, Terraform files, or workflow files are present.
## Key Dependencies
- Not detected - No dependency manifest exists.
- Not detected - No infrastructure-as-code dependencies or cloud SDKs are present in tracked repository files.
## Configuration
- No `.env` files were detected in the repository scan.
- No environment contract file such as `.env.example` exists.
- No build configuration files were detected.
## Platform Requirements
- A Markdown-capable editor is sufficient for the current repository state because the repo contains only `README.md` and prompt artifacts.
- Not applicable in the current state. The intended production target is described only as a future plan in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`.
## Project Maturity
- Stub / planning-only repository.
- Evidence: `README.md` contains only the heading `# autographs`.
- Evidence: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` is an implementation prompt describing a desired OCI, Next.js, Terraform, and GitHub Actions stack, but those technologies are not implemented in the repository.
- Evidence: `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md` summarizes the prompt artifact rather than shipped code.
- Treat the OCI, Next.js, Oracle Autonomous Database, Object Storage, Docker, Terraform, and GitHub Actions references in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` as planned architecture, not current stack.
- Add the first real stack documentation only after manifests and source files land, so `STACK.md` can be updated from observed code rather than prompt intent.
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

## Naming Patterns
- Markdown planning artifacts use uppercase summary-style names in `.planning/codebase/` such as `CONVENTIONS.md` and `TESTING.md`.
- Prompt artifacts use numeric prefixes plus kebab-case descriptive names, as seen in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`.
- Supporting prompt summaries use simple uppercase filenames such as `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`.
- No application source files exist under `src/`, `app/`, `tests/`, or similar directories, so source-code file naming conventions are not yet established.
- Not detected. The repository does not contain implementation files with function definitions.
- Not detected in code.
- Prompt and documentation text favors clear domain-oriented phrases such as "single admin account", "private image access", and "GitHub Actions". This is documentation language, not an established variable naming convention.
- Not detected. No typed source files are present.
## Code Style
- No formatting tool configuration is present. `eslint.config.*`, `.eslintrc*`, `.prettierrc*`, `prettier.config.*`, and `biome.json` are not present in the repository root.
- Current tracked files are short Markdown documents: `README.md`, `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`, and `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`.
- Observed Markdown style uses ATX headings, sentence-style prose, and flat bullet lists.
- Not detected. No linter configuration, no package manifest, and no lint scripts are present.
## Import Organization
- Not detected. No `tsconfig.json`, `jsconfig.json`, or bundler configuration is present.
## Error Handling
- No runtime error-handling code exists yet.
- Documentation does show a habit of calling out operational constraints and missing prerequisites explicitly in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` and `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`.
- Inference: future project documentation is likely expected to distinguish between automated steps and manual prerequisites because the prompt repeatedly requires that separation.
## Logging
- No logging implementation exists.
- No documented logging standard exists outside the prompt's general requirement for operator-facing documentation.
## Comments
- Inline code comments are not observable because no code exists.
- Documentation is currently the primary communication mechanism. The main conventions are:
- State scope explicitly, as seen in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`.
- Separate requirements, implementation guidance, output expectations, and verification steps into dedicated sections.
- Use bullets for constraints and numbered lists for execution order.
- Not detected.
## Function Design
## Module Design
- Not detected. No modules are present.
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
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

## Pattern Overview
- The repository is centered on `.prompts/001-autograph-gallery-bootstrap-do/`, which is the main source of product scope and implementation intent.
- The primary architectural artifact is `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`, not application source code.
- The checked-in codebase currently documents a target system rather than containing that system's source code.
- There are no application, infrastructure, database, or test entry points present beyond documentation artifacts such as `README.md` and `.prompts/.../SUMMARY.md`.
## Layers
- Purpose: Identify the project at the top level.
- Location: `README.md`
- Contains: Minimal repository title only.
- Depends on: Nothing in-repo.
- Used by: Humans and future automation reading the repo root.
- Purpose: Define the intended product, delivery model, target stack, and verification expectations for a future implementation pass.
- Location: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`
- Contains: The authoritative project prompt for the repository, including objective, requirements, implementation guidance, output expectations, and success criteria.
- Depends on: `README.md` as referenced context.
- Used by: A future coding run that will scaffold the actual platform.
- Purpose: Group the repository's main product-definition artifacts in one place.
- Location: `.prompts/001-autograph-gallery-bootstrap-do/`
- Contains: The main execution prompt, `SUMMARY.md`, and workflow support directory `completed/`.
- Depends on: Repository-level planning context.
- Used by: Humans and agents to understand what this repository is intended to become.
- Purpose: Summarize what the prompt artifact is for and what it decided.
- Location: `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`
- Contains: One-line summary, findings, files created, blockers, and next step.
- Depends on: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`
- Used by: Humans reviewing prompt intent without reading the full prompt.
- Purpose: Store generated repository analysis for later planning/execution commands.
- Location: `.planning/codebase/`
- Contains: Architecture and structure documents such as `.planning/codebase/ARCHITECTURE.md` and `.planning/codebase/STRUCTURE.md`.
- Depends on: Observed repository state.
- Used by: Follow-on planning and execution tooling.
## Data Flow
- State is document-based and file-backed. There is no runtime state, persisted application data model, or request lifecycle implemented in the repository.
## Key Abstractions
- Purpose: Acts as the primary architectural artifact for the intended system.
- Examples: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`
- Pattern: Structured prompt sections using XML-like headings such as `<objective>`, `<requirements>`, and `<verification>`.
- Purpose: Establishes a package-level boundary around the repository's current source of truth for what should be built.
- Examples: `.prompts/001-autograph-gallery-bootstrap-do/`
- Pattern: Numbered directory containing the full prompt plus a companion summary.
- Purpose: Provides a lightweight derivative view of the execution prompt.
- Examples: `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`
- Pattern: Markdown summary with fixed headings for findings, blockers, and next step.
- Purpose: Capture current-state repository intelligence in a reusable format.
- Examples: `.planning/codebase/ARCHITECTURE.md`, `.planning/codebase/STRUCTURE.md`
- Pattern: Markdown reference documents consumed by later planning workflows.
## Entry Points
- Location: `README.md`
- Triggers: Opening the repository root.
- Responsibilities: Present the repository name only.
- Location: `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md`
- Triggers: A future implementation run using the prompt.
- Responsibilities: Specify the desired OCI, Next.js, database, CI/CD, and admin/public gallery architecture to be created.
- Location: `.prompts/001-autograph-gallery-bootstrap-do/`
- Triggers: Any review of current repository intent or startup planning.
- Responsibilities: Serve as the main source of product definition and implementation direction for the repo in its current state.
- Location: `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`
- Triggers: Quick review of prompt scope.
- Responsibilities: Summarize the prompt artifact and recommended next action.
## Error Handling
- Requirements and constraints are expressed declaratively in `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` rather than enforced by code.
- Any validation described today is aspirational and belongs to the future implementation, not the current repository state.
## Cross-Cutting Concerns
## Notable Absences
- No `app/`, `src/`, `pages/`, or `components/` directory exists for a web application.
- No `.github/workflows/` directory exists for CI/CD workflows.
- No `infra/`, `terraform/`, or equivalent infrastructure-as-code directory exists.
- No `db/`, schema, migration, or seed files exist.
- No package manifest such as `package.json`, `pyproject.toml`, `go.mod`, or `Cargo.toml` exists.
- No runtime configuration, container definitions, or deployment scripts are present.
<!-- GSD:architecture-end -->

<!-- GSD:skills-start source:skills/ -->
## Project Skills

No project skills found. Add skills to any of: `.claude/skills/`, `.agents/skills/`, `.cursor/skills/`, or `.github/skills/` with a `SKILL.md` index file.
<!-- GSD:skills-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd-quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd-debug` for investigation and bug fixing
- `/gsd-execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->

## Git Commit Branch Guardrails

- Never commit directly to `main` or `master`.
- If work starts on `main` or `master`, create or switch to a dedicated work branch before editing files that will be committed.
- Keep all commits for a task inside the current work branch. Merge back to `main` only through the project's normal PR/merge path.
- If a commit command would run on `main` or `master`, stop and report the current branch plus the branch that should contain the work.

## Connectivity and Publishing Failures

- If `git push`, `git fetch`, `gh`, SSH, DNS, or GitHub API calls fail because of local connectivity, local SSH configuration, credentials, network restrictions, or sandbox/network access, stop and tell the user immediately.
- Do not spend time inventing alternate push/fetch workarounds such as overriding `GIT_SSH_COMMAND`, switching remotes, bypassing SSH config, or retrying through a different protocol unless the user explicitly asks for that approach.
- Prefer a concise report with the failing command, the important error text, and the suggested user-side action. The user can often resolve host connectivity issues faster outside the Codex environment.
- After the user confirms the issue is fixed, retry the original straightforward command.

## Pull Request Review Findings

- When a review agent is deployed on a GitHub PR, every actionable finding from that agent must be written back to the PR as a GitHub comment.
- Prefer inline review comments when a finding maps to a specific changed line; otherwise post a single PR-level review/comment that groups the remaining findings by severity.
- Do not leave review-agent findings only in chat, local files, or agent summaries. The PR must contain the review feedback so it is visible during code review.
- If GitHub comment creation fails because of connectivity, credentials, permissions, or API errors, stop and report the failed command/API action plus the finding text that still needs to be posted.



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd-profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
