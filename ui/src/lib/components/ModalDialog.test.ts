import { cleanup, render, waitFor } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("$lib/api", async (importOriginal) => ({
  ...(await importOriginal<typeof import("$lib/api")>()),
  updateSettings: vi.fn(),
}));

import * as api from "$lib/api";
import ModalDialog from "./ModalDialog.svelte";
import { getSettings, updateSettingsStore } from "$lib/stores/settings.svelte";

afterEach(cleanup);

describe("ModalDialog", () => {
  it("updates its default confirmation label when the locale changes", async () => {
    vi.mocked(api.updateSettings).mockImplementation(async (update) => ({
      ...getSettings(),
      ...update,
    }));
    const { container } = render(ModalDialog, {
      props: { mode: "confirm" },
    });

    const confirmButton = () => container.querySelectorAll("button")[1];
    expect(confirmButton()?.textContent?.trim()).toBe("Confirm");

    await updateSettingsStore({ locale: "de" });

    await waitFor(() =>
      expect(confirmButton()?.textContent?.trim()).toBe("Bestätigen"),
    );
  });
});
