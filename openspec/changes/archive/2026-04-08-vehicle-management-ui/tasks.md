## 1. API client

- [x] 1.1 Create `ui/src/lib/api.ts` with `ApiError` class (extends `Error`, has `status` and `code` properties)
- [x] 1.2 Implement `request<T>(method, url, body?)` helper — JSON serialization, response parsing, `ApiError` on non-OK, `undefined` on 204
- [x] 1.3 Export `Vehicle` and `CreateVehicle` TypeScript interfaces matching backend API contract
- [x] 1.4 Export typed functions: `fetchVehicles()`, `fetchVehicle(id)`, `createVehicle(data)`, `updateVehicle(id, data)`, `deleteVehicle(id)`

## 2. Vehicle store

- [x] 2.1 Create `ui/src/lib/stores/vehicles.svelte.ts` with runes state: `vehicles` (Vehicle[]), `loading` (boolean), `error` (string | null)
- [x] 2.2 Implement `loadVehicles()` — clears error, sets loading, fetches from API, updates list
- [x] 2.3 Implement `createVehicle(data)` — clears error, calls API, appends to list, returns vehicle or null
- [x] 2.4 Implement `updateVehicle(id, data)` — clears error, calls API, replaces in list, returns vehicle or null
- [x] 2.5 Implement `deleteVehicle(id)` — clears error, calls API, removes from list, returns boolean
- [x] 2.6 Export getter functions: `getVehicles()`, `getLoading()`, `getError()`

## 3. Vehicle store tests

- [x] 3.1 Create `ui/src/lib/stores/vehicles.test.ts` with `vi.mock('$lib/api')` pattern
- [x] 3.2 Test `loadVehicles` success: sets vehicle list, clears error
- [x] 3.3 Test `loadVehicles` failure: sets error message, keeps list unchanged
- [x] 3.4 Test `createVehicle` success: appends to list, returns vehicle
- [x] 3.5 Test `createVehicle` failure: returns null, sets error
- [x] 3.6 Test `updateVehicle` success: replaces in list, returns vehicle
- [x] 3.7 Test `deleteVehicle` success: removes from list, returns true
- [x] 3.8 Test `deleteVehicle` failure: keeps list unchanged, returns false
- [x] 3.9 Test error clearing: each action clears previous error before API call

## 4. VehicleForm component

- [x] 4.1 Create `ui/src/lib/components/VehicleForm.svelte` with props: `initial?` (Vehicle), `onsave` callback, `saving` (boolean)
- [x] 4.2 Form fields: name (text, required), make (text), model (text), year (number), fuel type (select), notes (textarea)
- [x] 4.3 All inputs use `.input-wrap` + `.input` CSS classes, buttons use `.btn` classes
- [x] 4.4 Fuel type select with options: gasoline, diesel, electric, hybrid, lpg, cng, other — default "gasoline"
- [x] 4.5 Client-side name validation: show inline error if empty/whitespace on submit, prevent `onsave` call
- [x] 4.6 Pre-fill all fields from `initial` prop when provided (edit mode)
- [x] 4.7 Disable form and show saving state when `saving` prop is true

## 5. Create vehicle page

- [x] 5.1 Create `ui/src/routes/settings/vehicles/new/+page.svelte` with `PageContainer` (width="narrow")
- [x] 5.2 Render `VehicleForm` without `initial`, wire `onsave` to store's `createVehicle`
- [x] 5.3 On success, navigate to `/settings` via `goto()`
- [x] 5.4 On failure, display error message from store

## 6. Edit vehicle page

- [x] 6.1 Create `ui/src/routes/settings/vehicles/[id]/edit/+page.svelte` with `PageContainer` (width="narrow")
- [x] 6.2 Load vehicle by ID from route param on mount using `fetchVehicle()` from API
- [x] 6.3 Render `VehicleForm` with `initial` set to the loaded vehicle
- [x] 6.4 Wire `onsave` to store's `updateVehicle`, navigate to `/settings` on success
- [x] 6.5 Show error if vehicle not found or load fails

## 7. Settings vehicles section

- [x] 7.1 Update `ui/src/routes/settings/+page.svelte` — import vehicle store, call `loadVehicles()` on mount
- [x] 7.2 Add "Vehicles" section using `.section` + `.section-title` CSS classes
- [x] 7.3 Render vehicle list: each row shows name, make/model/year subtitle, edit button (`.btn-icon`), delete button (`.btn-icon`)
- [x] 7.4 Vehicle rows use `.card` class with corner triangle
- [x] 7.5 Add "Add vehicle" button (`.btn .btn-primary`) linking to `/settings/vehicles/new`
- [x] 7.6 Show empty state when no vehicles exist with add vehicle action
- [x] 7.7 Implement inline delete confirmation: row transforms to "Delete [name]?" with confirm/cancel buttons
- [x] 7.8 Wire confirm to store's `deleteVehicle()`, revert row on cancel

## 8. Dashboard update

- [x] 8.1 Update `ui/src/routes/+page.svelte` — import vehicle store, load vehicles on mount
- [x] 8.2 When vehicle list is empty, show `EmptyState` with `Car` icon, heading "No vehicles yet", and button linking to `/settings/vehicles/new`
- [x] 8.3 When vehicles exist, show vehicle chips as tabs (placeholder for future dashboard content)

## 9. Verification

- [x] 9.1 Run `npm run format:check --prefix ui` and fix formatting
- [x] 9.2 Run `npm run lint --prefix ui` and fix lint errors
- [x] 9.3 Run `npm run check --prefix ui` and fix type errors
- [x] 9.4 Run `npm run test --prefix ui` and verify all tests pass
- [x] 9.5 Run `cargo test` to verify backend + UI tests pass
- [x] 9.6 Manually test: add vehicle from settings, edit vehicle, delete vehicle with confirmation, verify dashboard shows vehicle chips
