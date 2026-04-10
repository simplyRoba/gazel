import type { SegmentHistory, Fillup, VehicleStats, Vehicle } from "$lib/api";

/**
 * Build a map from fill-up key (date|odometer) to segment efficiency.
 * Only valid segments are included.
 */
export function buildEfficiencyMap(
  segments: SegmentHistory[],
): Map<string, number> {
  const map = new Map<string, number>();
  for (const seg of segments) {
    if (seg.is_valid) {
      const key = `${seg.end_date}|${seg.end_odometer}`;
      map.set(key, seg.efficiency);
    }
  }
  return map;
}

/**
 * Look up the efficiency for a specific fill-up using a pre-built map.
 */
export function getEfficiencyForFillup(
  fillup: Fillup,
  effMap: Map<string, number>,
): number | null {
  const key = `${fillup.date}|${fillup.odometer}`;
  return effMap.get(key) ?? null;
}

/**
 * Aggregate summary across all vehicles.
 */
export interface FleetSummary {
  totalDistance: number;
  totalFillups: number;
  totalCost: number;
  costPerDistance: number | null;
  costPerVolume: number | null;
}

export function computeFleetSummary(
  vehicles: Vehicle[],
  getStats: (vehicleId: number) => VehicleStats | undefined,
): FleetSummary | null {
  if (vehicles.length === 0) return null;

  let totalDistance = 0;
  let totalFillups = 0;
  let totalCost = 0;
  let totalFuel = 0;
  let hasAnyStats = false;

  for (const v of vehicles) {
    const s = getStats(v.id);
    if (!s) continue;
    hasAnyStats = true;
    totalDistance += s.total_distance;
    totalFillups += s.fill_up_count;
    totalCost += s.total_cost;
    totalFuel += s.total_fuel;
  }

  if (!hasAnyStats) {
    return {
      totalDistance: 0,
      totalFillups: 0,
      totalCost: 0,
      costPerDistance: null,
      costPerVolume: null,
    };
  }

  const costPerDistance = totalDistance > 0 ? totalCost / totalDistance : null;
  const costPerVolume = totalFuel > 0 ? totalCost / totalFuel : null;

  return {
    totalDistance,
    totalFillups,
    totalCost,
    costPerDistance,
    costPerVolume,
  };
}
