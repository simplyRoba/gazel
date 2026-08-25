import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import {
  exceptionDetails,
  installGlobalClientDiagnostics,
  reportClientDiagnostic,
} from "./client-diagnostics";

const mockFetch = vi.fn((_input: RequestInfo | URL, _init?: RequestInit) =>
  Promise.resolve(new Response(null, { status: 204 })),
);

beforeEach(() => {
  mockFetch.mockClear();
  vi.stubGlobal("fetch", mockFetch);
});

afterEach(() => {
  vi.unstubAllGlobals();
});

describe("client diagnostics", () => {
  it("posts only the allowlisted diagnostic fields to the public endpoint", () => {
    reportClientDiagnostic({
      stage: "dashboard_loading_snapshot",
      outcome: "snapshot",
      settings_loading: false,
      vehicles_loading: false,
      active_vehicle_selected: true,
      fillups_loading: true,
      stats_loading: false,
    });

    expect(mockFetch).toHaveBeenCalledOnce();
    expect(mockFetch).toHaveBeenCalledWith(
      "/client-diagnostics",
      expect.objectContaining({
        method: "POST",
        keepalive: true,
      }),
    );
    const init = mockFetch.mock.calls[0][1];
    expect(JSON.parse(String(init?.body))).toEqual({
      stage: "dashboard_loading_snapshot",
      outcome: "snapshot",
      settings_loading: false,
      vehicles_loading: false,
      active_vehicle_selected: true,
      fillups_loading: true,
      stats_loading: false,
      pathname: "/",
    });
  });

  it("reports global errors and unhandled rejections without stacks", () => {
    const remove = installGlobalClientDiagnostics();

    window.dispatchEvent(
      new ErrorEvent("error", { error: new TypeError("render failed") }),
    );
    const rejection = new Event("unhandledrejection") as PromiseRejectionEvent;
    Object.defineProperty(rejection, "reason", {
      value: new Error("promise failed"),
    });
    window.dispatchEvent(rejection);
    remove();

    const payloads = mockFetch.mock.calls.map(([, init]) =>
      JSON.parse(String(init?.body)),
    );
    expect(payloads).toEqual([
      expect.objectContaining({
        stage: "window_error",
        exception_type: "TypeError",
        exception_message: "render failed",
      }),
      expect.objectContaining({
        stage: "unhandled_rejection",
        exception_type: "Error",
        exception_message: "promise failed",
      }),
    ]);
    expect(JSON.stringify(payloads)).not.toContain("stack");
  });

  it("normalizes control characters and non-Error exceptions", () => {
    expect(exceptionDetails(new Error("line one\nline two"))).toEqual({
      exception_type: "Error",
      exception_message: "line one line two",
    });
    expect(exceptionDetails(new Error("token=do-not-log"))).toEqual({
      exception_type: "Error",
      exception_message: "Sensitive exception message redacted",
    });
    expect(exceptionDetails({ secret: "not serialized" })).toEqual({
      exception_type: "NonError",
      exception_message: "Non-Error exception",
    });
  });
});
