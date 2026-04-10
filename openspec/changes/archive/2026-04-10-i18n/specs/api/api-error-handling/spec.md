## ADDED Requirements

### Requirement: Error codes as stable i18n keys

The `code` field in API error responses SHALL serve as the stable identifier for frontend i18n mapping. The `message` field SHALL remain an English-language fallback only. The frontend SHALL use `resolveError()` to map codes to localized strings rather than displaying `message` directly.

#### Scenario: Error code is the translation key

- **WHEN** the backend returns `{ "code": "VEHICLE_NOT_FOUND", "message": "Vehicle not found." }`
- **THEN** the frontend SHALL look up translation key `error.VEHICLE_NOT_FOUND` for the active locale
- **AND** SHALL fall back to the `message` field only if the translation key is missing

#### Scenario: All error codes have translation entries

- **WHEN** the translation system is configured
- **THEN** every error code used by `default_message()` in the backend SHALL have a corresponding `error.<CODE>` key in all translation files
