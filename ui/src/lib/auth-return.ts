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
