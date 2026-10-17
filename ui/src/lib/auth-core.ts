export interface EnabledAuthConfig {
  enabled: true;
  provider_name: string;
}

export interface DisabledAuthConfig {
  enabled: false;
}

export type AuthConfig = EnabledAuthConfig | DisabledAuthConfig;

export function parseAuthConfig(value: unknown): AuthConfig | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    return null;
  }
  const keys = Object.keys(value);
  if (!keys.includes("enabled") || !("enabled" in value)) return null;
  if (value.enabled === false && keys.length === 1) {
    return { enabled: false };
  }
  if (
    value.enabled !== true ||
    keys.length !== 2 ||
    !keys.includes("provider_name") ||
    !("provider_name" in value) ||
    typeof value.provider_name !== "string"
  ) {
    return null;
  }
  const name = value.provider_name;
  if (
    name.length === 0 ||
    // Rust str::trim() uses Unicode White_Space, not JS trim's U+FEFF.
    /^\p{White_Space}|\p{White_Space}$/u.test(name) ||
    [...name].length > 80 ||
    [...name].some((character) => {
      const code = character.codePointAt(0) ?? 0;
      return code <= 0x1f || (code >= 0x7f && code <= 0x9f);
    })
  ) {
    return null;
  }
  return { enabled: true, provider_name: name };
}

export const MAX_RETURN_TO_BYTES = 2048;

const RESERVED_PATHS = ["/auth", "/login", "/api", "/health"];

function containsUnsafeCharacter(value: string): boolean {
  return [...value].some((character) => {
    const code = character.codePointAt(0) ?? 0;
    return character === "\\" || code <= 0x1f || (code >= 0x7f && code <= 0x9f);
  });
}

/** Preserves a safe serialized local target, otherwise returns `/`. */
export function safeLocalTarget(
  target: string | null | undefined,
  extraReservedPaths: readonly string[] = [],
): string {
  if (
    !target ||
    new TextEncoder().encode(target).byteLength > MAX_RETURN_TO_BYTES ||
    !target.startsWith("/") ||
    target.startsWith("//")
  ) {
    return "/";
  }

  try {
    const decoded = decodeURIComponent(target);
    if (containsUnsafeCharacter(decoded)) return "/";

    const pathEnd = target.search(/[?#]/);
    const path = pathEnd === -1 ? target : target.slice(0, pathEnd);
    const decodedPath = decodeURIComponent(path);
    if (
      /[%?#]/.test(decodedPath) ||
      decodedPath.split("/").length !== path.split("/").length ||
      decodedPath.split("/").some((part) => part === "." || part === "..") ||
      [...RESERVED_PATHS, ...extraReservedPaths].some(
        (reserved) =>
          decodedPath === reserved || decodedPath.startsWith(`${reserved}/`),
      )
    ) {
      return "/";
    }

    if (
      new URL(target, "https://auth.invalid").origin !== "https://auth.invalid"
    ) {
      return "/";
    }
  } catch {
    return "/";
  }
  return target;
}

/** Strictly form-decodes the first recognized return_to, preserving its serialization. */
export function returnToFromQuery(
  rawQuery: string,
  extraReservedPaths: readonly string[] = [],
): string {
  const query = rawQuery.startsWith("?") ? rawQuery.slice(1) : rawQuery;
  for (const pair of query.split("&")) {
    const separator = pair.indexOf("=");
    const key = separator === -1 ? pair : pair.slice(0, separator);
    const value = separator === -1 ? "" : pair.slice(separator + 1);
    let name: string;
    try {
      name = decodeURIComponent(key.replace(/\+/g, " "));
    } catch {
      continue;
    }
    if (name !== "return_to") continue;
    try {
      return safeLocalTarget(
        decodeURIComponent(value.replace(/\+/g, " ")),
        extraReservedPaths,
      );
    } catch {
      return "/";
    }
  }
  return "/";
}

/** Constructs a login-page URL without normalizing the accepted target. */
export function loginUrl(
  target: string | null | undefined,
  additionalExcludedRoots: readonly string[] = [],
): string {
  const returnTo = safeLocalTarget(target, additionalExcludedRoots);
  return `/login?${new URLSearchParams({ return_to: returnTo })}`;
}
