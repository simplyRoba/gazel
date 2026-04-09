## 1. Backend: Tighten fill-up API contract

- [x] 1.1 Make `odometer` required in `CreateFillup` -- validate presence with `FILLUP_ODOMETER_REQUIRED` error code; add error code to `ApiError`
- [x] 1.2 Make `cost` required in `CreateFillup` -- validate presence with `FILLUP_COST_REQUIRED` error code; add error code to `ApiError`
- [x] 1.3 Make `odometer` and `cost` required in `UpdateFillup` (change from `Option<f64>` to `f64` in the struct)
- [x] 1.4 Auto-populate `fuel_unit` and `currency` from settings table on create and update -- read settings row, use `volume_unit` for `fuel_unit` and `currency` for `currency`; remove these fields from `CreateFillup`/`UpdateFillup` structs
- [x] 1.5 Change `is_full_tank` default from `false` to `true` on create
- [x] 1.6 Update existing backend integration tests in `tests/fillups.rs` to match new required fields and defaults

## 2. Frontend: API client and types

- [x] 2.1 Add `Fillup`, `CreateFillup`, `UpdateFillup` TypeScript interfaces to `ui/src/lib/api.ts`
- [x] 2.2 Add `fetchFillups(vehicleId)`, `fetchFillup(vehicleId, id)`, `createFillup(vehicleId, data)`, `updateFillup(vehicleId, id, data)`, `deleteFillup(vehicleId, id)` functions to `ui/src/lib/api.ts`

## 3. Frontend: Fill-up store

- [x] 3.1 Create `ui/src/lib/stores/fillups.svelte.ts` with `$state` runes: fill-up cache (Map keyed by vehicleId), loading, error, activeVehicleId
- [x] 3.2 Implement getter functions: `getFillups()`, `getLoading()`, `getError()`, `getActiveVehicleId()`
- [x] 3.3 Implement `loadFillups(vehicleId)` action -- fetch from API, update cache
- [x] 3.4 Implement `createFillup(vehicleId, data)` action -- call API, insert into cache in sort order
- [x] 3.5 Implement `updateFillup(vehicleId, fillupId, data)` action -- call API, replace in cache
- [x] 3.6 Implement `deleteFillup(vehicleId, fillupId)` action -- call API, remove from cache
- [x] 3.7 Implement `setActiveVehicle(vehicleId)` action -- update activeVehicleId, trigger loadFillups
- [x] 3.8 Write vitest tests for the fill-up store (mock API, test all actions, error handling, cache behavior)

## 4. Frontend: Fill-up form modal

- [x] 4.1 Create `ui/src/lib/components/FillupForm.svelte` -- dual-mode create/edit form with `$props()` for `initial` (optional Fillup), `vehicleId`, `onsave`, `oncancel`, `ondelete` callbacks
- [x] 4.2 Implement form fields: date (default today), odometer (with unit label from settings), fuel_amount (with unit label from settings), cost (with currency symbol from settings), station, notes, is_full_tank toggle (default ON), is_missed toggle (default OFF)
- [x] 4.3 Implement client-side validation: required fields, positive numbers
- [x] 4.4 Create `ui/src/lib/components/FillupModal.svelte` -- wraps FillupForm in a native `<dialog>` element; handles open/close, passes data to form; includes delete button in edit mode that triggers a ModalDialog confirmation
- [x] 4.5 Implement smart missed fill-up prompt: compare odometer gap to 1.75x average gap from store data; show inline suggestion below odometer field when gap is suspicious
- [x] 4.6 Write vitest tests for FillupForm (validation, defaults, create/edit modes)

## 5. Frontend: Dashboard integration

- [x] 5.1 Make vehicle chips interactive in `ui/src/routes/+page.svelte` -- add click handlers, bind active state to `getActiveVehicleId()` from fillup store, load fill-ups on chip click
- [x] 5.2 Auto-select first vehicle on dashboard load and trigger fill-up fetch
- [x] 5.3 Render fill-up card list below chips -- show date, odometer, fuel amount, cost, station, full-tank badge; use format utilities for display
- [x] 5.4 Add loading state (shimmer/skeleton) while fill-ups are being fetched
- [x] 5.5 Add empty state for vehicle with no fill-ups (message + "Add fill-up" CTA)
- [x] 5.6 Add "Add fill-up" button that opens FillupModal in create mode for the active vehicle
- [x] 5.7 Wire fill-up card tap to open FillupModal in edit mode with the card's fill-up data
- [x] 5.8 Write vitest tests for dashboard fill-up interactions

## 6. Frontend: Global CTA wiring

- [x] 6.1 Update `handleCta()` in `ui/src/routes/+layout.svelte` -- if 1 vehicle: open FillupModal; if >1 vehicle: show vehicle picker then open FillupModal; if 0 vehicles: navigate to add vehicle page
- [x] 6.2 Implement vehicle picker UI (simple list inside the modal or a selection step before the form)

## 7. Quality gate

- [x] 7.1 Run `npm run check --prefix ui` and fix any type errors
- [x] 7.2 Run `npm run lint --prefix ui` and `npm run format:check --prefix ui` and fix issues
- [x] 7.3 Run `cargo fmt -- --check` and `cargo clippy -- -D warnings` and fix issues
- [x] 7.4 Run `cargo test` (full suite including UI tests) and ensure all tests pass
