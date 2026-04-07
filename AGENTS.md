# AGENTS.md

Guidance for coding agents working in `gazel` (Rust backend + SvelteKit frontend).

## Personas
- Developer: maintains the Rust service, Docker artifacts, and integration behavior.
- End user: runs `gazel` locally/self-hosted and accesses API/UI directly.

## Ground rules
- Ask clarifying questions when requirements materially affect behavior and cannot be inferred from code/specs.
- When the user asks for a staged or limited change, do only that requested step and stop before follow-on changes until asked.
- Prefer small, focused changes; avoid drive-by refactors unless needed for correctness.
- Never merge or push to `main` without asking; keep branches short-lived and purpose-specific.
- Use Conventional Commits for commit messages (`feat:`, `fix:`, `refactor:`, etc.).
- Treat all changes as pending until a human review is completed.

## Build, lint, and test commands

### Setup
- Install Rust stable toolchain (`rustup toolchain install stable`).
- Install UI deps: `npm ci --prefix ui`.

### Build
- Build full app (includes UI build via `build.rs`): `cargo build`.
- Build release binary: `cargo build --release`.
- Fast backend-only dev build (skip UI embed step): `SKIP_UI_BUILD=1 cargo build`.
- Run app locally: `cargo run`.

### Lint and format
- Rust format check: `cargo fmt -- --check`.
- Rust format apply: `cargo fmt`.
- Rust lint (CI parity): `cargo clippy -- -D warnings`.
- UI format check: `npm run format:check --prefix ui`.
- UI format apply: `npm run format --prefix ui`.
- UI lint: `npm run lint --prefix ui`.
- UI lint fix: `npm run lint:fix --prefix ui`.
- UI type check: `npm run check --prefix ui`.

### Test suites
- Full test suite (Rust + UI via integration test): `cargo test`.
- Rust-only tests (skip UI bridge test): `cargo test -- --skip ui_tests`.
- UI tests through Rust integration target: `cargo test --test ui`.
- UI tests directly with vitest: `npm run test --prefix ui`.

### Running a single test (important)
- Single Rust test by name (any target): `cargo test <test_name>`.
- Single integration test file: `cargo test --test fillups`.
- Single test in an integration file: `cargo test --test fillups create_fillup`.
- Exact single Rust test match: `cargo test create_fillup -- --exact`.
- Single UI test file: `npm run test --prefix ui -- ui/src/lib/stores/fillups.test.ts`.
- Single UI test by name: `npm run test --prefix ui -- ui/src/lib/stores/fillups.test.ts -t "loads fillups"`.
- Vitest watch mode for one file: `npm --prefix ui exec vitest ui/src/lib/stores/fillups.test.ts --watch`.

### Recommended pre-review gate
- Run in this order for quick feedback: `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, `npm run check --prefix ui`, `cargo test`.

## Repository-specific testing behavior
- Rust integration tests live in `tests/*.rs`; shared helpers live in `tests/common/mod.rs`.
- Many backend tests build an in-memory SQLite app via `common::test_app()` and exercise the Axum router with `tower::ServiceExt::oneshot`.
- Backend endpoint tests usually assert HTTP status first, then inspect JSON response bodies.
- `tests/ui.rs` runs `npm run test` in `ui/` and auto-runs `npm install` if `ui/node_modules` is missing.
- UI tests are co-located: store/utility tests near source, component tests near components, route tests under `ui/src/tests/routes/`.
- Test names should describe scenario/state; assertions describe expected outcomes.

## Code style guidelines

### Cross-cutting
- Follow existing code patterns in touched files before introducing new idioms.
- Keep functions cohesive and focused; avoid large mixed-responsibility blocks.
- Prefer explicit types and clear data-shape boundaries between backend and frontend.
- Avoid adding dependencies unless there is a clear need and repeated benefit.

### Rust style (`src/`, `tests/`)
- Use Rust stable only; no nightly features.
- Formatting is `rustfmt` default; do not hand-format against formatter output.
- Clippy policy is strict (`pedantic = deny` in `Cargo.toml`); satisfy lints without adding blanket `allow`s.
- Exception: localized `#[allow(...)]` is acceptable when modeling intent (example: `Option<Option<T>>` for nullable-vs-absent patch semantics).
- Imports are typically grouped as: std, external crates, internal modules (`super`/`crate`). Keep groups tidy.
- Naming: snake_case for functions/modules/variables, PascalCase for structs/enums/traits, SCREAMING_SNAKE_CASE for constants.
- Use `Result<T, ApiError>` for API handlers and map errors to explicit variants.
- API errors should return JSON `{ "message": string }` via `ApiError`/`IntoResponse`.
- Validate request inputs at boundary handlers (required fields, enum domains, ranges).
- Prefer parameterized SQL with `sqlx` binds; avoid string interpolation for user input.
- Add `/// # Errors` docs for public functions that can fail.
- In async services, favor graceful degradation/logging over panics except during startup invariants.

### TypeScript/Svelte style (`ui/`)
- TypeScript runs in `strict` mode; keep strict typing intact.
- Use named `interface`/`type` for API payloads and store state.
- Prefer `import type` for type-only imports.
- Prefer single quotes and semicolons, matching existing UI code style.
- Formatting is enforced with Prettier; linting is enforced with ESLint plus `eslint-plugin-svelte`.
- File names: Svelte components use `PascalCase.svelte`; TS modules/stores/utils use lower-case or feature names like `fillups.ts`.
- Svelte 5 runes are the established pattern (`$state`, `$derived`, `$effect`, `$props`). Do not mix legacy patterns without reason.
- Keep UI state updates immutable (map/filter/spread) in stores and component logic.
- Catch async UI errors and surface user-facing messages via store/page error state.
- Keep API access centralized in `ui/src/lib/api.ts`; avoid ad-hoc fetch logic in many components.

### API and data contracts
- Backend JSON fields are snake_case; frontend interfaces mirror this exactly.
- Keep nullability semantics explicit and consistent across Rust structs and TS interfaces.
- If changing API shape, update backend handlers/tests, `ui/src/lib/api.ts`, and affected UI tests.

## Architecture and workflow notes
- Backend: Axum + SQLx + SQLite.
- Frontend: SvelteKit static build embedded in Rust binary via `rust-embed`.
- For fast dev loop with hot reload, run UI and backend separately (see `CONTRIBUTING.md`).
- OpenSpec artifacts live under `openspec/`; keep implementation aligned with active specs.

## Spec and review expectations
- Check relevant OpenSpec docs before implementing behavior changes: canonical specs in `openspec/specs/**` and proposals/tasks in `openspec/changes/**`.
- Before requesting review, ensure lint/tests pass and changes are spec-consistent.

## Practical implementation checklist
- For backend endpoint changes, usually touch handler in `src/api/**`, request/response types, SQL query + migration (if schema changed), and integration tests in `tests/**`.
- For frontend API contract changes, usually touch `ui/src/lib/api.ts`, impacted stores, route/component usage, and vitest coverage.
- Keep migration files append-only in `migrations/`; do not edit already-applied migrations.
- Prefer minimal, targeted assertions in tests that verify behavior (status code, message, key payload fields).

## Import and module conventions
- Rust `use` statements should be grouped as std, third-party crates, then local modules (`super`, `crate`).
- In TypeScript, import external packages first, `$lib/*` aliases next, and relative imports last.
- Re-export modules sparingly (`mod.rs`/`index.ts`) and only when it improves call-site clarity.

## Error-handling conventions
- Backend: convert infrastructure errors to `ApiError::*` with user-safe messages.
- Backend: reserve panics/`expect` for startup invariants and test code, not request paths.
- Frontend: catch async failures in stores/routes and expose a user-visible error state.
- Frontend: throw `Error` objects (not strings) when propagating failures.

## Useful references
- `README.md`: runtime configuration, deployment, and feature overview.
- `CONTRIBUTING.md`: local dev workflow and testing notes.
- `Cargo.toml`: lint strictness and dependency/tooling context.
- `ui/package.json`: frontend scripts and toolchain entry points.
- `.github/workflows/ci.yml`: CI source of truth for enforced checks.
