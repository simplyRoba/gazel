import type { ScaleLinear, ScaleTime, ScaleBand, ScalePoint } from "d3-scale";

import type {
  FuelPricePoint,
  PeriodSeries,
  SparklineSeries,
} from "#lib/api.js";

export interface ChartScales {
  x:
    | ScaleLinear<number, number>
    | ScaleTime<number, number>
    | ScaleBand<string>
    | ScalePoint<string>;
  y: ScaleLinear<number, number>;
}

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

/** Adapt server periods without aggregating, filling gaps, or trimming them. */
export function toPeriodData(series: PeriodSeries): MonthlyCostPoint[] {
  return series.points.map(({ period, value }) => ({ month: period, value }));
}

/** Parse calendar dates locally; values are already in the response units. */
export function toPriceData(
  points: readonly FuelPricePoint[],
): TimeSeriesPoint[] {
  return points.map(({ date, value }) => {
    const [year, month, day] = date.split("-").map(Number);
    return { date: new Date(year, month - 1, day), value };
  });
}

/** Preserve the server's original positions and display-convention values. */
export function toSparklineData(series: SparklineSeries): SparklinePoint[] {
  return series.points.map(({ position, value }) => ({
    x: position,
    y: value,
  }));
}
