## Context

gazel stores all data in a single SQLite file. Users currently have no application-level way to back up or restore their vehicles, fill-ups, and settings. The flowl reference project (same stack) implements a backup/restore system using versioned JSON inside a ZIP archive with major.minor version gating on import.

gazel has no binary assets (no photos), so a plain JSON document is sufficient -- no ZIP needed. The existing CRUD endpoints already validate all inputs, and those validation functions can be reused during import.

## Goals / Non-Goals

**Goals:**
- Provide full-data and per-vehicle JSON export via API endpoints
- Provide a JSON import endpoint with validation, version compatibility checks, and two modes (replace/merge)
- Surface export and import controls in the settings UI
- Ensure import is atomic (transaction-based, rollback on failure)
- Include a version field in the export so future gazel versions can detect incompatible formats

**Non-Goals:**
- CSV or other export formats (JSON only for v1)
- Scheduled/automatic backups
- Import from third-party fuel tracking apps
- Streaming or chunked export for very large datasets (SQLite in-process makes this unnecessary at gazel's scale)
- Export/import of settings (settings are instance-specific; only vehicles and fill-ups are portable)

## Decisions

### 1. Plain JSON, no ZIP

**Decision**: Export produces a single JSON document; import accepts the same.

**Rationale**: gazel has no binary assets (photos, files). A plain JSON response is simpler to produce, consume, and inspect than a ZIP. It also avoids adding a compression dependency.

**Alternative considered**: ZIP like flowl -- rejected because flowl needs it for photo files; gazel does not.

### 2. Version field uses `CARGO_PKG_VERSION`

**Decision**: The export JSON includes a `version` field set to the crate version from `Cargo.toml` at compile time. Import checks that the major.minor of the archive matches the running server; patch differences are allowed.

**Rationale**: This mirrors flowl's approach. It's zero-maintenance (no separate format version to track) and sufficient for a single-user app. If the data shape changes, it will coincide with a minor version bump.

**Alternative considered**: A separate integer `schema_version` -- simpler but decoupled from the release cadence, adding another thing to remember to bump.

### 3. Two import modes: replace and merge

**Decision**: Import accepts a `mode` query parameter (`replace` or `merge`, default `replace`).

- **Replace**: DELETE all vehicles + fill-ups, INSERT imported data. Simple, total overwrite.
- **Merge**: For each imported vehicle, match by `name` (case-insensitive). If a match exists, update its fields and merge fill-ups. Fill-ups are matched by `(vehicle_id, date, odometer)` tuple -- if a match exists, skip; otherwise insert. New vehicles are inserted.

**Rationale**: Replace covers the backup/restore use case. Merge covers the "transfer data from another instance" case without losing existing data. The TODO explicitly calls for merge vs. replace semantics.

**Alternative considered**: Only replace (like flowl) -- simpler but loses existing data on import, which is destructive for a merge/transfer workflow.

### 4. Preview via dry-run, not a separate endpoint

**Decision**: `POST /api/import?preview=true` validates and returns a summary (vehicle count, fill-up count, conflicts) without writing to the database. `POST /api/import` (without `preview=true`) performs the actual import.

**Rationale**: Reuses the same endpoint and validation logic. The UI calls preview first, shows results, then calls the real import on confirmation. Avoids server-side state or a two-step session.

### 5. Export excludes settings, includes only vehicles and fill-ups

**Decision**: The export JSON contains `version`, `exported_at`, `vehicles[]`, and each vehicle embeds its `fillups[]`. Settings are excluded.

**Rationale**: Settings are instance-specific (unit system, theme, locale). Importing someone else's settings would be confusing. Vehicles and fill-ups are the portable data.

### 6. Handler modules follow existing pattern

**Decision**: Add `src/api/export.rs` and `src/api/import.rs` with routes registered in `src/api/mod.rs`. Reuse `AppState`, `ApiError`, and existing validation helpers.

**Rationale**: Consistent with `vehicles.rs`, `fillups.rs`, `settings.rs` pattern.

### 7. UI placement

**Decision**: Export/import controls go on the settings page in a new "Data" section. Per-vehicle export is available via a menu/action on the vehicle cards or vehicle detail context.

**Rationale**: Settings page is the natural home for administrative actions. Per-vehicle export is a convenience shortcut.

## Risks / Trade-offs

- **[Large import body]** A user with thousands of fill-ups could send a large JSON body. Mitigation: apply a body size limit (e.g., 10 MB) on the import route, consistent with flowl's approach.
- **[Replace mode is destructive]** Replace deletes all existing data. Mitigation: the UI shows a preview first and requires explicit confirmation. The preview clearly states "this will replace all existing data."
- **[Merge matching heuristic]** Matching vehicles by name and fill-ups by (date, odometer) is imperfect -- renamed vehicles won't match. Mitigation: acceptable for v1; the preview shows what will be created vs. skipped so users can verify.
- **[No migration between versions]** If the export format changes in a future minor version, old exports are rejected. Mitigation: acceptable for a personal-use app; users can re-export from the same version. Could add migration logic later if needed.
- **[Atomicity on merge]** Merge mode within a single transaction could be slow for very large datasets. Mitigation: gazel targets personal use (tens of vehicles, thousands of fill-ups at most), so performance is not a concern.
