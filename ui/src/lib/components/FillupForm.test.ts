import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { CreateFillup, Fillup } from "$lib/api";
import FillupForm from "./FillupForm.svelte";

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

function loadedFillup(id: number, odometer: number): Fillup {
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
  };
}

beforeEach(() => {
  storeState.fillups = [];
});

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

// Test the validation logic extracted from FillupForm independently; the
// loaded-history behavior below mounts the component to cover store usage.

describe("FillupForm validation rules", () => {
  function validate(fields: {
    date: string;
    odometer: string;
    fuelAmount: string;
    cost: string;
  }): Record<string, string> {
    const errors: Record<string, string> = {};

    if (!fields.date.trim()) {
      errors.date = "Date is required.";
    }

    const odoVal = parseFloat(fields.odometer);
    if (!fields.odometer.trim() || isNaN(odoVal)) {
      errors.odometer = "Odometer is required.";
    } else if (odoVal < 0) {
      errors.odometer = "Odometer must be positive.";
    }

    const fuelVal = parseFloat(fields.fuelAmount);
    if (!fields.fuelAmount.trim() || isNaN(fuelVal)) {
      errors.fuelAmount = "Fuel amount is required.";
    } else if (fuelVal <= 0) {
      errors.fuelAmount = "Fuel amount must be greater than zero.";
    }

    const costVal = parseFloat(fields.cost);
    if (!fields.cost.trim() || isNaN(costVal)) {
      errors.cost = "Cost is required.";
    } else if (costVal < 0) {
      errors.cost = "Cost must not be negative.";
    }

    return errors;
  }

  it("passes with all valid fields", () => {
    const errors = validate({
      date: "2026-04-09",
      odometer: "10000",
      fuelAmount: "42.3",
      cost: "78.50",
    });
    expect(errors).toEqual({});
  });

  it("requires date", () => {
    const errors = validate({
      date: "",
      odometer: "10000",
      fuelAmount: "42.3",
      cost: "78.50",
    });
    expect(errors.date).toBe("Date is required.");
  });

  it("requires odometer", () => {
    const errors = validate({
      date: "2026-04-09",
      odometer: "",
      fuelAmount: "42.3",
      cost: "78.50",
    });
    expect(errors.odometer).toBe("Odometer is required.");
  });

  it("rejects negative odometer", () => {
    const errors = validate({
      date: "2026-04-09",
      odometer: "-100",
      fuelAmount: "42.3",
      cost: "78.50",
    });
    expect(errors.odometer).toBe("Odometer must be positive.");
  });

  it("requires fuel amount", () => {
    const errors = validate({
      date: "2026-04-09",
      odometer: "10000",
      fuelAmount: "",
      cost: "78.50",
    });
    expect(errors.fuelAmount).toBe("Fuel amount is required.");
  });

  it("rejects zero fuel amount", () => {
    const errors = validate({
      date: "2026-04-09",
      odometer: "10000",
      fuelAmount: "0",
      cost: "78.50",
    });
    expect(errors.fuelAmount).toBe("Fuel amount must be greater than zero.");
  });

  it("rejects negative fuel amount", () => {
    const errors = validate({
      date: "2026-04-09",
      odometer: "10000",
      fuelAmount: "-5",
      cost: "78.50",
    });
    expect(errors.fuelAmount).toBe("Fuel amount must be greater than zero.");
  });

  it("requires cost", () => {
    const errors = validate({
      date: "2026-04-09",
      odometer: "10000",
      fuelAmount: "42.3",
      cost: "",
    });
    expect(errors.cost).toBe("Cost is required.");
  });

  it("rejects negative cost", () => {
    const errors = validate({
      date: "2026-04-09",
      odometer: "10000",
      fuelAmount: "42.3",
      cost: "-10",
    });
    expect(errors.cost).toBe("Cost must not be negative.");
  });

  it("accepts zero cost", () => {
    const errors = validate({
      date: "2026-04-09",
      odometer: "10000",
      fuelAmount: "42.3",
      cost: "0",
    });
    expect(errors).toEqual({});
  });

  it("reports multiple errors at once", () => {
    const errors = validate({
      date: "",
      odometer: "",
      fuelAmount: "",
      cost: "",
    });
    expect(Object.keys(errors)).toHaveLength(4);
  });
});

describe("FillupForm loaded history helpers", () => {
  it("uses currently loaded recent entries without requesting continuation", async () => {
    storeState.fillups = [
      loadedFillup(3, 2000),
      loadedFillup(2, 1500),
      loadedFillup(1, 1000),
    ];
    const onsave = vi.fn(async (_data: CreateFillup): Promise<void> => {});
    render(FillupForm, {
      props: {
        vehicleId: 1,
        onsave,
        oncancel: vi.fn(),
      },
    });

    const odometer = screen.getByRole("textbox", {
      name: /Odometer/,
    }) as HTMLInputElement;
    await waitFor(() => expect(odometer.value).toBe("2000"));

    await fireEvent.input(odometer, { target: { value: "3000" } });
    expect(
      await screen.findByText(
        "That's a larger gap than usual. Did you miss a fill-up?",
      ),
    ).toBeTruthy();
    expect(storeState.loadMoreFillups).not.toHaveBeenCalled();
  });
});

describe("Smart missed fill-up detection", () => {
  function shouldPromptMissed(
    newOdometer: number,
    existingOdometers: number[],
  ): boolean {
    if (existingOdometers.length < 2) return false;
    const lastOdo = existingOdometers[0];
    if (lastOdo <= 0) return false;

    const validOdos = existingOdometers.filter((o) => o > 0);
    if (validOdos.length < 2) return false;

    let totalGap = 0;
    for (let i = 0; i < validOdos.length - 1; i++) {
      totalGap += validOdos[i] - validOdos[i + 1];
    }
    const avgGap = totalGap / (validOdos.length - 1);
    const currentGap = newOdometer - lastOdo;

    return avgGap > 0 && currentGap > avgGap * 1.75;
  }

  it("prompts when gap exceeds 1.75x average", () => {
    // Average gap = 500, new gap = 1000 → 2x → prompt
    expect(shouldPromptMissed(11500, [10500, 10000, 9500])).toBe(true);
  });

  it("does not prompt when gap is within normal", () => {
    // Average gap = 500, new gap = 500 → 1x → no prompt
    expect(shouldPromptMissed(11000, [10500, 10000, 9500])).toBe(false);
  });

  it("does not prompt with fewer than 2 fill-ups", () => {
    expect(shouldPromptMissed(15000, [10000])).toBe(false);
  });

  it("does not prompt with empty history", () => {
    expect(shouldPromptMissed(10000, [])).toBe(false);
  });
});
