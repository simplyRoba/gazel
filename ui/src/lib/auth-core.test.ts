import { describe, expect, it } from "vitest";

import vectors from "../../../tests/fixtures/auth/return-targets.json";
import { loginUrl, parseAuthConfig, safeLocalTarget } from "./auth-core.js";

describe.each([
  { app: "gazel" as const, additionalRoots: [] },
  { app: "flowl" as const, additionalRoots: ["/uploads"] },
])("pure authentication navigation ($app)", ({ app, additionalRoots }) => {
  it.each(vectors.vectors)("preserves or rejects $id", (vector) => {
    const input = vector.input;
    const value =
      input === null
        ? null
        : typeof input === "string"
          ? input
          : input.prefix + input.repeat.repeat(input.count) + input.suffix;
    const expected = vector.target[app] ? (value ?? "/") : "/";
    expect(safeLocalTarget(value, additionalRoots)).toBe(expected);

    const wire =
      value === null ? "" : new URLSearchParams({ return_to: value });
    const onceDecoded = new URLSearchParams(wire).get("return_to");
    expect(safeLocalTarget(onceDecoded, additionalRoots)).toBe(expected);

    const login = new URL(
      loginUrl(onceDecoded, additionalRoots),
      "https://auth.invalid",
    );
    expect(login.pathname).toBe("/login");
    expect([...login.searchParams.keys()]).toEqual(["return_to"]);
    expect(login.searchParams.get("return_to")).toBe(expected);
    expect(login.hash).toBe("");
  });

  it("retains accepted query and fragment serialization through standard form encoding", () => {
    const target = "/settings?label=a b&ratio=100%25&next=a%2Fb#part%20one";
    const login = loginUrl(target, additionalRoots);
    expect(login).toContain("a+b");
    expect(
      new URL(login, "https://auth.invalid").searchParams.get("return_to"),
    ).toBe(target);
  });
});

describe("pure public authentication config decoding", () => {
  it.each([
    { enabled: false },
    { enabled: true, provider_name: "OpenID Connect" },
    { enabled: true, provider_name: "é".repeat(80) },
    { enabled: true, provider_name: "🌱".repeat(80) },
  ])("accepts the normalized contract %j", (value) => {
    expect(parseAuthConfig(value)).toEqual(value);
  });

  it.each([
    null,
    undefined,
    true,
    [],
    "enabled",
    {},
    { enabled: "true" },
    { enabled: false, provider_name: "ignored" },
    { enabled: false, extra: true },
    { enabled: true },
    { provider_name: "Provider" },
    { enabled: true, provider_name: "Provider", extra: true },
    { enabled: true, provider_name: null },
    { enabled: true, provider_name: 1 },
    { enabled: true, provider_name: "" },
    { enabled: true, provider_name: " " },
    { enabled: true, provider_name: " Provider" },
    { enabled: true, provider_name: "Provider " },
    { enabled: true, provider_name: "bad\nname" },
    { enabled: true, provider_name: "before\u0085after" },
    { enabled: true, provider_name: "before\u009fafter" },
    { enabled: true, provider_name: "é".repeat(81) },
    { enabled: true, provider_name: "🌱".repeat(81) },
  ])("rejects an invalid contract %j", (value) => {
    expect(parseAuthConfig(value)).toBeNull();
  });

  it("does not accept an inherited discriminant or provider name", () => {
    expect(parseAuthConfig(Object.create({ enabled: false }))).toBeNull();
    expect(
      parseAuthConfig(
        Object.assign(Object.create({ provider_name: "Provider" }), {
          enabled: true,
        }),
      ),
    ).toBeNull();
  });
});
