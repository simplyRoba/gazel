export interface MonthlySelection {
  vehicleId: number | null;
  start: string;
  followingLatest: boolean;
}

export function monthlyWindow<T extends { period: string }>(
  points: T[],
  vehicleId: number | null,
  selection: MonthlySelection | null,
) {
  const lastStart = Math.max(0, points.length - 12);
  let start = lastStart;
  if (
    selection &&
    selection.vehicleId === vehicleId &&
    !selection.followingLatest
  ) {
    const index = points.findIndex((point) => point.period >= selection.start);
    start = Math.min(index < 0 ? lastStart : index, lastStart);
  }
  const normalizedSelection: MonthlySelection | null =
    selection &&
    selection.vehicleId === vehicleId &&
    !selection.followingLatest &&
    points.length > 0 &&
    start < lastStart
      ? { vehicleId, start: points[start].period, followingLatest: false }
      : null;
  return {
    selection: normalizedSelection,
    points: points.slice(start, start + 12),
    canPrevious: start > 0,
    canNext: start < lastStart,
    move(direction: -1 | 1): MonthlySelection | null {
      if (!points.length) return null;
      const next = Math.max(0, Math.min(lastStart, start + direction * 12));
      return {
        vehicleId,
        start: points[next].period,
        followingLatest: next === lastStart,
      };
    },
  };
}
