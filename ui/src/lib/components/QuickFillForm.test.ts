import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { CreateFillup, Fillup } from "$lib/api";
import QuickFillForm from "./QuickFillForm.svelte";

const storeState = vi.hoisted(() => ({
  fillups: [] as Fillup[],
  loadMoreFillups: vi.fn(),
}));

vi.mock("$lib/stores/settings.svelte", () => ({
  getSettings: () => ({
    unit_system: "metric",
    distance_unit: "km",
    volume_unit: "l",
    currency: "USD",
    color_mode: "system",
    locale: "en",
  }),
}));

vi.mock("$lib/stores/fillups.svelte", () => ({
  getFillupsByVehicle: () => storeState.fillups,
  loadMoreFillups: storeState.loadMoreFillups,
}));

function fillup(
  id: number,
  odometer: number,
  overrides: Partial<Fillup> = {},
): Fillup {
  return {
    id,
    vehicle_id: 1,
    date: `2026-01-${String(id).padStart(2, "0")}`,
    odometer,
    fuel_amount: 40,
    fuel_unit: "l",
    cost: 60,
    currency: "USD",
    is_full_tank: true,
    is_missed: false,
    station: null,
    notes: null,
    created_at: "2026-01-01T00:00:00Z",
    updated_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}

function renderForm() {
  const onsave = vi.fn(async (_data: CreateFillup): Promise<void> => {});
  const oncancel = vi.fn();
  render(QuickFillForm, {
    props: { vehicleId: 1, onsave, oncancel },
  });
  return { onsave, oncancel };
}

function inputNamed(name: RegExp): HTMLInputElement {
  return screen.getByRole("textbox", { name }) as HTMLInputElement;
}

beforeEach(() => {
  storeState.fillups = [];
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("QuickFillForm", () => {
  it("calculates total after fuel and price are entered", async () => {
    renderForm();

    await fireEvent.input(inputNamed(/Fuel amount/), {
      target: { value: "40" },
    });
    await fireEvent.input(inputNamed(/Price/), {
      target: { value: "1.5" },
    });

    await waitFor(() => {
      expect(inputNamed(/Total/).value).toBe("60");
    });
  });

  it("shows field errors and does not save invalid input", async () => {
    const { onsave } = renderForm();

    await fireEvent.click(screen.getByRole("button", { name: "Add fill-up" }));

    expect(screen.getByText("Odometer is required.")).toBeTruthy();
    expect(screen.getByText("Fuel amount is required.")).toBeTruthy();
    expect(screen.getByText("Cost is required.")).toBeTruthy();
    expect(onsave).not.toHaveBeenCalled();
  });

  it("shows tank-to-tank efficiency and converts intervening partial fuel", async () => {
    storeState.fillups = [
      fillup(2, 1200, {
        fuel_amount: 10 / 3.785_411_784,
        fuel_unit: "gal",
        is_full_tank: false,
      }),
      fillup(1, 1000),
    ];
    renderForm();

    await fireEvent.input(inputNamed(/Odometer/), {
      target: { value: "1500" },
    });
    await fireEvent.input(inputNamed(/Fuel amount/), {
      target: { value: "30" },
    });

    expect(await screen.findByText("Est. 8.0 L/100 km")).toBeTruthy();
  });

  it("hides efficiency when there is no previous full-tank reading", async () => {
    storeState.fillups = [fillup(1, 1000, { is_full_tank: false })];
    renderForm();

    await fireEvent.input(inputNamed(/Odometer/), {
      target: { value: "1500" },
    });
    await fireEvent.input(inputNamed(/Fuel amount/), {
      target: { value: "40" },
    });

    expect(screen.queryByText(/^Est\./)).toBeNull();
  });

  it("hides efficiency for an invalid tank-to-tank segment", async () => {
    storeState.fillups = [
      fillup(2, 1200, { is_full_tank: false, is_missed: true }),
      fillup(1, 1000),
    ];
    renderForm();

    await fireEvent.input(inputNamed(/Odometer/), {
      target: { value: "1500" },
    });
    await fireEvent.input(inputNamed(/Fuel amount/), {
      target: { value: "40" },
    });

    expect(screen.queryByText(/^Est\./)).toBeNull();
  });

  it("ignores a full-tank baseline without a valid odometer", async () => {
    storeState.fillups = [fillup(1, 0)];
    renderForm();

    await fireEvent.input(inputNamed(/Odometer/), {
      target: { value: "500" },
    });
    await fireEvent.input(inputNamed(/Fuel amount/), {
      target: { value: "40" },
    });

    expect(screen.queryByText(/^Est\./)).toBeNull();
  });

  it("hides efficiency when the current fill-up is partial", async () => {
    storeState.fillups = [fillup(1, 1000)];
    renderForm();

    await fireEvent.input(inputNamed(/Odometer/), {
      target: { value: "1500" },
    });
    await fireEvent.input(inputNamed(/Fuel amount/), {
      target: { value: "40" },
    });
    expect(await screen.findByText("Est. 8.0 L/100 km")).toBeTruthy();

    await fireEvent.click(screen.getByRole("button", { name: "More details" }));
    await fireEvent.click(screen.getByLabelText("Full tank"));

    expect(screen.queryByText(/^Est\./)).toBeNull();
  });

  it("keeps optional details collapsed until requested", async () => {
    renderForm();

    expect(screen.queryByLabelText(/Date/)).toBeNull();
    expect(screen.queryByLabelText("Station")).toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "More details" }));

    expect(screen.getByLabelText(/Date/)).toBeTruthy();
    expect(screen.getByLabelText("Station")).toBeTruthy();
    expect(screen.getByLabelText("Notes")).toBeTruthy();
    expect(screen.getByLabelText("Full tank")).toBeTruthy();
    expect(screen.getByLabelText("Missed fill-up before this")).toBeTruthy();
  });

  it("uses currently loaded recent entries without requesting continuation", async () => {
    storeState.fillups = [fillup(3, 2000), fillup(2, 1500), fillup(1, 1000)];
    renderForm();

    await waitFor(() => expect(inputNamed(/Odometer/).value).toBe("2000"));
    await fireEvent.input(inputNamed(/Odometer/), {
      target: { value: "3000" },
    });

    expect(
      await screen.findByText(
        "That's a larger gap than usual. Did you miss a fill-up?",
      ),
    ).toBeTruthy();
    expect(storeState.loadMoreFillups).not.toHaveBeenCalled();
  });

  it("offers the smart missed-fill-up action for an unusually large gap", async () => {
    storeState.fillups = [fillup(3, 2000), fillup(2, 1500), fillup(1, 1000)];
    renderForm();

    await fireEvent.input(inputNamed(/Odometer/), {
      target: { value: "3000" },
    });

    expect(
      await screen.findByText(
        "That's a larger gap than usual. Did you miss a fill-up?",
      ),
    ).toBeTruthy();
    await fireEvent.click(
      screen.getByRole("button", { name: "Yes, mark as missed" }),
    );
    expect(
      screen.queryByText(
        "That's a larger gap than usual. Did you miss a fill-up?",
      ),
    ).toBeNull();
  });

  it("submits a valid create payload without price per unit", async () => {
    const { onsave } = renderForm();

    await fireEvent.input(inputNamed(/Odometer/), {
      target: { value: "1000" },
    });
    await fireEvent.input(inputNamed(/Fuel amount/), {
      target: { value: "40" },
    });
    await fireEvent.input(inputNamed(/Total/), {
      target: { value: "60" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Add fill-up" }));

    await waitFor(() => expect(onsave).toHaveBeenCalledOnce());
    const payload = onsave.mock.calls[0][0];
    expect(payload).toEqual({
      date: new Date().toISOString().slice(0, 10),
      odometer: 1000,
      fuel_amount: 40,
      cost: 60,
      is_full_tank: true,
      is_missed: false,
      station: null,
      notes: null,
    });
    expect(payload).not.toHaveProperty("price_per_unit");
  });
});
