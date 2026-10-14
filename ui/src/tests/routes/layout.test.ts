import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
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
  vehicles: [] as import("#lib/api.js").Vehicle[],
  createFillup: vi.fn(),
  setActiveVehicle: vi.fn(),
  refreshStats: vi.fn(),
}));

function buildTouchEvent(
  type: string,
  yPositions: number[],
  cancelable = false,
): TouchEvent {
  const event = new Event(type, { bubbles: true, cancelable }) as TouchEvent;
  Object.defineProperty(event, "touches", {
    value: yPositions.map((clientY) => ({ clientY })),
  });
  return event;
}

vi.stubGlobal("fetch", mockFetch);

vi.mock("$app/state", () => ({
  page: {
    get url() {
      if (!pageState.url) throw new Error("test page URL is not initialized");
      return pageState.url;
    },
  },
}));

vi.mock("#lib/stores/settings.svelte.js", async (importOriginal) => ({
  ...(await importOriginal<typeof import("#lib/stores/settings.svelte.js")>()),
  initSettings: storeSpies.initSettings,
}));

vi.mock("#lib/stores/vehicles.svelte.js", async (importOriginal) => ({
  ...(await importOriginal<typeof import("#lib/stores/vehicles.svelte.js")>()),
  loadVehicles: storeSpies.loadVehicles,
  getVehicles: () => storeSpies.vehicles,
}));

vi.mock("#lib/stores/stats.svelte.js", () => ({
  refreshStats: storeSpies.refreshStats,
}));
vi.mock("#lib/stores/fillups.svelte.js", async (original) => ({
  ...(await original<typeof import("#lib/stores/fillups.svelte.js")>()),
  createFillup: storeSpies.createFillup,
  setActiveVehicle: storeSpies.setActiveVehicle,
}));

describe("root layout public login branch", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    storeSpies.vehicles = [];
    vi.stubGlobal(
      "matchMedia",
      vi.fn(() => ({ matches: false })),
    );
    pageState.url = new SvelteURL("http://localhost/login");
    HTMLDialogElement.prototype.showModal = function () {
      this.open = true;
    };
    HTMLDialogElement.prototype.close = function () {
      this.open = false;
    };
  });

  afterEach(() => {
    cleanup();
    vi.restoreAllMocks();
  });

  it.each([true, false])(
    "refreshes displayed statistics only after a successful global CTA save (success=%s)",
    async (success) => {
      pageState.url = new SvelteURL("http://localhost/");
      storeSpies.vehicles = [
        { id: 1, name: "First" },
        { id: 2, name: "Second" },
      ] as import("#lib/api.js").Vehicle[];
      storeSpies.createFillup.mockResolvedValueOnce(success ? { id: 1 } : null);
      const { container } = render(LoginLayoutFixture);
      await tick();
      await fireEvent.click(container.querySelector(".sidebar-cta")!);
      await fireEvent.click(screen.getByRole("button", { name: "Second" }));
      expect(storeSpies.setActiveVehicle).toHaveBeenCalledExactlyOnceWith(2);
      expect(storeSpies.refreshStats).not.toHaveBeenCalled();
      for (const [id, value] of [
        ["q-odometer", "11000"],
        ["q-fuel", "40"],
        ["q-total", "80"],
      ]) {
        await fireEvent.input(container.querySelector(`#${id}`)!, {
          target: { value },
        });
      }
      await fireEvent.submit(container.querySelector("#quick-fill-form")!);
      await waitFor(() =>
        expect(storeSpies.createFillup).toHaveBeenCalledTimes(1),
      );
      if (success) {
        await waitFor(() =>
          expect(storeSpies.refreshStats).toHaveBeenCalledExactlyOnceWith(),
        );
      } else {
        await waitFor(() =>
          expect(container.querySelector(".save-error")).toBeTruthy(),
        );
        expect(storeSpies.refreshStats).not.toHaveBeenCalled();
      }
    },
  );

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

  it("resets an active pull when a second touch starts", async () => {
    vi.stubGlobal(
      "matchMedia",
      vi.fn((query: string) => ({
        matches:
          query === "(display-mode: standalone)" ||
          query === "(pointer: coarse)",
      })),
    );
    pageState.url = new SvelteURL("http://localhost/");

    const { container } = render(LoginLayoutFixture);
    await tick();
    await tick();

    await fireEvent(window, buildTouchEvent("touchstart", [100]));
    await fireEvent(window, buildTouchEvent("touchmove", [160], true));
    expect(container.querySelector(".pull-indicator")?.classList).toContain(
      "visible",
    );

    await fireEvent(window, buildTouchEvent("touchstart", [160, 200]));
    expect(container.querySelector(".pull-indicator")?.classList).not.toContain(
      "visible",
    );
  });

  it("requires touched scroll containers to be at the top", async () => {
    vi.stubGlobal(
      "matchMedia",
      vi.fn((query: string) => ({
        matches:
          query === "(display-mode: standalone)" ||
          query === "(pointer: coarse)",
      })),
    );
    pageState.url = new SvelteURL("http://localhost/");

    const { container } = render(LoginLayoutFixture);
    await tick();
    await tick();

    const scrollContainer = screen.getByTestId("fixture-scroll-container");
    const touchTarget = scrollContainer.querySelector("p")!;
    const indicator = container.querySelector(".pull-indicator")!;

    scrollContainer.scrollTop = 120;
    await fireEvent(touchTarget, buildTouchEvent("touchstart", [100]));
    await fireEvent(touchTarget, buildTouchEvent("touchmove", [160], true));
    expect(indicator.classList).not.toContain("visible");

    scrollContainer.scrollTop = 0;
    await fireEvent(touchTarget, buildTouchEvent("touchmove", [170], true));
    expect(indicator.classList).not.toContain("visible");

    await fireEvent(touchTarget, buildTouchEvent("touchstart", [100]));
    await fireEvent(touchTarget, buildTouchEvent("touchmove", [160], true));
    expect(indicator.classList).toContain("visible");

    scrollContainer.scrollTop = 10;
    await fireEvent(touchTarget, buildTouchEvent("touchmove", [170], true));
    expect(indicator.classList).not.toContain("visible");
  });
});
