import { beforeEach, describe, expect, it, vi } from "vitest";

describe("notification store", () => {
  let store: typeof import("./notifications.svelte");

  beforeEach(async () => {
    vi.resetModules();
    store = await import("./notifications.svelte");
  });

  it("pushes a notification with generated ID", () => {
    const id = store.pushNotification({ message: "Hello" });
    expect(id).toMatch(/^notification-/);
    const visible = store.getVisibleNotifications();
    expect(visible).toHaveLength(1);
    expect(visible[0].message).toBe("Hello");
    expect(visible[0].variant).toBe("info");
  });

  it("dismisses a notification by ID", () => {
    const id = store.pushNotification({ message: "To dismiss" });
    expect(store.getVisibleNotifications()).toHaveLength(1);
    store.dismissNotification(id);
    expect(store.getVisibleNotifications()).toHaveLength(0);
  });

  it("returns max 3 visible notifications", () => {
    store.pushNotification({ message: "1" });
    store.pushNotification({ message: "2" });
    store.pushNotification({ message: "3" });
    store.pushNotification({ message: "4" });
    expect(store.getAllNotifications()).toHaveLength(4);
    expect(store.getVisibleNotifications()).toHaveLength(3);
  });

  it("clears all notifications", () => {
    store.pushNotification({ message: "1" });
    store.pushNotification({ message: "2" });
    store.clearNotifications();
    expect(store.getVisibleNotifications()).toHaveLength(0);
    expect(store.getAllNotifications()).toHaveLength(0);
  });

  it("uses provided variant", () => {
    store.pushNotification({ message: "Error!", variant: "error" });
    expect(store.getVisibleNotifications()[0].variant).toBe("error");
  });
});
