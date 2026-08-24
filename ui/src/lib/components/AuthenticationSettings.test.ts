import { render, screen } from "@testing-library/svelte";
import { describe, expect, it } from "vitest";
import type { AppInfo } from "$lib/api";
import AuthenticationSettings from "./AuthenticationSettings.svelte";

const disabledInfo: AppInfo = {
  version: "1.0.0",
  repository: "https://example.test/gazel",
  license: "AGPL-3.0-or-later",
};
const enabledInfo: AppInfo = { ...disabledInfo, auth_enabled: true };

describe("AuthenticationSettings", () => {
  it("does not render logout controls when optional auth_enabled is omitted", () => {
    render(AuthenticationSettings, {
      props: { authEnabled: disabledInfo.auth_enabled === true },
    });

    expect(screen.queryByRole("button", { name: "Sign out" })).toBeNull();
  });

  it("posts local logout when authentication is enabled", () => {
    const { container } = render(AuthenticationSettings, {
      props: { authEnabled: enabledInfo.auth_enabled === true },
    });

    expect(
      screen.getByRole("heading", { name: "Authentication" }),
    ).toBeTruthy();
    expect(screen.getByRole("button", { name: "Sign out" })).toBeTruthy();
    const form = container.querySelector("form");
    expect(form?.getAttribute("method")).toBe("POST");
    expect(form?.getAttribute("action")).toBe("/auth/logout");
  });
});
