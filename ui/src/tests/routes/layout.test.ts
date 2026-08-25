import { cleanup, render, screen } from "@testing-library/svelte";
import { SvelteURL } from "svelte/reactivity";
import { tick } from "svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import LoginLayoutFixture from "./LoginLayoutFixture.svelte";

type ReactiveUrl = Pick<SvelteURL, "pathname">;

const mockFetch = vi.fn();
const pageState = vi.hoisted(() => ({
  url: null as ReactiveUrl | null,
}));
const storeSpies = vi.hoisted(() => ({
  initSettings: vi.fn(() => Promise.resolve()),
  loadVehicles: vi.fn(() => Promise.resolve()),
}));

vi.stubGlobal("fetch", mockFetch);

vi.mock("$app/state", () => ({
  page: {
    get url() {
      if (!pageState.url) throw new Error("test page URL is not initialized");
      return pageState.url;
    },
  },
}));

vi.mock("$lib/stores/settings.svelte", async (importOriginal) => ({
  ...(await importOriginal<typeof import("$lib/stores/settings.svelte")>()),
  initSettings: storeSpies.initSettings,
}));

vi.mock("$lib/stores/vehicles.svelte", async (importOriginal) => ({
  ...(await importOriginal<typeof import("$lib/stores/vehicles.svelte")>()),
  loadVehicles: storeSpies.loadVehicles,
}));

describe("root layout public login branch", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal(
      "matchMedia",
      vi.fn(() => ({ matches: false })),
    );
    pageState.url = new SvelteURL("http://localhost/login");
  });

  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it("renders only login content without protected shell or hydration", () => {
    const { container } = render(LoginLayoutFixture);

    expect(screen.getByText("Public login content")).toBeTruthy();
    expect(container.querySelector(".sidebar")).toBeNull();
    expect(container.querySelector(".bottom-bar")).toBeNull();
    expect(container.querySelector(".pull-indicator")).toBeNull();
    expect(storeSpies.initSettings).not.toHaveBeenCalled();
    expect(storeSpies.loadVehicles).not.toHaveBeenCalled();
    expect(mockFetch).not.toHaveBeenCalled();
  });

  it("initializes protected stores once after leaving login and removes the shell on return", async () => {
    const { container } = render(LoginLayoutFixture);

    pageState.url!.pathname = "/";
    await tick();
    await tick();
    expect(container.querySelector(".app-shell")).toBeTruthy();
    expect(storeSpies.initSettings).toHaveBeenCalledTimes(1);
    expect(storeSpies.loadVehicles).toHaveBeenCalledTimes(1);

    pageState.url!.pathname = "/settings";
    await tick();
    expect(storeSpies.initSettings).toHaveBeenCalledTimes(1);
    expect(storeSpies.loadVehicles).toHaveBeenCalledTimes(1);

    pageState.url!.pathname = "/login";
    await tick();
    expect(container.querySelector(".app-shell")).toBeNull();
    expect(container.querySelector(".pull-indicator")).toBeNull();
  });

  it("registers standalone touch listeners once without recursively rerunning the effect", async () => {
    const matchMedia = vi.fn((query: string) => ({
      matches:
        query === "(display-mode: standalone)" || query === "(pointer: coarse)",
    }));
    vi.stubGlobal("matchMedia", matchMedia);
    pageState.url = new SvelteURL("http://localhost/");

    const addEventListener = vi.spyOn(window, "addEventListener");
    const unhandledRejections: unknown[] = [];
    const handleUnhandledRejection = (event: PromiseRejectionEvent) => {
      unhandledRejections.push(event.reason);
    };
    window.addEventListener("unhandledrejection", handleUnhandledRejection);

    render(LoginLayoutFixture);
    await tick();
    await tick();

    for (const eventName of [
      "touchstart",
      "touchmove",
      "touchend",
      "touchcancel",
    ]) {
      expect(
        addEventListener.mock.calls.filter(([name]) => name === eventName),
      ).toHaveLength(1);
    }
    expect(matchMedia).toHaveBeenCalledTimes(2);
    expect(unhandledRejections).toEqual([]);

    window.removeEventListener("unhandledrejection", handleUnhandledRejection);
  });
});
