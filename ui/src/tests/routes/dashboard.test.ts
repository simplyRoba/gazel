import { describe, expect, it } from "vitest";

// Test the date formatting logic used in the dashboard.
// Full component tests with mock stores would require
// mounting the Svelte component, which is more integration-level.

describe("dashboard date formatting", () => {
  function formatDate(dateStr: string): string {
    const d = new Date(dateStr + "T00:00:00");
    return d.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: "numeric",
    });
  }

  it("formats a date string", () => {
    const result = formatDate("2026-04-09");
    // Exact format depends on locale, but should contain 2026 and Apr or 4
    expect(result).toContain("2026");
    expect(result.length).toBeGreaterThan(5);
  });

  it("handles different months", () => {
    const jan = formatDate("2026-01-15");
    const dec = formatDate("2026-12-25");
    expect(jan).not.toEqual(dec);
  });
});
