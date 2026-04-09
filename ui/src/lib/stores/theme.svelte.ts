const STORAGE_KEY = "gazel.theme";

export type ThemePreference = "light" | "dark" | "system";
export type EffectiveTheme = "light" | "dark";

let themePreference = $state<ThemePreference>("system");
let systemPrefersDark = $state(false);

const effectiveTheme = $derived<EffectiveTheme>(
  themePreference === "system"
    ? systemPrefersDark
      ? "dark"
      : "light"
    : themePreference,
);

let mediaQuery: MediaQueryList | null = null;

function isValidPreference(value: unknown): value is ThemePreference {
  return value === "light" || value === "dark" || value === "system";
}

function resolve(pref: ThemePreference): EffectiveTheme {
  return pref === "system" ? (systemPrefersDark ? "dark" : "light") : pref;
}

function applyTheme(theme: EffectiveTheme): void {
  document.documentElement.setAttribute("data-theme", theme);
}

export function setTheme(pref: ThemePreference): void {
  themePreference = pref;
  localStorage.setItem(STORAGE_KEY, pref);
  applyTheme(resolve(pref));
  // Async server sync — fire-and-forget.
  import("$lib/api")
    .then(({ updateSettings }) => updateSettings({ color_mode: pref }))
    .catch(() => {});
}

export function initTheme(serverColorMode?: string): void {
  const stored = localStorage.getItem(STORAGE_KEY);
  const localPref: ThemePreference = isValidPreference(stored)
    ? stored
    : "system";

  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  systemPrefersDark = mediaQuery.matches;

  mediaQuery.addEventListener("change", (e: MediaQueryListEvent) => {
    systemPrefersDark = e.matches;
    if (themePreference === "system") {
      applyTheme(e.matches ? "dark" : "light");
    }
  });

  if (serverColorMode !== undefined && isValidPreference(serverColorMode)) {
    // Server is authoritative — unless this is the first sync and the server
    // still has the default while localStorage has an explicit user choice.
    if (
      serverColorMode === "system" &&
      (localPref === "light" || localPref === "dark")
    ) {
      // First-sync upgrade: push localStorage value to server.
      themePreference = localPref;
      import("$lib/api")
        .then(({ updateSettings }) => updateSettings({ color_mode: localPref }))
        .catch(() => {});
    } else {
      // Server wins — update localStorage to match.
      themePreference = serverColorMode;
      localStorage.setItem(STORAGE_KEY, serverColorMode);
    }
  } else {
    // No server value available — use localStorage as-is.
    themePreference = localPref;
  }

  applyTheme(resolve(themePreference));
}

export function getThemePreference(): ThemePreference {
  return themePreference;
}

export function getEffectiveTheme(): EffectiveTheme {
  return effectiveTheme;
}
