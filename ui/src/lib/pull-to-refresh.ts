// Pull-to-refresh pure logic module.
// All functions are side-effect-free and fully testable.
// No DOM, framework, or browser API dependencies beyond the
// Window/Document objects passed as arguments.

// ── Constants ──────────────────────────────────────────────

/** Pixel distance the user must pull before the gesture triggers a refresh. */
export const PULL_TO_REFRESH_THRESHOLD = 128;

/** Maximum visual offset for the pull indicator (elastic cap). */
export const MAX_PULL_TO_REFRESH_OFFSET = 140;

/** Milliseconds to wait before calling reload, so the spinner is visible. */
export const PULL_TO_REFRESH_RELOAD_DELAY_MS = 120;

/** Internal elastic range for content displacement past threshold. */
const CONTENT_ELASTIC_RANGE = 100;

// ── Types ──────────────────────────────────────────────────

export type PullIndicatorState = "idle" | "pulling" | "release" | "refreshing";

export interface PullToRefreshEligibility {
  standalone: boolean;
  touchCapable: boolean;
  pathname: string;
  scrollTop: number;
  overlayOpen: boolean;
}

// ── Capability detection ───────────────────────────────────

interface StandaloneWindow {
  matchMedia: (query: string) => { matches: boolean };
  navigator: { standalone?: boolean };
}

/** True when the app is running as an installed standalone PWA. */
export function isStandalonePwaSession(win: StandaloneWindow): boolean {
  if (win.matchMedia("(display-mode: standalone)").matches) return true;
  if (win.navigator.standalone === true) return true;
  return false;
}

interface TouchWindow {
  matchMedia: (query: string) => { matches: boolean };
  navigator: { maxTouchPoints: number };
  ontouchstart?: unknown;
}

/** True when the device supports touch input. */
export function isTouchCapableDevice(win: TouchWindow): boolean {
  if (win.matchMedia("(pointer: coarse)").matches) return true;
  if (win.navigator.maxTouchPoints > 0) return true;
  if ("ontouchstart" in win) return true;
  return false;
}

// ── Route eligibility ──────────────────────────────────────

const ALLOWED_ROUTES: string[] = ["/", "/settings"];

/** True when pull-to-refresh is allowed on the given route. */
export function isPullToRefreshRoute(pathname: string): boolean {
  return ALLOWED_ROUTES.includes(pathname);
}

// ── Overlay blocking ───────────────────────────────────────

interface MinimalDocument {
  querySelector: (selector: string) => Element | null;
}

/** True when a modal dialog is open and should block pull-to-refresh. */
export function hasBlockingPullToRefreshOverlay(doc: MinimalDocument): boolean {
  return doc.querySelector("dialog[open]") !== null;
}

// ── Eligibility gate ───────────────────────────────────────

/** True only when ALL preconditions for starting a pull gesture are met. */
export function canStartPullToRefresh(
  params: PullToRefreshEligibility,
): boolean {
  return (
    params.standalone &&
    params.touchCapable &&
    isPullToRefreshRoute(params.pathname) &&
    params.scrollTop <= 0 &&
    !params.overlayOpen
  );
}

// ── State derivation ───────────────────────────────────────

/** Derive the indicator state from the raw pull distance. */
export function getPullIndicatorState(
  rawPullDistance: number,
): PullIndicatorState {
  if (rawPullDistance <= 0) return "idle";
  if (rawPullDistance < PULL_TO_REFRESH_THRESHOLD) return "pulling";
  return "release";
}

// ── Elastic offset calculations ────────────────────────────

/**
 * Calculate the indicator's visual offset with exponential decay past threshold.
 * Linear 1:1 up to THRESHOLD, then asymptotically approaches MAX_OFFSET.
 */
export function calculatePullOffset(distance: number): number {
  if (distance <= 0) return 0;
  if (distance <= PULL_TO_REFRESH_THRESHOLD) return distance;

  const elasticRange = MAX_PULL_TO_REFRESH_OFFSET - PULL_TO_REFRESH_THRESHOLD;
  const overThreshold = distance - PULL_TO_REFRESH_THRESHOLD;
  return (
    PULL_TO_REFRESH_THRESHOLD +
    elasticRange * (1 - Math.exp(-overThreshold / elasticRange))
  );
}

/**
 * Calculate the content's vertical displacement with a wider elastic range.
 * Linear 1:1 up to THRESHOLD, then asymptotically approaches THRESHOLD + 100.
 */
export function calculateContentOffset(distance: number): number {
  if (distance <= 0) return 0;
  if (distance <= PULL_TO_REFRESH_THRESHOLD) return distance;

  const overThreshold = distance - PULL_TO_REFRESH_THRESHOLD;
  return (
    PULL_TO_REFRESH_THRESHOLD +
    CONTENT_ELASTIC_RANGE *
      (1 - Math.exp(-overThreshold / CONTENT_ELASTIC_RANGE))
  );
}

// ── Trigger check ──────────────────────────────────────────

/** True when the pull distance has crossed the threshold. */
export function shouldTriggerPullToRefresh(rawPullDistance: number): boolean {
  return rawPullDistance >= PULL_TO_REFRESH_THRESHOLD;
}

// ── Reload scheduling ──────────────────────────────────────

/**
 * Schedule a page reload after a short delay so the refreshing spinner
 * is visible before the page blanks.
 * Returns the timeout ID for cancellation.
 */
export function schedulePullToRefreshReload(
  win: { setTimeout: (fn: () => void, ms: number) => number },
  reloadFn: () => void,
  delayMs: number = PULL_TO_REFRESH_RELOAD_DELAY_MS,
): number {
  return win.setTimeout(reloadFn, delayMs);
}
