import { describe, it, expect, vi, beforeEach } from "vitest";

// Mock fetch globally
const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

// Mock URL.createObjectURL/revokeObjectURL and DOM methods for download tests
vi.stubGlobal("URL", {
  createObjectURL: vi.fn(() => "blob:test-url"),
  revokeObjectURL: vi.fn(),
});

import {
  ApiError,
  exportAll,
  exportVehicle,
  previewImport,
  importData,
} from "./api";

describe("Export/Import API", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Mock DOM for download
    const mockElement = {
      href: "",
      download: "",
      click: vi.fn(),
    };
    vi.spyOn(document, "createElement").mockReturnValue(
      mockElement as unknown as HTMLElement,
    );
    vi.spyOn(document.body, "appendChild").mockReturnValue(
      null as unknown as HTMLElement,
    );
    vi.spyOn(document.body, "removeChild").mockReturnValue(
      null as unknown as HTMLElement,
    );
  });

  describe("exportAll", () => {
    it("downloads file on success", async () => {
      const blob = new Blob(["{}"], { type: "application/json" });
      mockFetch.mockResolvedValue({
        ok: true,
        blob: () => Promise.resolve(blob),
        headers: new Headers({
          "content-disposition": 'attachment; filename="gazel-export.json"',
        }),
      });

      await exportAll();

      expect(mockFetch).toHaveBeenCalledWith("/api/export");
      expect(document.createElement).toHaveBeenCalledWith("a");
    });

    it("throws ApiError on failure", async () => {
      mockFetch.mockResolvedValue({
        ok: false,
        status: 500,
        statusText: "Internal Server Error",
        json: () =>
          Promise.resolve({ code: "INTERNAL_ERROR", message: "fail" }),
      });

      await expect(exportAll()).rejects.toThrow(ApiError);
    });
  });

  describe("exportVehicle", () => {
    it("downloads file with vehicle-specific URL", async () => {
      const blob = new Blob(["{}"], { type: "application/json" });
      mockFetch.mockResolvedValue({
        ok: true,
        blob: () => Promise.resolve(blob),
        headers: new Headers({
          "content-disposition":
            'attachment; filename="gazel-export-my-car.json"',
        }),
      });

      await exportVehicle(42);

      expect(mockFetch).toHaveBeenCalledWith("/api/vehicles/42/export");
    });

    it("throws ApiError on 404", async () => {
      mockFetch.mockResolvedValue({
        ok: false,
        status: 404,
        statusText: "Not Found",
        json: () =>
          Promise.resolve({ code: "VEHICLE_NOT_FOUND", message: "not found" }),
      });

      await expect(exportVehicle(999)).rejects.toThrow(ApiError);
    });
  });

  describe("previewImport", () => {
    it("sends preview request in replace mode", async () => {
      const previewResult = { preview: true, vehicles: 2, fillups: 10 };
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve(previewResult),
      });

      const result = await previewImport({ version: "1.0.0", vehicles: [] });

      expect(mockFetch).toHaveBeenCalledWith(
        "/api/import?preview=true&mode=replace",
        expect.objectContaining({
          method: "POST",
          body: expect.any(String),
        }),
      );
      expect(result).toEqual(previewResult);
    });

    it("sends preview request in merge mode", async () => {
      const previewResult = {
        preview: true,
        vehicles_new: 1,
        vehicles_existing: 1,
        fillups_new: 5,
        fillups_existing: 3,
      };
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve(previewResult),
      });

      const result = await previewImport(
        { version: "1.0.0", vehicles: [] },
        "merge",
      );

      expect(mockFetch).toHaveBeenCalledWith(
        "/api/import?preview=true&mode=merge",
        expect.objectContaining({ method: "POST" }),
      );
      expect(result).toEqual(previewResult);
    });

    it("throws on version mismatch", async () => {
      mockFetch.mockResolvedValue({
        ok: false,
        status: 422,
        statusText: "Unprocessable Entity",
        json: () =>
          Promise.resolve({
            code: "IMPORT_VERSION_MISMATCH",
            message: "mismatch",
          }),
      });

      await expect(
        previewImport({ version: "99.0.0", vehicles: [] }),
      ).rejects.toThrow(ApiError);
    });
  });

  describe("importData", () => {
    it("sends import request in replace mode by default", async () => {
      const importResult = { vehicles_created: 2, fillups_created: 10 };
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve(importResult),
      });

      const result = await importData({ version: "1.0.0", vehicles: [] });

      expect(mockFetch).toHaveBeenCalledWith(
        "/api/import?mode=replace",
        expect.objectContaining({ method: "POST" }),
      );
      expect(result).toEqual(importResult);
    });

    it("sends import request in merge mode", async () => {
      const importResult = {
        vehicles_created: 1,
        vehicles_updated: 1,
        fillups_created: 5,
        fillups_skipped: 3,
      };
      mockFetch.mockResolvedValue({
        ok: true,
        status: 200,
        json: () => Promise.resolve(importResult),
      });

      const result = await importData(
        { version: "1.0.0", vehicles: [] },
        "merge",
      );

      expect(mockFetch).toHaveBeenCalledWith(
        "/api/import?mode=merge",
        expect.objectContaining({ method: "POST" }),
      );
      expect(result).toEqual(importResult);
    });

    it("throws on validation error", async () => {
      mockFetch.mockResolvedValue({
        ok: false,
        status: 422,
        statusText: "Unprocessable Entity",
        json: () =>
          Promise.resolve({
            code: "IMPORT_VALIDATION_ERROR",
            message: "invalid data",
          }),
      });

      const error = await importData({ version: "1.0.0", vehicles: [] }).catch(
        (e: unknown) => e,
      );
      expect(error).toBeInstanceOf(ApiError);
      expect((error as ApiError).code).toBe("IMPORT_VALIDATION_ERROR");
    });
  });
});
