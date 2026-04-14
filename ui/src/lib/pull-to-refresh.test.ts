import { describe, expect, it, vi } from "vitest";
import {
  PULL_TO_REFRESH_THRESHOLD,
  MAX_PULL_TO_REFRESH_OFFSET,
  PULL_TO_REFRESH_RELOAD_DELAY_MS,
  isStandalonePwaSession,
  isTouchCapableDevice,
  isPullToRefreshRoute,
  hasBlockingPullToRefreshOverlay,
  canStartPullToRefresh,
  getPullIndicatorState,
  calculatePullOffset,
  calculateContentOffset,
  shouldTriggerPullToRefresh,
  schedulePullToRefreshReload,
} from "./pull-to-refresh";

// ── Constants ──────────────────────────────────────────────

describe("constants", () => {
  it("exports correct threshold", () => {
    expect(PULL_TO_REFRESH_THRESHOLD).toBe(128);
  });

  it("exports correct max offset", () => {
    expect(MAX_PULL_TO_REFRESH_OFFSET).toBe(140);
  });

  it("exports correct reload delay", () => {
    expect(PULL_TO_REFRESH_RELOAD_DELAY_MS).toBe(120);
  });
});

// ── isStandalonePwaSession ─────────────────────────────────

describe("isStandalonePwaSession", () => {
  it("returns true when display-mode standalone matches", () => {
    const win = {
      matchMedia: (q: string) => ({
        matches: q === "(display-mode: standalone)",
      }),
      navigator: {},
    };
    expect(isStandalonePwaSession(win)).toBe(true);
  });

  it("returns true when navigator.standalone is true (iOS)", () => {
    const win = {
      matchMedia: () => ({ matches: false }),
      navigator: { standalone: true },
    };
    expect(isStandalonePwaSession(win)).toBe(true);
  });

  it("returns false in a regular browser tab", () => {
    const win = {
      matchMedia: () => ({ matches: false }),
      navigator: { standalone: false },
    };
    expect(isStandalonePwaSession(win)).toBe(false);
  });

  it("returns false when navigator.standalone is undefined", () => {
    const win = {
      matchMedia: () => ({ matches: false }),
      navigator: {},
    };
    expect(isStandalonePwaSession(win)).toBe(false);
  });
});

// ── isTouchCapableDevice ───────────────────────────────────

describe("isTouchCapableDevice", () => {
  it("returns true when pointer: coarse matches", () => {
    const win = {
      matchMedia: (q: string) => ({ matches: q === "(pointer: coarse)" }),
      navigator: { maxTouchPoints: 0 },
    };
    expect(isTouchCapableDevice(win)).toBe(true);
  });

  it("returns true when maxTouchPoints > 0", () => {
    const win = {
      matchMedia: () => ({ matches: false }),
      navigator: { maxTouchPoints: 5 },
    };
    expect(isTouchCapableDevice(win)).toBe(true);
  });

  it("returns true when ontouchstart is in window", () => {
    const win = {
      matchMedia: () => ({ matches: false }),
      navigator: { maxTouchPoints: 0 },
      ontouchstart: null,
    };
    expect(isTouchCapableDevice(win)).toBe(true);
  });

  it("returns false when no touch support detected", () => {
    const win = {
      matchMedia: () => ({ matches: false }),
      navigator: { maxTouchPoints: 0 },
    };
    expect(isTouchCapableDevice(win)).toBe(false);
  });
});

// ── isPullToRefreshRoute ───────────────────────────────────

describe("isPullToRefreshRoute", () => {
  it("allows dashboard route", () => {
    expect(isPullToRefreshRoute("/")).toBe(true);
  });

  it("allows settings route", () => {
    expect(isPullToRefreshRoute("/settings")).toBe(true);
  });

  it("blocks new vehicle form", () => {
    expect(isPullToRefreshRoute("/settings/vehicles/new")).toBe(false);
  });

  it("blocks edit vehicle form", () => {
    expect(isPullToRefreshRoute("/settings/vehicles/1/edit")).toBe(false);
  });

  it("blocks unknown routes", () => {
    expect(isPullToRefreshRoute("/unknown")).toBe(false);
  });

  it("blocks empty string", () => {
    expect(isPullToRefreshRoute("")).toBe(false);
  });
});

// ── hasBlockingPullToRefreshOverlay ─────────────────────────

describe("hasBlockingPullToRefreshOverlay", () => {
  it("returns true when a dialog[open] exists", () => {
    const doc = {
      querySelector: (s: string) =>
        s === "dialog[open]" ? document.createElement("dialog") : null,
    };
    expect(hasBlockingPullToRefreshOverlay(doc)).toBe(true);
  });

  it("returns false when no dialog[open] exists", () => {
    const doc = {
      querySelector: () => null,
    };
    expect(hasBlockingPullToRefreshOverlay(doc)).toBe(false);
  });
});

// ── canStartPullToRefresh ──────────────────────────────────

describe("canStartPullToRefresh", () => {
  const allMet = {
    standalone: true,
    touchCapable: true,
    pathname: "/",
    scrollTop: 0,
    overlayOpen: false,
  };

  it("returns true when all conditions met", () => {
    expect(canStartPullToRefresh(allMet)).toBe(true);
  });

  it("returns false when not standalone", () => {
    expect(canStartPullToRefresh({ ...allMet, standalone: false })).toBe(false);
  });

  it("returns false when not touch capable", () => {
    expect(canStartPullToRefresh({ ...allMet, touchCapable: false })).toBe(
      false,
    );
  });

  it("returns false when route is not allowed", () => {
    expect(
      canStartPullToRefresh({ ...allMet, pathname: "/settings/vehicles/new" }),
    ).toBe(false);
  });

  it("returns false when scrolled down", () => {
    expect(canStartPullToRefresh({ ...allMet, scrollTop: 100 })).toBe(false);
  });

  it("returns false when overlay is open", () => {
    expect(canStartPullToRefresh({ ...allMet, overlayOpen: true })).toBe(false);
  });

  it("returns true when scrollTop is exactly 0", () => {
    expect(canStartPullToRefresh({ ...allMet, scrollTop: 0 })).toBe(true);
  });

  it("returns true when scrollTop is negative (bounce)", () => {
    expect(canStartPullToRefresh({ ...allMet, scrollTop: -5 })).toBe(true);
  });
});

// ── getPullIndicatorState ──────────────────────────────────

describe("getPullIndicatorState", () => {
  it("returns idle for zero distance", () => {
    expect(getPullIndicatorState(0)).toBe("idle");
  });

  it("returns idle for negative distance", () => {
    expect(getPullIndicatorState(-50)).toBe("idle");
  });

  it("returns pulling for distance below threshold", () => {
    expect(getPullIndicatorState(64)).toBe("pulling");
  });

  it("returns pulling just below threshold", () => {
    expect(getPullIndicatorState(127)).toBe("pulling");
  });

  it("returns release at exact threshold", () => {
    expect(getPullIndicatorState(128)).toBe("release");
  });

  it("returns release above threshold", () => {
    expect(getPullIndicatorState(200)).toBe("release");
  });
});

// ── calculatePullOffset ────────────────────────────────────

describe("calculatePullOffset", () => {
  it("returns 0 for zero distance", () => {
    expect(calculatePullOffset(0)).toBe(0);
  });

  it("returns 0 for negative distance", () => {
    expect(calculatePullOffset(-10)).toBe(0);
  });

  it("returns distance unchanged in linear range", () => {
    expect(calculatePullOffset(64)).toBe(64);
  });

  it("returns threshold at exact threshold", () => {
    expect(calculatePullOffset(128)).toBe(128);
  });

  it("returns value between threshold and max above threshold", () => {
    const offset = calculatePullOffset(200);
    expect(offset).toBeGreaterThan(PULL_TO_REFRESH_THRESHOLD);
    expect(offset).toBeLessThan(MAX_PULL_TO_REFRESH_OFFSET);
  });

  it("approaches max offset for very large distances", () => {
    const offset = calculatePullOffset(10000);
    expect(offset).toBeCloseTo(MAX_PULL_TO_REFRESH_OFFSET, 1);
  });

  it("increases monotonically past threshold", () => {
    const a = calculatePullOffset(150);
    const b = calculatePullOffset(200);
    const c = calculatePullOffset(300);
    expect(b).toBeGreaterThan(a);
    expect(c).toBeGreaterThan(b);
  });
});

// ── calculateContentOffset ─────────────────────────────────

describe("calculateContentOffset", () => {
  it("returns 0 for zero distance", () => {
    expect(calculateContentOffset(0)).toBe(0);
  });

  it("returns 0 for negative distance", () => {
    expect(calculateContentOffset(-10)).toBe(0);
  });

  it("returns distance unchanged in linear range", () => {
    expect(calculateContentOffset(64)).toBe(64);
  });

  it("returns threshold at exact threshold", () => {
    expect(calculateContentOffset(128)).toBe(128);
  });

  it("has wider elastic range than pull offset", () => {
    const pullOffset = calculatePullOffset(200);
    const contentOffset = calculateContentOffset(200);
    // Content should be larger because it has a wider elastic range (100 vs 12)
    expect(contentOffset).toBeGreaterThan(pullOffset);
  });

  it("approaches threshold + 100 for very large distances", () => {
    const offset = calculateContentOffset(10000);
    expect(offset).toBeCloseTo(PULL_TO_REFRESH_THRESHOLD + 100, 1);
  });

  it("increases monotonically past threshold", () => {
    const a = calculateContentOffset(150);
    const b = calculateContentOffset(200);
    const c = calculateContentOffset(300);
    expect(b).toBeGreaterThan(a);
    expect(c).toBeGreaterThan(b);
  });
});

// ── shouldTriggerPullToRefresh ──────────────────────────────

describe("shouldTriggerPullToRefresh", () => {
  it("returns false below threshold", () => {
    expect(shouldTriggerPullToRefresh(100)).toBe(false);
  });

  it("returns true at exact threshold", () => {
    expect(shouldTriggerPullToRefresh(128)).toBe(true);
  });

  it("returns true above threshold", () => {
    expect(shouldTriggerPullToRefresh(200)).toBe(true);
  });

  it("returns false for zero", () => {
    expect(shouldTriggerPullToRefresh(0)).toBe(false);
  });

  it("returns false for negative", () => {
    expect(shouldTriggerPullToRefresh(-10)).toBe(false);
  });
});

// ── schedulePullToRefreshReload ─────────────────────────────

describe("schedulePullToRefreshReload", () => {
  it("calls reload after the specified delay", () => {
    vi.useFakeTimers();
    const reloadFn = vi.fn();
    const win = { setTimeout: globalThis.setTimeout.bind(globalThis) };

    schedulePullToRefreshReload(win, reloadFn, 120);

    expect(reloadFn).not.toHaveBeenCalled();
    vi.advanceTimersByTime(120);
    expect(reloadFn).toHaveBeenCalledOnce();

    vi.useRealTimers();
  });

  it("returns a timeout ID", () => {
    vi.useFakeTimers();
    const win = { setTimeout: globalThis.setTimeout.bind(globalThis) };
    const id = schedulePullToRefreshReload(win, vi.fn(), 100);
    expect(id).toBeDefined();
    vi.useRealTimers();
  });

  it("uses default delay when not specified", () => {
    vi.useFakeTimers();
    const reloadFn = vi.fn();
    const win = { setTimeout: globalThis.setTimeout.bind(globalThis) };

    schedulePullToRefreshReload(win, reloadFn);

    vi.advanceTimersByTime(119);
    expect(reloadFn).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(reloadFn).toHaveBeenCalledOnce();

    vi.useRealTimers();
  });
});
