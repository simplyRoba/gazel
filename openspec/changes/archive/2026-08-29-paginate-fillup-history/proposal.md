## Why

The dashboard currently downloads every fill-up for the selected vehicle before rendering the card list. The card list should instead load a bounded first page and fetch older entries only as the user scrolls.

## What Changes

- Replace the fill-up list response with a cursor-paginated page envelope.
- Load 25 fill-ups initially and append older pages through endless scrolling.
- Keep loaded cards visible while loading more and provide explicit retry after a continuation failure.
- Prevent stale requests from a previous page chain or vehicle from changing current state.
- Reload the first page after fill-up mutations.
- Do not make form helpers trigger loading of the complete history.
- Leave statistics and chart-history endpoints unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `api-fillup-crud`: Make vehicle fill-up listing cursor-paginated.
- `ui-fillup-ui`: Load and render fill-up history incrementally.

## Impact

- Fill-up list API response is **BREAKING** for direct clients.
- Backend fill-up handler/tests and frontend API/store/dashboard/tests are affected.
- Add a direct `base64` dependency for URL-safe opaque cursor encoding; no database migration is planned.
