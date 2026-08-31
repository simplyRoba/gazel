import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import LoginPage from "../../routes/login/+page.svelte";

const mockFetch = vi.fn();
vi.stubGlobal("fetch", mockFetch);

vi.mock("$app/state", () => ({
  page: {
    get url() {
      return new URL(window.location.href);
    },
  },
}));

function setUrl(path: string): void {
  window.history.replaceState({}, "", path);
}

describe("login page", () => {
  beforeEach(() => {
    mockFetch.mockReset();
    setUrl("/login");
  });

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
    vi.stubGlobal("fetch", mockFetch);
  });

  it("shows a branded loading state without a provider action while config is pending", () => {
    mockFetch.mockReturnValue(new Promise(() => {}));

    render(LoginPage);

    expect(screen.getByRole("heading", { name: "Gazel" })).toBeTruthy();
    expect(screen.getByRole("status").textContent).toBe("Loading...");
    expect(screen.queryByRole("button")).toBeNull();
    expect(document.querySelector("[aria-busy='true']")).toBeTruthy();
  });

  it("renders Gazel branding and one provider action without local credentials", async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: () =>
        Promise.resolve({ enabled: true, provider_name: "Authentik" }),
    });

    render(LoginPage);

    expect(
      await screen.findByRole("button", { name: "Continue with Authentik" }),
    ).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Gazel" })).toBeTruthy();
    expect(
      screen.getByText("Authentication is required to use Gazel."),
    ).toBeTruthy();
    expect(screen.getAllByRole("button")).toHaveLength(1);
    const form = document.querySelector("form");
    expect(form?.getAttribute("action")).toBe("/auth/login");
    expect(form?.getAttribute("method")).toBe("GET");
    expect(form?.querySelector("input")?.value).toBe("/");
    expect(screen.queryByLabelText(/password|username|email/i)).toBeNull();
  });

  it("propagates decoded return_to as encoded auth-login query data", async () => {
    setUrl("/login?return_to=%2Fsettings%3Ftab%3Ddata%23export");
    mockFetch.mockResolvedValue({
      ok: true,
      json: () =>
        Promise.resolve({ enabled: true, provider_name: "OpenID Connect" }),
    });

    render(LoginPage);

    expect(
      await screen.findByRole("button", {
        name: "Continue with OpenID Connect",
      }),
    ).toBeTruthy();
    const returnTo = document.querySelector("input")?.value;
    expect(returnTo).toBe("/settings?tab=data#export");
    expect(new URLSearchParams({ return_to: returnTo ?? "" }).toString()).toBe(
      "return_to=%2Fsettings%3Ftab%3Ddata%23export",
    );
  });

  it.each([
    "https://attacker.example/",
    "//attacker.example/",
    "/auth/login",
    `/${"a".repeat(2_049)}`,
  ])(
    "never turns untrusted return_to data into an external action %#",
    async (returnTo) => {
      setUrl(`/login?${new URLSearchParams({ return_to: returnTo })}`);
      mockFetch.mockResolvedValue({
        ok: true,
        json: () =>
          Promise.resolve({ enabled: true, provider_name: "OpenID Connect" }),
      });

      render(LoginPage);

      expect(
        await screen.findByRole("button", {
          name: "Continue with OpenID Connect",
        }),
      ).toBeTruthy();
      const form = document.querySelector("form");
      expect(form?.getAttribute("action")).toBe("/auth/login");
      expect(form?.querySelector("input")?.value).toBe(returnTo);
      expect(document.querySelector('a[href^="http"]')).toBeNull();
    },
  );

  it("renders the default provider label returned by public config", async () => {
    mockFetch.mockResolvedValue({
      ok: true,
      json: () =>
        Promise.resolve({ enabled: true, provider_name: "OpenID Connect" }),
    });

    render(LoginPage);

    expect(
      await screen.findByRole("button", {
        name: "Continue with OpenID Connect",
      }),
    ).toBeTruthy();
  });

  it("shows stable failure and signed-out states without automatic login", async () => {
    setUrl("/login?error=untrusted_provider_message&logged_out=1");
    mockFetch.mockResolvedValue({
      ok: true,
      json: () =>
        Promise.resolve({ enabled: true, provider_name: "OpenID Connect" }),
    });

    render(LoginPage);

    expect(
      await screen.findByText("Authentication failed. Please try again."),
    ).toBeTruthy();
    expect(screen.queryByText("untrusted_provider_message")).toBeNull();
    expect(screen.getByRole("button")).toBeTruthy();
    expect(mockFetch).toHaveBeenCalledTimes(1);
  });

  it("treats an empty error value as an unknown authentication failure", async () => {
    setUrl("/login?error=");
    mockFetch.mockResolvedValue({
      ok: true,
      json: () =>
        Promise.resolve({ enabled: true, provider_name: "OpenID Connect" }),
    });

    render(LoginPage);

    expect(
      await screen.findByText("Authentication failed. Please try again."),
    ).toBeTruthy();
    expect(screen.getByRole("button")).toBeTruthy();
  });

  it("shows provider-unavailable and signed-out states without starting login", async () => {
    setUrl("/login?error=provider_unavailable");
    mockFetch.mockResolvedValue({
      ok: true,
      json: () =>
        Promise.resolve({ enabled: true, provider_name: "OpenID Connect" }),
    });

    render(LoginPage);

    expect(
      await screen.findByText(
        "The authentication provider is temporarily unavailable. Please try again.",
      ),
    ).toBeTruthy();
    expect(screen.getByRole("button")).toBeTruthy();

    cleanup();
    setUrl("/login?logged_out=1");
    render(LoginPage);

    expect(
      await screen.findByText("You have been signed out of Gazel."),
    ).toBeTruthy();
    expect(screen.getByRole("button")).toBeTruthy();
    expect(mockFetch).toHaveBeenCalledTimes(2);
  });

  it("replace-navigates without authentication controls when auth config is disabled", async () => {
    const replace = vi.fn();
    vi.stubGlobal("window", {
      location: { href: "http://localhost:3000/login", replace },
    });
    mockFetch.mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ enabled: false }),
    });

    render(LoginPage);

    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(mockFetch).toHaveBeenCalledWith(
      "/auth/config",
      expect.objectContaining({ signal: expect.any(AbortSignal) }),
    );
    expect(replace).toHaveBeenCalledWith("/");
    expect(
      screen.queryByText("Authentication is required to use Gazel."),
    ).toBeNull();
    expect(screen.queryByRole("button")).toBeNull();
  });

  it.each([
    { enabled: true },
    { enabled: true, provider_name: "" },
    { enabled: false, provider_name: "unexpected" },
  ])(
    "shows a generic usable message for malformed public config %#",
    async (value) => {
      mockFetch.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve(value),
      });

      render(LoginPage);

      expect(
        await screen.findByText(
          "Authentication is currently unavailable. Please try again later.",
        ),
      ).toBeTruthy();
      expect(screen.queryByRole("button")).toBeNull();
    },
  );

  it("shows a generic usable message when public auth config fails", async () => {
    mockFetch.mockRejectedValue(new Error("network unavailable"));

    render(LoginPage);

    expect(
      await screen.findByText(
        "Authentication is currently unavailable. Please try again later.",
      ),
    ).toBeTruthy();
    expect(screen.queryByRole("button")).toBeNull();
  });

  it("ignores a late auth-config response after the page is unmounted", async () => {
    const replace = vi.fn();
    vi.stubGlobal("window", {
      location: { href: "http://localhost:3000/login", replace },
    });
    let resolveConfig: ((response: unknown) => void) | undefined;
    mockFetch.mockReturnValue(
      new Promise((resolve) => {
        resolveConfig = resolve;
      }),
    );

    render(LoginPage);
    cleanup();
    resolveConfig?.({
      ok: true,
      json: () => Promise.resolve({ enabled: false }),
    });
    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(replace).not.toHaveBeenCalled();
  });
});
