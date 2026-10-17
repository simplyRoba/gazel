import { loginUrl, parseAuthConfig } from "#lib/auth-core.js";
import type { AuthConfig } from "#lib/auth-core.js";

export type { AuthConfig } from "#lib/auth-core.js";

// ── ApiError ──────────────────────────────────────────────

export class ApiError extends Error {
  status: number;
  code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

// ── Core request helper ──────────────────────────────────

const authenticationNavigationTargets = new WeakSet<object>();

export function authenticationLoginUrl(
  location: Pick<Location, "pathname" | "search" | "hash">,
): string {
  return loginUrl(`${location.pathname}${location.search}${location.hash}`);
}

function navigateToLogin(): void {
  if (typeof window === "undefined") return;
  const path = window.location.pathname;
  if (path === "/login" || path === "/auth" || path.startsWith("/auth/"))
    return;

  const targetWindow = window.top ?? window;
  if (authenticationNavigationTargets.has(targetWindow)) return;

  authenticationNavigationTargets.add(targetWindow);
  targetWindow.location.assign(authenticationLoginUrl(window.location));
}

async function apiErrorFromResponse(resp: Response): Promise<ApiError> {
  const parsed: unknown = await resp.json().catch(() => ({}));
  const data =
    parsed !== null && typeof parsed === "object" && !Array.isArray(parsed)
      ? (parsed as Record<string, unknown>)
      : {};
  const code = typeof data.code === "string" ? data.code : "UNKNOWN_ERROR";
  const message =
    typeof data.message === "string" ? data.message : resp.statusText;

  if (resp.status === 401 && code === "AUTHENTICATION_REQUIRED") {
    navigateToLogin();
  }

  return new ApiError(resp.status, code, message);
}

async function request<T>(
  method: string,
  url: string,
  body?: unknown,
): Promise<T> {
  const init: RequestInit = { method };
  if (body !== undefined) {
    init.headers = { "Content-Type": "application/json" };
    init.body = JSON.stringify(body);
  }

  const resp = await fetch(url, init);

  if (!resp.ok) {
    throw await apiErrorFromResponse(resp);
  }

  if (resp.status === 204) {
    return undefined as T;
  }

  return resp.json();
}

// ── App info types ───────────────────────────────────────

export interface AppInfo {
  version: string;
  repository: string;
  license: string;
  auth_enabled?: true;
}

// ── Public auth config API ───────────────────────────────

export async function fetchAuthConfig(
  signal?: AbortSignal,
): Promise<AuthConfig> {
  const response = await fetch("/auth/config", { signal });
  if (!response.ok) throw new Error("Auth configuration unavailable");

  const value: unknown = await response.json();
  const config = parseAuthConfig(value);
  if (!config) throw new Error("Invalid auth configuration");
  return config;
}

// ── App info API functions ──────────────────────────────

export function fetchAppInfo(): Promise<AppInfo> {
  return request("GET", "/api/info");
}

// ── Vehicle types ────────────────────────────────────────

export interface Vehicle {
  id: number;
  name: string;
  make: string | null;
  model: string | null;
  year: number | null;
  fuel_type: string;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateVehicle {
  name: string;
  make?: string | null;
  model?: string | null;
  year?: number | null;
  fuel_type?: string;
  notes?: string | null;
}

// ── Vehicle API functions ────────────────────────────────

export function fetchVehicles(): Promise<Vehicle[]> {
  return request("GET", "/api/vehicles");
}

export function fetchVehicle(id: number): Promise<Vehicle> {
  return request("GET", `/api/vehicles/${id}`);
}

export function createVehicle(data: CreateVehicle): Promise<Vehicle> {
  return request("POST", "/api/vehicles", data);
}

export function updateVehicle(
  id: number,
  data: CreateVehicle,
): Promise<Vehicle> {
  return request("PUT", `/api/vehicles/${id}`, data);
}

export function deleteVehicle(id: number): Promise<void> {
  return request("DELETE", `/api/vehicles/${id}`);
}

// ── Fill-up types ────────────────────────────────────────

export interface Fillup {
  id: number;
  vehicle_id: number;
  date: string;
  odometer: number;
  fuel_amount: number;
  fuel_unit: string;
  cost: number;
  currency: string;
  is_full_tank: boolean;
  is_missed: boolean;
  station: string | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface FillupPageEfficiency {
  distance_unit: string;
  volume_unit: string;
  values: Array<{ fillup_id: number; efficiency: number | null }>;
}

export interface FillupPage {
  items: Fillup[];
  next_cursor: string | null;
  efficiency: FillupPageEfficiency;
}

export interface CreateFillup {
  date: string;
  odometer: number;
  fuel_amount: number;
  cost: number;
  is_full_tank?: boolean;
  is_missed?: boolean;
  station?: string | null;
  notes?: string | null;
}

export interface UpdateFillup {
  date: string;
  odometer: number;
  fuel_amount: number;
  cost: number;
  is_full_tank?: boolean;
  is_missed?: boolean;
  station?: string | null;
  notes?: string | null;
}

// ── Fill-up API functions ────────────────────────────────

export function fetchFillups(
  vehicleId: number,
  cursor?: string,
): Promise<FillupPage> {
  const query =
    cursor === undefined ? "" : `?${new URLSearchParams({ cursor })}`;
  return request("GET", `/api/vehicles/${vehicleId}/fillups${query}`);
}

export function fetchFillup(
  vehicleId: number,
  fillupId: number,
): Promise<Fillup> {
  return request("GET", `/api/vehicles/${vehicleId}/fillups/${fillupId}`);
}

export function createFillup(
  vehicleId: number,
  data: CreateFillup,
): Promise<Fillup> {
  return request("POST", `/api/vehicles/${vehicleId}/fillups`, data);
}

export function updateFillup(
  vehicleId: number,
  fillupId: number,
  data: UpdateFillup,
): Promise<Fillup> {
  return request("PUT", `/api/vehicles/${vehicleId}/fillups/${fillupId}`, data);
}

export function deleteFillup(
  vehicleId: number,
  fillupId: number,
): Promise<void> {
  return request("DELETE", `/api/vehicles/${vehicleId}/fillups/${fillupId}`);
}

// ── Stats types ─────────────────────────────────────────

export interface StatsUnits {
  distance_unit: string;
  volume_unit: string;
  currency: string;
}

export interface StatsVehicleSummary {
  vehicle_id: number;
  total_distance: number;
  total_fuel: number;
  total_cost: number;
  fill_up_count: number;
  average_efficiency: number | null;
  average_cost_per_distance: number | null;
}

export interface StatsFleetSummary {
  total_distance: number;
  total_fuel: number;
  total_cost: number;
  fill_up_count: number;
  average_efficiency: number | null;
  cost_per_distance: number | null;
  cost_per_volume: number | null;
}

export interface StatsRange {
  start: string;
  end: string;
}

export interface PeriodSeries {
  points: Array<{ period: string; value: number }>;
  source_count: number;
  source_range: StatsRange | null;
  displayed_range: StatsRange | null;
  omitted_older_periods: boolean;
}

export interface PeriodModes {
  monthly: PeriodSeries;
  yearly: PeriodSeries;
}

export interface FuelPricePoint {
  date: string;
  value: number;
}

export interface FuelPriceSeries {
  points: FuelPricePoint[];
  source_count: number;
  reduced: boolean;
  source_range: StatsRange | null;
  displayed_range: StatsRange | null;
  trend: [FuelPricePoint, FuelPricePoint] | null;
}

export interface SparklineSeries {
  points: Array<{ position: number; value: number }>;
  source_count: number;
  reduced: boolean;
}

export interface SelectedStats {
  vehicle_id: number;
  summary: StatsVehicleSummary;
  charts: {
    cost: PeriodModes;
    distance: PeriodModes;
    fuel_price: FuelPriceSeries;
  };
  cost_per_distance: SparklineSeries;
}

export interface StatsResponse {
  units: StatsUnits;
  fleet: StatsFleetSummary;
  vehicles: StatsVehicleSummary[];
  fleet_efficiency: SparklineSeries;
  selected: SelectedStats | null;
}

// ── Stats API functions ─────────────────────────────────

export function fetchStats(
  vehicleId: number | null = null,
): Promise<StatsResponse> {
  const query =
    vehicleId === null
      ? ""
      : `?${new URLSearchParams({ vehicle_id: String(vehicleId) })}`;
  return request("GET", `/api/stats${query}`);
}

// ── Settings types ───────────────────────────────────────

export interface Settings {
  unit_system: string;
  distance_unit: string;
  volume_unit: string;
  currency: string;
  color_mode: string;
  locale: string;
}

export interface UpdateSettingsRequest {
  unit_system?: string;
  distance_unit?: string;
  volume_unit?: string;
  currency?: string;
  color_mode?: string;
  locale?: string;
}

// ── Settings API functions ───────────────────────────────

export function fetchSettings(): Promise<Settings> {
  return request("GET", "/api/settings");
}

export function updateSettings(data: UpdateSettingsRequest): Promise<Settings> {
  return request("PUT", "/api/settings", data);
}

// ── Export/Import types ─────────────────────────────────

export interface ImportReplaceResult {
  vehicles_created: number;
  fillups_created: number;
}

export interface ImportMergeResult {
  vehicles_created: number;
  vehicles_updated: number;
  fillups_created: number;
  fillups_skipped: number;
}

export interface ImportReplacePreview {
  preview: true;
  vehicles: number;
  fillups: number;
}

export interface ImportMergePreview {
  preview: true;
  vehicles_new: number;
  vehicles_existing: number;
  fillups_new: number;
  fillups_existing: number;
}

export type ImportMode = "replace" | "merge";

export type ImportPreviewResult = ImportReplacePreview | ImportMergePreview;
export type ImportResult = ImportReplaceResult | ImportMergeResult;

// ── Export/Import API functions ──────────────────────────

export async function exportAll(): Promise<void> {
  const resp = await fetch("/api/export");
  if (!resp.ok) {
    throw await apiErrorFromResponse(resp);
  }
  const blob = await resp.blob();
  const disposition = resp.headers.get("content-disposition");
  const filename =
    disposition?.match(/filename="(.+)"/)?.[1] ?? "gazel-export.json";
  downloadBlob(blob, filename);
}

export async function exportVehicle(id: number): Promise<void> {
  const resp = await fetch(`/api/vehicles/${id}/export`);
  if (!resp.ok) {
    throw await apiErrorFromResponse(resp);
  }
  const blob = await resp.blob();
  const disposition = resp.headers.get("content-disposition");
  const filename =
    disposition?.match(/filename="(.+)"/)?.[1] ?? "gazel-export.json";
  downloadBlob(blob, filename);
}

export function previewImport(
  data: unknown,
  mode: ImportMode = "replace",
): Promise<ImportPreviewResult> {
  return request("POST", `/api/import?preview=true&mode=${mode}`, data);
}

export function importData(
  data: unknown,
  mode: ImportMode = "replace",
): Promise<ImportResult> {
  return request("POST", `/api/import?mode=${mode}`, data);
}

function downloadBlob(blob: Blob, filename: string): void {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}
