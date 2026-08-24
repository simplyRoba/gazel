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
  const returnTo = `${location.pathname}${location.search}${location.hash}`;
  return `/login?${new URLSearchParams({ return_to: returnTo })}`;
}

function navigateToLogin(): void {
  if (typeof window === "undefined") return;

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

export type AuthConfig =
  { enabled: false } | { enabled: true; provider_name: string };

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
  if (!isAuthConfig(value)) throw new Error("Invalid auth configuration");
  return value;
}

function isAuthConfig(value: unknown): value is AuthConfig {
  if (!value || typeof value !== "object" || Array.isArray(value)) return false;

  const record = value as Record<string, unknown>;
  const keys = Object.keys(record).sort();
  if (record.enabled === false) {
    return keys.length === 1 && keys[0] === "enabled";
  }
  if (
    record.enabled !== true ||
    keys.length !== 2 ||
    keys[0] !== "enabled" ||
    keys[1] !== "provider_name" ||
    typeof record.provider_name !== "string"
  ) {
    return false;
  }

  const providerName = record.provider_name;
  return (
    providerName === providerName.trim() &&
    providerName.length > 0 &&
    [...providerName].length <= 80 &&
    ![...providerName].some((character) => {
      const codePoint = character.codePointAt(0) ?? 0;
      return codePoint <= 0x1f || (codePoint >= 0x7f && codePoint <= 0x9f);
    })
  );
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

export function fetchFillups(vehicleId: number): Promise<Fillup[]> {
  return request("GET", `/api/vehicles/${vehicleId}/fillups`);
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

export interface VehicleStats {
  total_distance: number;
  total_fuel: number;
  total_cost: number;
  fill_up_count: number;
  average_efficiency: number | null;
  average_cost_per_distance: number | null;
  distance_unit: string;
  volume_unit: string;
  currency: string;
}

export interface SegmentHistory {
  start_date: string;
  end_date: string;
  start_odometer: number;
  end_odometer: number;
  distance: number;
  fuel: number;
  cost: number;
  efficiency: number;
  cost_per_distance: number;
  is_valid: boolean;
  distance_unit: string;
  volume_unit: string;
  currency: string;
}

// ── Stats API functions ─────────────────────────────────

export function fetchVehicleStats(vehicleId: number): Promise<VehicleStats> {
  return request("GET", `/api/vehicles/${vehicleId}/stats`);
}

export function fetchVehicleStatsHistory(
  vehicleId: number,
): Promise<SegmentHistory[]> {
  return request("GET", `/api/vehicles/${vehicleId}/stats/history`);
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
