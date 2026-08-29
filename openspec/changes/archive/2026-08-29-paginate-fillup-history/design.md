## Context

`GET /api/vehicles/{vehicle_id}/fillups` currently returns one unbounded array ordered by `(date DESC, id DESC)`. The frontend caches and renders that array. This change only bounds fill-up-card retrieval; `/stats/history` remains unchanged.

## Goals / Non-Goals

**Goals:**

- Return at most 25 fill-ups by default.
- Append older pages without duplicates or stale responses.
- Support loading, retry, and exhausted behavior in the existing nested scroll layout.

**Non-Goals:**

- Chart/statistics optimization.
- Server-side form-assistance calculations.
- Card virtualization.
- Fill-up validation changes.

## Decisions

### One paginated response contract

The list endpoint always returns:

```text
{ items: Fillup[], next_cursor: string | null }
```

It accepts optional `limit` and `cursor`. `limit` defaults to 25 and must be an integer from 1 through 100. `cursor` is an opaque string returned by the server and passed back unchanged by clients. Internally it contains the final item's date and ID.

Continuation uses `(date < cursor.date OR (date = cursor.date AND id < cursor.id))`, ordered by `(date DESC, id DESC)`. The server requests `limit + 1` rows to detect another page. Invalid limits return `FILLUP_INVALID_PAGE_LIMIT`; malformed cursors return `FILLUP_INVALID_CURSOR`, both as `400 Bad Request` JSON errors.

Keyset pagination is used instead of offsets so inserts ahead of the cursor do not shift later pages and deep pages do not discard preceding rows.

### Per-vehicle page chains with generations

Each cached vehicle keeps loaded items, next cursor, initial/loading-more flags, continuation error, and a generation number. Starting a fresh initial load increments the generation. A response is applied only if its captured generation still matches, preventing stale continuation responses after vehicle changes, refreshes, or mutations.

Continuation is ignored while already loading, after failure until explicit retry, or when no cursor remains. Appended items are de-duplicated by ID.

Successful create, update, or delete performs the existing local cache update, then starts a fresh first-page load so cursor state is valid. Form helpers can use currently loaded recent entries but never trigger continuation themselves.

### Endless-scroll sentinel

An `IntersectionObserver` watches a sentinel after the cards. On desktop its root is the `.fillups-column` scroll container; on mobile it uses the viewport. A forward root margin starts loading shortly before the end. The observer is disconnected and recreated when the component, active vehicle, or scroll root changes.

Already-loaded cards remain visible during continuation. A failed request replaces automatic loading with a retry button. A null cursor removes the sentinel and prevents further requests.

## Risks / Trade-offs

- **Direct API break** → Ship backend and embedded frontend together and document the envelope.
- **Concurrent external edits can move records across cursors** → De-duplicate appended IDs; a refresh starts a new chain.
- **Form heuristics use only recent loaded history** → This is intentional; pagination must not secretly fetch all pages.
- **Stats history remains unbounded** → Optimize it in a separate change rather than expanding this one.
- **DOM grows after prolonged scrolling** → Initial work is bounded; virtualization is deferred.

## Migration Plan

Update backend and bundled frontend in one release. Rollback requires only the previous binary because there is no schema change.

## Open Questions

None.
