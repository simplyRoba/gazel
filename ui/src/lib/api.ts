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
    const data = await resp.json().catch(() => ({ message: resp.statusText }));
    throw new ApiError(
      resp.status,
      data.code || "UNKNOWN_ERROR",
      data.message || resp.statusText,
    );
  }

  if (resp.status === 204) {
    return undefined as T;
  }

  return resp.json();
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
