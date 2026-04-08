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
