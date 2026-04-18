<objective>
Build a production-lean autograph photo sharing website from scratch in a nearly empty repository, using Oracle Cloud Infrastructure Always Free services wherever feasible.

Purpose: Launch a public site for anonymous browsing, search, and detail views of autograph pieces, with each photo stored in private OCI Object Storage and each autograph record linked to metadata in Oracle Autonomous Database Free. Treat CI/CD as part of the initial platform bootstrap: start with deployment and infrastructure as code for a brand-new tenancy, wire GitHub Actions to validate and deploy that infrastructure on merge to `main`, then scaffold the application, admin upload workflow, AI-assisted metadata extraction, and public browsing experience.

Output: Infrastructure code, tenancy bootstrap guidance and automation, GitHub Actions pipelines, deployment scripts, containerized application code, documentation, and a completion summary saved in this repository.
</objective>

<context>
Current repository state: @README.md

This is a greenfield repo. Assume no existing application structure, no cloud resources, and no preconfigured OCI tenancy assets beyond a brand-new tenancy that the operator can authenticate against with the OCI CLI.

Assume GitHub is the source of truth for delivery. The repository should be able to validate, build, and auto-deploy the stack after in-repo checks pass, using GitHub Actions and GitHub Secrets for OCI credentials and deployment inputs.

Product scope already chosen:
- Public users are fully anonymous and read-only.
- There is exactly one admin account for uploads and content management.
- Use a single `Next.js` full-stack application for v1 rather than splitting frontend and backend into separate services.
- The deployed application stack should be fully containerized.
- Auto-deploy should happen on merge to `main` after validations pass.
- No staging environment is required for v1.
</context>

<requirements>
Thoroughly analyze the OCI Always Free constraints and choose an implementation that stays realistic for a new tenancy while still meeting the product goal.

Functional requirements:
- Provision infrastructure as code first.
- Use OCI Object Storage to store autograph photos.
- Use Oracle Autonomous Database Free to store autograph metadata and object references, unless a fallback is truly required by implementation friction.
- Serve a public website from OCI Free Tier services.
- Support a public gallery view with search or filtering by at least signer name, category, and tags.
- Support viewing a single autograph item with full metadata and image display.
- Support an upload/admin path for adding autograph entries and uploading images.
- Support exactly one admin account for authentication and content management.
- Persist enough metadata to make the gallery genuinely useful, including title, signer, description, event/source, date if known, tags, object storage location, teams/people categories, event/location, inscription text, authentication/certification company and ID, and estimated year.
- Include AI-assisted processing during admin upload to suggest fields such as signer/actor, item type, and other metadata, with the admin reviewing and confirming before save.

Delivery requirements:
- From the beginning, configure the repository for continuous validation and automatic deployment after GitHub Actions checks pass.
- Use GitHub Secrets for OCI credentials and sensitive deployment configuration.
- Include workflows for validation on pull requests and deployment on merges to `main` after checks pass.
- Make the deployment workflow capable of applying infrastructure and deploying the application with minimal manual intervention.
- Treat CI/CD as a foundational part of the platform bootstrap, not a later enhancement.
- Ensure GitHub Actions can provision or update the tenancy-facing infrastructure needed for the app, including object storage, networking, compute, and related deploy resources, as part of the merge-to-main flow.

Tenancy bootstrap requirements:
- Include initialization guidance and automation for a brand-new OCI tenancy.
- Include best practices for break-glass access, IAM setup, compartment strategy, virtual networking, and minimal compute footprint.
- Keep privileges narrow and explicit. Avoid broad administrator access for routine deploy workflows.
- Document what must be created manually at tenancy bootstrap time versus what can be safely automated.
- Size the tenancy baseline for a single-app environment without drifting into enterprise sprawl.

Operational requirements:
- The stack must be bootstrappable in a blank tenancy.
- The infrastructure must be reproducible via code and documented.
- Configuration and secrets handling must be explicit and environment-driven.
- The solution must prefer low-cost or Always Free OCI primitives over paid managed services.

Quality requirements:
- Keep the architecture simple enough for one developer to operate.
- Prefer boring, maintainable technologies over novelty.
- Include automated checks that fit the selected stack.
- Document any unavoidable manual OCI setup steps clearly.
- Keep v1 intentionally narrow: no bulk import, no multiple images per item unless absolutely necessary, no public user accounts, no advanced search beyond metadata-based filters, and no edit history/versioning system.
</requirements>

<implementation>
Choose and justify a minimal architecture before writing code. A strong default is:
- Terraform for OCI infrastructure.
- One OCI Compute A1 Flex Always Free instance to host the web app and API.
- OCI Object Storage bucket for autograph images.
- Oracle Autonomous Database Free as the preferred system of record for metadata.
- A single `Next.js` full-stack application handling public pages, admin UI, API routes, auth, and app-mediated image delivery.
- GitHub Actions for validation and deployment orchestration.
- Containerized deployment artifacts for the application stack.

Prefer this execution order:
1. Define the target architecture, security posture, repository layout, delivery flow, and GitHub-driven deployment model.
2. Create tenancy bootstrap documentation and automation for IAM, compartments, networking, break-glass account handling, and minimal compute policy boundaries.
3. Create infrastructure as code for networking, instance, bucket, IAM/policies, and database-related setup where automation is possible.
4. Establish GitHub Actions and secret contracts early so merge-to-main can validate and deploy the infrastructure baseline.
5. Scaffold the containerized `Next.js` application with database access, admin authentication, and object storage integration.
6. Implement schema, migrations, and seed/dev helpers.
7. Implement admin upload flow that uploads files to Object Storage, runs AI-assisted metadata extraction, and writes reviewed metadata to the database.
8. Implement public gallery and detail pages.
9. Complete the deploy pipeline so GitHub Actions can roll forward infra and app changes together after validations pass.
10. Add deployment scripts, environment examples, and operator documentation.
11. Verify locally as far as possible, and document what requires live OCI credentials or tenancy access.

Carefully evaluate trade-offs for:
- Whether private images should be served through the app/backend or exposed through signed access with tight controls. For v1, prefer app-mediated delivery so Object Storage remains private and access control stays centralized in the application.
- Which Oracle database client and ORM strategy is most practical for the selected runtime.
- How to keep the single-admin authentication path simple and secure.
- Use a hybrid AI metadata extraction path for v1: OCR plus AI-assisted suggestions when helpful, with the admin able to review and edit all suggested fields before persistence.
- How GitHub Actions should authenticate to OCI with the least risky credential pattern available for this stack.
- Which tenancy bootstrap items must remain human-executed because of account recovery or break-glass concerns.

For IAM and tenancy bootstrap, prefer:
- Separate compartments for shared/security-sensitive resources versus app runtime resources if that keeps the model understandable.
- Least-privilege policies for CI/CD principals.
- Explicit documentation for emergency access and credential rotation.
- Minimal public exposure in networking rules.

Avoid:
- Architectures that require multiple paid OCI services to function.
- Overengineering with microservices.
- Deeply coupling storage object paths to UI assumptions without a stable metadata model.
- Vague "set this up in the console" instructions without documenting exact intent.
- Giving GitHub Actions unnecessary tenancy-wide permissions.
- Designing for multi-user account systems when the product only needs one admin account.
- Adding bulk-ingestion or version-history systems to v1.

For maximum efficiency, invoke independent file reads, searches, and non-dependent setup tasks in parallel where tool support allows it.
</implementation>

<output>
Create or modify files needed to ship the first working version, including artifacts similar to:
- `./infra/` - Terraform or equivalent OCI infrastructure as code for tenancy bootstrap and app resources
- `./.github/workflows/` - CI and auto-deploy workflows
- `./app/` or framework-standard source directories - website and API implementation
- `./db/` or framework-standard database directories - schema, migrations, and seed helpers
- `./scripts/` - deployment/bootstrap helpers for OCI CLI, app setup, and operator tasks
- `./docs/` - tenancy bootstrap, deployment, architecture, IAM, and operator documentation
- `./.env.example` - required configuration variables with safe placeholders
- `./README.md` - updated project overview and quickstart

If the selected framework uses a different conventional layout, follow that convention instead of forcing these exact folders.

Also create documentation that explicitly lists the required GitHub Secrets and what each is used for.
</output>

<verification>
Before declaring complete:
- Run the project’s install/build/test/lint/type-check commands for the chosen stack.
- Validate the infrastructure code format and syntax, such as `terraform fmt -check` and `terraform validate`, if Terraform is used.
- Validate GitHub Actions workflow syntax and ensure the validation and deploy jobs are logically wired together.
- Confirm the initial bootstrap path can be driven by GitHub Actions for infra deployment after merge to `main`, subject to the documented required secrets and any one-time manual tenancy prerequisites.
- Confirm the app can create and read autograph metadata records in local or testable conditions.
- Confirm the upload flow persists object references and metadata together.
- Confirm AI-assisted upload suggestions are reviewable by the admin before final save.
- Confirm the hybrid OCR-plus-AI extraction flow can be exercised against sample images and that the admin can correct imperfect suggestions.
- Confirm the public gallery and item detail experience render meaningful seeded or test data.
- Confirm documentation clearly distinguishes between manual tenancy bootstrap steps and automated deploy steps.
- Document any remaining live-cloud verification steps that require OCI credentials, tenancy identifiers, or manual console actions.
</verification>

<summary_requirements>
Create `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md`.

Include:
- One-liner
- Version
- Key Findings
- Files Created
- Decisions Needed
- Blockers
- Next Step

For Files Created, list the major implementation artifacts and what each one does. Be explicit about what was verified locally versus what still needs a live OCI tenancy.
</summary_requirements>

<success_criteria>
- The repository contains a coherent end-to-end starter implementation for the autograph gallery.
- Infrastructure as code exists for tenancy bootstrap, OCI hosting baseline, and supporting resources.
- The application stores autograph metadata in an Oracle Free Tier database and image assets in OCI Object Storage.
- The application uses Oracle Autonomous Database Free unless a documented, justified fallback proved necessary.
- The website can present a public gallery and item detail pages from stored data.
- Image objects are not directly public, and the chosen access pattern uses app-mediated delivery while still supporting anonymous browsing.
- There is exactly one admin authentication path and no public user account system.
- The implementation uses a single `Next.js` full-stack app for v1 rather than separate frontend/backend services.
- The deployed application stack is containerized.
- Admin uploads include hybrid OCR-plus-AI metadata suggestions with human confirmation before persistence.
- GitHub Actions can validate the repo and automatically deploy the stack after checks pass, using documented GitHub Secrets.
- GitHub Actions are part of the initial bootstrap path and can deploy the infrastructure baseline on merge to `main`.
- The tenancy bootstrap guidance covers break-glass access, IAM, networking, and minimal compute with clear operator instructions.
- An operator can understand how to deploy the system into a new OCI tenancy from the repo docs.
- `.prompts/001-autograph-gallery-bootstrap-do/SUMMARY.md` exists and clearly states outcomes, gaps, and next action.
</success_criteria>
