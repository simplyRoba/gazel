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

Gazelle-inspired: sleek, elegant, minimal. Warm but modern — graceful, not heavy.
Tokens live in `ui/src/lib/styles/tokens.css` as CSS custom properties.
Components use CSS Modules (one `.module.css` per component) referencing these tokens.

### Color Palette

Derived from the logo: warm amber/gold gazelle tones on a cool dark slate blue.
Light and dark themes swap via `[data-theme="light"]` / `[data-theme="dark"]` on `<html>`.
Default follows system preference via `prefers-color-scheme`.

#### Neutrals

| Token | Light | Dark | Use |
|---|---|---|---|
| `--color-bg` | `#FAF7F2` | `#1A2030` | Page background |
| `--color-bg-raised` | `#FFFFFF` | `#222B3A` | Cards, modals |
| `--color-bg-sunken` | `#F0EBE3` | `#151B28` | Inset areas, inputs |
| `--color-border` | `#E0D8CC` | `#2E3A4A` | Borders, dividers |
| `--color-border-subtle` | `#EBE5DB` | `#253040` | Subtle separators |

#### Text

| Token | Light | Dark | Use |
|---|---|---|---|
| `--color-text` | `#2C2418` | `#E8E0D4` | Primary text |
| `--color-text-secondary` | `#6B6054` | `#9A9184` | Secondary / muted text |
| `--color-text-tertiary` | `#9A9084` | `#6B6258` | Placeholders, hints |
| `--color-text-inverse` | `#FAF7F2` | `#1A2030` | Text on accent backgrounds |

#### Accent (gazelle amber/gold)

| Token | Light | Dark | Use |
|---|---|---|---|
| `--color-accent` | `#C08A45` | `#D4A56A` | Primary actions, links |
| `--color-accent-hover` | `#A87538` | `#E0B878` | Hover state |
| `--color-accent-subtle` | `#F5EBD8` | `#2A2520` | Accent backgrounds (tags, badges) |
| `--color-accent-text` | `#8B6530` | `#E8C88A` | Accent text on neutral bg |

#### Semantic

| Token | Light | Dark | Use |
|---|---|---|---|
| `--color-success` | `#3D8B4F` | `#5AAE6B` | Positive states |
| `--color-warning` | `#C0882A` | `#D4A040` | Warnings |
| `--color-error` | `#C44B3F` | `#E06858` | Errors, destructive |
| `--color-info` | `#3A7ABF` | `#5A9ADF` | Informational |

#### Logo / brand

| Token | Value | Use |
|---|---|---|
| `--color-brand-bg` | `#2F3F53` | Logo background, splash |
| `--color-brand-gold-1` | `#E8C88A` | Lightest gazelle tone |
| `--color-brand-gold-2` | `#D4A56A` | Medium gazelle tone |
| `--color-brand-gold-3` | `#C49A5E` | Darker gazelle tone |

### Typography

Font: **Inter** via Google Fonts, with system sans-serif fallback.
All sizes use `rem` for accessibility. Base is `16px`.

| Token | Size | Weight | Line height | Use |
|---|---|---|---|---|
| `--font-2xl` | `1.875rem` (30px) | 700 | 1.2 | Page titles |
| `--font-xl` | `1.5rem` (24px) | 600 | 1.25 | Section headings |
| `--font-lg` | `1.25rem` (20px) | 600 | 1.3 | Card headings |
| `--font-md` | `1rem` (16px) | 400 | 1.5 | Body text |
| `--font-sm` | `0.875rem` (14px) | 400 | 1.5 | Secondary text, labels |
| `--font-xs` | `0.75rem` (12px) | 500 | 1.4 | Captions, badges |

```
--font-family: 'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
--font-family-mono: 'SF Mono', SFMono-Regular, Consolas, 'Liberation Mono', Menlo, monospace;
```

### Spacing & Layout

4px base grid. Use multiples consistently.

| Token | Value | Use |
|---|---|---|
| `--space-1` | `4px` | Tight gaps (between icon and label) |
| `--space-2` | `8px` | Compact spacing (inline elements) |
| `--space-3` | `12px` | Default inner padding |
| `--space-4` | `16px` | Standard gap, card padding |
| `--space-5` | `20px` | — |
| `--space-6` | `24px` | Section spacing |
| `--space-8` | `32px` | Large spacing between sections |
| `--space-10` | `40px` | — |
| `--space-12` | `48px` | Page-level padding |
| `--space-16` | `64px` | Major section breaks |

### Radii

| Token | Value | Use |
|---|---|---|
| `--radius-sm` | `4px` | Inputs, small elements |
| `--radius-md` | `8px` | Cards, modals |
| `--radius-lg` | `12px` | Large containers |
| `--radius-full` | `9999px` | Pills, avatars |

### Shadows

| Token | Light | Dark | Use |
|---|---|---|---|
| `--shadow-sm` | `0 1px 2px rgba(44,36,24,0.06)` | `0 1px 2px rgba(0,0,0,0.2)` | Subtle lift (buttons) |
| `--shadow-md` | `0 2px 8px rgba(44,36,24,0.08)` | `0 2px 8px rgba(0,0,0,0.3)` | Cards |
| `--shadow-lg` | `0 8px 24px rgba(44,36,24,0.12)` | `0 8px 24px rgba(0,0,0,0.4)` | Modals, dropdowns |

### Transitions

```
--transition-fast: 120ms ease;
--transition-normal: 200ms ease;
--transition-slow: 300ms ease;
```

Use `transition-fast` for micro-interactions (hover, focus).
Use `transition-normal` for visibility changes (show/hide).
Use `transition-slow` for layout shifts (collapse/expand).
No decorative animations.
