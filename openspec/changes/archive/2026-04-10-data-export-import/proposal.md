## Why

Users have no way to back up their data or transfer it between gazel instances. A single corrupted SQLite file means total data loss. Export and import endpoints let users safeguard their fuel-tracking history and move data across devices or reinstalls.

## What Changes

- Add `GET /api/export` endpoint that serializes all vehicles, fill-ups, and settings into a versioned JSON document.
- Add `GET /api/vehicles/:id/export` endpoint for single-vehicle export (vehicle + its fill-ups).
- Add `POST /api/import` endpoint that accepts the same JSON format, validates it, and writes data to the database inside a transaction.
- Include an `export_version` field (semver, matching `CARGO_PKG_VERSION` major.minor) so the import endpoint can reject incompatible formats.
- Import supports two modes: **replace** (clear existing data, insert imported data) and **merge** (skip duplicates by matching vehicle name + fill-up date/odometer, insert new records).
- Import validates all records using the same rules as the CRUD endpoints before committing.
- Add a download button on the settings page and per-vehicle export action in the UI.
- Add an import section on the settings page with file picker, preview summary (N vehicles, M fill-ups), and confirm/cancel.
- Version compatibility gate on import: major.minor of the export must match the running server; patch differences are allowed.

## Capabilities

### New Capabilities
- `data-export`: Backend export endpoints, JSON schema with versioning, and UI download controls.
- `data-import`: Backend import endpoint with validation, version gating, replace/merge modes, preview, and UI upload flow.

### Modified Capabilities
- `ui/settings`: Settings page gains export download button and import upload section.

## Impact

- **Backend**: New handler module `src/api/export.rs` and `src/api/import.rs`, new routes in `src/api/mod.rs`.
- **Frontend**: New API client functions in `api.ts`, new components for export button and import flow, settings page additions.
- **Tests**: New integration tests in `tests/export.rs` and `tests/import.rs`; new vitest coverage for UI import/export flows.
- **No new dependencies**: serde already handles JSON serialization; no ZIP needed since gazel has no binary assets.
- **No migrations**: Export/import operates on existing tables.
