import { cleanup, fireEvent, render } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import ToastHost from "./ToastHost.svelte";
import {
  clearNotifications,
  getAllNotifications,
  pushNotification,
} from "$lib/stores/notifications.svelte";

describe("ToastHost", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    clearNotifications();
  });

  afterEach(() => {
    cleanup();
    vi.useRealTimers();
  });

  it("resumes automatic dismissal with the remaining time after hover", async () => {
    pushNotification({ message: "Saved", variant: "success" });
    const { container } = render(ToastHost);
    const toast = container.querySelector<HTMLElement>(".toast");
    expect(toast).not.toBeNull();

    await vi.advanceTimersByTimeAsync(3000);
    await fireEvent.mouseEnter(toast!);
    await vi.advanceTimersByTimeAsync(5000);
    expect(getAllNotifications()).toHaveLength(1);

    await fireEvent.mouseLeave(toast!);
    await vi.advanceTimersByTimeAsync(499);
    expect(getAllNotifications()).toHaveLength(1);

    await vi.advanceTimersByTimeAsync(1);
    expect(getAllNotifications()).toHaveLength(0);
  });
});
