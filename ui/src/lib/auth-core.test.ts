import { describe, expect, it } from "vitest";

import vectors from "../../../tests/fixtures/auth/return-targets.json";
import queryVectors from "../../../tests/fixtures/auth/return-queries.json";
import labelVectors from "../../../tests/fixtures/auth/provider-labels.json";
import {
  loginUrl,
  parseAuthConfig,
  returnToFromQuery,
  safeLocalTarget,
  shouldAutoRedirect,
} from "./auth-core.js";

type Label =
  | string
  | null
  | { repeat: string; count: number; prefix?: string; suffix?: string };

function expandLabel(value: Label): string | null {
  return value === null || typeof value === "string"
    ? value
    : (value.prefix ?? "") +
        value.repeat.repeat(value.count) +
        (value.suffix ?? "");
}

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

  it.each(queryVectors.vectors)(
    "strictly decodes raw return query: $id",
    (vector) => {
      expect(returnToFromQuery(vector.query, additionalRoots)).toBe(
        vector.target[app],
      );
      expect(returnToFromQuery(`?${vector.query}`, additionalRoots)).toBe(
        vector.target[app],
      );
    },
  );

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
  it.each(labelVectors.vectors)(
    "accepts backend-normalized labels unchanged: $id",
    (vector) => {
      const normalized = expandLabel(vector.normalized);
      if (normalized !== null) {
        const config = {
          enabled: true,
          provider_name: normalized,
          auto_redirect: false,
        };
        expect(parseAuthConfig(config)).toEqual(config);
      }
      const raw = expandLabel(vector.input);
      if (raw !== null) {
        const config = {
          enabled: true,
          provider_name: raw,
          auto_redirect: false,
        };
        expect(parseAuthConfig(config)).toEqual(
          raw === normalized ? config : null,
        );
      }
    },
  );

  it.each([
    { enabled: false },
    { enabled: true, provider_name: "OpenID Connect", auto_redirect: false },
    { enabled: true, provider_name: "OpenID Connect", auto_redirect: true },
    { enabled: true, provider_name: "é".repeat(80), auto_redirect: false },
    { enabled: true, provider_name: "🌱".repeat(80), auto_redirect: false },
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
    {
      enabled: true,
      provider_name: "Provider",
      auto_redirect: false,
      extra: true,
    },
    { enabled: true, provider_name: "Provider" },
    { enabled: true, provider_name: "Provider", auto_redirect: null },
    { enabled: true, provider_name: "Provider", auto_redirect: "true" },
    { enabled: true, provider_name: "Provider", auto_redirect: 1 },
    { enabled: false, auto_redirect: false },
    { enabled: false, auto_redirect: true },
    { enabled: true, provider_name: null, auto_redirect: false },
    { enabled: true, provider_name: 1, auto_redirect: false },
    { enabled: true, provider_name: "", auto_redirect: false },
    { enabled: true, provider_name: " ", auto_redirect: false },
    { enabled: true, provider_name: " Provider", auto_redirect: false },
    { enabled: true, provider_name: "Provider ", auto_redirect: false },
    { enabled: true, provider_name: "bad\nname", auto_redirect: false },
    { enabled: true, provider_name: "before\u0085after", auto_redirect: false },
    { enabled: true, provider_name: "before\u009fafter", auto_redirect: false },
    { enabled: true, provider_name: "é".repeat(81), auto_redirect: false },
    { enabled: true, provider_name: "🌱".repeat(81), auto_redirect: false },
  ])("rejects an invalid contract %j", (value) => {
    expect(parseAuthConfig(value)).toBeNull();
  });

  it("does not accept an inherited discriminant or provider name", () => {
    expect(parseAuthConfig(Object.create({ enabled: false }))).toBeNull();
    expect(
      parseAuthConfig(
        Object.assign(Object.create({ provider_name: "Provider" }), {
          enabled: true,
          auto_redirect: false,
        }),
      ),
    ).toBeNull();
    expect(
      parseAuthConfig(
        Object.assign(Object.create({ auto_redirect: true }), {
          enabled: true,
          provider_name: "Provider",
        }),
      ),
    ).toBeNull();
  });
});

describe("automatic provider navigation", () => {
  const automatic = {
    enabled: true,
    provider_name: "Provider",
    auto_redirect: true,
  } as const;

  it.each(["", "?return_to=%2Fsettings%3Ftab%3Ddata%23export"])(
    "starts on an ordinary login query %j",
    (query) => {
      expect(shouldAutoRedirect(automatic, query)).toBe(true);
      expect(
        shouldAutoRedirect({ ...automatic, auto_redirect: false }, query),
      ).toBe(false);
      expect(shouldAutoRedirect({ enabled: false }, query)).toBe(false);
    },
  );

  it.each([
    "?error=authentication_failed",
    "?error=provider_unavailable",
    "?error=unknown",
    "?error=",
    "?error",
    "?%65rror=unknown",
    "?logged_out=1",
    "?logged_out=0",
    "?logged_out=",
    "?logged_out",
    "?error=&logged_out=1&return_to=%2Fsettings",
  ])("does not restart login from a result query %j", (query) => {
    expect(shouldAutoRedirect(automatic, query)).toBe(false);
  });
});
