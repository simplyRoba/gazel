import en from "./en.json";

export type TranslationMap = Record<string, string>;

export const translations: Record<string, TranslationMap> = { en };

const loadingLocales = new Map<string, Promise<void>>();

export async function loadLocale(locale: string): Promise<void> {
  if (locale in translations || locale !== "de") return;

  const inFlight = loadingLocales.get(locale);
  if (inFlight) return inFlight;

  const promise = import("./de.json")
    .then((module) => {
      translations.de = module.default;
    })
    .finally(() => {
      loadingLocales.delete(locale);
    });
  loadingLocales.set(locale, promise);
  return promise;
}
