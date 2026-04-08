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

function applyTheme(theme: EffectiveTheme): void {
  document.documentElement.setAttribute("data-theme", theme);
}

export function setTheme(pref: ThemePreference): void {
  themePreference = pref;
  localStorage.setItem(STORAGE_KEY, pref);
  applyTheme(pref === "system" ? (systemPrefersDark ? "dark" : "light") : pref);
}

export function initTheme(): void {
  const stored = localStorage.getItem(STORAGE_KEY) as ThemePreference | null;
  if (stored === "light" || stored === "dark" || stored === "system") {
    themePreference = stored;
  }

  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  systemPrefersDark = mediaQuery.matches;

  mediaQuery.addEventListener("change", (e: MediaQueryListEvent) => {
    systemPrefersDark = e.matches;
    if (themePreference === "system") {
      applyTheme(e.matches ? "dark" : "light");
    }
  });

  applyTheme(
    themePreference === "system"
      ? systemPrefersDark
        ? "dark"
        : "light"
      : themePreference,
  );
}

export function getThemePreference(): ThemePreference {
  return themePreference;
}

export function getEffectiveTheme(): EffectiveTheme {
  return effectiveTheme;
}
