## 1. Notification store

- [x] 1.1 Create `ui/src/lib/stores/notifications.svelte.ts` with `$state` array, `Notification` and `NotificationInput` types, auto-incrementing ID
- [x] 1.2 Implement `pushNotification(input)` — adds notification with generated ID, returns ID
- [x] 1.3 Implement `dismissNotification(id)` — removes by ID
- [x] 1.4 Implement `clearNotifications()` — clears all
- [x] 1.5 Implement `getVisibleNotifications()` — returns first 3

## 2. ToastHost component

- [x] 2.1 Create `ui/src/lib/components/ToastHost.svelte` rendering visible notifications as fixed-position toasts
- [x] 2.2 Position: bottom-right on desktop (>768px), top on mobile (≤768px) below safe-area
- [x] 2.3 Left border accent colored by variant (success=green, error=red, info=blue)
- [x] 2.4 Close button (X) on each toast calling `dismissNotification()`
- [x] 2.5 Auto-dismiss: success/info after 3500ms, errors persist
- [x] 2.6 Hover pauses auto-dismiss timer, resume on mouse leave
- [x] 2.7 Optional action button on toast (label + onClick)
- [x] 2.8 Sharp styling with corner triangle on each toast

## 3. ModalDialog component

- [x] 3.1 Create `ui/src/lib/components/ModalDialog.svelte` using native `<dialog>` element
- [x] 3.2 Props: `open`, `title`, `message`, `mode` (confirm/alert), `variant` (danger/warning), `confirmLabel`, `onconfirm`, `oncancel`, `onclose`
- [x] 3.3 Confirm mode: Cancel + Confirm buttons, backdrop click calls `oncancel`
- [x] 3.4 Alert mode: single OK button, Escape calls `onclose`
- [x] 3.5 Danger variant: confirm/OK button uses `--color-error` background
- [x] 3.6 Sharp styling: no border-radius, corner triangle on the dialog, backdrop overlay
- [x] 3.7 `$effect` to sync `open` prop with `dialogEl.showModal()` / `dialogEl.close()`

## 4. Layout integration

- [x] 4.1 Import and render `<ToastHost />` in `ui/src/routes/+layout.svelte` after `<main>`

## 5. Replace inline delete with ModalDialog

- [x] 5.1 Update `ui/src/routes/settings/+page.svelte`: replace `deletingId` pattern with `deleteTarget` Vehicle state + `<ModalDialog>`
- [x] 5.2 Remove inline delete confirmation row markup and CSS
- [x] 5.3 Add `<ModalDialog>` at the bottom of the template with danger variant

## 6. Wire vehicle store to notifications

- [x] 6.1 Import `pushNotification` in `ui/src/lib/stores/vehicles.svelte.ts`
- [x] 6.2 On every catch block, call `pushNotification({ variant: 'error', message })` in addition to setting error state

## 7. Notification store tests

- [x] 7.1 Create `ui/src/lib/stores/notifications.test.ts`
- [x] 7.2 Test push adds notification with generated ID
- [x] 7.3 Test dismiss removes by ID
- [x] 7.4 Test visible returns max 3
- [x] 7.5 Test clear removes all

## 8. Verification

- [x] 8.1 Run `npm run format:check --prefix ui` and fix formatting
- [x] 8.2 Run `npm run lint --prefix ui` and fix lint errors
- [x] 8.3 Run `npm run check --prefix ui` and fix type errors
- [x] 8.4 Run `npm run test --prefix ui` and verify all tests pass
- [x] 8.5 Run `cargo test` to verify backend still passes
