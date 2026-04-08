## Context

gazel has an app shell, CSS component classes, a vehicle CRUD UI with inline delete confirmation, and a vehicle store that sets error strings. flowl has a mature modal dialog and toast notification system that we adopt, adapted for gazel's Svelte 5 runes and angular design language (sharp edges, corner triangles).

## Goals / Non-Goals

**Goals:**

- A reusable `ModalDialog` component for confirmations and alerts across the whole app
- A global notification system (store + toast renderer) for success/error/info feedback
- Replace the inline vehicle delete confirmation with a proper modal
- Wire vehicle store errors to toast notifications

**Non-Goals:**

- No stacking/nested modals
- No i18n for modal labels (hardcoded English, i18n added in chunk 14)
- No notification sound or vibration
- No undo-via-notification (just informational)

## Decisions

### ModalDialog uses native `<dialog>` element

Same as flowl. The native `<dialog>` provides:
- `showModal()` / `close()` for proper focus trapping
- `::backdrop` pseudo-element for overlay
- `cancel` event on Escape key
- Accessibility out of the box (focus management, ARIA)

Controlled by a local `$state` boolean per usage. No global store for modals — each page renders its own `<ModalDialog>` inline.

**Props:**
- `open: boolean` — controls visibility
- `title: string` — dialog heading
- `message: string` — body text
- `mode: 'confirm' | 'alert'` — confirm has Cancel+Confirm, alert has single OK
- `variant: 'danger' | 'warning'` — danger uses error color for confirm button
- `confirmLabel?: string` — custom confirm button text (default: "Confirm")
- `onconfirm / oncancel / onclose` — callbacks

**Styling:** Sharp edges (no border-radius), corner triangle on the dialog, backdrop uses `color-mix()` with `--color-bg`.

### Notification store uses Svelte 5 runes

Unlike flowl's Svelte 4 writables, gazel uses `$state`:

```typescript
let notifications = $state<Notification[]>([]);
export function pushNotification(input: NotificationInput): string { ... }
export function dismissNotification(id: string): void { ... }
export function getVisibleNotifications(): Notification[] { return notifications.slice(0, 3); }
```

**Notification shape:**
- `id: string` — auto-generated
- `message: string` — displayed text
- `title?: string` — optional heading
- `variant: 'success' | 'info' | 'error'` — determines color and auto-dismiss behavior
- `action?: { label: string; onClick: () => void }` — optional action button

### ToastHost positioning and behavior

- **Desktop:** fixed bottom-right, 16px from edges, max-width 360px
- **Mobile (≤768px):** fixed top, full width with 16px margins, below safe-area
- **Auto-dismiss:** success/info after 3500ms, errors persist until X clicked
- **Max visible:** 3, oldest queued
- **Hover pauses** auto-dismiss timer
- **Styling:** Sharp edges, left border accent (4px) colored by variant, corner triangle on each toast

### ToastHost in root layout

Rendered after `<main>` in `+layout.svelte`:

```svelte
<main class="content">{@render children()}</main>
<ToastHost />
```

### Vehicle delete uses ModalDialog

Replace the `deletingId` inline confirmation pattern in settings with:

```svelte
let deleteTarget = $state<Vehicle | null>(null);

<ModalDialog
  open={!!deleteTarget}
  title="Delete vehicle"
  message={`Delete "${deleteTarget?.name}"? This cannot be undone.`}
  mode="confirm"
  variant="danger"
  confirmLabel="Delete"
  onconfirm={() => { deleteVehicle(deleteTarget!.id); deleteTarget = null; }}
  oncancel={() => { deleteTarget = null; }}
/>
```

### Vehicle store pushes notifications on error

Instead of only setting `error` state, the store also calls `pushNotification({ variant: 'error', message })`. The inline error banner in settings can be removed since toast handles it globally.

## Risks / Trade-offs

**No global modal store** → Each page manages its own dialog state. This means two dialogs on the same page need two separate booleans. Acceptable for gazel's simple pages (max 1 dialog per page).

**Runes notification store** → Less tested pattern than flowl's writables, but consistent with the rest of gazel's stores.
