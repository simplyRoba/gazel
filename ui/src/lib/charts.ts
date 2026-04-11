import type { Readable } from "svelte/store";
import type { ScaleLinear, ScaleTime, ScaleBand } from "d3-scale";

import type { SegmentHistory } from "$lib/api";

// ── LayerCake context type ──────────────────────────────

/* eslint-disable @typescript-eslint/no-explicit-any */
export interface LayerCakeContext {
  data: Readable<Record<string, any>[]>;
  xGet: Readable<(d: Record<string, any>) => number>;
  yGet: Readable<(d: Record<string, any>) => number>;
  xScale: Readable<
    ScaleLinear<number, number> | ScaleTime<number, number> | ScaleBand<string>
  >;
  yScale: Readable<ScaleLinear<number, number>>;
  width: Readable<number>;
  height: Readable<number>;
}
/* eslint-enable @typescript-eslint/no-explicit-any */

// ── Chart data shapes ───────────────────────────────────

export interface TimeSeriesPoint {
  date: Date;
  value: number;
}

export interface MonthlyCostPoint {
  month: string;
  value: number;
}

export interface SparklinePoint {
  x: number;
  y: number;
}

// ── Data transformation functions ───────────────────────

/**
 * Maps valid segments to efficiency time-series data.
 * Filters out invalid segments (where is_valid is false).
 */
export function toEfficiencyData(
  segments: SegmentHistory[],
): TimeSeriesPoint[] {
  return segments
    .filter((s) => s.is_valid)
    .map((s) => ({
      date: new Date(s.end_date),
      value: s.efficiency,
    }));
}

/**
 * Aggregates segment costs by calendar month.
 * Returns sorted (chronological) monthly totals with zero-filled
 * gaps so every month between the first and last entry is present.
 */
export function toMonthlyCostData(
  segments: SegmentHistory[],
): MonthlyCostPoint[] {
  if (segments.length === 0) return [];

  const monthMap = new Map<string, number>();

  for (const s of segments) {
    const month = s.end_date.slice(0, 7); // YYYY-MM
    monthMap.set(month, (monthMap.get(month) ?? 0) + s.cost);
  }

  const keys = Array.from(monthMap.keys()).sort();
  const first = keys[0];
  const last = keys[keys.length - 1];

  const result: MonthlyCostPoint[] = [];
  let [y, m] = first.split("-").map(Number);
  const [endY, endM] = last.split("-").map(Number);

  while (y < endY || (y === endY && m <= endM)) {
    const key = `${y}-${String(m).padStart(2, "0")}`;
    result.push({ month: key, value: monthMap.get(key) ?? 0 });
    m++;
    if (m > 12) {
      m = 1;
      y++;
    }
  }

  return result;
}

/**
 * Aggregates segment costs by calendar year.
 * Returns sorted (chronological) yearly totals.
 */
export function toYearlyCostData(
  segments: SegmentHistory[],
): MonthlyCostPoint[] {
  const yearMap = new Map<string, number>();

  for (const s of segments) {
    const year = s.end_date.slice(0, 4); // YYYY
    yearMap.set(year, (yearMap.get(year) ?? 0) + s.cost);
  }

  return Array.from(yearMap.entries())
    .sort(([a], [b]) => a.localeCompare(b))
    .map(([month, value]) => ({ month, value }));
}

/** Parse a YYYY-MM-DD string as a local-time Date (avoids UTC shifting). */
function parseLocalDate(dateStr: string): Date {
  const [y, m, d] = dateStr.split("-").map(Number);
  return new Date(y, m - 1, d);
}

/**
 * Distributes segment distances proportionally across the months they span.
 * A segment from Jan 15 to Mar 20 has its distance split across Jan, Feb,
 * and Mar based on how many days of the segment fall in each month.
 * Returns sorted (chronological) monthly totals with zero-filled gaps.
 */
export function toMonthlyDistanceData(
  segments: SegmentHistory[],
): MonthlyCostPoint[] {
  if (segments.length === 0) return [];

  const MS_PER_DAY = 86_400_000;
  const monthMap = new Map<string, number>();

  for (const s of segments) {
    const start = parseLocalDate(s.start_date);
    // End date is inclusive (fill-up happened that day), so use next day
    // as the exclusive boundary for proportional splitting.
    const endExcl = parseLocalDate(s.end_date);
    endExcl.setDate(endExcl.getDate() + 1);
    const totalDays = Math.max(
      1,
      (endExcl.getTime() - start.getTime()) / MS_PER_DAY,
    );

    // Walk month by month from start to endExcl, distributing distance
    let cursor = new Date(start);
    while (cursor < endExcl) {
      const y = cursor.getFullYear();
      const m = cursor.getMonth();
      const key = `${y}-${String(m + 1).padStart(2, "0")}`;

      // Slice: from cursor to min(firstOfNextMonth, endExcl)
      const nextMonth = new Date(y, m + 1, 1);
      const sliceEnd =
        nextMonth.getTime() <= endExcl.getTime() ? nextMonth : endExcl;
      const daysInSlice = (sliceEnd.getTime() - cursor.getTime()) / MS_PER_DAY;

      if (daysInSlice > 0) {
        const fraction = daysInSlice / totalDays;
        monthMap.set(key, (monthMap.get(key) ?? 0) + s.distance * fraction);
      }

      cursor = nextMonth;
    }
  }

  const keys = Array.from(monthMap.keys()).sort();
  const first = keys[0];
  const last = keys[keys.length - 1];

  const result: MonthlyCostPoint[] = [];
  let [fy, fm] = first.split("-").map(Number);
  const [endY, endM] = last.split("-").map(Number);

  while (fy < endY || (fy === endY && fm <= endM)) {
    const key = `${fy}-${String(fm).padStart(2, "0")}`;
    result.push({ month: key, value: Math.round(monthMap.get(key) ?? 0) });
    fm++;
    if (fm > 12) {
      fm = 1;
      fy++;
    }
  }

  return result;
}

/**
 * Maps segments to fuel price time-series data (cost / fuel).
 * Excludes segments with zero fuel.
 */
export function toFuelPriceData(segments: SegmentHistory[]): TimeSeriesPoint[] {
  return segments
    .filter((s) => s.fuel > 0)
    .map((s) => ({
      date: new Date(s.end_date),
      value: s.cost / s.fuel,
    }));
}

/**
 * Generic mapper for sparkline-ready data.
 * Returns index-based x values with y from the accessor.
 */
export function toSparklineData(
  segments: SegmentHistory[],
  accessor: (s: SegmentHistory) => number,
): SparklinePoint[] {
  return segments.map((s, i) => ({
    x: i,
    y: accessor(s),
  }));
}
