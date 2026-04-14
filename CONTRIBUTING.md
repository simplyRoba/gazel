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
| `--color-text-tertiary` | `#736860` | `#928880` | Placeholders, hints |
| `--color-text-inverse` | `#FFFFFF` | `#1A2030` | Text on accent backgrounds |

#### Accent (gazelle amber/gold)

| Token | Light | Dark | Use |
|---|---|---|---|
| `--color-accent` | `#996B2E` | `#D4A56A` | Primary actions, links |
| `--color-accent-hover` | `#7D5824` | `#E0B878` | Hover state |
| `--color-accent-subtle` | `#F5EBD8` | `#2A2520` | Accent backgrounds (tags, badges) |
| `--color-accent-text` | `#8B6530` | `#E8C88A` | Accent text on neutral bg |

#### Semantic

| Token | Light | Dark | Use |
|---|---|---|---|
| `--color-success` | `#2E7A40` | `#5AAE6B` | Positive states |
| `--color-warning` | `#8D6518` | `#D4A040` | Warnings |
| `--color-error` | `#B33D32` | `#E06858` | Errors, destructive |
| `--color-info` | `#2E6BA8` | `#5A9ADF` | Informational |

#### Logo / brand

| Token | Value | Use |
|---|---|---|
| `--color-brand-bg` | `#2F3F53` | Logo background, sidebar, active chips |
| `--color-brand-gold-1` | `#E8C88A` | Lightest gazelle tone |
| `--color-brand-gold-2` | `#D4A56A` | Medium gazelle tone |
| `--color-brand-gold-3` | `#C49A5E` | Darker gazelle tone |

#### Navigation

The sidebar and bottom bar use the dark slate brand background in both
themes — this breaks the monotone and creates the warm-gold-on-cool-slate
contrast from the logo.

| Token | Light | Dark | Use |
|---|---|---|---|
| `--color-nav-bg` | `#2F3F53` | `#151B28` | Sidebar / bottom bar background |
| `--color-nav-text` | `#B0B8C8` | `#7A8494` | Inactive nav item text |
| `--color-nav-text-active` | `#E8C88A` | `#E8C88A` | Active nav item text (gold) |
| `--color-nav-border` | `rgba(255,255,255,0.08)` | `rgba(255,255,255,0.06)` | Nav border |
| `--color-nav-hover` | `rgba(255,255,255,0.06)` | `rgba(255,255,255,0.04)` | Nav item hover |

#### Feature surface

A cool blue-tinted surface for data/stat cards, distinguishing them from
warm neutral content cards.

| Token | Light | Dark | Use |
|---|---|---|---|
| `--color-bg-feature` | `#F0F2F6` | `#1E2636` | Summary stat card background |
| `--color-border-feature` | `#D8DCE6` | `#2A3444` | Summary stat card border |

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

### Corner Triangle (Brand Motif)

gazel's defining visual element: a small **accent-colored triangle** in the
**top-right corner** of cards, buttons, and interactive elements. Directly
references the triangular logo and makes the app instantly recognizable.

| Token | Value | Use |
|---|---|---|
| `--corner-tri-sm` | `10px` | Buttons, small elements |
| `--corner-tri-md` | `14px` | Cards, list items |
| `--corner-tri-lg` | `20px` | Modals, hero sections |
| `--radius-full` | `9999px` | CTA circle, pills only |

Implement via `::after` pseudo-element with CSS border trick:

```css
/* Card with accent corner triangle */
.card {
  position: relative;
  overflow: hidden;
}
.card::after {
  content: '';
  position: absolute;
  top: 0;
  right: 0;
  width: 0;
  height: 0;
  border-style: solid;
  border-width: 0 var(--corner-tri-md) var(--corner-tri-md) 0;
  border-color: transparent var(--color-accent) transparent transparent;
  transition: border-width var(--transition-fast);
}
/* Triangle grows on hover */
.card:hover::after {
  border-width: 0 calc(var(--corner-tri-md) + 4px) calc(var(--corner-tri-md) + 4px) 0;
}
```

**Rules:**
- Cards, modals: corner triangle always visible, grows on hover
- Buttons: corner triangle appears on hover (hidden by default)
- Inputs: no corner triangle — sharp rectangles only
- Only `--radius-full` for CTA circle and pills
- Never use `border-radius` on cards or buttons

### Triangle Accent Markers

Small triangular motifs used as active-state indicators and section accents.
Implemented via CSS borders (zero-width trick) or inline SVG.

**Usage:**
- Active nav item: small triangle pointing right (sidebar) or up (bottom bar)
- Section headers: small triangular bullet before the label
- Dividers: thin accent line with a triangle notch

```css
/* Triangle marker via CSS border trick */
.marker::before {
  content: '';
  width: 0;
  height: 0;
  border-left: 5px solid var(--color-accent);
  border-top: 4px solid transparent;
  border-bottom: 4px solid transparent;
}
```

### Data Treatment

Stats and numbers are the hero of a fuel tracking app. They get special treatment:

| Token | Value | Use |
|---|---|---|
| `--font-stat` | `2.25rem` (36px) | Large stat values (L/100km, cost) |
| `--font-stat-weight` | `700` | Bold weight for stat numbers |
| `--font-family-mono` | `'SF Mono', Consolas, ...` | All numerical data |

**Rules:**
- Stat values use `--font-stat` + `--font-family-mono` + `--color-accent-text`
- All numerical data (stats, fill-up costs, odometer, efficiency) uses `--font-family-mono`
- Stat labels use `--font-xs` + `--color-text-secondary` (normal font, not mono)
- Stats are always paired: value above, label below
- On mobile, stats can flow horizontally in a compact row

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

### Design Language Summary

| Element | Treatment |
|---|---|
| **Cards, modals** | Accent triangle in top-right corner (visible, grows on hover) |
| **Buttons** | Sharp rectangles, accent triangle appears on hover |
| **Inputs** | Sharp rectangles, no decoration |
| **CTA, pills** | Full circle (`--radius-full`) |
| **Sidebar, bottom bar** | Dark slate brand background (`--color-nav-bg`) in both themes |
| **Active nav** | Gold text on dark slate, triangle accent marker |
| **Active chip** | Brand-bg with gold text (ties to sidebar color language) |
| **Stat cards** | Blue-tinted feature surface (`--color-bg-feature`) |
| **Stat values** | Oversized (`--font-stat`), monospace, accent-colored |
| **All numbers** | Monospace font (`--font-family-mono`) |
| **Corners** | Never `border-radius` on cards or buttons |
