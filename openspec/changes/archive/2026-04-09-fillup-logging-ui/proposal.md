## Why

The fill-up CRUD API is complete but has no frontend. Users cannot record, view, edit, or delete fill-ups. The dashboard home page is a placeholder showing vehicle chips with no content. The global CTA button in the navigation is a stub. Without a fill-up logging UI the app delivers no value beyond vehicle management.

## What Changes

- Add fill-up TypeScript types and API client functions to `ui/src/lib/api.ts`
- Add a fill-up store with reactive state per vehicle (load, create, update, delete)
- Build the dashboard home page: make vehicle chips interactive, show fill-up card list for the selected vehicle, empty state per vehicle
- Build a fill-up form as a modal/bottom sheet (not a full page) for create and edit, opened by tapping a fill-up card (edit) or the add button (create)
- Wire the global CTA button (diamond on mobile, sidebar button on desktop) to open the fill-up form modal; if multiple vehicles exist, show a vehicle picker first; if only one vehicle, go straight to the form
- Delete fill-up with confirmation dialog (reuse existing `ModalDialog`)
- **BREAKING**: Make `odometer` and `cost` required fields in the fill-up create/update API (currently optional); auto-populate `fuel_unit` and `currency` from user settings instead of accepting them in the request
- Change `is_full_tank` default from `false` to `true`
- Add smart prompt: when odometer gap significantly exceeds the vehicle's historical average, suggest setting `is_missed` flag
- Add vitest tests for store, form, and dashboard interactions

## Capabilities

### New Capabilities
- `fillup-ui`: Fill-up logging frontend -- dashboard fill-up list, fill-up form modal, fill-up store, and global CTA wiring

### Modified Capabilities
- `api/fillup-crud`: Make `odometer` and `cost` required on create/update; auto-fill `fuel_unit` and `currency` from settings; change `is_full_tank` default to `true`

## Impact

- **Backend**: `src/api/fillups.rs` -- change `CreateFillup`/`UpdateFillup` structs to require `odometer` and `cost`; read settings to auto-populate `fuel_unit`/`currency`; update defaults. Existing integration tests in `tests/fillups.rs` need updating for new required fields.
- **Frontend**: `ui/src/lib/api.ts` (new types + functions), new `ui/src/lib/stores/fillups.svelte.ts`, new `ui/src/lib/components/FillupForm.svelte`, updated `ui/src/routes/+page.svelte` (dashboard), updated `ui/src/routes/+layout.svelte` (CTA wiring)
- **No new dependencies** -- reuses existing ModalDialog, design tokens, and format utilities
- **No migrations** -- schema is unchanged; only validation rules tighten
