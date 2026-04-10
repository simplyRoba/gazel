import type { ApiError } from "$lib/api";

/**
 * Resolve an API error to a user-facing localized message.
 *
 * Looks up `error.<code>` in translations via the provided `t` function.
 * Falls back to `error.message` (the backend's English default) if no
 * translation key matches.
 */
export function resolveError(
  error: ApiError,
  t: (key: string, params?: Record<string, string | number>) => string,
): string {
  const key = `error.${error.code}`;
  const translated = t(key);

  // If t() returned the key itself, it means no translation was found.
  // Fall back to the backend's message.
  if (translated === key) {
    return error.message;
  }

  return translated;
}
