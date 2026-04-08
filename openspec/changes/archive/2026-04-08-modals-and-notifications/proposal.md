## Why

The app currently uses inline confirmation for vehicle deletion, which is inconsistent and doesn't scale. As more destructive actions are added (delete fill-ups, clear data), each page would need its own inline confirmation logic. Similarly, there's no way to show success/error feedback after async operations (API errors are shown as inline text that's easy to miss).

flowl solves this with two framework-level components: a `ModalDialog` for confirmations and a `ToastHost` for notifications. Both are proven patterns that we adopt here.

## What Changes

- **ModalDialog component**: Native `<dialog>` element with two modes (confirm/alert), two variants (danger/warning), backdrop click to dismiss. Rendered inline per page, controlled by local `$state` boolean. Uses the design system's sharp/angular styling with corner triangle.
- **Notification store + ToastHost**: Global notification store with `pushNotification()` / `dismissNotification()`. Toast renderer in the root layout. Auto-dismiss success/info after 3500ms, errors persist until manually closed. Max 3 visible, queued. Positioned bottom-right on desktop, top on mobile.
- **Replace inline delete confirmation**: The settings page's vehicle delete switches from inline row transformation to `ModalDialog` with danger variant.
- **Wire error toasts**: Vehicle store errors push notifications instead of only setting error state.

## Capabilities

### New Capabilities

- `modal-dialog`: Reusable confirmation and alert dialog component using native `<dialog>` element
- `notifications`: Global notification store and toast renderer for success, info, and error feedback

### Modified Capabilities

- `vehicle-forms`: Replace inline delete confirmation with ModalDialog, wire error notifications

## Impact

- **UI files created**: `ui/src/lib/components/ModalDialog.svelte`, `ui/src/lib/components/ToastHost.svelte`, `ui/src/lib/stores/notifications.svelte.ts`
- **UI files modified**: `ui/src/routes/+layout.svelte` (add ToastHost), `ui/src/routes/settings/+page.svelte` (replace inline delete with ModalDialog), `ui/src/lib/stores/vehicles.svelte.ts` (push notifications on error)
- **Backend**: No changes
- **Dependencies**: No new packages
