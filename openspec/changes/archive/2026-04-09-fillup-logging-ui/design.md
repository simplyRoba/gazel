## Context

The fill-up CRUD API is fully implemented (`src/api/fillups.rs`) with endpoints for list, get, create, update, and delete. The frontend has no fill-up awareness: no types, no API client functions, no store, no UI. The dashboard home page (`ui/src/routes/+page.svelte`) renders vehicle filter chips (first one hard-coded active, no click handler) and a "coming soon" placeholder. The global CTA button in the layout navigation (`+layout.svelte`) calls a stub `handleCta()` with a TODO comment.

Existing patterns to follow: the vehicle store (`vehicles.svelte.ts`) uses Svelte 5 module-level `$state` runes with getter functions; the vehicle form (`VehicleForm.svelte`) is a dual-mode create/edit component with `$props()` and client-side validation; the API client (`api.ts`) uses thin typed wrappers around a generic `request<T>()` helper; the modal dialog (`ModalDialog.svelte`) uses native `<dialog>` with confirm/alert modes.

The backend currently treats `odometer` and `cost` as optional. This change tightens them to required and auto-populates `fuel_unit`/`currency` from settings.

## Goals / Non-Goals

**Goals:**
- Let users record, view, edit, and delete fill-ups entirely from the dashboard
- Make vehicle chips interactive so tapping a chip loads that vehicle's fill-ups
- Provide a fill-up form as a modal/bottom sheet accessible from the global CTA and per-vehicle add button
- Tighten the API contract: require `odometer` and `cost`, auto-fill `fuel_unit`/`currency` from settings
- Default `is_full_tank` to `true` (most fill-ups are full tanks)
- Smart UX: detect suspiciously large odometer gaps and prompt the user about missed fill-ups

**Non-Goals:**
- Vehicle detail page (no `/vehicles/:id` route) -- fill-ups live on the dashboard
- Efficiency calculations or statistics (chunk 9)
- Charts or visualizations (chunk 11)
- Pagination of fill-up list (defer until data volume warrants it)
- Drag-to-reorder or swipe-to-delete gestures

## Decisions

### 1. Fill-ups on the dashboard, not a separate page

**Decision**: Show fill-up cards directly on the dashboard under the vehicle filter chips. No `/vehicles/:id` route.

**Rationale**: The dashboard already has vehicle chips. Adding a separate detail page creates redundancy -- the detail page would show the same fill-up list with no additional content until stats/charts arrive in chunks 9-11. Keeping everything on the dashboard means fewer navigation hops.

**Alternative considered**: Dedicated vehicle detail page at `/vehicles/:id`. Rejected because it adds a route with no unique content beyond what the dashboard provides.

### 2. Modal/bottom sheet for the fill-up form

**Decision**: The fill-up form opens as a modal dialog (reusing the native `<dialog>` element pattern from `ModalDialog`), not as a full-page route.

**Rationale**: The user stays in context on the dashboard. On mobile, the modal naturally behaves as a bottom sheet with CSS (full-width, anchored to bottom, slides up). No new route needed, no navigation state to manage.

**Alternative considered**: Full page at `/vehicles/:id/fillups/new`. Rejected because it breaks context and requires route params and back-navigation handling.

### 3. Fill-up store keyed by vehicle ID

**Decision**: A single fillups store module that maintains a map of `vehicleId -> Fillup[]`. The active vehicle's fill-ups are loaded on chip selection and cached.

**Rationale**: Avoids re-fetching when switching between vehicles. Keeps one store module (not one per vehicle). Follows the vehicle store pattern with module-level `$state`.

**Structure**:
```
$state: {
  fillups: Map<number, Fillup[]>   // keyed by vehicleId
  loading: boolean
  error: string | null
  activeVehicleId: number | null
}

Exported getters: getFillups(), getLoading(), getError(), getActiveVehicleId()
Exported actions: loadFillups(vehicleId), createFillup(vehicleId, data), 
                  updateFillup(vehicleId, fillupId, data), deleteFillup(vehicleId, fillupId),
                  setActiveVehicle(vehicleId)
```

**Alternative considered**: Separate store instance per vehicle. Rejected as over-engineered for a personal-use app with few vehicles.

### 4. Global CTA wiring with vehicle picker

**Decision**: The CTA button in the layout calls a callback that either (a) opens the fill-up form modal directly if one vehicle exists, or (b) shows a vehicle picker first if multiple vehicles exist. The vehicle picker is a simple list inside the same modal, not a separate dialog.

**Flow**:
```
CTA tap → getVehicles().length === 1 → open form with that vehicle
CTA tap → getVehicles().length > 1  → show vehicle picker → select → open form
```

The dashboard's per-vehicle "Add fill-up" button skips the picker since the vehicle is already known.

### 5. Required fields: odometer and cost (breaking API change)

**Decision**: Make `odometer` and `cost` required in `CreateFillup` and `UpdateFillup`. Remove `fuel_unit` and `currency` from the request body; read them from the settings table on the backend.

**Rationale**: Without odometer, efficiency calculation (chunk 9) is impossible. Without cost, cost tracking is pointless. The `fuel_unit` and `currency` are app-wide settings, not per-fill-up choices -- the user shouldn't enter them on every form submission.

**Backend change**: In `fillups.rs`, change `odometer: Option<f64>` to `odometer: Option<f64>` (still Option in the struct for deserialization) but validate as required with a new error code `FILLUP_ODOMETER_REQUIRED`. Same pattern for `cost` with `FILLUP_COST_REQUIRED`. Read settings from the database to populate `fuel_unit` and `currency` on insert/update.

**Migration**: No schema migration needed. Validation-only change. Existing test data with null odometer/cost won't be retroactively rejected -- only new creates/updates enforce the requirement.

### 6. Smart missed-fill-up prompt

**Decision**: On the frontend, when the user enters an odometer value, compare the gap (new odometer - last known odometer for that vehicle) against the vehicle's average gap. If the gap is more than 1.75x the average, show an inline prompt suggesting they toggle `is_missed`.

**Rationale**: `is_missed` is a confusing concept for most users. Surfacing it contextually when it's likely relevant is better UX than always showing an unexplained checkbox.

**Implementation**: The last known odometer and average gap are derived from the fill-up list already loaded in the store. No extra API call needed. The prompt is a non-blocking inline message below the odometer field -- not a modal or alert.

### 7. Tap card to edit

**Decision**: Tapping a fill-up card opens the same form modal in edit mode with the fill-up data pre-filled. A delete button is available inside the edit modal.

**Rationale**: Avoids cluttering each card with icon buttons. Single interaction model: tap to open. Consistent with "the modal is the interaction surface."

## Risks / Trade-offs

**[Breaking API change]** → Existing API consumers (if any beyond the UI) will get 422 errors when omitting `odometer` or `cost`. Mitigated by: this is a personal self-hosted app with no external API consumers. Backend tests will be updated in the same change.

**[No pagination]** → If a vehicle has hundreds of fill-ups, the list will be long. Mitigated by: personal-use app unlikely to have more than ~100 fill-ups per vehicle in the near term. Pagination can be added later without breaking the UI structure.

**[Modal form on mobile]** → Modals can be awkward on small screens. Mitigated by: CSS makes the modal full-width and anchored to the bottom on mobile viewports, behaving like a native bottom sheet. The form is short enough (6-7 fields) to fit without scrolling on most devices.

**[Settings dependency for fuel_unit/currency]** → The create/update handler now reads from the settings table, adding a query. Mitigated by: settings is a singleton row, effectively free to read. Could cache in `AppState` later if needed.
