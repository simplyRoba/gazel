## Context

gazel has a vehicle CRUD API (`/api/vehicles`), a responsive app shell with sidebar/bottom bar navigation, CSS component classes (buttons, inputs, chips, badges, cards), and design system tokens. The UI currently shows placeholder pages. This is the first frontend feature connecting to the backend. The sibling project flowl has a mature API client (`api.ts`), store pattern (`stores/plants.ts`), and form pattern (`PlantForm.svelte`) that we follow closely but adapt for Svelte 5 runes (flowl uses Svelte 4 writables).

## Goals / Non-Goals

**Goals:**

- Establish the API client pattern (`request()` + `ApiError`) that all future features reuse
- Establish the store pattern (runes-based, async actions, error handling) that all future stores follow
- Let users add, edit, and delete vehicles from the Settings page
- Show a helpful empty state on the dashboard when no vehicles exist

**Non-Goals:**

- No vehicle photos or image upload
- No drag-to-reorder vehicles
- No vehicle detail page (vehicle details will be shown on the dashboard in a future change)
- No i18n — all strings are hardcoded English for now
- No offline support or optimistic updates
- No loading skeletons yet (simple loading states only)

## Decisions

### API client: `request()` helper + `ApiError` class

```typescript
export class ApiError extends Error {
  status: number;
  code: string;
  constructor(status: number, code: string, message: string) { ... }
}

async function request<T>(method: string, url: string, body?: unknown): Promise<T> { ... }
```

The `request()` function handles JSON serialization, response parsing, and error mapping. On non-OK responses, it reads `{ code, message }` from the body and throws `ApiError`. On 204, returns `undefined as T`.

**Rationale:** Identical to flowl's pattern. Centralizes all HTTP logic. The `ApiError.code` field enables future i18n mapping (frontend can translate codes to localized messages). The generic `T` gives type-safe returns at each call site.

**Difference from flowl:** No `recheckHealth()` call on network failure (gazel doesn't have offline detection yet). No FormData handling (no file uploads for vehicles).

### Vehicle store: Svelte 5 runes, not Svelte 4 writables

flowl uses `writable()` stores. gazel uses Svelte 5 runes (`$state`, `$derived`) since the rest of the app already uses them (theme store, layout).

```typescript
let vehicles = $state<Vehicle[]>([]);
let loading = $state(false);
let error = $state<string | null>(null);
```

Exported as getter functions and action functions:

```typescript
export function getVehicles(): Vehicle[] { return vehicles; }
export function getError(): string | null { return error; }
export async function loadVehicles(): Promise<void> { ... }
export async function createVehicle(data: CreateVehicle): Promise<Vehicle | null> { ... }
```

**Rationale:** Runes are the Svelte 5 way. The file uses `.svelte.ts` extension (like the theme store) to enable runes in plain TypeScript. Getter functions expose reactive state to components. Action functions follow flowl's pattern: clear error → call API → update state → return entity or null.

**Error handling:** Errors are stored as human-readable strings. For now, `ApiError.message` is used directly (the backend already provides English messages via `default_message()`). When i18n is added, a `resolveError()` function will map `ApiError.code` to translation keys.

### Vehicle form: shared `VehicleForm.svelte` component

A single form component used by both the create and edit pages. Props:

- `initial?: Vehicle` — when provided, form is in edit mode with pre-filled values
- `onsave: (data: CreateVehicle) => Promise<void>` — callback for submit
- `saving: boolean` — disables form during save

Fields:
- **Name** (text, required, client-side validation)
- **Make** (text, optional)
- **Model** (text, optional)
- **Year** (number, optional)
- **Fuel type** (select from allowed values, defaults to "gasoline")
- **Notes** (textarea, optional)

**Rationale:** Same shared-form pattern as flowl's `PlantForm.svelte`. The form doesn't call the API directly — the parent page handles that via the `onsave` callback. This keeps the form pure (presentation + validation) and the page responsible for navigation and state updates.

### Settings page: vehicles section with inline list

The settings page shows a "Vehicles" section using the `.section` / `.section-title` CSS classes. Vehicles are listed as simple rows with name, make/model/year subtitle, and edit/delete action buttons.

The "Add vehicle" button uses `.btn .btn-primary` and navigates to `/settings/vehicles/new`.

**Delete flow:** Clicking delete shows an inline confirmation row (the vehicle row transforms to "Delete [name]? [Confirm] [Cancel]"). No modal — keeps it lightweight. On confirm, calls the store's `deleteVehicle()` action.

**Rationale:** Inline confirmation is simpler than a modal and doesn't require a modal component. flowl uses this pattern for some delete actions.

### Route structure

```
/settings                          → Settings page with vehicles section
/settings/vehicles/new             → Create vehicle form
/settings/vehicles/[id]/edit       → Edit vehicle form
```

Both form pages use `PageContainer` with `width="narrow"` since forms are focused, single-column content.

After successful create/edit, the user is navigated back to `/settings`. After delete, the list updates in-place.

**Rationale:** Nested under `/settings` since vehicle management is an admin task, not the daily workflow. The dashboard (future change) will show vehicle data for daily use.

### Dashboard empty state update

When the vehicle store has zero vehicles, the dashboard shows the `EmptyState` component with:
- Icon: `Car` (from lucide-svelte)
- Heading: "No vehicles yet"
- Description: "Add your first vehicle to start tracking."
- Action: Button linking to `/settings/vehicles/new`

**Rationale:** Direct link to the add form (not just settings) reduces clicks for first-time setup.

### Form inputs use `.input-wrap` + `.input` CSS classes

All form fields use the global CSS component classes from `inputs.css`. This ensures the corner triangle motif appears on inputs consistently.

```svelte
<div class="input-wrap">
  <input class="input" type="text" bind:value={name} />
</div>
```

Select dropdowns also use `.input` class for consistent styling.

### Fuel type select options

The fuel type dropdown uses a static list matching the backend's `VALID_FUEL_TYPES`:

```typescript
const FUEL_TYPES = [
  { value: 'gasoline', label: 'Gasoline' },
  { value: 'diesel', label: 'Diesel' },
  { value: 'electric', label: 'Electric' },
  { value: 'hybrid', label: 'Hybrid' },
  { value: 'lpg', label: 'LPG' },
  { value: 'cng', label: 'CNG' },
  { value: 'other', label: 'Other' },
];
```

**Rationale:** Hardcoded to match the backend enum. When i18n is added, labels will be translation keys.

## Risks / Trade-offs

**No `resolveError()` yet** → Error messages come directly from the backend's `default_message()`. This means error text is English-only and not customizable on the frontend. Acceptable until i18n is built (chunk 14).

**Runes store pattern is newer** → Less community documentation than Svelte 4 writables. But the pattern is simple (state + getters + actions) and the theme store already proves it works in this codebase.

**No loading skeletons** → The vehicle list shows "Loading..." text during fetch. Skeleton CSS exists but isn't wired into components yet. Acceptable for a list that's typically 1-3 items.

**Fuel type labels not localized** → "Gasoline", "Diesel" etc. are English strings. Will be replaced with i18n keys in chunk 14.

**Delete has no undo** → Once confirmed, the vehicle is permanently deleted (hard delete on the backend). Acceptable for a rare admin action. The confirmation step prevents accidental deletion.
