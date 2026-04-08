## Why

The vehicle CRUD API exists but there's no way to use it from the UI. Users need to add, edit, and delete vehicles before they can track fill-ups. This is the first frontend feature that connects to the backend, establishing the API client, store patterns, and form patterns that all future UI features will follow.

## What Changes

- **API client** (`ui/src/lib/api.ts`): Centralized `request()` helper with `ApiError` class (status + code), typed vehicle endpoint functions (list, get, create, update, delete). Following flowl's proven pattern.
- **Vehicle store** (`ui/src/lib/stores/vehicles.svelte.ts`): Svelte 5 runes-based store managing vehicle list, loading/error state, and async CRUD actions. Each action clears errors, calls the API, and updates state immutably.
- **Settings vehicles section**: The settings page (`/settings`) gets a "Vehicles" section showing all vehicles as a list with edit/delete actions and an "Add vehicle" button.
- **Vehicle form page**: Shared form at `/settings/vehicles/new` (create) and `/settings/vehicles/:id/edit` (edit). Fields: name, make, model, year, fuel type, notes. Client-side name validation; all other validation is server-side via ApiError.
- **Delete confirmation**: Inline confirmation in the vehicle list before deleting.
- **Dashboard empty state update**: When no vehicles exist, the dashboard empty state links to settings to add the first vehicle.
- **Vitest tests**: Store tests (mock API, assert state), following flowl's vi.mock pattern.

## Capabilities

### New Capabilities

- `api-client`: Centralized HTTP request helper, ApiError class, and typed endpoint functions for communicating with the backend API
- `vehicle-store`: Frontend state management for vehicles — loading, CRUD actions, error handling, reactive state
- `vehicle-forms`: Vehicle add/edit form pages under settings, delete confirmation, settings vehicles section

### Modified Capabilities

_(none)_

## Impact

- **UI files created**: `ui/src/lib/api.ts`, `ui/src/lib/stores/vehicles.svelte.ts`, `ui/src/lib/stores/vehicles.test.ts`, `ui/src/lib/components/VehicleForm.svelte`, `ui/src/routes/settings/vehicles/new/+page.svelte`, `ui/src/routes/settings/vehicles/[id]/edit/+page.svelte`
- **UI files modified**: `ui/src/routes/settings/+page.svelte` (add vehicles section), `ui/src/routes/+page.svelte` (update empty state link)
- **Backend**: No changes
- **Dependencies**: No new packages
- **Vite proxy**: Already configured to proxy `/api` to the Rust backend on port 4110
