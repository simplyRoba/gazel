# Contributing

## Architecture

The Rust backend serves a SvelteKit SPA. The UI is built as static files (`ui/build/`) and embedded into the Rust binary at compile time via `rust-embed`. The result is a single self-contained binary with no external file dependencies. However for development, it is nice to have hot reloading so the UI can be updated independently for a faster feedback loop. Here is how to set that up:

### Dev server with hot reloading

Requires Node.js (LTS) and Rust (stable). A devcontainer config is included.

Install UI dependencies first:

```bash
npm ci --prefix ui
```

Run two terminals:

```bash
# Terminal 1: UI with hot module reload (--host exposes on network for phone testing)
npm run dev --prefix ui -- --host

# Terminal 2: Rust backend with auto-restart on code changes
GAZEL_DB_PATH=/tmp/gazel.db SKIP_UI_BUILD=1 cargo watch -x run
```

Open `http://localhost:5173` (or the network URL printed by Vite for phone testing). Vite proxies `/api` and `/health` to the Rust backend on port 4110.

`SKIP_UI_BUILD=1` tells `build.rs` to skip the SvelteKit build so Rust recompiles fast. `cargo-watch` is installed in the devcontainer automatically.

### Testing

Run the full test suite (Rust + UI):

```bash
cargo test
```

The `ui_tests` integration test in `tests/ui.rs` shells out to `npm run test` in `ui/`, so vitest runs as part of `cargo test`. Node.js must be installed and `ui/node_modules` present (the test runs `npm install` automatically if missing).

Run only Rust backend tests (faster, no Node.js needed):

```bash
cargo test -- --skip ui_tests
```

Run only UI tests:

```bash
cargo test --test ui
```

Or run vitest directly for watch mode during UI development:

```bash
npm --prefix ui exec vitest --watch
```

### Linting and formatting

Run UI formatting check:

```bash
npm run format:check --prefix ui
```

Apply UI formatting:

```bash
npm run format --prefix ui
```

Run UI linting:

```bash
npm run lint --prefix ui
```

Auto-fix UI lint issues where possible:

```bash
npm run lint:fix --prefix ui
```

Run Svelte/TypeScript checks:

```bash
npm run check --prefix ui
```

### Build with embedded UI

To compile a binary with the UI baked in (like production), run `cargo build` without `SKIP_UI_BUILD`. This triggers `build.rs` to build the SvelteKit frontend and embed it via `rust-embed`.

## Design

<!-- TODO: define design language -->

### Color Palette

<!-- TODO: define color palette -->

### Typography

<!-- TODO: define typography -->

### Spacing & Layout

<!-- TODO: define spacing and layout -->
