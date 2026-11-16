import { describe, expect, it } from "vitest";
import { monthlyWindow } from "./monthly-window.js";

const months = (count: number) =>
  Array.from({ length: count }, (_, i) => ({
    period: `${2000 + Math.floor(i / 12)}-${String((i % 12) + 1).padStart(2, "0")}`,
    value: i,
  }));

describe("monthly windows", () => {
  it.each([0, 1, 12, 13, 25, 160])(
    "covers %i months and clamps both ends",
    (count) => {
      const points = months(count);
      let view = monthlyWindow(points, 1, null);
      expect(view.points).toEqual(points.slice(-12));
      expect(view.canNext).toBe(false);
      const seen = new Set(view.points.map((p) => p.period));
      while (view.canPrevious) {
        view = monthlyWindow(points, 1, view.move(-1));
        view.points.forEach((p) => seen.add(p.period));
      }
      expect(seen.size).toBe(count);
      expect(view.points.length).toBe(Math.min(count, 12));
      while (view.canNext) view = monthlyWindow(points, 1, view.move(1));
      expect(view.points).toEqual(points.slice(-12));
    },
  );

  it("persists clamped selections across subsequent appends and prepends", () => {
    const points = months(36);
    const historical = monthlyWindow(points, 1, null).move(-1);
    const clampedLatest = monthlyWindow(points.slice(0, 24), 1, historical);
    expect(clampedLatest.points[0].period).toBe("2001-01");
    expect(clampedLatest.selection).toBeNull();
    expect(
      monthlyWindow(points, 1, clampedLatest.selection).points[0].period,
    ).toBe("2002-01");

    const clampedOldest = monthlyWindow(points.slice(15), 1, historical);
    expect(clampedOldest.selection?.start).toBe("2001-04");
    expect(
      monthlyWindow(points, 1, clampedOldest.selection).points[0].period,
    ).toBe("2001-04");
  });

  it("preserves historical calendar keys across prepends and clamps deletions; latest follows and vehicle changes reset", () => {
    const points = months(36);
    const historical = monthlyWindow(points, 1, null).move(-1);
    expect(monthlyWindow(points.slice(3), 1, historical).points[0].period).toBe(
      "2001-01",
    );
    expect(
      monthlyWindow(points.slice(15), 1, historical).points[0].period,
    ).toBe("2001-04");
    expect(monthlyWindow(months(48), 1, historical).points[0].period).toBe(
      "2001-01",
    );
    expect(monthlyWindow(months(48), 2, historical).points[0].period).toBe(
      "2003-01",
    );
    const latest = monthlyWindow(points, 1, null).move(1);
    expect(monthlyWindow(months(48), 1, latest).points[0].period).toBe(
      "2003-01",
    );
  });
});
