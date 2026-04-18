# Domain Pitfalls

**Domain:** OCI-hosted autograph gallery bootstrap
**Researched:** 2026-04-18

## Critical Pitfalls

### Pitfall 1: Treating the Prompt as the Product
**What goes wrong:** The team mistakes the completeness of the bootstrap prompt for repository progress and tries to deliver the full vision in one pass.
**Why it happens:** The implementation brief is unusually detailed, but the repository still lacks the thin executable baseline that would make progress measurable.
**Consequences:** Phase 1 balloons into infra, CI/CD, app scaffold, auth, upload, OCR, AI metadata extraction, search, and docs all at once; review gets muddy; nothing is truly done.
**Prevention:** Define Phase 1 as a narrow vertical slice only: deployable `Next.js` shell, one database-backed record path, one private image fetch path, one admin login path, and explicit bootstrap docs.
**Detection:** PRs add many top-level directories at once, acceptance criteria are prose-heavy instead of command-verifiable, or “almost complete” appears before any route can run end to end.

### Pitfall 2: Over-Privileged CI/CD Into a Fresh Tenancy
**What goes wrong:** GitHub Actions receives OCI credentials broad enough to manage the whole tenancy because that is the fastest way to make automation work.
**Why it happens:** OCI access commonly uses API signing keys and OCIDs, and greenfield tenancy bootstrap blurs the line between one-time admin setup and routine deploy automation.
**Consequences:** A leaked key, mis-scoped policy, or workflow mistake can affect compartments, networking, storage, or database resources well beyond this app.
**Prevention:** Keep break-glass bootstrap human-run; create a dedicated CI principal with compartment-scoped permissions; document exactly which OCIDs and secrets live in GitHub versus local operator storage; rotate keys on a schedule.
**Detection:** CI secrets include tenancy-wide admin credentials, policy docs use vague verbs like “manage all-resources,” or deployment steps require root-compartment access for normal app updates.

### Pitfall 3: Private Image Delivery Turning the App Into a Bottleneck
**What goes wrong:** The app proxies every image request in a naive buffer-to-memory path.
**Why it happens:** The product intentionally keeps Object Storage private and routes reads through the application, but that pattern becomes expensive if implemented as synchronous full-file proxying.
**Consequences:** Gallery pages feel slow, the A1 instance burns CPU and memory on image relay work, and anonymous browsing becomes less reliable under even modest traffic.
**Prevention:** Stream objects instead of buffering, generate thumbnails or constrained variants, set cache headers deliberately, and keep gallery pages from requesting full-resolution originals by default.
**Detection:** Image routes read full objects into memory, no thumbnail strategy exists, TTFB rises sharply on gallery pages, or one page load triggers many repeated storage fetches.

### Pitfall 4: Oracle Driver and Runtime Friction Arriving Late
**What goes wrong:** The project commits to Autonomous Database Free without proving that the selected Node.js driver, ORM, migrations, and deployment image behave cleanly in the chosen runtime.
**Why it happens:** Oracle is the preferred store, but JavaScript ecosystem examples and defaults more often target Postgres or MySQL, so hidden client and packaging friction is easy to underestimate.
**Consequences:** Late rework around connection handling, container build dependencies, ORM limitations, or a forced fallback database decision after application code has already spread.
**Prevention:** Prove the hardest path first: container build, DB connection, schema migration, read/write cycle, and CI validation story against the intended Oracle stack before building higher-level features.
**Detection:** Database choice discussions stay abstract, migration tooling is not exercised in a real container, or “fallback if needed” remains undefined past the first implementation phase.

### Pitfall 5: Ambiguous Tenancy Bootstrap Ownership
**What goes wrong:** The repo mixes manual tenancy setup, break-glass recovery, IAM creation, networking, and app deploy steps without clearly stating who does what and in what order.
**Why it happens:** A blank OCI tenancy makes bootstrap feel like one continuous automation story, but some steps should remain manual for safety and recoverability.
**Consequences:** Operators get stuck, CI cannot run because prerequisite identities or compartments do not exist, or the project quietly depends on console actions that were never documented precisely.
**Prevention:** Split bootstrap into three tracks: manual security bootstrap, automatable baseline infra, and routine app deploy. Mark every step as `manual once`, `automated`, or `manual break-glass only`.
**Detection:** Setup docs say “configure OCI” without exact intent, bootstrap order is unclear, or a fresh operator cannot tell which OCIDs must exist before the first merge to `main`.

### Pitfall 6: Prompt-Complete vs Product-Complete Confusion
**What goes wrong:** Status reporting treats prompt artifacts, summaries, or planning docs as evidence that the platform is operational.
**Why it happens:** This repo began prompt-first, and the written specification is more mature than the implementation.
**Consequences:** Roadmaps become optimistic, verification gets deferred, and missing essentials like `.env.example`, workflows, or a runnable app shell hide behind polished documentation.
**Prevention:** Require every “complete” claim to map to a committed artifact plus a local or live verification command. Planning docs should point at code, not substitute for it.
**Detection:** Milestone summaries cite documents more often than runnable files, or success criteria are checked off without commands, screenshots, or deployed proof.

## Moderate Pitfalls

### Pitfall 1: Always Free Capacity Assumptions
**What goes wrong:** The design assumes OCI Always Free limits are generous enough for storage, API calls, and database usage without budgeting for headroom.
**Prevention:** Define expected image sizes, thumbnail policy, retention assumptions, and request budgets early; prefer metadata-light MVP usage patterns and monitor for quota pressure from day one.

### Pitfall 2: Single-Admin Auth Growing Into a Mini Identity System
**What goes wrong:** A simple admin path grows into invitations, password reset flows, roles, or audit features that the product does not need yet.
**Prevention:** Keep one explicit admin credential path for v1, document rotation and recovery manually, and defer multi-user features entirely.

### Pitfall 3: Search Scope Expanding Before the Catalog Exists
**What goes wrong:** The team reaches for advanced search, fuzzy indexing, or semantic retrieval before basic metadata filters are proven.
**Prevention:** Ship signer, category, and tags first; treat richer discovery as a later optimization tied to real usage.

### Pitfall 4: OCR and AI Suggestions Becoming Blocking Dependencies
**What goes wrong:** Upload success depends on perfect OCR or model output rather than graceful admin review.
**Prevention:** Make OCR/AI advisory only; allow manual entry to succeed even when extraction is poor, slow, or unavailable.

## Minor Pitfalls

### Pitfall 1: Metadata Model Coupled to Object Paths
**What goes wrong:** UI assumptions leak into object naming or storage layout, making future changes to image handling painful.
**Prevention:** Store stable object references and metadata separately; avoid encoding display logic in bucket paths.

### Pitfall 2: Missing Secret Contract Early
**What goes wrong:** Contributors invent ad hoc environment names for OCI credentials, OCIDs, and app secrets.
**Prevention:** Publish `.env.example` and GitHub secret naming conventions before the first deploy workflow lands.

### Pitfall 3: No Reverse Proxy Plan for Self-Hosted Next.js
**What goes wrong:** The app is exposed directly without the reverse-proxy layer recommended for self-hosting.
**Prevention:** Treat the proxy, TLS termination, and request forwarding rules as part of the base deployment shape, not post-launch polish.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Bootstrap planning | Turning the entire prompt into Phase 1 scope | Define a thin vertical slice with explicit non-goals |
| IAM and secrets | Giving GitHub Actions broad tenancy power | Separate manual bootstrap admin from compartment-scoped CI identity |
| OCI bootstrap docs | Unclear manual vs automated steps | Label each step by ownership and frequency |
| App deployment | Exposing Next.js directly on the instance | Put a reverse proxy in front of the app server from day one |
| Database integration | Discovering Oracle client/tooling issues late | Validate containerized read/write/migration flow before feature work |
| Image delivery | Full-resolution proxying for every gallery request | Stream, cache, and serve thumbnails for list views |
| Upload workflow | Making OCR/AI success mandatory | Keep admin review/manual entry as the source of truth |
| MVP acceptance | Counting planning docs as shipped functionality | Gate completion on committed artifacts plus runnable verification |

## Sources

- `.planning/PROJECT.md` (project scope and constraints) — HIGH confidence
- `.prompts/001-autograph-gallery-bootstrap-do/001-autograph-gallery-bootstrap-do.md` (authoritative implementation brief) — HIGH confidence
- `.planning/codebase/CONCERNS.md` (current repo risks and planning drift) — HIGH confidence
- Orchestrator-supplied official evidence: OCI Always Free quotas and Autonomous Database Free constraints — HIGH confidence
- Orchestrator-supplied official evidence: Next.js self-hosting guidance recommends a reverse proxy in front of the app server — HIGH confidence
- Orchestrator-supplied official evidence: OCI API access commonly relies on API signing keys and OCIDs, creating secret and IAM boundary risk — HIGH confidence
