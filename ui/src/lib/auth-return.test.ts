import { describe, expect, it } from "vitest";

import vectors from "../../../tests/fixtures/auth/return-targets.json";
import { safeLocalTarget } from "./auth-return.js";

describe("safe local authentication targets", () => {
  it.each(vectors.vectors)("preserves or rejects $id", (vector) => {
    const input = vector.input;
    const value =
      input === null
        ? null
        : typeof input === "string"
          ? input
          : input.prefix + input.repeat.repeat(input.count) + input.suffix;
    const expected = vector.target.gazel ? (value ?? "/") : "/";
    expect(safeLocalTarget(value)).toBe(expected);

    const query =
      value === null ? "" : new URLSearchParams({ return_to: value });
    expect(safeLocalTarget(new URLSearchParams(query).get("return_to"))).toBe(
      expected,
    );
  });

  it("supports additional app-owned reserved namespaces", () => {
    expect(safeLocalTarget("/uploads/photo.jpg")).toBe("/uploads/photo.jpg");
    expect(safeLocalTarget("/uploads/photo.jpg", ["/uploads"])).toBe("/");
    expect(safeLocalTarget("/uploads-other", ["/uploads"])).toBe(
      "/uploads-other",
    );
  });
});
