import { describe, expect, it } from "vitest";
import { toPeriodData, toPriceData, toSparklineData } from "./charts.js";
import en from "./i18n/en.json";
import de from "./i18n/de.json";

describe("bounded chart presentation mappers", () => {
  it("keeps English and German range-label keys and interpolation fields aligned", () => {
    expect(Object.keys(en).sort()).toEqual(Object.keys(de).sort());
    for (const key of [
      "charts.limitedRange",
      "charts.reducedHistory",
    ] as const) {
      expect(en[key].match(/\{\w+\}/g)?.sort()).toEqual(
        de[key].match(/\{\w+\}/g)?.sort(),
      );
    }
  });

  it("maps supplied periods exactly without filling gaps, sorting, or rounding", () => {
    const series = {
      points: [
        { period: "2026-01", value: 123.456789 },
        { period: "2026-03", value: 0 },
      ],
      source_count: 200,
      source_range: { start: "2000-01", end: "2026-03" },
      displayed_range: { start: "2026-01", end: "2026-03" },
      omitted_older_periods: true,
    };
    const before = structuredClone(series);
    expect(toPeriodData(series)).toEqual([
      { month: "2026-01", value: 123.456789 },
      { month: "2026-03", value: 0 },
    ]);
    expect(series).toEqual(before);
    expect(toPeriodData({ ...series, points: [] })).toEqual([]);
  });

  it("parses local calendar dates without changing already converted unrounded prices or same-day order", () => {
    const points = [
      { date: "2026-03-01", value: 1.23456789 },
      { date: "2026-03-01", value: 0 },
    ];
    const result = toPriceData(points);
    expect(result.map((point) => point.value)).toEqual([1.23456789, 0]);
    expect(result[0].date.getFullYear()).toBe(2026);
    expect(result[0].date.getMonth()).toBe(2);
    expect(result[0].date.getDate()).toBe(1);
    expect(result[0].date.getHours()).toBe(0);
    expect(points[0].date).toBe("2026-03-01");
    expect(toPriceData([])).toEqual([]);
  });

  it("preserves unequal sampled positions and display-convention efficiency without inversion", () => {
    expect(
      toSparklineData({
        points: [
          { position: 0, value: 8 },
          { position: 0.1, value: 10 },
          { position: 0.9, value: 6 },
          { position: 1, value: 9 },
        ],
        source_count: 1000,
        reduced: true,
      }),
    ).toEqual([
      { x: 0, y: 8 },
      { x: 0.1, y: 10 },
      { x: 0.9, y: 6 },
      { x: 1, y: 9 },
    ]);
    expect(
      toSparklineData({ points: [], source_count: 0, reduced: false }),
    ).toEqual([]);
  });
});
